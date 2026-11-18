// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic runtime for typed model state/event contracts.

use std::collections::{BTreeMap, BTreeSet};
use xdl_gnc_core::{DurationNs, Timestamp};
use xdl_model::{
    BlockId, Model, PortDirection, PortId, SignalShape, StateId, StateMachine, StateMachineError,
    TimedTransition,
};

use crate::guard_expression::GuardExpression;
use crate::{BlockRuntime, ExecutionError, ScheduledEvent, Scheduler, SignalValue, TaskId};

pub struct StateMachineRuntime {
    machine: StateMachine,
    current: StateId,
    current_time: Timestamp,
    state_entered_at: Timestamp,
}

/// State-machine runtime registered as a periodic scheduler task.
pub struct ScheduledStateMachineRuntime {
    runtime: StateMachineRuntime,
    task: TaskId,
}

/// Scheduled graph block for guarded, event-driven state machines.
/// Event code zero means no external event; guards are scalar inputs where
/// zero is false and any finite nonzero value is true.
pub struct StateMachineBlock {
    runtime: StateMachineRuntime,
    event_input: PortId,
    guard_inputs: BTreeMap<String, PortId>,
    output: PortId,
    event_codes: BTreeMap<i64, String>,
    guard_expressions: BTreeMap<String, GuardExpression>,
}

impl StateMachineBlock {
    /// Builds a state-machine block from model-document parameters.
    /// Required parameters: `state_machine`, `event_input`, `state_output`,
    /// and `event_codes`; `guard_inputs` maps expression variables to input
    /// port names.
    pub fn from_model(model: &Model, block_id: BlockId) -> Result<Self, ExecutionError> {
        let block = model.block(block_id)?;
        let machine_value = block
            .parameters
            .get("state_machine")
            .cloned()
            .ok_or_else(|| missing_state_machine_parameter("state_machine"))?;
        let machine: StateMachine = serde_json::from_value(machine_value).map_err(|error| {
            ExecutionError::Runtime(format!("invalid state_machine parameter: {error}"))
        })?;
        let event_input_name = parameter_string(&block.parameters, "event_input")?;
        let state_output_name = parameter_string(&block.parameters, "state_output")?;
        let event_input =
            model_port_by_name(model, block_id, event_input_name, PortDirection::Input)?;
        let output = model_port_by_name(model, block_id, state_output_name, PortDirection::Output)?;

        let event_codes_value = block
            .parameters
            .get("event_codes")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| {
                ExecutionError::Runtime(
                    "state-machine event_codes parameter must map event names to integer codes"
                        .to_string(),
                )
            })?;
        let event_codes =
            event_codes_value
                .iter()
                .try_fold(BTreeMap::new(), |mut mappings, (event, code)| {
                    let code = code.as_i64().ok_or_else(|| {
                        ExecutionError::Runtime(format!(
                            "state-machine event code for {event} must be an integer"
                        ))
                    })?;
                    if mappings.insert(code, event.clone()).is_some() {
                        return Err(ExecutionError::Runtime(format!(
                            "duplicate state-machine event code {code}"
                        )));
                    }
                    Ok(mappings)
                })?;

