// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

use crate::Timestamp;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Quality {
    Nominal,
    Degraded,
    Invalid,
    Stale,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignalStatus {
    pub quality: Quality,
    pub timestamp: Timestamp,
}

impl SignalStatus {
    pub const fn nominal(timestamp: Timestamp) -> Self {
        Self {
            quality: Quality::Nominal,
            timestamp,
        }
    }

    pub const fn is_usable(self) -> bool {
        matches!(self.quality, Quality::Nominal | Quality::Degraded)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepEvent {
    TargetReached,
    Saturated,
    RateLimited,
    StaleInput,
    DegradedInput,
    InvalidInput,
    NumericalWarning,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StepResult<T> {
    pub value: T,
    pub status: SignalStatus,
    pub events: Vec<StepEvent>,
}

impl<T> StepResult<T> {
    pub fn nominal(value: T, timestamp: Timestamp) -> Self {
        Self {
            value,
            status: SignalStatus::nominal(timestamp),
            events: Vec::new(),
        }
    }

    pub fn with_event(mut self, event: StepEvent) -> Self {
        self.events.push(event);
        self
    }
}
