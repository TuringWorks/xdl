// SPDX-License-Identifier: GPL-2.0-only OR MIT
// Copyright (C) 2026 Ravindra Boddipalli

use crate::{GncError, Quaternion, Vec3};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum FrameId {
    Inertial,
    EarthCenteredEarthFixed,
    NorthEastDown,
    NorthEastUp,
    Body,
    Sensor(u16),
    /// Application-defined frame, identified by number so that `FrameId` stays `Copy`.
    Custom(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Transform {
    pub translation: Vec3,
    pub rotation: Quaternion,
}

impl Transform {
    pub const IDENTITY: Self = Self {
        translation: Vec3::ZERO,
        rotation: Quaternion::IDENTITY,
    };

    pub fn new(translation: Vec3, rotation: Quaternion) -> Result<Self, GncError> {
        if !translation.is_finite() {
            return Err(GncError::NonFinite {
                context: "transform translation".to_string(),
            });
        }
        Ok(Self {
            translation,
            rotation: rotation.try_normalize()?,
        })
    }

    pub fn apply(self, point: Vec3) -> Result<Vec3, GncError> {
        Ok(self.rotation.rotate_vector(point)? + self.translation)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pose {
    pub position: Vec3,
    pub attitude: Quaternion,
}

impl Pose {
    pub fn new(position: Vec3, attitude: Quaternion) -> Result<Self, GncError> {
        if !position.is_finite() {
            return Err(GncError::NonFinite {
                context: "pose position".to_string(),
            });
        }
        Ok(Self {
            position,
            attitude: attitude.try_normalize()?,
        })
    }
}