        let guard_inputs = match block.parameters.get("guard_inputs") {
            None => BTreeMap::new(),
            Some(value) => {
                let mappings = value.as_object().ok_or_else(|| {
                    ExecutionError::Runtime(
                        "state-machine guard_inputs parameter must map guard names to port names"
                            .to_string(),
                    )
                })?;
                mappings.iter().try_fold(
                    BTreeMap::<String, PortId>::new(),
                    |mut ports, (guard, name)| -> Result<_, ExecutionError> {
                        let name = name.as_str().ok_or_else(|| {
                            ExecutionError::Runtime(format!(
                                "state-machine guard port for {guard} must be a string"
                            ))
                        })?;
                        let port = model_port_by_name(model, block_id, name, PortDirection::Input)?;
                        ports.insert(guard.clone(), port);
                        Ok(ports)
                    },
                )?
            }
        };
        Self::new(machine, event_input, guard_inputs, output, event_codes)
    }

    pub fn new(
        machine: StateMachine,
        event_input: PortId,
        guard_inputs: BTreeMap<String, PortId>,
        output: PortId,
        event_codes: BTreeMap<i64, String>,
    ) -> Result<Self, ExecutionError> {
        let known_events: BTreeSet<_> = machine
            .transitions
            .iter()
            .map(|transition| transition.event.as_str())
            .collect();
        const MAX_EXACT_EVENT_CODE: i64 = 9_007_199_254_740_992;
        if event_codes.iter().any(|(code, event)| {
            *code == 0
                || !(-MAX_EXACT_EVENT_CODE..=MAX_EXACT_EVENT_CODE).contains(code)
                || event.trim().is_empty()
                || !known_events.contains(event.as_str())
        }) {
            return Err(ExecutionError::Runtime(
                "state-machine event codes must be nonzero and map to declared events".to_string(),
            ));
        }
        let known_guards: BTreeSet<_> = machine
            .transitions
            .iter()
            .filter_map(|transition| transition.guard.as_deref())
            .collect();
        let guard_expressions = known_guards.iter().try_fold(
            BTreeMap::<String, GuardExpression>::new(),
            |mut expressions, source| {
                let expression = GuardExpression::parse(source).map_err(|error| {
                    ExecutionError::Runtime(format!("invalid guard expression {source:?}: {error}"))
                })?;
                expressions.insert((*source).to_string(), expression);
                Ok::<_, ExecutionError>(expressions)
            },
        )?;
        let required_variables: BTreeSet<_> = guard_expressions
            .values()
            .flat_map(|expression| expression.variables().iter().cloned())
            .collect();
        if guard_inputs.keys().any(|name| name.trim().is_empty())
            || required_variables
                .iter()
                .any(|variable| !guard_inputs.contains_key(variable))
            || guard_inputs
                .keys()
                .any(|variable| !required_variables.contains(variable))
        {
            return Err(ExecutionError::Runtime(
                "state-machine guard_inputs must map every expression variable exactly by name"
                    .to_string(),
            ));
        }
        let guard_ports: BTreeSet<_> = guard_inputs.values().copied().collect();
        if event_input == output
            || guard_inputs
                .values()
                .any(|port| *port == output || *port == event_input)
            || guard_ports.len() != guard_inputs.len()
        {
            return Err(ExecutionError::Runtime(
                "state-machine input and output ports must be distinct".to_string(),
            ));
        }
        let runtime = StateMachineRuntime::new(machine)?;
        Ok(Self {
            runtime,
            event_input,
            guard_inputs,
            output,
            event_codes,
            guard_expressions,
        })
    }
}

impl BlockRuntime for StateMachineBlock {
    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let event_code = scalar_input(inputs, self.event_input, "state-machine event code")?;
        const MAX_EXACT_SIGNAL_INTEGER: f64 = 9_007_199_254_740_992.0;
        if event_code.fract() != 0.0
            || !(-MAX_EXACT_SIGNAL_INTEGER..=MAX_EXACT_SIGNAL_INTEGER).contains(&event_code)
        {
            return Err(ExecutionError::Runtime(
                "state-machine event code must be an exactly representable integer".to_string(),
            ));
        }
        let event_code = event_code as i64;
        let event = if event_code == 0 {
            None
        } else {
            Some(self.event_codes.get(&event_code).ok_or_else(|| {
                ExecutionError::Runtime(format!(
                    "state-machine event code {event_code} is not configured"
                ))
            })?)
        };
        let guard_values = self
            .guard_inputs
            .iter()
            .map(|(variable, port)| {
                let value = scalar_input(inputs, *port, "state-machine guard expression")?;
                Ok((variable.clone(), value))
            })
            .collect::<Result<BTreeMap<_, _>, ExecutionError>>()?;
        let active_guards = self
            .guard_expressions
            .values()
            .map(|expression| {
                expression
                    .evaluate(&guard_values)
                    .map(|active| active.then(|| expression.source().to_string()))
                    .map_err(|error| {
                        ExecutionError::Runtime(format!(
                            "could not evaluate state-machine guard {:?}: {error}",
                            expression.source()
                        ))
                    })
            })
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .flatten()
            .collect::<BTreeSet<_>>();

