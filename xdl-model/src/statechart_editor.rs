// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Serializable geometry and editing operations for state/event diagrams.

use crate::{StateId, StateMachine, StateMachineError};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use thiserror::Error;

pub const STATE_CHART_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub enum TransitionKey {
    Event(usize),
    Timed(usize),
}

impl From<TransitionKey> for String {
    fn from(key: TransitionKey) -> Self {
        match key {
            TransitionKey::Event(index) => format!("event:{index}"),
            TransitionKey::Timed(index) => format!("timed:{index}"),
        }
    }
}

impl TryFrom<String> for TransitionKey {
    type Error = String;

    fn try_from(key: String) -> Result<Self, Self::Error> {
        if let Some(index) = key.strip_prefix("event:") {
            return index
                .parse()
                .map(TransitionKey::Event)
                .map_err(|_| format!("invalid event transition key {key:?}"));
        }
        if let Some(index) = key.strip_prefix("timed:") {
            return index
                .parse()
                .map(TransitionKey::Timed)
                .map_err(|_| format!("invalid timed transition key {key:?}"));
        }
        Err(format!("invalid state-chart transition key {key:?}"))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StateNodeLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl StateNodeLayout {
    pub const fn new(x: f64, y: f64, width: f64, height: f64) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    fn is_valid(self) -> bool {
        self.x.is_finite()
            && self.y.is_finite()
            && self.width.is_finite()
            && self.height.is_finite()
            && self.width > 0.0
            && self.height > 0.0
    }

    fn center(self) -> (f64, f64) {
        (self.x + self.width / 2.0, self.y + self.height / 2.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct TransitionLabelLayout {
    pub x: f64,
    pub y: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct StateChartLayout {
    pub states: BTreeMap<StateId, StateNodeLayout>,
    pub transitions: BTreeMap<TransitionKey, TransitionLabelLayout>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateChartDocument {
    #[serde(default = "state_chart_schema_version")]
    pub schema_version: u32,
    pub machine: StateMachine,
    pub layout: StateChartLayout,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum StateChartEditorError {
    #[error("state-machine is invalid: {0}")]
    InvalidMachine(#[from] StateMachineError),
    #[error("unsupported state-chart schema version: {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("state-chart layout is missing state {0:?}")]
    MissingState(StateId),
    #[error("state-chart layout has unknown state {0:?}")]
    UnknownState(StateId),
    #[error("state-chart layout for state {0:?} is invalid")]
    InvalidStateLayout(StateId),
    #[error("state-chart layout is missing transition {0:?}")]
    MissingTransition(TransitionKey),
    #[error("state-chart layout has unknown transition {0:?}")]
    UnknownTransition(TransitionKey),
    #[error("state-chart transition label geometry is invalid for {0:?}")]
    InvalidTransitionLabel(TransitionKey),
    #[error("state-chart serialization failed: {0}")]
    Serialization(String),
    #[error("state-chart deserialization failed: {0}")]
    Deserialization(String),
    #[error("invalid state-chart interaction: {0}")]
    InvalidInteraction(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StateChartHitTarget {
    State(StateId),
    Transition(TransitionKey),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum StateChartEditorEvent {
    Select(StateChartHitTarget),
    ClearSelection,
    MoveSelected { dx: f64, dy: f64 },
}

#[derive(Debug, Clone, PartialEq)]
pub struct StateChartEditor {
    pub document: StateChartDocument,
    pub selection: Option<StateChartHitTarget>,
}

impl StateChartDocument {
    pub fn from_machine(machine: StateMachine) -> Result<Self, StateChartEditorError> {
        machine.validate()?;
        let states = machine
            .states
            .keys()
            .enumerate()
            .map(|(index, state)| {
                (
                    *state,
                    StateNodeLayout::new(
                        80.0 + (index % 3) as f64 * 240.0,
                        80.0 + (index / 3) as f64 * 160.0,
                        150.0,
                        64.0,
                    ),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let transitions = machine
            .transitions
            .iter()
            .enumerate()
            .map(|(index, transition)| {
                let from = states[&transition.from].center();
                let to = states[&transition.to].center();
                (
                    TransitionKey::Event(index),
                    TransitionLabelLayout {
                        x: (from.0 + to.0) / 2.0,
                        y: if transition.from == transition.to {
                            states[&transition.from].y - 18.0
                        } else {
                            (from.1 + to.1) / 2.0 - 14.0
                        },
                    },
                )
            })
            .chain(
                machine
                    .timed_transitions
                    .iter()
                    .enumerate()
                    .map(|(index, transition)| {
                        let from = states[&transition.from].center();
                        let to = states[&transition.to].center();
                        (
                            TransitionKey::Timed(index),
                            TransitionLabelLayout {
                                x: (from.0 + to.0) / 2.0,
                                y: if transition.from == transition.to {
                                    states[&transition.from].y - 4.0
                                } else {
                                    (from.1 + to.1) / 2.0 + 14.0
                                },
                            },
                        )
                    }),
            )
            .collect();
        let document = Self {
            schema_version: STATE_CHART_SCHEMA_VERSION,
            machine,
            layout: StateChartLayout {
                states,
                transitions,
            },
        };
        document.validate()?;
        Ok(document)
    }

    pub fn validate(&self) -> Result<(), StateChartEditorError> {
        if self.schema_version != STATE_CHART_SCHEMA_VERSION {
            return Err(StateChartEditorError::UnsupportedSchemaVersion(
                self.schema_version,
            ));
        }
        self.machine.validate()?;
        for state in self.machine.states.keys() {
            let layout = self
                .layout
                .states
                .get(state)
                .ok_or(StateChartEditorError::MissingState(*state))?;
            if !layout.is_valid() {
                return Err(StateChartEditorError::InvalidStateLayout(*state));
            }
        }
        if let Some(state) = self
            .layout
            .states
            .keys()
            .find(|state| !self.machine.states.contains_key(state))
        {
            return Err(StateChartEditorError::UnknownState(*state));
        }
        let expected_transitions: BTreeSet<_> = (0..self.machine.transitions.len())
            .map(TransitionKey::Event)
            .chain((0..self.machine.timed_transitions.len()).map(TransitionKey::Timed))
            .collect();
        for transition in &expected_transitions {
            let layout = self
                .layout
                .transitions
                .get(transition)
                .ok_or(StateChartEditorError::MissingTransition(*transition))?;
            if !layout.x.is_finite() || !layout.y.is_finite() {
                return Err(StateChartEditorError::InvalidTransitionLabel(*transition));
            }
        }
        if let Some(transition) = self
            .layout
            .transitions
            .keys()
            .find(|transition| !expected_transitions.contains(transition))
        {
            return Err(StateChartEditorError::UnknownTransition(*transition));
        }
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, StateChartEditorError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| StateChartEditorError::Serialization(error.to_string()))
    }

    /// Converts the chart into parameters accepted by `xdl-sim`'s
    /// `StateMachineBlock`. Event codes are assigned deterministically in
    /// sorted event-name order; guard inputs use `guard_<name>` port names.
    pub fn runtime_block_parameters(
        &self,
        period_ns: i64,
    ) -> Result<serde_json::Value, StateChartEditorError> {
        self.validate()?;
        if period_ns <= 0 {
            return Err(StateChartEditorError::InvalidInteraction(
                "state-machine sample period must be positive".to_string(),
            ));
        }
        let event_names: BTreeSet<_> = self
            .machine
            .transitions
            .iter()
            .map(|transition| transition.event.as_str())
            .collect();
        let event_codes = event_names
            .into_iter()
            .enumerate()
            .map(|(index, event)| {
                let code = i64::try_from(index + 1).map_err(|_| {
                    StateChartEditorError::Serialization(
                        "too many distinct state-machine events".to_string(),
                    )
                })?;
                Ok((event.to_string(), code))
            })
            .collect::<Result<BTreeMap<_, _>, StateChartEditorError>>()?;
        let guard_inputs: BTreeMap<_, _> = self
            .machine
            .transitions
            .iter()
            .filter_map(|transition| transition.guard.as_ref())
            .map(|guard| (guard.clone(), format!("guard_{guard}")))
            .collect();
        let machine = serde_json::to_value(&self.machine)
            .map_err(|error| StateChartEditorError::Serialization(error.to_string()))?;
        Ok(serde_json::json!({
            "state_machine": machine,
            "event_input": "event_code",
            "state_output": "state_id",
            "event_codes": event_codes,
            "guard_inputs": guard_inputs,
            "period_ns": period_ns,
            "phase_ns": 0,
            "priority": 0
        }))
    }

    pub fn from_json(json: &str) -> Result<Self, StateChartEditorError> {
        let document: Self = serde_json::from_str(json)
            .map_err(|error| StateChartEditorError::Deserialization(error.to_string()))?;
        document.validate()?;
        Ok(document)
    }

    pub fn move_state(
        &mut self,
        state: StateId,
        x: f64,
        y: f64,
    ) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        let layout = candidate
            .layout
            .states
            .get_mut(&state)
            .ok_or(StateChartEditorError::MissingState(state))?;
        layout.x = x;
        layout.y = y;
        candidate.validate()?;
        self.layout = candidate.layout;
        Ok(())
    }

    pub fn move_transition_label(
        &mut self,
        transition: TransitionKey,
        x: f64,
        y: f64,
    ) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        let layout = candidate
            .layout
            .transitions
            .get_mut(&transition)
            .ok_or(StateChartEditorError::MissingTransition(transition))?;
        layout.x = x;
        layout.y = y;
        candidate.validate()?;
        self.layout = candidate.layout;
        Ok(())
    }

    pub fn add_state(&mut self, name: impl Into<String>) -> Result<StateId, StateChartEditorError> {
        let mut candidate = self.clone();
        let state = candidate.machine.add_state(name)?;
        let mut index = 0usize;
        let geometry = loop {
            let geometry = StateNodeLayout::new(
                80.0 + (index % 3) as f64 * 240.0,
                80.0 + (index / 3) as f64 * 160.0,
                150.0,
                64.0,
            );
            let overlaps = candidate.layout.states.values().any(|existing| {
                geometry.x < existing.x + existing.width
                    && geometry.x + geometry.width > existing.x
                    && geometry.y < existing.y + existing.height
                    && geometry.y + geometry.height > existing.y
            });
            if !overlaps {
                break geometry;
            }
            index += 1;
        };
        candidate.layout.states.insert(state, geometry);
        candidate.validate()?;
        self.machine = candidate.machine;
        self.layout = candidate.layout;
        Ok(state)
    }

    pub fn add_event_transition(
        &mut self,
        from: StateId,
        to: StateId,
        event: impl Into<String>,
        guard: Option<String>,
    ) -> Result<TransitionKey, StateChartEditorError> {
        let mut candidate = self.clone();
        match guard {
            Some(guard) => candidate
                .machine
                .add_guarded_transition(from, to, event, guard)?,
            None => candidate.machine.add_transition(from, to, event)?,
        }
        let index = candidate.machine.transitions.len() - 1;
        let from_center = candidate.layout.states[&from].center();
        let to_center = candidate.layout.states[&to].center();
        candidate.layout.transitions.insert(
            TransitionKey::Event(index),
            TransitionLabelLayout {
                x: (from_center.0 + to_center.0) / 2.0,
                y: (from_center.1 + to_center.1) / 2.0,
            },
        );
        candidate.validate()?;
        self.machine = candidate.machine;
        self.layout = candidate.layout;
        Ok(TransitionKey::Event(index))
    }

    pub fn add_timed_transition(
        &mut self,
        from: StateId,
        to: StateId,
        after: xdl_gnc_core::DurationNs,
    ) -> Result<TransitionKey, StateChartEditorError> {
        let mut candidate = self.clone();
        candidate.machine.add_timed_transition(from, to, after)?;
        let index = candidate.machine.timed_transitions.len() - 1;
        let from_center = candidate.layout.states[&from].center();
        let to_center = candidate.layout.states[&to].center();
        candidate.layout.transitions.insert(
            TransitionKey::Timed(index),
            TransitionLabelLayout {
                x: (from_center.0 + to_center.0) / 2.0,
                y: (from_center.1 + to_center.1) / 2.0,
            },
        );
        candidate.validate()?;
        self.machine = candidate.machine;
        self.layout = candidate.layout;
        Ok(TransitionKey::Timed(index))
    }

    pub fn edit_event_transition(
        &mut self,
        index: usize,
        from: StateId,
        to: StateId,
        event: impl Into<String>,
        guard: Option<String>,
    ) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        let transition = candidate.machine.transitions.get_mut(index).ok_or(
            StateChartEditorError::UnknownTransition(TransitionKey::Event(index)),
        )?;
        transition.from = from;
        transition.to = to;
        transition.event = event.into();
        transition.guard = guard;
        candidate.validate()?;
        self.machine = candidate.machine;
        Ok(())
    }

    pub fn edit_timed_transition(
        &mut self,
        index: usize,
        from: StateId,
        to: StateId,
        after: xdl_gnc_core::DurationNs,
    ) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        let transition = candidate.machine.timed_transitions.get_mut(index).ok_or(
            StateChartEditorError::UnknownTransition(TransitionKey::Timed(index)),
        )?;
        transition.from = from;
        transition.to = to;
        transition.after = after;
        candidate.validate()?;
        self.machine = candidate.machine;
        Ok(())
    }

    pub fn remove_state(&mut self, state: StateId) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        if !candidate.machine.states.contains_key(&state) {
            return Err(StateChartEditorError::UnknownState(state));
        }
        if candidate.machine.states.len() == 1 {
            return Err(StateChartEditorError::InvalidInteraction(
                "a state chart must retain at least one state".to_string(),
            ));
        }
        candidate.machine.states.remove(&state);
        candidate.layout.states.remove(&state);
        if candidate.machine.initial_state == Some(state) {
            candidate.machine.initial_state = candidate.machine.states.keys().next().copied();
        }
        let old_event_layouts = std::mem::take(&mut candidate.layout.transitions);
        let retained_events: Vec<_> = candidate
            .machine
            .transitions
            .iter()
            .enumerate()
            .filter(|(_, transition)| transition.from != state && transition.to != state)
            .map(|(index, transition)| (index, transition.clone()))
            .collect();
        candidate.machine.transitions = retained_events
            .iter()
            .map(|(_, transition)| transition.clone())
            .collect();
        let mut transition_layouts = BTreeMap::new();
        for (new_index, (old_index, _)) in retained_events.iter().enumerate() {
            if let Some(layout) = old_event_layouts.get(&TransitionKey::Event(*old_index)) {
                transition_layouts.insert(TransitionKey::Event(new_index), *layout);
            }
        }
        let retained_timed: Vec<_> = candidate
            .machine
            .timed_transitions
            .iter()
            .enumerate()
            .filter(|(_, transition)| transition.from != state && transition.to != state)
            .map(|(index, transition)| (index, transition.clone()))
            .collect();
        candidate.machine.timed_transitions = retained_timed
            .iter()
            .map(|(_, transition)| transition.clone())
            .collect();
        for (new_index, (old_index, _)) in retained_timed.iter().enumerate() {
            if let Some(layout) = old_event_layouts.get(&TransitionKey::Timed(*old_index)) {
                transition_layouts.insert(TransitionKey::Timed(new_index), *layout);
            }
        }
        candidate.layout.transitions = transition_layouts;
        candidate.validate()?;
        self.machine = candidate.machine;
        self.layout = candidate.layout;
        Ok(())
    }

    pub fn remove_transition(
        &mut self,
        transition: TransitionKey,
    ) -> Result<(), StateChartEditorError> {
        let mut candidate = self.clone();
        let old_layouts = std::mem::take(&mut candidate.layout.transitions);
        let mut transition_layouts = BTreeMap::new();
        match transition {
            TransitionKey::Event(remove_index) => {
                if remove_index >= candidate.machine.transitions.len() {
                    return Err(StateChartEditorError::UnknownTransition(transition));
                }
                candidate.machine.transitions.remove(remove_index);
                for (key, layout) in old_layouts {
                    match key {
                        TransitionKey::Event(index) if index == remove_index => {}
                        TransitionKey::Event(index) if index > remove_index => {
                            transition_layouts.insert(TransitionKey::Event(index - 1), layout);
                        }
                        _ => {
                            transition_layouts.insert(key, layout);
                        }
                    }
                }
            }
            TransitionKey::Timed(remove_index) => {
                if remove_index >= candidate.machine.timed_transitions.len() {
                    return Err(StateChartEditorError::UnknownTransition(transition));
                }
                candidate.machine.timed_transitions.remove(remove_index);
                for (key, layout) in old_layouts {
                    match key {
                        TransitionKey::Timed(index) if index == remove_index => {}
                        TransitionKey::Timed(index) if index > remove_index => {
                            transition_layouts.insert(TransitionKey::Timed(index - 1), layout);
                        }
                        _ => {
                            transition_layouts.insert(key, layout);
                        }
                    }
                }
            }
        }
        candidate.layout.transitions = transition_layouts;
        candidate.validate()?;
        self.machine = candidate.machine;
        self.layout = candidate.layout;
        Ok(())
    }

    pub fn hit_test(
        &self,
        x: f64,
        y: f64,
        tolerance: f64,
    ) -> Result<Option<StateChartHitTarget>, StateChartEditorError> {
        self.validate()?;
        if !x.is_finite() || !y.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
            return Ok(None);
        }
        for (state, layout) in &self.layout.states {
            if (layout.x..=layout.x + layout.width).contains(&x)
                && (layout.y..=layout.y + layout.height).contains(&y)
            {
                return Ok(Some(StateChartHitTarget::State(*state)));
            }
        }
        let tolerance_squared = tolerance * tolerance;
        for (index, transition) in self.machine.transitions.iter().enumerate() {
            if transition_distance_squared(&self.layout, transition.from, transition.to, (x, y))
                <= tolerance_squared
            {
                return Ok(Some(StateChartHitTarget::Transition(TransitionKey::Event(
                    index,
                ))));
            }
        }
        for (index, transition) in self.machine.timed_transitions.iter().enumerate() {
            if transition_distance_squared(&self.layout, transition.from, transition.to, (x, y))
                <= tolerance_squared
            {
                return Ok(Some(StateChartHitTarget::Transition(TransitionKey::Timed(
                    index,
                ))));
            }
        }
        Ok(None)
    }

    pub fn to_svg(&self) -> Result<String, StateChartEditorError> {
        self.validate()?;
        let (min_x, min_y, max_x, max_y) = self.layout.states.values().fold(
            (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
            |(min_x, min_y, max_x, max_y), state| {
                (
                    min_x.min(state.x),
                    min_y.min(state.y),
                    max_x.max(state.x + state.width),
                    max_y.max(state.y + state.height),
                )
            },
        );
        let (min_x, min_y, max_x, max_y) = self.layout.transitions.values().fold(
            (min_x, min_y, max_x, max_y),
            |(min_x, min_y, max_x, max_y), label| {
                (
                    min_x.min(label.x - 96.0),
                    min_y.min(label.y - 20.0),
                    max_x.max(label.x + 96.0),
                    max_y.max(label.y + 8.0),
                )
            },
        );
        let margin = 32.0;
        let mut svg = String::new();
        let _ = writeln!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" role=\"img\">",
            min_x - margin,
            min_y - margin,
            (max_x - min_x + 2.0 * margin).max(1.0),
            (max_y - min_y + 2.0 * margin).max(1.0)
        );
        let _ = writeln!(svg, "<defs><marker id=\"state-arrow\" markerWidth=\"8\" markerHeight=\"8\" refX=\"7\" refY=\"4\" orient=\"auto\"><path d=\"M0,0 L8,4 L0,8 z\" fill=\"currentColor\" /></marker></defs>");
        let _ = writeln!(
            svg,
            "<g class=\"transitions\" fill=\"none\" stroke=\"currentColor\">"
        );
        for transition in &self.machine.transitions {
            render_transition(&mut svg, &self.layout, transition.from, transition.to)?;
        }
        for transition in &self.machine.timed_transitions {
            render_transition(&mut svg, &self.layout, transition.from, transition.to)?;
        }
        let _ = writeln!(
            svg,
            "</g><g class=\"states\" fill=\"none\" stroke=\"currentColor\">"
        );
        for state in self.machine.states.values() {
            let geometry = self.layout.states[&state.id];
            let class = if Some(state.id) == self.machine.initial_state {
                "state initial"
            } else {
                "state"
            };
            let _ = writeln!(
                svg,
                "<rect class=\"{class}\" x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"12\" />",
                geometry.x, geometry.y, geometry.width, geometry.height
            );
            let _ = writeln!(svg, "<text x=\"{}\" y=\"{}\" text-anchor=\"middle\" fill=\"currentColor\" stroke=\"none\">{}</text>", geometry.x + geometry.width / 2.0, geometry.y + geometry.height / 2.0 + 5.0, escape_xml(&state.name));
        }
        let _ = writeln!(
            svg,
            "</g><g class=\"transition-labels\" fill=\"currentColor\" stroke=\"none\">"
        );
        for (index, transition) in self.machine.transitions.iter().enumerate() {
            let label = match &transition.guard {
                Some(guard) => format!("{} [{}]", transition.event, guard),
                None => transition.event.clone(),
            };
            render_label(&mut svg, &self.layout, TransitionKey::Event(index), &label)?;
        }
        for (index, transition) in self.machine.timed_transitions.iter().enumerate() {
            render_label(
                &mut svg,
                &self.layout,
                TransitionKey::Timed(index),
                &format!("after {} ns", transition.after.as_nanoseconds()),
            )?;
        }
        let _ = writeln!(svg, "</g></svg>");
        Ok(svg)
    }
}

impl StateChartEditor {
    pub fn new(document: StateChartDocument) -> Result<Self, StateChartEditorError> {
        document.validate()?;
        Ok(Self {
            document,
            selection: None,
        })
    }

    pub fn select_at(
        &mut self,
        x: f64,
        y: f64,
        tolerance: f64,
    ) -> Result<Option<StateChartHitTarget>, StateChartEditorError> {
        let selection = self.document.hit_test(x, y, tolerance)?;
        self.selection = selection;
        Ok(selection)
    }

    pub fn apply(&mut self, event: StateChartEditorEvent) -> Result<(), StateChartEditorError> {
        match event {
            StateChartEditorEvent::Select(target) => {
                match target {
                    StateChartHitTarget::State(state)
                        if !self.document.layout.states.contains_key(&state) =>
                    {
                        return Err(StateChartEditorError::MissingState(state));
                    }
                    StateChartHitTarget::Transition(transition)
                        if !self.document.layout.transitions.contains_key(&transition) =>
                    {
                        return Err(StateChartEditorError::MissingTransition(transition));
                    }
                    _ => {}
                }
                self.selection = Some(target);
                Ok(())
            }
            StateChartEditorEvent::ClearSelection => {
                self.selection = None;
                Ok(())
            }
            StateChartEditorEvent::MoveSelected { dx, dy } => {
                if !dx.is_finite() || !dy.is_finite() {
                    return Err(StateChartEditorError::InvalidInteraction(
                        "movement delta must be finite".to_string(),
                    ));
                }
                match self.selection {
                    Some(StateChartHitTarget::State(state)) => {
                        let geometry = *self
                            .document
                            .layout
                            .states
                            .get(&state)
                            .ok_or(StateChartEditorError::MissingState(state))?;
                        self.document
                            .move_state(state, geometry.x + dx, geometry.y + dy)
                    }
                    Some(StateChartHitTarget::Transition(transition)) => {
                        let label = *self
                            .document
                            .layout
                            .transitions
                            .get(&transition)
                            .ok_or(StateChartEditorError::MissingTransition(transition))?;
                        self.document
                            .move_transition_label(transition, label.x + dx, label.y + dy)
                    }
                    None => Err(StateChartEditorError::InvalidInteraction(
                        "select a state or transition before moving".to_string(),
                    )),
                }
            }
        }
    }
}

fn render_transition(
    svg: &mut String,
    layout: &StateChartLayout,
    from: StateId,
    to: StateId,
) -> Result<(), StateChartEditorError> {
    let from_layout = *layout
        .states
        .get(&from)
        .ok_or(StateChartEditorError::MissingState(from))?;
    let to_layout = *layout
        .states
        .get(&to)
        .ok_or(StateChartEditorError::MissingState(to))?;
    let start = from_layout.center();
    let end = to_layout.center();
    if from == to {
        let left = start.0 - from_layout.width * 0.2;
        let right = start.0 + from_layout.width * 0.2;
        let top = from_layout.y;
        let _ = writeln!(
            svg,
            "<path d=\"M {} {} C {} {}, {} {}, {} {}\" marker-end=\"url(#state-arrow)\" />",
            right,
            top,
            right + 56.0,
            top - 72.0,
            left - 56.0,
            top - 72.0,
            left,
            top
        );
    } else {
        let direction = (end.0 - start.0, end.1 - start.1);
        let start = rectangle_edge(from_layout, direction);
        let end = rectangle_edge(to_layout, (-direction.0, -direction.1));
        let _ = writeln!(
            svg,
            "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" marker-end=\"url(#state-arrow)\" />",
            start.0, start.1, end.0, end.1
        );
    }
    Ok(())
}

fn rectangle_edge(layout: StateNodeLayout, direction: (f64, f64)) -> (f64, f64) {
    let center = layout.center();
    if direction.0 == 0.0 && direction.1 == 0.0 {
        return center;
    }
    let horizontal_scale = if direction.0 == 0.0 {
        f64::INFINITY
    } else {
        layout.width / 2.0 / direction.0.abs()
    };
    let vertical_scale = if direction.1 == 0.0 {
        f64::INFINITY
    } else {
        layout.height / 2.0 / direction.1.abs()
    };
    let scale = horizontal_scale.min(vertical_scale);
    (
        center.0 + direction.0 * scale,
        center.1 + direction.1 * scale,
    )
}

fn transition_distance_squared(
    layout: &StateChartLayout,
    from: StateId,
    to: StateId,
    point: (f64, f64),
) -> f64 {
    let from_layout = layout.states[&from];
    let start = from_layout.center();
    if from == to {
        let left = start.0 - from_layout.width * 0.2;
        let right = start.0 + from_layout.width * 0.2;
        let top = from_layout.y;
        let first_control = (right + 56.0, top - 72.0);
        let second_control = (left - 56.0, top - 72.0);
        let end = (left, top);
        let mut previous = (right, top);
        let mut minimum = f64::INFINITY;
        for index in 1..=24 {
            let t = index as f64 / 24.0;
            let inverse = 1.0 - t;
            let current = (
                inverse.powi(3) * right
                    + 3.0 * inverse.powi(2) * t * first_control.0
                    + 3.0 * inverse * t.powi(2) * second_control.0
                    + t.powi(3) * end.0,
                inverse.powi(3) * top
                    + 3.0 * inverse.powi(2) * t * first_control.1
                    + 3.0 * inverse * t.powi(2) * second_control.1
                    + t.powi(3) * end.1,
            );
            minimum = minimum.min(distance_to_segment_squared(point, previous, current));
            previous = current;
        }
        return minimum;
    }
    let to_layout = layout.states[&to];
    let end = to_layout.center();
    let direction = (end.0 - start.0, end.1 - start.1);
    distance_to_segment_squared(
        point,
        rectangle_edge(from_layout, direction),
        rectangle_edge(to_layout, (-direction.0, -direction.1)),
    )
}

fn render_label(
    svg: &mut String,
    layout: &StateChartLayout,
    key: TransitionKey,
    label: &str,
) -> Result<(), StateChartEditorError> {
    let position = layout
        .transitions
        .get(&key)
        .ok_or(StateChartEditorError::MissingTransition(key))?;
    let _ = writeln!(
        svg,
        "<text x=\"{}\" y=\"{}\">{}</text>",
        position.x,
        position.y,
        escape_xml(label)
    );
    Ok(())
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn distance_to_segment_squared(point: (f64, f64), start: (f64, f64), end: (f64, f64)) -> f64 {
    let direction = (end.0 - start.0, end.1 - start.1);
    let length_squared = direction.0 * direction.0 + direction.1 * direction.1;
    if length_squared == 0.0 {
        return (point.0 - start.0).powi(2) + (point.1 - start.1).powi(2);
    }
    let projection =
        ((point.0 - start.0) * direction.0 + (point.1 - start.1) * direction.1) / length_squared;
    let projection = projection.clamp(0.0, 1.0);
    (point.0 - start.0 - projection * direction.0).powi(2)
        + (point.1 - start.1 - projection * direction.1).powi(2)
}

fn state_chart_schema_version() -> u32 {
    STATE_CHART_SCHEMA_VERSION
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdl_gnc_core::DurationNs;

    fn example_machine() -> StateMachine {
        let mut machine = StateMachine::new("flight_mode");
        let wait = machine.add_state("WAIT").unwrap();
        let ready = machine.add_state("READY").unwrap();
        let complete = machine.add_state("COMPLETE").unwrap();
        machine.set_initial_state(wait).unwrap();
        machine
            .add_guarded_transition(wait, ready, "enable", "thermal_ready")
            .unwrap();
        machine
            .add_timed_transition(ready, complete, DurationNs::from_nanoseconds(5_000_000_000))
            .unwrap();
        machine
    }

    #[test]
    fn document_layout_and_json_round_trip_cover_event_and_timed_transitions() {
        let document = StateChartDocument::from_machine(example_machine()).unwrap();
        assert_eq!(document.layout.states.len(), 3);
        assert_eq!(document.layout.transitions.len(), 2);
        let restored = StateChartDocument::from_json(&document.to_json().unwrap()).unwrap();
        assert_eq!(restored, document);
    }

    #[test]
    fn editor_hit_testing_selection_and_move_are_transactional() {
        let document = StateChartDocument::from_machine(example_machine()).unwrap();
        let mut editor = StateChartEditor::new(document).unwrap();
        let state = *editor.document.machine.initial_state.as_ref().unwrap();
        let layout = editor.document.layout.states[&state];
        assert_eq!(
            editor
                .select_at(layout.x + 10.0, layout.y + 10.0, 0.0)
                .unwrap(),
            Some(StateChartHitTarget::State(state))
        );
        editor
            .apply(StateChartEditorEvent::MoveSelected { dx: 15.0, dy: 20.0 })
            .unwrap();
        assert_eq!(editor.document.layout.states[&state].x, layout.x + 15.0);
        let before_invalid_move = editor.document.clone();
        assert!(editor
            .apply(StateChartEditorEvent::MoveSelected {
                dx: f64::NAN,
                dy: 1.0
            })
            .is_err());
        assert_eq!(editor.document, before_invalid_move);
    }

    #[test]
    fn svg_renders_guards_and_timed_transition_labels_safely() {
        let document = StateChartDocument::from_machine(example_machine()).unwrap();
        let svg = document.to_svg().unwrap();
        assert!(svg.contains("enable [thermal_ready]"), "{svg}");
        assert!(svg.contains("after 5000000000 ns"), "{svg}");
        assert!(svg.contains("class=\"state initial\""), "{svg}");
    }
}
