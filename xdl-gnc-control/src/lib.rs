// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Bounded, deterministic control primitives for the XDL GNC vertical slice.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{GncError, ScalarLimit, SignalStatus, StepEvent, StepResult, Timestamp};

pub type Matrix<const ROWS: usize, const COLUMNS: usize> = [[f64; COLUMNS]; ROWS];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct DiscreteStateSpace<const STATES: usize, const INPUTS: usize, const OUTPUTS: usize> {
    pub a: Matrix<STATES, STATES>,
    pub b: Matrix<STATES, INPUTS>,
    pub c: Matrix<OUTPUTS, STATES>,
    pub d: Matrix<OUTPUTS, INPUTS>,
    pub state: [f64; STATES],
}

impl<const STATES: usize, const INPUTS: usize, const OUTPUTS: usize>
    DiscreteStateSpace<STATES, INPUTS, OUTPUTS>
{
    pub fn new(
        a: Matrix<STATES, STATES>,
        b: Matrix<STATES, INPUTS>,
        c: Matrix<OUTPUTS, STATES>,
        d: Matrix<OUTPUTS, INPUTS>,
        initial_state: [f64; STATES],
    ) -> Result<Self, GncError> {
        if !matrix_is_finite(&a)
            || !matrix_is_finite(&b)
            || !matrix_is_finite(&c)
            || !matrix_is_finite(&d)
            || !initial_state.iter().all(|value| value.is_finite())
        {
            return Err(GncError::NonFinite {
                context: "discrete state-space model".to_string(),
            });
        }
        Ok(Self {
            a,
            b,
            c,
            d,
            state: initial_state,
        })
    }

    pub fn reset(&mut self, state: [f64; STATES]) -> Result<(), GncError> {
        if !state.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "state-space state".to_string(),
            });
        }
        self.state = state;
        Ok(())
    }

    pub fn step(&mut self, input: [f64; INPUTS]) -> Result<[f64; OUTPUTS], GncError> {
        if !input.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "state-space input".to_string(),
            });
        }
        let output = add_vectors(mat_vec(&self.c, &self.state), mat_vec(&self.d, &input));
        self.state = add_vectors(mat_vec(&self.a, &self.state), mat_vec(&self.b, &input));
        Ok(output)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct LinearGainSchedule<const INPUTS: usize, const STATES: usize> {
    pub lower_parameter: f64,
    pub upper_parameter: f64,
    pub lower_gain: Matrix<INPUTS, STATES>,
    pub upper_gain: Matrix<INPUTS, STATES>,
}

impl<const INPUTS: usize, const STATES: usize> LinearGainSchedule<INPUTS, STATES> {
    pub fn new(
        lower_parameter: f64,
        upper_parameter: f64,
        lower_gain: Matrix<INPUTS, STATES>,
        upper_gain: Matrix<INPUTS, STATES>,
    ) -> Result<Self, GncError> {
        if !lower_parameter.is_finite()
            || !upper_parameter.is_finite()
            || !matrix_is_finite(&lower_gain)
            || !matrix_is_finite(&upper_gain)
        {
            return Err(GncError::NonFinite {
                context: "gain schedule".to_string(),
            });
        }
        if lower_parameter >= upper_parameter {
            return Err(GncError::InvalidValue {
                context: "gain schedule".to_string(),
                reason: "lower parameter must be less than upper parameter".to_string(),
            });
        }
        Ok(Self {
            lower_parameter,
            upper_parameter,
            lower_gain,
            upper_gain,
        })
    }

    pub fn gain(&self, parameter: f64) -> Result<Matrix<INPUTS, STATES>, GncError> {
        if !parameter.is_finite() {
            return Err(GncError::NonFinite {
                context: "gain scheduling parameter".to_string(),
            });
        }
        let fraction = ((parameter - self.lower_parameter)
            / (self.upper_parameter - self.lower_parameter))
            .clamp(0.0, 1.0);
        let mut gain = [[0.0; STATES]; INPUTS];
        for row in 0..INPUTS {
            for column in 0..STATES {
                gain[row][column] = self.lower_gain[row][column]
                    + fraction * (self.upper_gain[row][column] - self.lower_gain[row][column]);
            }
        }
        Ok(gain)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ScheduledStateFeedback<const INPUTS: usize, const STATES: usize> {
    schedule: LinearGainSchedule<INPUTS, STATES>,
}

impl<const INPUTS: usize, const STATES: usize> ScheduledStateFeedback<INPUTS, STATES> {
    pub const fn new(schedule: LinearGainSchedule<INPUTS, STATES>) -> Self {
        Self { schedule }
    }

    pub fn step(
        &self,
        timestamp: Timestamp,
        parameter: f64,
        state_error: [f64; STATES],
    ) -> Result<StepResult<[f64; INPUTS]>, GncError> {
        if !state_error.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "scheduled feedback state error".to_string(),
            });
        }
        let gain = self.schedule.gain(parameter)?;
        let command = mat_vec(&gain, &state_error);
        Ok(StepResult::nominal(command, timestamp))
    }
}

fn matrix_is_finite<const ROWS: usize, const COLUMNS: usize>(
    matrix: &Matrix<ROWS, COLUMNS>,
) -> bool {
    matrix.iter().flatten().all(|value| value.is_finite())
}

fn mat_vec<const ROWS: usize, const COLUMNS: usize>(
    matrix: &Matrix<ROWS, COLUMNS>,
    vector: &[f64; COLUMNS],
) -> [f64; ROWS] {
    core::array::from_fn(|row| {
        (0..COLUMNS)
            .map(|column| matrix[row][column] * vector[column])
            .sum()
    })
}

fn add_vectors<const SIZE: usize>(left: [f64; SIZE], right: [f64; SIZE]) -> [f64; SIZE] {
    core::array::from_fn(|index| left[index] + right[index])
}

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

    #[test]
    fn discrete_state_space_updates_state_after_computing_output() {
        let mut model =
            DiscreteStateSpace::<1, 1, 1>::new([[1.0]], [[1.0]], [[1.0]], [[0.0]], [0.0])
                .expect("finite state-space model");
        assert_eq!(model.step([2.0]).expect("valid step"), [0.0]);
        assert_eq!(model.step([2.0]).expect("valid step"), [2.0]);
    }

    #[test]
    fn gain_schedule_interpolates_and_clamps_endpoints() {
        let schedule =
            LinearGainSchedule::new(0.0, 10.0, [[1.0, 0.0]], [[3.0, 2.0]]).expect("valid schedule");
        assert_eq!(schedule.gain(-1.0).expect("lower clamp"), [[1.0, 0.0]]);
        assert_eq!(schedule.gain(5.0).expect("interpolation"), [[2.0, 1.0]]);
        assert_eq!(schedule.gain(11.0).expect("upper clamp"), [[3.0, 2.0]]);
    }

    #[test]
    fn scheduled_feedback_returns_interpolated_command() {
        let schedule =
            LinearGainSchedule::new(0.0, 10.0, [[1.0, 0.0]], [[3.0, 2.0]]).expect("valid schedule");
        let controller = ScheduledStateFeedback::new(schedule);
        let result = controller
            .step(Timestamp::ZERO, 5.0, [2.0, 4.0])
            .expect("valid feedback");
        assert_eq!(result.value, [8.0]);
    }

    #[test]
    fn state_space_rejects_non_finite_input() {
        let mut model =
            DiscreteStateSpace::<1, 1, 1>::new([[1.0]], [[1.0]], [[1.0]], [[0.0]], [0.0])
                .expect("finite state-space model");
        assert!(model.step([f64::NAN]).is_err());
    }
}