        self.runtime.advance_to(timestamp)?;
        if let Some(event) = event {
            self.runtime.step_with_guards(event, &active_guards)?;
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(f64::from(self.runtime.current_state().value())),
        )]))
    }
}

fn missing_state_machine_parameter(name: &str) -> ExecutionError {
    ExecutionError::Runtime(format!(
        "state-machine block is missing required parameter {name}"
    ))
}

fn parameter_string<'a>(
    parameters: &'a BTreeMap<String, serde_json::Value>,
    name: &str,
) -> Result<&'a str, ExecutionError> {
    parameters
        .get(name)
        .and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| missing_state_machine_parameter(name))
}

fn model_port_by_name(
    model: &Model,
    block_id: BlockId,
    name: &str,
    direction: PortDirection,
) -> Result<PortId, ExecutionError> {
    let block = model.block(block_id)?;
    let mut matches = block
        .ports
        .values()
        .filter(|port| port.name.eq_ignore_ascii_case(name));
    let port = matches.next().ok_or_else(|| {
        ExecutionError::Runtime(format!("state-machine block has no port named {name}"))
    })?;
    if matches.next().is_some() {
        return Err(ExecutionError::Runtime(format!(
            "state-machine block has ambiguous port name {name}"
        )));
    }
    if port.direction != direction || port.signal.shape != SignalShape::Scalar {
        return Err(ExecutionError::Runtime(format!(
            "state-machine port {name} must be a scalar {direction:?} port"
        )));
    }
    Ok(port.id)
}

fn scalar_input(
    inputs: &BTreeMap<PortId, SignalValue>,
    port: PortId,
    context: &str,
) -> Result<f64, ExecutionError> {
    match inputs.get(&port) {
        Some(SignalValue::Scalar(value)) if value.is_finite() => Ok(*value),
        Some(SignalValue::Scalar(_)) => {
            Err(ExecutionError::Runtime(format!("{context} must be finite")))
        }
        Some(_) => Err(ExecutionError::Runtime(format!(
            "{context} input must be scalar"
        ))),
        None => Err(ExecutionError::MissingInput(port)),
    }
}

impl ScheduledStateMachineRuntime {
    pub fn new(
        machine: StateMachine,
        scheduler: &mut Scheduler,
        name: impl Into<String>,
        period: DurationNs,
        phase: DurationNs,
        priority: u32,
    ) -> Result<Self, ExecutionError> {
        let runtime = StateMachineRuntime::new(machine)?;
        let task = scheduler.add_periodic_task(name, period, phase, priority)?;
        Ok(Self { runtime, task })
    }

    pub fn task_id(&self) -> TaskId {
        self.task
    }

    /// Dispatches one scheduler release. Due timed transitions run first,
    /// followed by the optional external event at that same timestamp.
    pub fn dispatch(
        &mut self,
        event: &ScheduledEvent,
        event_name: Option<&str>,
        active_guards: &BTreeSet<String>,
    ) -> Result<StateId, ExecutionError> {
        if event.task != self.task {
            return Err(ExecutionError::WrongStateMachineTask {
                expected: self.task,
                actual: event.task,
            });
        }
        self.runtime.advance_to(event.timestamp)?;
        match event_name {
            Some(name) => self.runtime.step_with_guards(name, active_guards),
            None => Ok(self.runtime.current_state()),
        }
    }
}

impl StateMachineRuntime {
    pub fn new(machine: StateMachine) -> Result<Self, ExecutionError> {
        let current = machine
            .initial_state
            .ok_or_else(|| state_machine_error(StateMachineError::MissingInitialState))?;
        Self::from_explicit_state(machine, current, Timestamp::ZERO, Timestamp::ZERO)
    }

