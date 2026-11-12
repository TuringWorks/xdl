// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

use crate::{FrameId, GncError};
use serde::{Deserialize, Serialize};

/// Closed set of physical units used by the GNC contracts.
///
/// A closed enum (not a free-form string) so that a unit mismatch between a
/// producer and a consumer is caught by a check at construction, not by a
/// silently wrong number downstream.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Unit {
    Meter,
    MeterPerSecond,
    MeterPerSecondSquared,
    Radian,
    RadianPerSecond,
}

/// Frame and unit attached to a vector-valued quantity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VectorMeta {
    pub frame: FrameId,
    pub unit: Unit,
}

impl VectorMeta {
    pub const fn new(frame: FrameId, unit: Unit) -> Self {
        Self { frame, unit }
    }

    /// Verifies that this metadata is exactly `frame` and `unit`.
    ///
    /// Returns `InvalidValue` naming `context` on any mismatch. Never coerces.
    pub fn require(&self, frame: &FrameId, unit: Unit, context: &str) -> Result<(), GncError> {
        if &self.frame != frame {
            return Err(GncError::InvalidValue {
                context: context.to_string(),
                reason: format!("frame mismatch: expected {frame:?}, found {:?}", self.frame),
            });
        }
        if self.unit != unit {
            return Err(GncError::InvalidValue {
                context: context.to_string(),
                reason: format!("unit mismatch: expected {unit:?}, found {:?}", self.unit),
            });
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn require_accepts_exact_frame_and_unit() {
        let meta = VectorMeta::new(FrameId::Body, Unit::MeterPerSecondSquared);
        assert!(meta
            .require(&FrameId::Body, Unit::MeterPerSecondSquared, "accel")
            .is_ok());
    }

    #[test]
    fn require_rejects_frame_mismatch() {
        let meta = VectorMeta::new(FrameId::Body, Unit::Meter);
        assert!(meta
            .require(&FrameId::Inertial, Unit::Meter, "position")
            .is_err());
    }

    #[test]
    fn require_rejects_unit_mismatch_without_coercion() {
        let meta = VectorMeta::new(FrameId::Body, Unit::RadianPerSecond);
        assert!(meta.require(&FrameId::Body, Unit::Radian, "rate").is_err());
    }
}
