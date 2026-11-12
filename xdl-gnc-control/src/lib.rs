// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Bounded, deterministic control primitives for the XDL GNC vertical slice.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{GncError, ScalarLimit, SignalStatus, StepEvent, StepResult, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PidConfig {
    pub proportional_gain: f64,
    pub integral_gain: f64,
    pub derivative_gain: f64,
    pub output_limit: ScalarLimit,
    pub integral_limit: ScalarLimit,
}

impl PidConfig {
    pub fn new(
        proportional_gain: f64,
        integral_gain: f64,
        derivative_gain: f64,
        output_limit: ScalarLimit,
        integral_limit: ScalarLimit,
    ) -> Result<Self, GncError> {
        let gains = [proportional_gain, integral_gain, derivative_gain];
        if !gains.iter().all(|gain| gain.is_finite()) {
            return Err(GncError::NonFinite {
                context: "PID gains".to_string(),
            });
        }
        Ok(Self {
            proportional_gain,
            integral_gain,
            derivative_gain,
            output_limit,
            integral_limit,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PidOutput {
    pub command: f64,
    pub error: f64,
    pub derivative: f64,
    pub integral: f64,
    pub status: SignalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PidController {
    config: PidConfig,
    integral: f64,
    previous_error: Option<f64>,
    previous_timestamp: Option<Timestamp>,
}

impl PidController {
    pub const fn new(config: PidConfig) -> Self {
        Self {
            config,
            integral: 0.0,
            previous_error: None,
            previous_timestamp: None,
        }
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = None;
        self.previous_timestamp = None;
    }

    pub fn step(
        &mut self,
        timestamp: Timestamp,
        setpoint: f64,
        measurement: f64,
    ) -> Result<StepResult<PidOutput>, GncError> {
        if !setpoint.is_finite() || !measurement.is_finite() {
            return Err(GncError::NonFinite {
                context: "PID input".to_string(),
            });
        }
        let error = setpoint - measurement;
        let (dt, derivative) = match (self.previous_timestamp, self.previous_error) {
            (Some(previous_timestamp), Some(previous_error)) => {
                let duration = timestamp.duration_since(previous_timestamp)?;
                if !duration.is_positive() {
                    return Err(GncError::InvalidTime(
                        "PID timestamps must be strictly increasing".to_string(),
                    ));
                }
                let seconds = duration.as_seconds();
                (seconds, (error - previous_error) / seconds)
            }
            (None, None) => (0.0, 0.0),
            _ => {
                return Err(GncError::InvalidValue {
                    context: "PID state".to_string(),
                    reason: "timestamp and previous error must be initialized together".to_string(),
                });
            }
        };

        let previous_integral = self.integral;
        let candidate_integral = if dt > 0.0 {
            self.config
                .integral_limit
                .apply(previous_integral + error * dt)?
                .value
        } else {
            previous_integral
        };
        let unclamped = self.config.proportional_gain * error
            + self.config.integral_gain * candidate_integral
            + self.config.derivative_gain * derivative;
        let limited = self.config.output_limit.apply(unclamped)?;
        let saturated_in_direction = limited.saturated
            && ((unclamped > self.config.output_limit.max && error > 0.0)
                || (unclamped < self.config.output_limit.min && error < 0.0));
        self.integral = if saturated_in_direction {
            previous_integral
        } else {
            candidate_integral
        };
        self.previous_error = Some(error);
        self.previous_timestamp = Some(timestamp);

        let mut events = Vec::new();
        if limited.saturated {
            events.push(StepEvent::Saturated);
        }
        Ok(StepResult {
            value: PidOutput {
                command: limited.value,
                error,
                derivative,
                integral: self.integral,
                status: SignalStatus::nominal(timestamp),
            },
            status: SignalStatus::nominal(timestamp),
            events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn controller() -> PidController {
        let output_limit = ScalarLimit::new(-1.0, 1.0).expect("valid output limit");
        let integral_limit = ScalarLimit::new(-10.0, 10.0).expect("valid integral limit");
        let config =
            PidConfig::new(1.0, 1.0, 0.0, output_limit, integral_limit).expect("valid PID config");
        PidController::new(config)
    }

    #[test]
    fn pid_reports_saturation_and_anti_windup() {
        let mut controller = controller();
        let first = controller
            .step(Timestamp::ZERO, 10.0, 0.0)
            .expect("initial step");
        let second = controller
            .step(
                Timestamp::from_seconds(1.0).expect("finite timestamp"),
                10.0,
                0.0,
            )
            .expect("second step");
        assert_eq!(first.value.command, 1.0);
        assert_eq!(second.value.command, 1.0);
        assert!(second.events.contains(&StepEvent::Saturated));
        assert_eq!(second.value.integral, 0.0);
    }

    #[test]
    fn pid_derivative_uses_timestamp_delta() {
        let mut controller = controller();
        controller
            .step(Timestamp::ZERO, 0.0, 0.0)
            .expect("initial step");
        let output = controller
            .step(
                Timestamp::from_seconds(0.5).expect("finite timestamp"),
                1.0,
                0.0,
            )
            .expect("second step");
        assert!((output.value.derivative - 2.0).abs() < 1e-12);
    }
}