    /// Restores a state-machine runtime from explicit serialized state.
    pub fn from_explicit_state(
        machine: StateMachine,
        current: StateId,
        state_entered_at: Timestamp,
        current_time: Timestamp,
    ) -> Result<Self, ExecutionError> {
        machine.validate().map_err(state_machine_error)?;
        if !machine.states.contains_key(&current) {
            return Err(state_machine_error(StateMachineError::UnknownState(
                current,
            )));
        }
        if state_entered_at > current_time {
            return Err(ExecutionError::Runtime(
                "state entry time cannot be later than current simulation time".to_string(),
            ));
        }
        Ok(Self {
            machine,
            current,
            current_time,
            state_entered_at,
        })
    }

    pub fn current_state(&self) -> StateId {
        self.current
    }

    pub fn state_name(&self) -> &str {
        &self.machine.states[&self.current].name
    }

    pub fn state_entered_at(&self) -> Timestamp {
        self.state_entered_at
    }

    pub fn step(&mut self, event: &str) -> Result<StateId, ExecutionError> {
        self.step_with_guards(event, &BTreeSet::new())
    }

    /// Processes an event using the host's currently true named guards.
    /// Guard expressions are evaluated outside the model crate. If more than
    /// one eligible transition matches, declaration order determines priority.
    pub fn step_with_guards(
        &mut self,
        event: &str,
        active_guards: &BTreeSet<String>,
    ) -> Result<StateId, ExecutionError> {
        if event.trim().is_empty() {
            return Err(ExecutionError::Runtime(
                "state-machine event must not be empty".to_string(),
            ));
        }
        if let Some(next) = self
            .machine
            .transition_with_guards(self.current, event, active_guards)
            .map_err(state_machine_error)?
        {
            self.current = next;
            self.state_entered_at = self.current_time;
        }
        Ok(self.current)
    }

    /// Advances simulation time and fires the first eligible timed transition
    /// from the active state. Repeated calls at the same timestamp can fire
    /// subsequent transitions in a timed chain, one at a time.
    pub fn advance_to(&mut self, timestamp: Timestamp) -> Result<StateId, ExecutionError> {
        if timestamp < self.current_time {
            return Err(ExecutionError::Runtime(
                "state-machine time cannot move backwards".to_string(),
            ));
        }
        let mut transition: Option<(&TimedTransition, Timestamp)> = None;
        for candidate in self
            .machine
            .timed_transitions
            .iter()
            .filter(|candidate| candidate.from == self.current)
        {
            let deadline = self
                .state_entered_at
                .checked_add(candidate.after)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
            if deadline <= timestamp
                && transition
                    .as_ref()
                    .map_or(true, |(_, earliest)| deadline < *earliest)
            {
                transition = Some((candidate, deadline));
            }
        }
        if let Some((transition, deadline)) = transition {
            self.current = transition.to;
            self.state_entered_at = deadline;
        }
        self.current_time = timestamp;
        Ok(self.current)
    }

    pub fn reset(&mut self) -> Result<StateId, ExecutionError> {
        self.current = self
            .machine
            .initial_state
            .ok_or_else(|| state_machine_error(StateMachineError::MissingInitialState))?;
        self.current_time = Timestamp::ZERO;
        self.state_entered_at = Timestamp::ZERO;
        Ok(self.current)
    }
}

