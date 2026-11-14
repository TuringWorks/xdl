// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Serializable geometry for a block-diagram editor.

use crate::{BlockId, Model, PortDirection, PortId};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PortSide {
    Left,
    Right,
    Top,
    Bottom,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct BlockLayout {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl BlockLayout {
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
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PortLayout {
    pub side: PortSide,
    pub offset: f64,
}

impl PortLayout {
    pub const fn new(side: PortSide, offset: f64) -> Self {
        Self { side, offset }
    }

    fn is_valid(self) -> bool {
        self.offset.is_finite() && (0.0..=1.0).contains(&self.offset)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct ModelLayout {
    pub blocks: BTreeMap<BlockId, BlockLayout>,
    pub ports: BTreeMap<PortId, PortLayout>,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum LayoutError {
    #[error("layout is missing block {0:?}")]
    MissingBlock(BlockId),
    #[error("layout references unknown block {0:?}")]
    UnknownBlock(BlockId),
    #[error("layout is missing port {0:?}")]
    MissingPort(PortId),
    #[error("layout references unknown port {0:?}")]
    UnknownPort(PortId),
    #[error("layout geometry for block {0:?} is invalid")]
    InvalidBlockGeometry(BlockId),
    #[error("layout geometry for port {0:?} is invalid")]
    InvalidPortGeometry(PortId),
    #[error("input port {0:?} must be on the left or top side")]
    InputSide(PortId),
    #[error("output port {0:?} must be on the right or bottom side")]
    OutputSide(PortId),
}

impl ModelLayout {
    pub fn from_model(model: &Model) -> Result<Self, LayoutError> {
        let mut layout = Self::default();
        for (index, block) in model.blocks.values().enumerate() {
            layout.blocks.insert(
                block.id,
                BlockLayout::new(
                    240.0 * (index % 4) as f64,
                    140.0 * (index / 4) as f64,
                    180.0,
                    80.0,
                ),
            );
            let inputs: Vec<_> = block
                .ports
                .values()
                .filter(|port| port.direction == PortDirection::Input)
                .collect();
            let outputs: Vec<_> = block
                .ports
                .values()
                .filter(|port| port.direction == PortDirection::Output)
                .collect();
            for (port_index, port) in inputs.into_iter().enumerate() {
                layout.ports.insert(
                    port.id,
                    PortLayout::new(
                        PortSide::Left,
                        normalized_offset(port_index, block.ports.len()),
                    ),
                );
            }
            for (port_index, port) in outputs.into_iter().enumerate() {
                layout.ports.insert(
                    port.id,
                    PortLayout::new(
                        PortSide::Right,
                        normalized_offset(port_index, block.ports.len()),
                    ),
                );
            }
        }
        layout.validate(model)?;
        Ok(layout)
    }

    pub fn validate(&self, model: &Model) -> Result<(), LayoutError> {
        for block in model.blocks.values() {
            let geometry = self
                .blocks
                .get(&block.id)
                .ok_or(LayoutError::MissingBlock(block.id))?;
            if !geometry.is_valid() {
                return Err(LayoutError::InvalidBlockGeometry(block.id));
            }
            for port in block.ports.values() {
                let layout = self
                    .ports
                    .get(&port.id)
                    .copied()
                    .ok_or(LayoutError::MissingPort(port.id))?;
                if !layout.is_valid() {
                    return Err(LayoutError::InvalidPortGeometry(port.id));
                }
                let input_side = matches!(layout.side, PortSide::Left | PortSide::Top);
                if port.direction == PortDirection::Input && !input_side {
                    return Err(LayoutError::InputSide(port.id));
                }
                if port.direction == PortDirection::Output && input_side {
                    return Err(LayoutError::OutputSide(port.id));
                }
            }
        }
        if let Some(block) = self
            .blocks
            .keys()
            .find(|block| !model.blocks.contains_key(block))
        {
            return Err(LayoutError::UnknownBlock(*block));
        }
        if let Some(port) = self.ports.keys().find(|port| model.port(**port).is_err()) {
            return Err(LayoutError::UnknownPort(*port));
        }
        Ok(())
    }
}

fn normalized_offset(index: usize, port_count: usize) -> f64 {
    (index + 1) as f64 / (port_count + 1) as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Model, SignalSpec};

    #[test]
    fn layout_validates_block_and_port_geometry() {
        let mut model = Model::new("diagram");
        let block = model.add_block("gain").expect("block");
        let input = model
            .add_port(block, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("input");
        let output = model
            .add_port(block, "out", PortDirection::Output, SignalSpec::scalar())
            .expect("output");
        let layout = ModelLayout {
            blocks: BTreeMap::from([(block, BlockLayout::new(10.0, 20.0, 120.0, 60.0))]),
            ports: BTreeMap::from([
                (input, PortLayout::new(PortSide::Left, 0.5)),
                (output, PortLayout::new(PortSide::Right, 0.5)),
            ]),
        };
        layout.validate(&model).expect("valid layout");
    }

    #[test]
    fn layout_rejects_wrong_port_side() {
        let mut model = Model::new("diagram");
        let block = model.add_block("gain").expect("block");
        let input = model
            .add_port(block, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("input");
        let layout = ModelLayout {
            blocks: BTreeMap::from([(block, BlockLayout::new(0.0, 0.0, 1.0, 1.0))]),
            ports: BTreeMap::from([(input, PortLayout::new(PortSide::Right, 0.5))]),
        };
        assert!(matches!(
            layout.validate(&model),
            Err(LayoutError::InputSide(_))
        ));
    }

    #[test]
    fn layout_reports_a_model_port_missing_from_layout_as_missing() {
        let mut model = Model::new("diagram");
        let block = model.add_block("gain").expect("block");
        let input = model
            .add_port(block, "in", PortDirection::Input, SignalSpec::scalar())
            .expect("input");
        // Block geometry present, port layout absent: the layout lacks the port.
        let layout = ModelLayout {
            blocks: BTreeMap::from([(block, BlockLayout::new(10.0, 20.0, 120.0, 60.0))]),
            ports: BTreeMap::new(),
        };
        assert_eq!(
            layout.validate(&model),
            Err(LayoutError::MissingPort(input))
        );
    }

    #[test]
    fn layout_can_be_generated_deterministically_from_model() {
        let mut model = Model::new("diagram");
        let block = model.add_block("controller").expect("block");
        model
            .add_port(block, "command", PortDirection::Input, SignalSpec::scalar())
            .expect("input");
        model
            .add_port(
                block,
                "actuator",
                PortDirection::Output,
                SignalSpec::scalar(),
            )
            .expect("output");
        let layout = ModelLayout::from_model(&model).expect("generated layout");
        layout.validate(&model).expect("valid layout");
        assert_eq!(
            layout.blocks[&block],
            BlockLayout::new(0.0, 0.0, 180.0, 80.0)
        );
        assert_eq!(layout.ports.len(), 2);
    }
}
