// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic actuator models with explicit saturation and rate limiting.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{GncError, ScalarLimit, SignalStatus, StepEvent, StepResult, Timestamp};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderActuatorConfig {
    pub time_constant_seconds: f64,
    pub command_limit: ScalarLimit,
    pub rate_limit_per_second: f64,
}

impl FirstOrderActuatorConfig {
    pub fn new(
        time_constant_seconds: f64,
        command_limit: ScalarLimit,
        rate_limit_per_second: f64,
    ) -> Result<Self, GncError> {
        if !time_constant_seconds.is_finite() || !rate_limit_per_second.is_finite() {
            return Err(GncError::NonFinite {
                context: "actuator configuration".to_string(),
            });
        }
        if time_constant_seconds <= 0.0 || rate_limit_per_second <= 0.0 {
            return Err(GncError::InvalidValue {
                context: "actuator configuration".to_string(),
                reason: "time constant and rate limit must be positive".to_string(),
            });
        }
        Ok(Self {
            time_constant_seconds,
            command_limit,
            rate_limit_per_second,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FirstOrderActuator {
    config: FirstOrderActuatorConfig,
    timestamp: Timestamp,
    output: f64,
}

impl FirstOrderActuator {
    pub fn new(
        config: FirstOrderActuatorConfig,
        timestamp: Timestamp,
        initial_output: f64,
    ) -> Result<Self, GncError> {
        if !initial_output.is_finite() {
            return Err(GncError::NonFinite {
                context: "actuator initial output".to_string(),
            });
        }
        let initial_output = config.command_limit.apply(initial_output)?.value;
        Ok(Self {
            config,
            timestamp,
            output: initial_output,
        })
    }

    /// Rehydrates an actuator for a stateless host boundary such as XDL.
    pub fn with_state(
        config: FirstOrderActuatorConfig,
        timestamp: Timestamp,
        output: f64,
    ) -> Result<Self, GncError> {
        Self::new(config, timestamp, output)
    }

    pub fn output(&self) -> f64 {
        self.output
    }

    pub fn step(
        &mut self,
        timestamp: Timestamp,
        command: f64,
    ) -> Result<StepResult<f64>, GncError> {
        if !command.is_finite() {
            return Err(GncError::NonFinite {
                context: "actuator command".to_string(),
            });
        }
        let dt = timestamp.duration_since(self.timestamp)?;
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "actuator timestamps must be strictly increasing".to_string(),
            ));
        }
        let seconds = dt.as_seconds();
        let limited_command = self.config.command_limit.apply(command)?;
        let alpha = seconds / (self.config.time_constant_seconds + seconds);
        let first_order_output = self.output + alpha * (limited_command.value - self.output);
        let maximum_delta = self.config.rate_limit_per_second * seconds;
        let requested_delta = first_order_output - self.output;
        let applied_delta = requested_delta.clamp(-maximum_delta, maximum_delta);
        let rate_limited = applied_delta != requested_delta;
        self.output += applied_delta;
        self.timestamp = timestamp;

        let mut events = Vec::new();
        if limited_command.saturated {
            events.push(StepEvent::Saturated);
        }
        if rate_limited {
            events.push(StepEvent::RateLimited);
        }
        Ok(StepResult {
            value: self.output,
            status: SignalStatus::nominal(timestamp),
            events,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn actuator() -> FirstOrderActuator {
        let limit = ScalarLimit::new(-1.0, 1.0).expect("valid command limit");
        let config =
            FirstOrderActuatorConfig::new(1.0, limit, 0.25).expect("valid actuator config");
        FirstOrderActuator::new(config, Timestamp::ZERO, 0.0).expect("valid actuator")
    }

    #[test]
    fn actuator_reports_command_saturation() {
        let mut actuator = actuator();
        let result = actuator
            .step(
                Timestamp::from_seconds(1.0).expect("finite timestamp"),
                10.0,
            )
            .expect("valid step");
        assert!(result.events.contains(&StepEvent::Saturated));
        assert!(result.value <= 1.0);
    }

    #[test]
    fn actuator_reports_rate_limiting() {
        let mut actuator = actuator();
        let result = actuator
            .step(Timestamp::from_seconds(1.0).expect("finite timestamp"), 1.0)
            .expect("valid step");
        assert!(result.events.contains(&StepEvent::RateLimited));
        assert!((result.value - 0.25).abs() < 1e-12);
    }

    #[test]
    fn actuator_rejects_non_monotonic_time() {
        let mut actuator = actuator();
        assert!(actuator.step(Timestamp::ZERO, 0.0).is_err());
    }
}
