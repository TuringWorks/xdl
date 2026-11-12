// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Typed guidance primitives for the XDL GNC vertical slice.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{GncError, Quality, SignalStatus, StepEvent, StepResult, Timestamp, Vec3};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaypointGuidanceConfig {
    pub cruise_speed: f64,
    pub acceptance_radius: f64,
}

impl WaypointGuidanceConfig {
    pub fn new(cruise_speed: f64, acceptance_radius: f64) -> Result<Self, GncError> {
        if !cruise_speed.is_finite() || !acceptance_radius.is_finite() {
            return Err(GncError::NonFinite {
                context: "waypoint guidance configuration".to_string(),
            });
        }
        if cruise_speed <= 0.0 || acceptance_radius < 0.0 {
            return Err(GncError::InvalidValue {
                context: "waypoint guidance configuration".to_string(),
                reason: "cruise speed must be positive and acceptance radius non-negative"
                    .to_string(),
            });
        }
        Ok(Self {
            cruise_speed,
            acceptance_radius,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GuidanceCommand {
    pub desired_velocity: Vec3,
    pub distance_to_waypoint: f64,
    pub reached: bool,
    pub status: SignalStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct WaypointGuidance {
    config: WaypointGuidanceConfig,
}

impl WaypointGuidance {
    pub const fn new(config: WaypointGuidanceConfig) -> Self {
        Self { config }
    }

    pub fn command(
        &self,
        timestamp: Timestamp,
        position: Vec3,
        waypoint: Vec3,
    ) -> Result<StepResult<GuidanceCommand>, GncError> {
        if !position.is_finite() || !waypoint.is_finite() {
            return Err(GncError::NonFinite {
                context: "waypoint guidance input".to_string(),
            });
        }
        let delta = waypoint - position;
        let distance = delta.norm();
        let reached = distance <= self.config.acceptance_radius;
        let desired_velocity = if reached {
            Vec3::ZERO
        } else {
            delta.try_normalize()? * self.config.cruise_speed
        };
        let status = SignalStatus {
            quality: if reached {
                Quality::Degraded
            } else {
                Quality::Nominal
            },
            timestamp,
        };
        let command = GuidanceCommand {
            desired_velocity,
            distance_to_waypoint: distance,
            reached,
            status,
        };
        let result = StepResult {
            value: command,
            status,
            events: Vec::new(),
        };
        Ok(if reached {
            result.with_event(StepEvent::TargetReached)
        } else {
            result
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn waypoint_command_points_toward_target_at_cruise_speed() {
        let config = WaypointGuidanceConfig::new(5.0, 0.5).expect("valid guidance config");
        let guidance = WaypointGuidance::new(config);
        let result = guidance
            .command(Timestamp::ZERO, Vec3::ZERO, Vec3::new(3.0, 4.0, 0.0))
            .expect("valid command");
        assert_eq!(result.value.desired_velocity, Vec3::new(3.0, 4.0, 0.0));
        assert!(!result.value.reached);
        assert!(result.events.is_empty());
    }

    #[test]
    fn waypoint_inside_acceptance_radius_stops() {
        let config = WaypointGuidanceConfig::new(5.0, 1.0).expect("valid guidance config");
        let guidance = WaypointGuidance::new(config);
        let result = guidance
            .command(Timestamp::ZERO, Vec3::ZERO, Vec3::new(0.5, 0.0, 0.0))
            .expect("valid command");
        assert_eq!(result.value.desired_velocity, Vec3::ZERO);
        assert!(result.value.reached);
        assert_eq!(result.events, vec![StepEvent::TargetReached]);
    }
}
