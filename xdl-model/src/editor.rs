// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Transactional editing operations for model diagrams.

use crate::{
    BlockId, BlockLayout, Connection, LayoutError, Model, ModelError, ModelLayout, PortId,
    PortLayout,
};
use std::fmt::Write;
use thiserror::Error;

pub const MODEL_DOCUMENT_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitTarget {
    Block(BlockId),
    Port(PortId),
    Connection(Connection),
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum EditorError {
    #[error("model error: {0}")]
    Model(#[from] ModelError),
    #[error("layout error: {0}")]
    Layout(#[from] LayoutError),
    #[error("unsupported model document schema version: {0}")]
    UnsupportedSchemaVersion(u32),
    #[error("model document serialization failed: {0}")]
    Serialization(String),
    #[error("model document deserialization failed: {0}")]
    Deserialization(String),
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct ModelDocument {
    #[serde(default = "current_schema_version")]
    pub schema_version: u32,
    pub model: Model,
    pub layout: ModelLayout,
}

const fn current_schema_version() -> u32 {
    MODEL_DOCUMENT_SCHEMA_VERSION
}

impl ModelDocument {
    pub fn new(model: Model, layout: ModelLayout) -> Result<Self, EditorError> {
        model.validate()?;
        layout.validate(&model)?;
        Ok(Self {
            schema_version: MODEL_DOCUMENT_SCHEMA_VERSION,
            model,
            layout,
        })
    }

    pub fn validate(&self) -> Result<(), EditorError> {
        if self.schema_version != MODEL_DOCUMENT_SCHEMA_VERSION {
            return Err(EditorError::UnsupportedSchemaVersion(self.schema_version));
        }
        self.model.validate()?;
        self.layout.validate(&self.model)?;
        Ok(())
    }

    pub fn to_json(&self) -> Result<String, EditorError> {
        self.validate()?;
        serde_json::to_string_pretty(self)
            .map_err(|error| EditorError::Serialization(error.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self, EditorError> {
        let document: Self = serde_json::from_str(json)
            .map_err(|error| EditorError::Deserialization(error.to_string()))?;
        document.validate()?;
        Ok(document)
    }

    /// Loads a document and emits canonical current-version JSON. This is the
    /// migration seam for future schema versions; version-one documents are
    /// currently normalized without changing their model semantics.
    pub fn migrate_json(json: &str) -> Result<String, EditorError> {
        Self::from_json(json)?.to_json()
    }

    pub fn to_svg(&self) -> Result<String, EditorError> {
        self.validate()?;
        let (min_x, min_y, max_x, max_y) = self.layout.blocks.values().fold(
            (0.0_f64, 0.0_f64, 0.0_f64, 0.0_f64),
            |(min_x, min_y, max_x, max_y), block| {
                (
                    min_x.min(block.x),
                    min_y.min(block.y),
                    max_x.max(block.x + block.width),
                    max_y.max(block.y + block.height),
                )
            },
        );
        let margin = 24.0;
        let width = (max_x - min_x + 2.0 * margin).max(1.0);
        let height = (max_y - min_y + 2.0 * margin).max(1.0);
        let mut svg = String::new();
        let _ = writeln!(
            svg,
            "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"{} {} {} {}\" role=\"img\">",
            min_x - margin,
            min_y - margin,
            width,
            height
        );
        let _ = writeln!(
            svg,
            "<g class=\"connections\" fill=\"none\" stroke=\"currentColor\">"
        );
        for connection in &self.model.connections {
            let start = self.port_position(connection.output)?;
            let end = self.port_position(connection.input)?;
            let _ = writeln!(
                svg,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" />",
                start.0, start.1, end.0, end.1
            );
        }
        let _ = writeln!(
            svg,
            "</g><g class=\"blocks\" fill=\"none\" stroke=\"currentColor\">"
        );
        for block in self.model.blocks.values() {
            let geometry = self.layout.blocks[&block.id];
            let _ = writeln!(
                svg,
                "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"4\" />",
                geometry.x, geometry.y, geometry.width, geometry.height
            );
        }
        let _ = writeln!(
            svg,
            "</g><g class=\"labels\" fill=\"currentColor\" stroke=\"none\">"
        );
        for block in self.model.blocks.values() {
            let geometry = self.layout.blocks[&block.id];
            let _ = writeln!(
                svg,
                "<text x=\"{}\" y=\"{}\">{}</text>",
                geometry.x + 8.0,
                geometry.y + 20.0,
                escape_xml(&block.name)
            );
            for port in block.ports.values() {
                let position = self.port_position(port.id)?;
                let _ = writeln!(
                    svg,
                    "<circle cx=\"{}\" cy=\"{}\" r=\"4\" /><text x=\"{}\" y=\"{}\">{}</text>",
                    position.0,
                    position.1,
                    position.0 + 6.0,
                    position.1 - 6.0,
                    escape_xml(&port.name)
                );
            }
        }
        let _ = writeln!(svg, "</g></svg>");
        Ok(svg)
    }

    pub fn hit_test(
        &self,
        x: f64,
        y: f64,
        tolerance: f64,
    ) -> Result<Option<HitTarget>, EditorError> {
        self.validate()?;
        if !x.is_finite() || !y.is_finite() || !tolerance.is_finite() || tolerance < 0.0 {
            return Ok(None);
        }
        let tolerance_squared = tolerance * tolerance;
        for block in self.model.blocks.values() {
            let geometry = self.layout.blocks[&block.id];
            if (geometry.x..=geometry.x + geometry.width).contains(&x)
                && (geometry.y..=geometry.y + geometry.height).contains(&y)
            {
                for port in block.ports.values() {
                    let position = self.port_position(port.id)?;
                    if distance_squared((x, y), position) <= tolerance_squared {
                        return Ok(Some(HitTarget::Port(port.id)));
                    }
                }
                return Ok(Some(HitTarget::Block(block.id)));
            }
        }
        for connection in &self.model.connections {
            let start = self.port_position(connection.output)?;
            let end = self.port_position(connection.input)?;
            if distance_to_segment_squared((x, y), start, end) <= tolerance_squared {
                return Ok(Some(HitTarget::Connection(*connection)));
            }
        }
        Ok(None)
    }

    fn port_position(&self, port: PortId) -> Result<(f64, f64), EditorError> {
        let block_id = self.model.block_for_port(port)?;
        let block = self.layout.blocks[&block_id];
        let port_layout = self.layout.ports[&port];
        let position = match port_layout.side {
            crate::PortSide::Left => (block.x, block.y + port_layout.offset * block.height),
            crate::PortSide::Right => (
                block.x + block.width,
                block.y + port_layout.offset * block.height,
            ),
            crate::PortSide::Top => (block.x + port_layout.offset * block.width, block.y),
            crate::PortSide::Bottom => (
                block.x + port_layout.offset * block.width,
                block.y + block.height,
            ),
        };
        Ok(position)
    }

    pub fn move_block(&mut self, block: BlockId, x: f64, y: f64) -> Result<(), EditorError> {
        let mut candidate = self.layout.clone();
        let geometry = candidate
            .blocks
            .get_mut(&block)
            .ok_or(LayoutError::MissingBlock(block))?;
        geometry.x = x;
        geometry.y = y;
        candidate.validate(&self.model)?;
        self.layout = candidate;
        Ok(())
    }

    pub fn resize_block(
        &mut self,
        block: BlockId,
        width: f64,
        height: f64,
    ) -> Result<(), EditorError> {
        let mut candidate = self.layout.clone();
        let geometry = candidate
            .blocks
            .get_mut(&block)
            .ok_or(LayoutError::MissingBlock(block))?;
        geometry.width = width;
        geometry.height = height;
        candidate.validate(&self.model)?;
        self.layout = candidate;
        Ok(())
    }

    pub fn set_port_layout(&mut self, port: PortId, layout: PortLayout) -> Result<(), EditorError> {
        let mut candidate = self.layout.clone();
        if !candidate.ports.contains_key(&port) {
            return Err(LayoutError::UnknownPort(port).into());
        }
        candidate.ports.insert(port, layout);
        candidate.validate(&self.model)?;
        self.layout = candidate;
        Ok(())
    }

    /// Reconnects an output to one input atomically. Existing connections for
    /// either endpoint are removed only from the candidate model; the current
    /// document is retained if validation fails.
    pub fn reconnect(&mut self, output: PortId, input: PortId) -> Result<(), EditorError> {
        let mut candidate = self.model.clone();
        candidate
            .connections
            .retain(|connection| connection.input != input && connection.output != output);
        candidate.connect(output, input)?;
        candidate.validate()?;
        self.layout.validate(&candidate)?;
        self.model = candidate;
        Ok(())
    }

    pub fn block_layout(&self, block: BlockId) -> Result<BlockLayout, EditorError> {
        self.layout
            .blocks
            .get(&block)
            .copied()
            .ok_or(LayoutError::MissingBlock(block).into())
    }
}

fn escape_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

fn distance_squared(first: (f64, f64), second: (f64, f64)) -> f64 {
    (first.0 - second.0).powi(2) + (first.1 - second.1).powi(2)
}

fn distance_to_segment_squared(point: (f64, f64), start: (f64, f64), end: (f64, f64)) -> f64 {
    let direction = (end.0 - start.0, end.1 - start.1);
    let length_squared = direction.0 * direction.0 + direction.1 * direction.1;
    if length_squared == 0.0 {
        return distance_squared(point, start);
    }
    let projection =
        ((point.0 - start.0) * direction.0 + (point.1 - start.1) * direction.1) / length_squared;
    let clamped = projection.clamp(0.0, 1.0);
    distance_squared(
        point,
        (
            start.0 + clamped * direction.0,
            start.1 + clamped * direction.1,
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PortDirection, PortSide, SignalSpec};
    use std::collections::BTreeMap;

    fn document() -> (ModelDocument, BlockId, PortId, PortId, PortId) {
        let mut model = Model::new("editor");
        let source = model.add_block("source").expect("source");
        let sink = model.add_block("sink").expect("sink");
        let alternate = model.add_block("alternate").expect("alternate");
        let source_out = model
            .add_port(source, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("port");
        let sink_in = model
            .add_port(sink, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        let alternate_in = model
            .add_port(alternate, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("port");
        model.connect(source_out, sink_in).expect("connection");
        let layout = ModelLayout {
            blocks: BTreeMap::from([
                (source, BlockLayout::new(0.0, 0.0, 100.0, 50.0)),
                (sink, BlockLayout::new(200.0, 0.0, 100.0, 50.0)),
                (alternate, BlockLayout::new(200.0, 100.0, 100.0, 50.0)),
            ]),
            ports: BTreeMap::from([
                (source_out, PortLayout::new(PortSide::Right, 0.5)),
                (sink_in, PortLayout::new(PortSide::Left, 0.5)),
                (alternate_in, PortLayout::new(PortSide::Left, 0.5)),
            ]),
        };
        (
            ModelDocument::new(model, layout).expect("document"),
            source,
            source_out,
            sink_in,
            alternate_in,
        )
    }

    #[test]
    fn edits_commit_only_after_validation() {
        let (mut document, source, _, _, _) = document();
        document.move_block(source, 30.0, 40.0).expect("move");
        assert_eq!(document.block_layout(source).expect("layout").x, 30.0);
        assert!(document.resize_block(source, 0.0, 50.0).is_err());
        assert_eq!(document.block_layout(source).expect("layout").width, 100.0);
    }

    #[test]
    fn reconnect_replaces_connection_atomically() {
        let (mut document, _, source_out, sink_in, alternate_in) = document();
        document
            .reconnect(source_out, alternate_in)
            .expect("reconnect");
        assert!(document.model.connections.iter().any(|connection| {
            connection.output == source_out && connection.input == alternate_in
        }));
        assert!(!document
            .model
            .connections
            .iter()
            .any(|connection| connection.input == sink_in));
        assert!(document.reconnect(PortId::new(999), sink_in).is_err());
        assert!(document.model.connections.iter().any(|connection| {
            connection.output == source_out && connection.input == alternate_in
        }));
    }

    #[test]
    fn document_round_trips_with_schema_version() {
        let (document, _, _, _, _) = document();
        let json = document.to_json().expect("serialize document");
        assert!(json.contains("schema_version"));
        let decoded = ModelDocument::from_json(&json).expect("deserialize document");
        assert_eq!(decoded, document);
    }

    #[test]
    fn unsupported_schema_is_rejected_before_use() {
        let (mut document, _, _, _, _) = document();
        document.schema_version = MODEL_DOCUMENT_SCHEMA_VERSION + 1;
        assert!(matches!(
            document.validate(),
            Err(EditorError::UnsupportedSchemaVersion(_))
        ));
    }

    #[test]
    fn migration_adds_missing_version_and_canonicalizes_json() {
        let (document, _, _, _, _) = document();
        let json = document.to_json().expect("serialize document");
        let legacy = json.replace("\"schema_version\": 1,\n", "");
        let migrated = ModelDocument::migrate_json(&legacy).expect("migrate document");
        assert!(migrated.contains("\"schema_version\": 1"));
        assert_eq!(
            ModelDocument::from_json(&migrated).expect("load migrated"),
            document
        );
    }

    #[test]
    fn document_renders_a_safe_connection_svg() {
        let (document, _, _, _, _) = document();
        let svg = document.to_svg().expect("render svg");
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("class=\"connections\""));
        assert!(svg.contains("<line "));
        assert!(svg.contains("<circle "));
        assert!(svg.contains("out"));
        assert!(svg.contains("source"));
    }

    #[test]
    fn hit_testing_prioritizes_ports_then_blocks_then_connections() {
        let (document, _, source_out, sink_in, _) = document();
        assert_eq!(
            document.hit_test(200.0, 25.0, 2.0).expect("hit test"),
            Some(HitTarget::Port(sink_in))
        );
        assert_eq!(
            document.hit_test(250.0, 20.0, 2.0).expect("hit test"),
            Some(HitTarget::Block(BlockId::new(1)))
        );
        assert_eq!(
            document.hit_test(150.0, 25.0, 2.0).expect("hit test"),
            Some(HitTarget::Connection(Connection {
                output: source_out,
                input: sink_in,
            }))
        );
    }
}
