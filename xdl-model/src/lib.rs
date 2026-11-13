// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Typed, serializable model graphs for XDL and GNC simulation.
//!
//! This crate contains no execution policy. It describes blocks, ports, and
//! connections, then validates the graph before a scheduler or solver runs it.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap, HashSet};
use thiserror::Error;
use xdl_gnc_core::{DurationNs, FrameId, Unit};

pub mod layout;
pub use layout::{BlockLayout, LayoutError, ModelLayout, PortLayout, PortSide};
pub mod editor;
pub use editor::{EditorError, HitTarget, ModelDocument};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BlockId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct PortId(u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct SubsystemId(u32);

impl BlockId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl PortId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

impl SubsystemId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortDirection {
    Input,
    Output,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SignalShape {
    Scalar,
    Vector(usize),
    Matrix { rows: usize, columns: usize },
    Bus(Vec<BusField>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusField {
    pub name: String,
    pub shape: SignalShape,
    pub unit: Option<Unit>,
    pub frame: Option<FrameId>,
}

impl BusField {
    pub fn new(name: impl Into<String>, shape: SignalShape) -> Self {
        Self {
            name: name.into(),
            shape,
            unit: None,
            frame: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalSpec {
    pub shape: SignalShape,
    pub unit: Option<Unit>,
    pub frame: Option<FrameId>,
    pub sample_period: Option<DurationNs>,
}

impl SignalSpec {
    pub fn scalar() -> Self {
        Self {
            shape: SignalShape::Scalar,
            unit: None,
            frame: None,
            sample_period: None,
        }
    }

    pub fn compatible_with(&self, other: &Self) -> bool {
        self == other
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Port {
    pub id: PortId,
    pub name: String,
    pub direction: PortDirection,
    pub signal: SignalSpec,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub id: BlockId,
    pub name: String,
    /// Whether outputs depend directly on current inputs during a step.
    /// Delays and state-holding blocks set this to `false`, allowing them to
    /// break a feedback cycle without creating an algebraic loop.
    pub direct_feedthrough: bool,
    pub ports: BTreeMap<PortId, Port>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Connection {
    pub output: PortId,
    pub input: PortId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BoundaryBinding {
    pub boundary: PortId,
    pub internal: PortId,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SubsystemError {
    #[error("unknown subsystem boundary port: {0:?}")]
    UnknownBoundaryPort(PortId),
    #[error("unknown internal port: {0:?}")]
    UnknownInternalPort(PortId),
    #[error("boundary port direction is invalid for this binding")]
    DirectionMismatch,
    #[error("subsystem boundary signal does not match internal signal")]
    SignalMismatch,
    #[error("subsystem boundary port is already bound: {0:?}")]
    AlreadyBound(PortId),
    #[error("internal subsystem model is invalid: {0}")]
    InternalModel(#[from] ModelError),
    #[error("subsystem boundary identifier space exhausted")]
    IdExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Subsystem {
    pub id: SubsystemId,
    pub name: String,
    pub model: Model,
    pub boundary_ports: BTreeMap<PortId, Port>,
    pub input_bindings: Vec<BoundaryBinding>,
    pub output_bindings: Vec<BoundaryBinding>,
    next_boundary_port: u32,
}

impl Subsystem {
    pub fn new(id: SubsystemId, name: impl Into<String>, model: Model) -> Self {
        Self {
            id,
            name: name.into(),
            model,
            boundary_ports: BTreeMap::new(),
            input_bindings: Vec::new(),
            output_bindings: Vec::new(),
            next_boundary_port: 0,
        }
    }

    pub fn add_boundary_port(
        &mut self,
        name: impl Into<String>,
        direction: PortDirection,
        signal: SignalSpec,
    ) -> Result<PortId, SubsystemError> {
        let id = PortId(self.next_boundary_port);
        self.next_boundary_port = self
            .next_boundary_port
            .checked_add(1)
            .ok_or(SubsystemError::IdExhausted)?;
        self.boundary_ports.insert(
            id,
            Port {
                id,
                name: name.into(),
                direction,
                signal,
            },
        );
        Ok(id)
    }

    pub fn bind_input(&mut self, boundary: PortId, internal: PortId) -> Result<(), SubsystemError> {
        self.bind(boundary, internal, PortDirection::Input)?;
        self.input_bindings
            .push(BoundaryBinding { boundary, internal });
        Ok(())
    }

    pub fn bind_output(
        &mut self,
        boundary: PortId,
        internal: PortId,
    ) -> Result<(), SubsystemError> {
        self.bind(boundary, internal, PortDirection::Output)?;
        self.output_bindings
            .push(BoundaryBinding { boundary, internal });
        Ok(())
    }

    pub fn validate(&self) -> Result<(), SubsystemError> {
        self.model.validate()?;
        for binding in self
            .input_bindings
            .iter()
            .chain(self.output_bindings.iter())
        {
            self.validate_compatibility(
                binding.boundary,
                binding.internal,
                self.boundary_ports
                    .get(&binding.boundary)
                    .ok_or(SubsystemError::UnknownBoundaryPort(binding.boundary))?
                    .direction,
            )?;
        }
        Ok(())
    }

    fn bind(
        &self,
        boundary: PortId,
        internal: PortId,
        direction: PortDirection,
    ) -> Result<(), SubsystemError> {
        self.validate_compatibility(boundary, internal, direction)?;
        let already_bound = self
            .input_bindings
            .iter()
            .chain(self.output_bindings.iter())
            .any(|binding| binding.boundary == boundary);
        if already_bound {
            return Err(SubsystemError::AlreadyBound(boundary));
        }
        Ok(())
    }

    fn validate_compatibility(
        &self,
        boundary: PortId,
        internal: PortId,
        direction: PortDirection,
    ) -> Result<(), SubsystemError> {
        let boundary_port = self
            .boundary_ports
            .get(&boundary)
            .ok_or(SubsystemError::UnknownBoundaryPort(boundary))?;
        let internal_port = self
            .model
            .port(internal)
            .map_err(|_| SubsystemError::UnknownInternalPort(internal))?;
        if boundary_port.direction != direction || internal_port.direction != direction {
            return Err(SubsystemError::DirectionMismatch);
        }
        if !boundary_port.signal.compatible_with(&internal_port.signal) {
            return Err(SubsystemError::SignalMismatch);
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ModelError {
    #[error("duplicate block name: {0}")]
    DuplicateBlockName(String),
    #[error("unknown block: {0:?}")]
    UnknownBlock(BlockId),
    #[error("unknown port: {0:?}")]
    UnknownPort(PortId),
    #[error("duplicate port name on block {block:?}: {name}")]
    DuplicatePortName { block: BlockId, name: String },
    #[error("port direction mismatch: output {output:?}, input {input:?}")]
    DirectionMismatch { output: PortId, input: PortId },
    #[error("signal specification mismatch from {output:?} to {input:?}")]
    SignalMismatch { output: PortId, input: PortId },
    #[error("input port is already connected: {0:?}")]
    InputAlreadyConnected(PortId),
    #[error("model graph contains a cycle")]
    Cycle,
    #[error("identifier space exhausted")]
    IdExhausted,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Model {
    pub name: String,
    pub blocks: BTreeMap<BlockId, Block>,
    pub connections: Vec<Connection>,
    next_block: u32,
    next_port: u32,
}

impl Model {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            blocks: BTreeMap::new(),
            connections: Vec::new(),
            next_block: 0,
            next_port: 0,
        }
    }

    pub fn add_block(&mut self, name: impl Into<String>) -> Result<BlockId, ModelError> {
        let name = name.into();
        if self.blocks.values().any(|block| block.name == name) {
            return Err(ModelError::DuplicateBlockName(name));
        }
        let id = BlockId(self.next_block);
        self.next_block = self
            .next_block
            .checked_add(1)
            .ok_or(ModelError::IdExhausted)?;
        self.blocks.insert(
            id,
            Block {
                id,
                name,
                direct_feedthrough: true,
                ports: BTreeMap::new(),
            },
        );
        Ok(id)
    }

    pub fn set_block_direct_feedthrough(
        &mut self,
        block: BlockId,
        direct_feedthrough: bool,
    ) -> Result<(), ModelError> {
        self.blocks
            .get_mut(&block)
            .ok_or(ModelError::UnknownBlock(block))?
            .direct_feedthrough = direct_feedthrough;
        Ok(())
    }

    pub fn add_port(
        &mut self,
        block_id: BlockId,
        name: impl Into<String>,
        direction: PortDirection,
        signal: SignalSpec,
    ) -> Result<PortId, ModelError> {
        let name = name.into();
        let block = self
            .blocks
            .get_mut(&block_id)
            .ok_or(ModelError::UnknownBlock(block_id))?;
        if block.ports.values().any(|port| port.name == name) {
            return Err(ModelError::DuplicatePortName {
                block: block_id,
                name,
            });
        }
        let id = PortId(self.next_port);
        self.next_port = self
            .next_port
            .checked_add(1)
            .ok_or(ModelError::IdExhausted)?;
        block.ports.insert(
            id,
            Port {
                id,
                name,
                direction,
                signal,
            },
        );
        Ok(id)
    }

    pub fn connect(&mut self, output: PortId, input: PortId) -> Result<(), ModelError> {
        let output_port = self.port(output)?;
        let input_port = self.port(input)?;
        if output_port.direction != PortDirection::Output
            || input_port.direction != PortDirection::Input
        {
            return Err(ModelError::DirectionMismatch { output, input });
        }
        if !output_port.signal.compatible_with(&input_port.signal) {
            return Err(ModelError::SignalMismatch { output, input });
        }
        if self
            .connections
            .iter()
            .any(|connection| connection.input == input)
        {
            return Err(ModelError::InputAlreadyConnected(input));
        }
        self.connections.push(Connection { output, input });
        Ok(())
    }

    pub fn block(&self, id: BlockId) -> Result<&Block, ModelError> {
        self.blocks.get(&id).ok_or(ModelError::UnknownBlock(id))
    }

    pub fn port(&self, id: PortId) -> Result<&Port, ModelError> {
        self.port_ref(id)
    }

    pub fn block_for_port(&self, port_id: PortId) -> Result<BlockId, ModelError> {
        self.block_for_port_ref(port_id)
    }

    pub fn validate(&self) -> Result<(), ModelError> {
        self.validate_connections()?;
        let mut adjacency: HashMap<BlockId, Vec<BlockId>> = HashMap::new();
        for connection in &self.connections {
            let source = self.block_for_port_ref(connection.output)?;
            let destination = self.block_for_port_ref(connection.input)?;
            adjacency.entry(source).or_default().push(destination);
        }
        let mut visiting = HashSet::new();
        let mut visited = HashSet::new();
        let mut stack = Vec::new();
        for block in self.blocks.keys().copied() {
            if has_algebraic_cycle(
                block,
                &adjacency,
                &self.blocks,
                &mut visiting,
                &mut visited,
                &mut stack,
            ) {
                return Err(ModelError::Cycle);
            }
        }
        Ok(())
    }

    fn validate_connections(&self) -> Result<(), ModelError> {
        for connection in &self.connections {
            let output = self.port(connection.output)?;
            let input = self.port(connection.input)?;
            if output.direction != PortDirection::Output || input.direction != PortDirection::Input
            {
                return Err(ModelError::DirectionMismatch {
                    output: connection.output,
                    input: connection.input,
                });
            }
            if !output.signal.compatible_with(&input.signal) {
                return Err(ModelError::SignalMismatch {
                    output: connection.output,
                    input: connection.input,
                });
            }
        }
        Ok(())
    }

    fn port_ref(&self, id: PortId) -> Result<&Port, ModelError> {
        self.blocks
            .values()
            .find_map(|block| block.ports.get(&id))
            .ok_or(ModelError::UnknownPort(id))
    }

    fn block_for_port_ref(&self, port_id: PortId) -> Result<BlockId, ModelError> {
        self.blocks
            .values()
            .find(|block| block.ports.contains_key(&port_id))
            .map(|block| block.id)
            .ok_or(ModelError::UnknownPort(port_id))
    }
}

fn has_algebraic_cycle(
    block: BlockId,
    adjacency: &HashMap<BlockId, Vec<BlockId>>,
    blocks: &BTreeMap<BlockId, Block>,
    visiting: &mut HashSet<BlockId>,
    visited: &mut HashSet<BlockId>,
    stack: &mut Vec<BlockId>,
) -> bool {
    if visiting.contains(&block) {
        let Some(start) = stack.iter().position(|candidate| *candidate == block) else {
            return true;
        };
        return stack[start..].iter().all(|candidate| {
            blocks
                .get(candidate)
                .is_some_and(|item| item.direct_feedthrough)
        });
    }
    if visited.contains(&block) {
        return false;
    }
    visiting.insert(block);
    stack.push(block);
    if adjacency
        .get(&block)
        .into_iter()
        .flatten()
        .any(|next| has_algebraic_cycle(*next, adjacency, blocks, visiting, visited, stack))
    {
        return true;
    }
    stack.pop();
    visiting.remove(&block);
    visited.insert(block);
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn graph() -> (Model, PortId, PortId) {
        let mut model = Model::new("test");
        let source = model.add_block("source").expect("block");
        let sink = model.add_block("sink").expect("block");
        let output = model
            .add_port(source, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("port");
        let input = model
            .add_port(sink, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        (model, output, input)
    }

    #[test]
    fn graph_accepts_typed_connection_and_serializes() {
        let (mut model, output, input) = graph();
        model.connect(output, input).expect("compatible ports");
        model.validate().expect("acyclic model");
        let encoded = serde_json::to_string(&model).expect("serializable model");
        let decoded: Model = serde_json::from_str(&encoded).expect("deserializable model");
        assert_eq!(decoded, model);
    }

    #[test]
    fn graph_rejects_unit_mismatch() {
        let mut model = Model::new("units");
        let source = model.add_block("source").expect("block");
        let sink = model.add_block("sink").expect("block");
        let output = model
            .add_port(
                source,
                "out",
                PortDirection::Output,
                SignalSpec {
                    shape: SignalShape::Vector(3),
                    unit: Some(Unit::Meter),
                    frame: Some(FrameId::Inertial),
                    sample_period: None,
                },
            )
            .expect("port");
        let input = model
            .add_port(
                sink,
                "in",
                PortDirection::Input,
                SignalSpec {
                    shape: SignalShape::Vector(3),
                    unit: Some(Unit::MeterPerSecond),
                    frame: Some(FrameId::Inertial),
                    sample_period: None,
                },
            )
            .expect("port");
        assert!(matches!(
            model.connect(output, input),
            Err(ModelError::SignalMismatch { .. })
        ));
    }

    #[test]
    fn graph_rejects_duplicate_input_and_cycles() {
        let (mut model, output, input) = graph();
        model.connect(output, input).expect("first connection");
        assert!(matches!(
            model.connect(output, input),
            Err(ModelError::InputAlreadyConnected(_))
        ));
        let source = model.add_block("source2").expect("block");
        let sink = model.add_block("sink2").expect("block");
        let source_out = model
            .add_port(source, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("port");
        let sink_in = model
            .add_port(sink, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        model.connect(source_out, sink_in).expect("connection");
        assert!(model.validate().is_ok());
    }

    #[test]
    fn delayed_feedback_cycle_is_valid_but_algebraic_cycle_is_rejected() {
        let mut model = Model::new("feedback");
        let first = model.add_block("first").expect("block");
        let delayed = model.add_block("delay").expect("block");
        let first_input = model
            .add_port(first, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        let first_output = model
            .add_port(first, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("port");
        let delayed_input = model
            .add_port(delayed, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        let delayed_output = model
            .add_port(delayed, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("port");
        model
            .connect(first_output, delayed_input)
            .expect("connection");
        model
            .connect(delayed_output, first_input)
            .expect("connection");
        assert!(matches!(model.validate(), Err(ModelError::Cycle)));
        model
            .set_block_direct_feedthrough(delayed, false)
            .expect("delay block");
        model.validate().expect("delay breaks algebraic loop");
    }

    #[test]
    fn bus_signal_schema_preserves_field_metadata() {
        let schema = SignalSpec {
            shape: SignalShape::Bus(vec![BusField {
                name: "position".to_string(),
                shape: SignalShape::Vector(3),
                unit: Some(Unit::Meter),
                frame: Some(FrameId::Inertial),
            }]),
            unit: None,
            frame: None,
            sample_period: None,
        };
        let encoded = serde_json::to_string(&schema).expect("serializable bus");
        let decoded: SignalSpec = serde_json::from_str(&encoded).expect("deserializable bus");
        assert_eq!(decoded, schema);
    }

    #[test]
    fn subsystem_boundary_validates_typed_input_binding() {
        let mut internal = Model::new("inner");
        let block = internal.add_block("controller").expect("block");
        let internal_input = internal
            .add_port(block, "command", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        let mut subsystem = Subsystem::new(SubsystemId::new(1), "controller_subsystem", internal);
        let boundary = subsystem
            .add_boundary_port("command", PortDirection::Input, SignalSpec::scalar())
            .expect("boundary port");
        subsystem
            .bind_input(boundary, internal_input)
            .expect("compatible binding");
        subsystem.validate().expect("valid subsystem");
        assert!(matches!(
            subsystem.bind_input(boundary, internal_input),
            Err(SubsystemError::AlreadyBound(_))
        ));
    }
}
