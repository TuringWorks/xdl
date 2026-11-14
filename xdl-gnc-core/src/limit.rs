// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

use crate::GncError;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScalarLimit {
    pub min: f64,
    pub max: f64,
}

impl ScalarLimit {
    pub fn new(min: f64, max: f64) -> Result<Self, GncError> {
        if !min.is_finite() || !max.is_finite() {
            return Err(GncError::NonFinite {
                context: "scalar limit".to_string(),
            });
        }
        if min > max {
            return Err(GncError::InvalidLimit { min, max });
        }
        Ok(Self { min, max })
    }

    pub fn apply(self, value: f64) -> Result<LimitResult, GncError> {
        if !value.is_finite() {
            return Err(GncError::NonFinite {
                context: "limited scalar".to_string(),
            });
        }
        let limited = value.clamp(self.min, self.max);
        Ok(LimitResult {
            value: limited,
            saturated: limited != value,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LimitResult {
    pub value: f64,
    pub saturated: bool,
}
