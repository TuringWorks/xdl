// SPDX-License-Identifier: GPL-2.0-only OR MIT
// Copyright (C) 2026 Ravindra Boddipalli

use thiserror::Error;

/// Errors that must remain explicit at a GNC library boundary.
#[derive(Debug, Clone, PartialEq, Error)]
pub enum GncError {
    #[error("non-finite value in {context}")]
    NonFinite { context: String },

    #[error("invalid value in {context}: {reason}")]
    InvalidValue { context: String, reason: String },

    #[error("invalid time operation: {0}")]
    InvalidTime(String),

    #[error("invalid limit: minimum {min} is greater than maximum {max}")]
    InvalidLimit { min: f64, max: f64 },

    #[error("zero-length vector cannot be normalized")]
    ZeroLengthVector,

    #[error("zero-length quaternion cannot be normalized")]
    ZeroLengthQuaternion,
}
