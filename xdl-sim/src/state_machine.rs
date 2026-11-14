// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic runtime for typed model state/event contracts.

use xdl_model::{StateId, StateMachine, StateMachineError};

use crate::ExecutionError;

pub struct StateMachineRuntime {
    machine: StateMachine,
    current: StateId,
}

impl StateMachineRuntime {
    pub fn new(machine: StateMachine) -> Result<Self, ExecutionError> {
        machine.validate().map_err(state_machine_error)?;
        let current = machine
            .initial_state
            .ok_or_else(|| state_machine_error(StateMachineError::MissingInitialState))?;
        Ok(Self { machine, current })
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
        }
        Ok(self.current)
    }

    pub fn reset(&mut self) -> Result<StateId, ExecutionError> {
        self.current = self
            .machine
            .initial_state
            .ok_or_else(|| state_machine_error(StateMachineError::MissingInitialState))?;
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
