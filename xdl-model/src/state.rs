// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic state/event contracts for model-based systems.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId(u32);

impl StateId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct State {
    pub id: StateId,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Transition {
    pub from: StateId,
    pub to: StateId,
    pub event: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StateMachineError {
    #[error("state name must not be empty")]
    EmptyStateName,
    #[error("event name must not be empty")]
    EmptyEventName,
    #[error("duplicate state name: {0}")]
    DuplicateStateName(String),
    #[error("unknown state: {0:?}")]
    UnknownState(StateId),
    #[error("state-machine initial state is not set")]
    MissingInitialState,
    #[error("duplicate transition from {from:?} on event {event}")]
    DuplicateTransition { from: StateId, event: String },
    #[error("state-machine must contain at least one state")]
    EmptyStateMachine,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine {
    pub name: String,
    pub states: BTreeMap<StateId, State>,
    pub transitions: Vec<Transition>,
    pub initial_state: Option<StateId>,
    next_state: u32,
}

impl StateMachine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            states: BTreeMap::new(),
            transitions: Vec::new(),
            initial_state: None,
            next_state: 0,
        }
    }

    pub fn add_state(&mut self, name: impl Into<String>) -> Result<StateId, StateMachineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(StateMachineError::EmptyStateName);
        }
        if self.states.values().any(|state| state.name == name) {
            return Err(StateMachineError::DuplicateStateName(name));
        }
        let id = StateId(self.next_state);
        self.next_state = self
            .next_state
            .checked_add(1)
            .ok_or(StateMachineError::UnknownState(id))?;
        self.states.insert(id, State { id, name });
        Ok(id)
    }

    pub fn set_initial_state(&mut self, state: StateId) -> Result<(), StateMachineError> {
        if !self.states.contains_key(&state) {
            return Err(StateMachineError::UnknownState(state));
        }
        self.initial_state = Some(state);
        Ok(())
    }

    pub fn add_transition(
        &mut self,
        from: StateId,
        to: StateId,
        event: impl Into<String>,
    ) -> Result<(), StateMachineError> {
        let event = event.into();
        if event.trim().is_empty() {
            return Err(StateMachineError::EmptyEventName);
        }
        if !self.states.contains_key(&from) {
            return Err(StateMachineError::UnknownState(from));
        }
        if !self.states.contains_key(&to) {
            return Err(StateMachineError::UnknownState(to));
        }
        if self
            .transitions
            .iter()
            .any(|transition| transition.from == from && transition.event == event)
        {
            return Err(StateMachineError::DuplicateTransition { from, event });
        }
        self.transitions.push(Transition { from, to, event });
        Ok(())
    }

    pub fn validate(&self) -> Result<(), StateMachineError> {
        if self.states.is_empty() {
            return Err(StateMachineError::EmptyStateMachine);
        }
        let initial = self
            .initial_state
            .ok_or(StateMachineError::MissingInitialState)?;
        if !self.states.contains_key(&initial) {
            return Err(StateMachineError::UnknownState(initial));
        }
        for transition in &self.transitions {
            if !self.states.contains_key(&transition.from) {
                return Err(StateMachineError::UnknownState(transition.from));
            }
            if !self.states.contains_key(&transition.to) {
                return Err(StateMachineError::UnknownState(transition.to));
            }
            if transition.event.trim().is_empty() {
                return Err(StateMachineError::EmptyEventName);
            }
        }
        Ok(())
    }

    pub fn transition(
        &self,
        current: StateId,
        event: &str,
    ) -> Result<Option<StateId>, StateMachineError> {
        self.validate()?;
        if !self.states.contains_key(&current) {
            return Err(StateMachineError::UnknownState(current));
        }
        Ok(self
            .transitions
            .iter()
            .find(|transition| transition.from == current && transition.event == event)
            .map(|transition| transition.to))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn readiness_machine() -> (StateMachine, StateId, StateId, StateId) {
        let mut machine = StateMachine::new("flight_readiness");
        let cold = machine.add_state("COLD").expect("cold");
        let ready = machine.add_state("READY").expect("ready");
        let flying = machine.add_state("FLYING").expect("flying");
        machine.set_initial_state(cold).expect("initial");
        machine
            .add_transition(cold, ready, "temperature_ready")
            .expect("cold to ready");
        machine
            .add_transition(ready, flying, "launch")
            .expect("ready to flying");
        machine
            .add_transition(flying, ready, "battery_low")
            .expect("flying to ready");
        (machine, cold, ready, flying)
    }

    #[test]
    fn readiness_machine_validates_and_transitions_deterministically() {
        let (machine, cold, ready, flying) = readiness_machine();
        machine.validate().expect("valid machine");
        assert_eq!(
            machine.transition(cold, "temperature_ready"),
            Ok(Some(ready))
        );
        assert_eq!(machine.transition(ready, "launch"), Ok(Some(flying)));
        assert_eq!(machine.transition(ready, "unknown"), Ok(None));
    }

    #[test]
    fn duplicate_event_from_a_state_is_rejected() {
        let (mut machine, cold, ready, _) = readiness_machine();
        assert_eq!(
            machine.add_transition(cold, ready, "temperature_ready"),
            Err(StateMachineError::DuplicateTransition {
                from: cold,
                event: "temperature_ready".to_string()
            })
        );
    }

    #[test]
    fn missing_initial_state_is_rejected() {
        let mut machine = StateMachine::new("invalid");
        machine.add_state("COLD").expect("state");
        assert_eq!(
            machine.validate(),
            Err(StateMachineError::MissingInitialState)
        );
    }

    #[test]
    fn duplicate_and_empty_state_names_are_rejected() {
        let mut machine = StateMachine::new("invalid");
        assert_eq!(
            machine.add_state(""),
            Err(StateMachineError::EmptyStateName)
        );
        machine.add_state("COLD").expect("state");
        assert_eq!(
            machine.add_state("COLD"),
            Err(StateMachineError::DuplicateStateName("COLD".to_string()))
        );
    }

    #[test]
    fn unknown_state_and_empty_event_are_rejected() {
        let mut machine = StateMachine::new("invalid");
        let cold = machine.add_state("COLD").expect("state");
        assert_eq!(
            machine.set_initial_state(StateId::new(99)),
            Err(StateMachineError::UnknownState(StateId::new(99)))
        );
        machine.set_initial_state(cold).expect("initial");
        assert_eq!(
            machine.add_transition(cold, StateId::new(99), "go"),
            Err(StateMachineError::UnknownState(StateId::new(99)))
        );
        assert_eq!(
            machine.add_transition(cold, cold, ""),
            Err(StateMachineError::EmptyEventName)
        );
    }
}
