// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic runtime for typed model state/event contracts.

use xdl_gnc_core::Timestamp;
use xdl_model::{StateId, StateMachine, StateMachineError, TimedTransition};

use crate::ExecutionError;

pub struct StateMachineRuntime {
    machine: StateMachine,
    current: StateId,
    current_time: Timestamp,
    state_entered_at: Timestamp,
}

impl StateMachineRuntime {
    pub fn new(machine: StateMachine) -> Result<Self, ExecutionError> {
        machine.validate().map_err(state_machine_error)?;
        let current = machine
            .initial_state
            .ok_or_else(|| state_machine_error(StateMachineError::MissingInitialState))?;
        Ok(Self {
            machine,
            current,
            current_time: Timestamp::ZERO,
            state_entered_at: Timestamp::ZERO,
        })
    }

    pub fn current_state(&self) -> StateId {
        self.current
    }

    pub fn step(&mut self, event: &str) -> Result<StateId, ExecutionError> {
        if event.trim().is_empty() {
            return Err(ExecutionError::Runtime(
                "state-machine event must not be empty".to_string(),
            ));
        }
        if let Some(next) = self
            .machine
            .transition(self.current, event)
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
    use xdl_model::StateMachine;

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
}
