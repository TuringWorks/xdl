// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Stable, dependency-light telemetry records for GNC products.
//!
//! The exporter uses JSON Lines so records can be streamed to a file, pipe, or
//! host application without buffering an entire flight run.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{FrameId, GncError, StepEvent, Timestamp, Vec3};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TelemetrySample {
    pub timestamp: Timestamp,
    pub sequence: u64,
    pub source: String,
    pub frame: Option<FrameId>,
    pub position: Option<Vec3>,
    pub velocity: Option<Vec3>,
    pub command: Option<f64>,
    pub events: Vec<StepEvent>,
}

impl TelemetrySample {
    pub fn new(timestamp: Timestamp, sequence: u64, source: impl Into<String>) -> Self {
        Self {
            timestamp,
            sequence,
            source: source.into(),
            frame: None,
            position: None,
            velocity: None,
            command: None,
            events: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct TelemetryLog {
    samples: Vec<TelemetrySample>,
}

impl TelemetryLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, sample: TelemetrySample) -> Result<(), GncError> {
        if let Some(previous) = self.samples.last() {
            if (sample.timestamp, sample.sequence) <= (previous.timestamp, previous.sequence) {
                return Err(GncError::InvalidTime(
                    "telemetry samples must be ordered by timestamp and sequence".to_string(),
                ));
            }
        }
        if !sample.command.is_none_or(f64::is_finite)
            || sample.position.is_some_and(|value| !value.is_finite())
            || sample.velocity.is_some_and(|value| !value.is_finite())
        {
            return Err(GncError::NonFinite {
                context: "telemetry sample".to_string(),
            });
        }
        self.samples.push(sample);
        Ok(())
    }

    pub fn samples(&self) -> &[TelemetrySample] {
        &self.samples
    }

    pub fn is_empty(&self) -> bool {
        self.samples.is_empty()
    }

    pub fn to_json_lines(&self) -> Result<String, GncError> {
        self.samples
            .iter()
            .map(|sample| {
                serde_json::to_string(sample).map_err(|error| GncError::InvalidValue {
                    context: "telemetry JSON export".to_string(),
                    reason: error.to_string(),
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|lines| {
                if lines.is_empty() {
                    String::new()
                } else {
                    format!("{}\n", lines.join("\n"))
                }
            })
    }

    pub fn from_json_lines(input: &str) -> Result<Self, GncError> {
        let mut log = Self::new();
        for line in input.lines().filter(|line| !line.trim().is_empty()) {
            let sample = serde_json::from_str(line).map_err(|error| GncError::InvalidValue {
                context: "telemetry JSON import".to_string(),
                reason: error.to_string(),
            })?;
            log.push(sample)?;
        }
        Ok(log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_lines_round_trip_preserves_order_and_values() {
        let mut log = TelemetryLog::new();
        let mut sample = TelemetrySample::new(Timestamp::from_nanoseconds(1), 0, "navigation");
        sample.frame = Some(FrameId::Inertial);
        sample.position = Some(Vec3::new(1.0, 2.0, 3.0));
        sample.events.push(StepEvent::DegradedInput);
        log.push(sample).expect("ordered sample");

        let encoded = log.to_json_lines().expect("JSON export");
        let decoded = TelemetryLog::from_json_lines(&encoded).expect("JSON import");
        assert_eq!(decoded, log);
        assert!(encoded.ends_with('\n'));
    }

    #[test]
    fn telemetry_rejects_invalid_order_and_non_finite_values() {
        let mut log = TelemetryLog::new();
        log.push(TelemetrySample::new(
            Timestamp::from_nanoseconds(2),
            0,
            "control",
        ))
        .expect("first sample");
        assert!(log
            .push(TelemetrySample::new(
                Timestamp::from_nanoseconds(1),
                0,
                "navigation"
            ))
            .is_err());

        let mut invalid = TelemetrySample::new(Timestamp::from_nanoseconds(3), 0, "control");
        invalid.command = Some(f64::NAN);
        assert!(log.push(invalid).is_err());
    }
}
