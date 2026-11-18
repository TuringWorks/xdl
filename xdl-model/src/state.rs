// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic state/event contracts for model-based systems.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use xdl_gnc_core::DurationNs;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct StateId(u32);

impl StateId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u32 {
        self.0
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guard: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimedTransition {
    pub from: StateId,
    pub to: StateId,
    pub after: DurationNs,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StateMachineError {
    #[error("state name must not be empty")]
    EmptyStateName,
    #[error("event name must not be empty")]
    EmptyEventName,
    #[error("guard name must not be empty")]
    EmptyGuardName,
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
    #[error("timed transition delay must be positive")]
    InvalidTransitionDelay,
    #[error("duplicate timed transition from {0:?} after {1} ns")]
    DuplicateTimedTransition(StateId, i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateMachine {
    pub name: String,
    pub states: BTreeMap<StateId, State>,
    pub transitions: Vec<Transition>,
    #[serde(default)]
    pub timed_transitions: Vec<TimedTransition>,
    pub initial_state: Option<StateId>,
    #[serde(default)]
    next_state: u32,
}

impl StateMachine {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            states: BTreeMap::new(),
            transitions: Vec::new(),
            timed_transitions: Vec::new(),
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
        if self.transitions.iter().any(|transition| {
            transition.from == from && transition.event == event && transition.guard.is_none()
        }) {
            return Err(StateMachineError::DuplicateTransition { from, event });
        }
        self.transitions.push(Transition {
            from,
            to,
            event,
            guard: None,
        });
        Ok(())
    }

    /// Adds an event transition eligible only while the named guard is true.
    /// Guard evaluation belongs to the caller/runtime host. When multiple
    /// guards for the same event are true, insertion order defines priority.
    pub fn add_guarded_transition(
        &mut self,
        from: StateId,
        to: StateId,
        event: impl Into<String>,
        guard: impl Into<String>,
    ) -> Result<(), StateMachineError> {
        let event = event.into();
        let guard = guard.into();
        if event.trim().is_empty() {
            return Err(StateMachineError::EmptyEventName);
        }
        if guard.trim().is_empty() {
            return Err(StateMachineError::EmptyGuardName);
        }
        if !self.states.contains_key(&from) {
            return Err(StateMachineError::UnknownState(from));
        }
        if !self.states.contains_key(&to) {
            return Err(StateMachineError::UnknownState(to));
        }
        if self.transitions.iter().any(|transition| {
            transition.from == from
                && transition.event == event
                && transition.guard.as_deref() == Some(guard.as_str())
        }) {
            return Err(StateMachineError::DuplicateTransition { from, event });
        }
        self.transitions.push(Transition {
            from,
            to,
            event,
            guard: Some(guard),
        });
        Ok(())
    }

    /// Adds a transition that becomes eligible after the source state has
    /// remained active for `after`. Delays are strictly positive so timed
    /// transition cycles cannot execute infinitely at one simulation time.
    pub fn add_timed_transition(
        &mut self,
        from: StateId,
        to: StateId,
        after: DurationNs,
    ) -> Result<(), StateMachineError> {
        if !after.is_positive() {
            return Err(StateMachineError::InvalidTransitionDelay);
        }
        if !self.states.contains_key(&from) {
            return Err(StateMachineError::UnknownState(from));
        }
        if !self.states.contains_key(&to) {
            return Err(StateMachineError::UnknownState(to));
        }
        if self
            .timed_transitions
            .iter()
            .any(|transition| transition.from == from && transition.after == after)
        {
            return Err(StateMachineError::DuplicateTimedTransition(
                from,
                after.as_nanoseconds(),
            ));
        }
        self.timed_transitions
            .push(TimedTransition { from, to, after });
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
        let mut transition_keys = BTreeSet::new();
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
            if transition
                .guard
                .as_deref()
                .map_or(false, |guard| guard.trim().is_empty())
            {
                return Err(StateMachineError::EmptyGuardName);
            }
            if !transition_keys.insert((
                transition.from,
                transition.event.as_str(),
                transition.guard.as_deref(),
            )) {
                return Err(StateMachineError::DuplicateTransition {
                    from: transition.from,
                    event: transition.event.clone(),
                });
            }
        }
        for transition in &self.timed_transitions {
            if !self.states.contains_key(&transition.from) {
                return Err(StateMachineError::UnknownState(transition.from));
            }
            if !self.states.contains_key(&transition.to) {
                return Err(StateMachineError::UnknownState(transition.to));
            }
            if !transition.after.is_positive() {
                return Err(StateMachineError::InvalidTransitionDelay);
            }
        }
        Ok(())
    }

    pub fn transition(
        &self,
        current: StateId,
        event: &str,
    ) -> Result<Option<StateId>, StateMachineError> {
        self.transition_with_guards(current, event, &BTreeSet::new())
    }

    pub fn transition_with_guards(
        &self,
        current: StateId,
        event: &str,
        active_guards: &BTreeSet<String>,
    ) -> Result<Option<StateId>, StateMachineError> {
        self.validate()?;
        if !self.states.contains_key(&current) {
            return Err(StateMachineError::UnknownState(current));
        }
        Ok(self
            .transitions
            .iter()
            .find(|transition| {
                transition.from == current
                    && transition.event == event
                    && transition
                        .guard
                        .as_ref()
                        .map_or(true, |guard| active_guards.contains(guard))
            })
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

    #[test]
    fn guarded_transitions_require_active_guard_and_keep_declared_priority() {
        let mut machine = StateMachine::new("guarded");
        let idle = machine.add_state("IDLE").expect("idle");
        let warm = machine.add_state("WARM").expect("warm");
        let hot = machine.add_state("HOT").expect("hot");
        machine.set_initial_state(idle).expect("initial");
        machine
            .add_guarded_transition(idle, warm, "tick", "temperature_ready")
            .expect("temperature guard");
        machine
            .add_guarded_transition(idle, hot, "tick", "pressure_ready")
            .expect("pressure guard");
        machine.validate().expect("valid guarded machine");

        assert_eq!(machine.transition(idle, "tick"), Ok(None));
        let active = BTreeSet::from(["pressure_ready".to_string()]);
        assert_eq!(
            machine.transition_with_guards(idle, "tick", &active),
            Ok(Some(hot))
        );
        let both_active = BTreeSet::from([
            "temperature_ready".to_string(),
            "pressure_ready".to_string(),
        ]);
        assert_eq!(
            machine.transition_with_guards(idle, "tick", &both_active),
            Ok(Some(warm))
        );
    }

    #[test]
    fn duplicate_guarded_transition_and_empty_guard_are_rejected() {
        let mut machine = StateMachine::new("guarded");
        let idle = machine.add_state("IDLE").expect("idle");
        let ready = machine.add_state("READY").expect("ready");
        machine.set_initial_state(idle).expect("initial");
        assert_eq!(
            machine.add_guarded_transition(idle, ready, "tick", "  "),
            Err(StateMachineError::EmptyGuardName)
        );
        machine
            .add_guarded_transition(idle, ready, "tick", "safe")
            .expect("first transition");
        assert_eq!(
            machine.add_guarded_transition(idle, ready, "tick", "safe"),
            Err(StateMachineError::DuplicateTransition {
                from: idle,
                event: "tick".to_string()
            })
        );
    }

    #[test]
    fn missing_guard_field_deserializes_as_unconditional() -> Result<(), serde_json::Error> {
        let transition: Transition =
            serde_json::from_str(r#"{"from":0,"to":1,"event":"advance"}"#)?;
        assert_eq!(transition.guard, None);
        let serialized = serde_json::to_value(transition)?;
        assert!(!serialized
            .as_object()
            .is_some_and(|object| object.contains_key("guard")));
        Ok(())
    }
}
