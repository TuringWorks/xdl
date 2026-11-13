// SPDX-License-Identifier: GPL-2.0-only OR MIT
// Copyright (C) 2026 Ravindra Boddipalli

use crate::GncError;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// `i64::MAX as f64` rounds up to exactly 2^63, so a `<=` test against it lets
/// 2^63 ns through and `as i64` then saturates silently. The valid range is
/// `[-2^63, 2^63)`, so the upper bound is an exclusive comparison.
const I64_LOWER_BOUND_NS: f64 = -9_223_372_036_854_775_808.0;
const I64_UPPER_BOUND_EXCLUSIVE_NS: f64 = 9_223_372_036_854_775_808.0;

fn nanos_fit_i64(nanos: f64) -> bool {
    (I64_LOWER_BOUND_NS..I64_UPPER_BOUND_EXCLUSIVE_NS).contains(&nanos)
}

/// Monotonic timestamp represented as signed nanoseconds from an application-defined epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Timestamp(i64);

impl Timestamp {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanoseconds(value: i64) -> Self {
        Self(value)
    }

    pub fn from_seconds(value: f64) -> Result<Self, GncError> {
        if !value.is_finite() {
            return Err(GncError::NonFinite {
                context: "timestamp seconds".to_string(),
            });
        }
        let nanos = value * 1_000_000_000.0;
        if !nanos_fit_i64(nanos) {
            return Err(GncError::InvalidTime(
                "timestamp is outside i64 nanoseconds".to_string(),
            ));
        }
        Ok(Self(nanos.round() as i64))
    }

    pub const fn as_nanoseconds(self) -> i64 {
        self.0
    }

    pub fn as_seconds(self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }

    pub fn checked_add(self, duration: DurationNs) -> Result<Self, GncError> {
        self.0
            .checked_add(duration.0)
            .map(Self)
            .ok_or_else(|| GncError::InvalidTime("timestamp addition overflow".to_string()))
    }

    pub fn duration_since(self, earlier: Self) -> Result<DurationNs, GncError> {
        self.0
            .checked_sub(earlier.0)
            .map(DurationNs)
            .ok_or_else(|| GncError::InvalidTime("timestamp subtraction overflow".to_string()))
    }
}

/// Signed nanosecond duration used for simulation and control steps.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DurationNs(i64);

impl DurationNs {
    pub const ZERO: Self = Self(0);

    pub const fn from_nanoseconds(value: i64) -> Self {
        Self(value)
    }

    pub fn from_seconds(value: f64) -> Result<Self, GncError> {
        if !value.is_finite() {
            return Err(GncError::NonFinite {
                context: "duration seconds".to_string(),
            });
        }
        let nanos = value * 1_000_000_000.0;
        if !nanos_fit_i64(nanos) {
            return Err(GncError::InvalidTime(
                "duration is outside i64 nanoseconds".to_string(),
            ));
        }
        Ok(Self(nanos.round() as i64))
    }

    pub const fn as_nanoseconds(self) -> i64 {
        self.0
    }

    pub fn as_seconds(self) -> f64 {
        self.0 as f64 / 1_000_000_000.0
    }

    pub fn is_positive(self) -> bool {
        self.0 > 0
    }

    pub fn try_as_std(self) -> Result<Duration, GncError> {
        u64::try_from(self.0)
            .map(Duration::from_nanos)
            .map_err(|_| {
                GncError::InvalidTime(
                    "negative duration cannot be converted to std::time::Duration".to_string(),
                )
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn timestamp_round_trips_seconds() {
        let timestamp = Timestamp::from_seconds(1.25).expect("finite timestamp");
        assert_eq!(timestamp.as_nanoseconds(), 1_250_000_000);
        assert_eq!(timestamp.as_seconds(), 1.25);
    }

    #[test]
    fn timestamp_from_seconds_rejects_values_at_or_above_i64_nanoseconds() {
        // 2^63 ns is about 9.223e9 s; 9.3e9 s is safely past it.
        assert!(Timestamp::from_seconds(9.3e9).is_err());
        assert!(Timestamp::from_seconds(1.0e10).is_err());
    }

    #[test]
    fn duration_from_seconds_rejects_values_below_i64_nanoseconds() {
        assert!(DurationNs::from_seconds(-9.3e9).is_err());
    }

    #[test]
    fn duration_rejects_negative_std_conversion() {
        let duration = DurationNs::from_nanoseconds(-1);
        assert!(duration.try_as_std().is_err());
    }
}
