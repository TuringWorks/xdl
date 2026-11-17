// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic runtime for typed model state/event contracts.

use std::collections::{BTreeMap, BTreeSet};
use xdl_gnc_core::{DurationNs, Timestamp};
use xdl_model::{
    BlockId, Model, PortDirection, PortId, SignalShape, StateId, StateMachine, StateMachineError,
    TimedTransition,
};

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
}

impl StateMachineBlock {
    /// Builds a state-machine block from model-document parameters.
    /// Required parameters: `state_machine`, `event_input`, `state_output`,
    /// and `event_codes`; `guard_inputs` maps guard names to input port names.
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
        if guard_inputs
            .keys()
            .any(|guard| guard.trim().is_empty() || !known_guards.contains(guard.as_str()))
            || known_guards
                .iter()
                .any(|guard| !guard_inputs.contains_key(*guard))
        {
            return Err(ExecutionError::Runtime(
                "state-machine guard inputs must map every declared guard exactly by name"
                    .to_string(),
            ));
        }
        if event_input == output || guard_inputs.values().any(|port| *port == output) {
            return Err(ExecutionError::Runtime(
                "state-machine output port must differ from all input ports".to_string(),
            ));
        }
        let runtime = StateMachineRuntime::new(machine)?;
        Ok(Self {
            runtime,
            event_input,
            guard_inputs,
            output,
            event_codes,
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
        let active_guards = self
            .guard_inputs
            .iter()
            .map(|(guard, port)| {
                let value = scalar_input(inputs, *port, "state-machine guard")?;
                Ok((value != 0.0).then(|| guard.clone()))
            })
            .collect::<Result<BTreeSet<_>, ExecutionError>>()?
            .into_iter()
            .flatten()
            .collect();

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
    use crate::SchedulerConfig;
    use std::collections::BTreeSet;
    use xdl_model::StateMachine;

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
}