fn state_machine_error(error: StateMachineError) -> ExecutionError {
    ExecutionError::Runtime(format!("state-machine error: {error}"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Constant, ExecutionEngine, Scheduler, SchedulerConfig};
    use std::collections::BTreeSet;
    use xdl_model::{
        Model, ModelDocument, ModelLayout, PortDirection, SignalSpec, StateChartDocument,
        StateMachine,
    };

    fn duration(nanoseconds: i64) -> DurationNs {
        DurationNs::from_nanoseconds(nanoseconds)
    }

    fn machine() -> (StateMachine, StateId, StateId, StateId) {
        let mut machine = StateMachine::new("flight_readiness");
        let cold = machine.add_state("COLD").expect("cold");
        let ready = machine.add_state("READY").expect("ready");
        let flying = machine.add_state("FLYING").expect("flying");
        machine.set_initial_state(cold).expect("initial");
        machine
            .add_transition(cold, ready, "temperature_ready")
            .expect("temperature");
        machine
            .add_transition(ready, flying, "launch")
            .expect("launch");
        (machine, cold, ready, flying)
    }

    #[test]
    fn runtime_starts_and_steps_deterministically() {
        let (machine, cold, ready, flying) = machine();
        let mut runtime = StateMachineRuntime::new(machine).expect("runtime");
        assert_eq!(runtime.current_state(), cold);
        assert_eq!(runtime.step("temperature_ready"), Ok(ready));
        assert_eq!(runtime.step("launch"), Ok(flying));
    }

    #[test]
    fn unknown_event_holds_state() {
        let (machine, cold, _, _) = machine();
        let mut runtime = StateMachineRuntime::new(machine).expect("runtime");
        assert_eq!(runtime.step("not_defined"), Ok(cold));
    }

    #[test]
    fn empty_event_is_rejected() {
        let (machine, _, _, _) = machine();
        let mut runtime = StateMachineRuntime::new(machine).expect("runtime");
        assert!(runtime.step(" ").is_err());
    }

    #[test]
    fn reset_returns_to_initial_state() {
        let (machine, cold, ready, _) = machine();
        let mut runtime = StateMachineRuntime::new(machine).expect("runtime");
        runtime.step("temperature_ready").expect("ready");
        assert_eq!(runtime.reset(), Ok(cold));
        assert_eq!(runtime.current_state(), cold);
        assert_ne!(runtime.current_state(), ready);
    }

    #[test]
    fn runtime_steps_only_when_host_reports_guard_active() {
        let mut machine = StateMachine::new("guarded");
        let cold = machine.add_state("COLD").expect("cold");
        let ready = machine.add_state("READY").expect("ready");
        machine.set_initial_state(cold).expect("initial");
        machine
            .add_guarded_transition(cold, ready, "launch", "flight_ready")
            .expect("guarded launch");
        let mut runtime = StateMachineRuntime::new(machine).expect("runtime");

        assert_eq!(runtime.step("launch"), Ok(cold));
        let active_guards = BTreeSet::from(["flight_ready".to_string()]);
        assert_eq!(
            runtime.step_with_guards("launch", &active_guards),
            Ok(ready)
        );
    }

    #[test]
    fn scheduled_runtime_advances_timers_before_same_time_event() {
        let mut machine = StateMachine::new("scheduled_guarded");
        let cold = machine.add_state("COLD").expect("cold");
        let ready = machine.add_state("READY").expect("ready");
        let flying = machine.add_state("FLYING").expect("flying");
        machine.set_initial_state(cold).expect("initial");
        machine
            .add_timed_transition(cold, ready, duration(10))
            .expect("timed readiness");
        machine
            .add_guarded_transition(ready, flying, "launch", "ready_to_fly")
            .expect("guarded launch");

        let mut scheduler =
            Scheduler::new(SchedulerConfig::new(duration(10), 4).expect("scheduler config"));
        let mut runtime = ScheduledStateMachineRuntime::new(
            machine,
            &mut scheduler,
            "flight_state",
            duration(10),
            DurationNs::ZERO,
            0,
        )
        .expect("scheduled runtime");

        let initial_events = scheduler
            .advance_to(Timestamp::ZERO)
            .expect("initial release");
        assert_eq!(
            runtime.dispatch(&initial_events[0], None, &BTreeSet::new()),
            Ok(cold)
        );
        let due_events = scheduler
            .advance_to(Timestamp::from_nanoseconds(10))
            .expect("timed release");
        let active_guards = BTreeSet::from(["ready_to_fly".to_string()]);
        assert_eq!(
            runtime.dispatch(&due_events[0], Some("launch"), &active_guards),
            Ok(flying)
        );
        assert_eq!(runtime.task_id(), due_events[0].task);
    }

    #[test]
    fn scheduled_runtime_rejects_release_from_another_task() {
        let (machine, _, _, _) = machine();
        let mut scheduler =
            Scheduler::new(SchedulerConfig::new(duration(10), 4).expect("scheduler config"));
        let mut runtime = ScheduledStateMachineRuntime::new(
            machine,
            &mut scheduler,
            "state_machine",
            duration(10),
            DurationNs::ZERO,
            0,
        )
        .expect("scheduled runtime");
        let other_task = scheduler
            .add_periodic_task("other", duration(10), DurationNs::ZERO, 1)
            .expect("other task");
        let events = scheduler.advance_to(Timestamp::ZERO).expect("releases");
        let other_event = events
            .iter()
            .find(|event| event.task == other_task)
            .expect("other task release");

        assert_eq!(
            runtime.dispatch(other_event, None, &BTreeSet::new()),
            Err(ExecutionError::WrongStateMachineTask {
                expected: runtime.task_id(),
                actual: other_task,
            })
        );
    }

    #[test]
    fn state_chart_document_builds_an_executable_guarded_model_block() {
        let mut machine = StateMachine::new("flight_readiness");
        let cold = machine.add_state("COLD").expect("cold state");
        let ready = machine.add_state("READY").expect("ready state");
        machine.set_initial_state(cold).expect("initial state");
        machine
            .add_guarded_transition(
                cold,
                ready,
                "launch",
                "battery_temperature_c >= 10 && battery_soc > 0.3",
            )
            .expect("guarded transition");
        let chart = StateChartDocument::from_machine(machine).expect("chart document");

        let mut model = Model::new("state_chart_host");
        model.add_block("plant").expect("plant block");
        let layout = ModelLayout::from_model(&model).expect("model layout");
        let mut document = ModelDocument::new(model, layout).expect("model document");
        let block_id = chart
            .insert_runtime_block(&mut document, "readiness", 100_000_000)
            .expect("insert runtime block");
        let model = &document.model;
        let block = model.block(block_id).expect("inserted block");
        let port_named = |name: &str| {
            block
                .ports
                .values()
                .find(|port| port.name == name)
                .map(|port| port.id)
                .expect("expected state-chart port")
        };
        let event_port = port_named("event_code");
        let temperature_port = port_named("guard_battery_temperature_c");
        let soc_port = port_named("guard_battery_soc");
        let output_port = port_named("state_id");
        let mut runtime = StateMachineBlock::from_model(model, block_id)
            .expect("runtime block from state-chart parameters");

        let event_code = document.model.block(block_id).unwrap().parameters["event_codes"]["launch"]
            .as_i64()
            .expect("configured event code") as f64;
        let cold_temperature = runtime
            .step(
                Timestamp::from_nanoseconds(0),
                &BTreeMap::from([
                    (event_port, SignalValue::Scalar(event_code)),
                    (temperature_port, SignalValue::Scalar(9.0)),
                    (soc_port, SignalValue::Scalar(0.45)),
                ]),
            )
            .expect("temperature guard false");
        assert_eq!(
            cold_temperature[&output_port],
            SignalValue::Scalar(f64::from(cold.value()))
        );

        let low_charge = runtime
            .step(
                Timestamp::from_nanoseconds(0),
                &BTreeMap::from([
                    (event_port, SignalValue::Scalar(event_code)),
                    (temperature_port, SignalValue::Scalar(12.0)),
                    (soc_port, SignalValue::Scalar(0.2)),
                ]),
            )
            .expect("charge guard false");
        assert_eq!(
            low_charge[&output_port],
            SignalValue::Scalar(f64::from(cold.value()))
        );

        let transitioned = runtime
            .step(
                Timestamp::from_nanoseconds(0),
                &BTreeMap::from([
                    (event_port, SignalValue::Scalar(event_code)),
                    (temperature_port, SignalValue::Scalar(12.0)),
                    (soc_port, SignalValue::Scalar(0.45)),
                ]),
            )
            .expect("compound guard active step");
        assert_eq!(
            transitioned[&output_port],
            SignalValue::Scalar(f64::from(ready.value()))
        );
    }

    #[test]
    fn mars_readiness_model_executes_connected_sensor_guards_and_events() {
        let mut machine = StateMachine::new("mars_flight_readiness");
        let cold = machine.add_state("COLD").expect("cold state");
        let ready = machine.add_state("READY").expect("ready state");
        let flying = machine.add_state("FLYING").expect("flying state");
        machine.set_initial_state(cold).expect("initial state");
        machine
            .add_guarded_transition(
                cold,
                ready,
                "launch",
                "battery_temperature_c >= 10 && battery_soc > 0.3",
            )
            .expect("readiness transition");
        machine
            .add_guarded_transition(ready, flying, "takeoff", "battery_soc > 0.3")
            .expect("takeoff transition");
        let chart = StateChartDocument::from_machine(machine).expect("chart document");

        let mut model = Model::new("mars_readiness_vertical_slice");
        let (temperature_source, temperature_output) =
            add_constant_source(&mut model, "BatteryTemperature");
        let (soc_source, soc_output) = add_constant_source(&mut model, "BatterySoc");
        let layout = ModelLayout::from_model(&model).expect("model layout");
        let mut document = ModelDocument::new(model, layout).expect("model document");
        let chart_block = chart
            .insert_runtime_block(&mut document, "FlightReadiness", 100_000_000)
            .expect("insert chart block");
        let chart_model = &mut document.model;
        let chart_ports = chart_model
            .block(chart_block)
            .expect("chart block")
            .ports
            .values()
            .map(|port| (port.name.clone(), port.id))
            .collect::<BTreeMap<_, _>>();
        let temperature_input = chart_ports["guard_battery_temperature_c"];
        let soc_input = chart_ports["guard_battery_soc"];
        let event_input = chart_ports["event_code"];
        let state_output = chart_ports["state_id"];
        chart_model
            .connect(temperature_output, temperature_input)
            .expect("connect temperature signal");
        chart_model
            .connect(soc_output, soc_input)
            .expect("connect SOC signal");

        let scheduler = Scheduler::new(
            SchedulerConfig::new(duration(100_000_000), 16).expect("scheduler config"),
        );
        let mut engine = ExecutionEngine::new(document.model, scheduler).expect("engine");
        for (block, output, value, name) in [
            (temperature_source, temperature_output, 12.0, "temperature"),
            (soc_source, soc_output, 0.45, "SOC"),
        ] {
            let task = engine
                .scheduler
                .add_periodic_task(name, duration(100_000_000), DurationNs::ZERO, 0)
                .expect("source task");
            engine
                .register_task(
                    task,
                    block,
                    Box::new(Constant::new(output, value).expect("constant")),
                )
                .expect("register source");
        }
        engine
            .register_state_machine_block(chart_block)
            .expect("register readiness chart");
        let launch_code = engine.model.block(chart_block).expect("chart").parameters["event_codes"]
            ["launch"]
            .as_i64()
            .expect("launch event code") as f64;
        let takeoff_code = engine.model.block(chart_block).expect("chart").parameters["event_codes"]
            ["takeoff"]
            .as_i64()
            .expect("takeoff event code") as f64;

        engine
            .inject_signal(event_input, SignalValue::Scalar(launch_code))
            .expect("inject launch command");
        engine.advance_to(Timestamp::ZERO).expect("cold to ready");
        assert_eq!(
            engine.signal(state_output),
            Some(&SignalValue::Scalar(f64::from(ready.value())))
        );

        engine
            .inject_signal(event_input, SignalValue::Scalar(0.0))
            .expect("clear event input");
        engine
            .advance_to(Timestamp::from_nanoseconds(100_000_000))
            .expect("hold ready state");
        assert_eq!(
            engine.signal(state_output),
            Some(&SignalValue::Scalar(f64::from(ready.value())))
        );

        engine
            .inject_signal(event_input, SignalValue::Scalar(takeoff_code))
            .expect("inject takeoff command");
        engine
            .advance_to(Timestamp::from_nanoseconds(200_000_000))
            .expect("ready to flying");
        assert_eq!(
            engine.signal(state_output),
            Some(&SignalValue::Scalar(f64::from(flying.value())))
        );
    }

    fn add_constant_source(model: &mut Model, name: &str) -> (BlockId, PortId) {
        let block = model.add_block(name).expect("constant source block");
        let output = model
            .add_port(block, "output", PortDirection::Output, SignalSpec::scalar())
            .expect("constant output port");
        (block, output)
    }
}
