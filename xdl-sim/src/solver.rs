// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic fixed-step continuous-time integration primitives.

use thiserror::Error;

const MAX_STATE_ELEMENTS: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Error)]
pub enum SolverError {
    #[error("solver step must be finite and positive")]
    InvalidStep,
    #[error("adaptive solver configuration is invalid")]
    InvalidConfiguration,
    #[error("state dimension {0} exceeds the solver bound")]
    StateTooLarge(usize),
    #[error("state and derivative dimensions differ")]
    DimensionMismatch,
    #[error("solver derivative returned a non-finite value at index {index}")]
    NonFiniteDerivative { index: usize },
    #[error("solver result is non-finite at index {index}")]
    NonFiniteResult { index: usize },
    #[error("integration interval must be finite and strictly increasing")]
    InvalidInterval,
    #[error("adaptive integration exceeded its step bound ({0})")]
    StepLimitExceeded(usize),
    #[error("adaptive integration exceeded {retries} retries at time {time}")]
    RetryLimitExceeded { time: f64, retries: usize },
    #[error("adaptive integration cannot meet tolerance at the minimum step at time {0}")]
    MinimumStepReached(f64),
    #[error("adaptive integration time step made no representable progress at time {0}")]
    TimeDidNotAdvance(f64),
}

pub type DerivativeFn<'a> = dyn FnMut(f64, &[f64], &mut [f64]) + 'a;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AdaptiveConfig {
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
    pub minimum_step: f64,
    pub maximum_step: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveStep {
    pub state: Vec<f64>,
    pub accepted_step: f64,
    pub next_step: f64,
    pub normalized_error: f64,
}

/// A time/state pair emitted by bounded adaptive integration.
#[derive(Debug, Clone, PartialEq)]
pub struct IntegrationSample {
    pub time: f64,
    pub state: Vec<f64>,
}

/// Result of integrating a finite interval with accepted RK4 steps.
#[derive(Debug, Clone, PartialEq)]
pub struct AdaptiveIntegration {
    /// Includes the initial state and the exact requested end time.
    pub samples: Vec<IntegrationSample>,
    pub accepted_steps: usize,
    pub rejected_steps: usize,
}

impl AdaptiveConfig {
    pub fn validate(self) -> Result<Self, SolverError> {
        if !self.absolute_tolerance.is_finite()
            || self.absolute_tolerance <= 0.0
            || !self.relative_tolerance.is_finite()
            || self.relative_tolerance < 0.0
            || !self.minimum_step.is_finite()
            || self.minimum_step <= 0.0
            || !self.maximum_step.is_finite()
            || self.maximum_step < self.minimum_step
        {
            Err(SolverError::InvalidConfiguration)
        } else {
            Ok(self)
        }
    }
}

fn validate_state(state: &[f64]) -> Result<(), SolverError> {
    if state.len() > MAX_STATE_ELEMENTS {
        return Err(SolverError::StateTooLarge(state.len()));
    }
    if let Some(index) = state.iter().position(|value| !value.is_finite()) {
        return Err(SolverError::NonFiniteResult { index });
    }
    Ok(())
}

fn validate_derivative(derivative: &[f64]) -> Result<(), SolverError> {
    derivative
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            (!value.is_finite()).then_some(SolverError::NonFiniteDerivative { index })
        })
        .map_or(Ok(()), Err)
}

fn validate_step(step: f64) -> Result<(), SolverError> {
    if step.is_finite() && step > 0.0 {
        Ok(())
    } else {
        Err(SolverError::InvalidStep)
    }
}

/// Integrate `y' = f(t, y)` one step with explicit Euler.
pub fn euler_step(
    time: f64,
    step: f64,
    state: &[f64],
    derivative: &mut DerivativeFn<'_>,
) -> Result<Vec<f64>, SolverError> {
    validate_step(step)?;
    validate_state(state)?;
    let mut slope = vec![0.0; state.len()];
    derivative(time, state, &mut slope);
    validate_derivative(&slope)?;
    let result: Vec<_> = state
        .iter()
        .zip(slope)
        .map(|(value, slope)| value + step * slope)
        .collect();
    validate_result(&result)
}

/// Integrate `y' = f(t, y)` one step with classical fourth-order Runge–Kutta.
pub fn rk4_step(
    time: f64,
    step: f64,
    state: &[f64],
    derivative: &mut DerivativeFn<'_>,
) -> Result<Vec<f64>, SolverError> {
    validate_step(step)?;
    validate_state(state)?;
    let mut k1 = vec![0.0; state.len()];
    let mut k2 = vec![0.0; state.len()];
    let mut k3 = vec![0.0; state.len()];
    let mut k4 = vec![0.0; state.len()];
    let mut intermediate = state.to_vec();

    derivative(time, state, &mut k1);
    validate_derivative(&k1)?;
    intermediate
        .iter_mut()
        .zip(&k1)
        .for_each(|(value, slope)| *value += 0.5 * step * slope);
    derivative(time + 0.5 * step, &intermediate, &mut k2);
    validate_derivative(&k2)?;
    intermediate.copy_from_slice(state);
    intermediate
        .iter_mut()
        .zip(&k2)
        .for_each(|(value, slope)| *value += 0.5 * step * slope);
    derivative(time + 0.5 * step, &intermediate, &mut k3);
    validate_derivative(&k3)?;
    intermediate.copy_from_slice(state);
    intermediate
        .iter_mut()
        .zip(&k3)
        .for_each(|(value, slope)| *value += step * slope);
    derivative(time + step, &intermediate, &mut k4);
    validate_derivative(&k4)?;

    let result: Vec<_> = state
        .iter()
        .zip(&k1)
        .zip(&k2)
        .zip(&k3)
        .zip(&k4)
        .map(|((((value, k1), k2), k3), k4)| value + step * (k1 + 2.0 * k2 + 2.0 * k3 + k4) / 6.0)
        .collect();
    validate_result(&result)
}

/// Integrate one step using RK4 step-doubling for a deterministic error estimate.
///
/// If the error is too large, the state is returned as a rejected full-step
/// estimate and `accepted_step` is zero. The caller retries with `next_step`;
/// this function never hides retries in an unbounded loop.
pub fn rk4_adaptive_step(
    time: f64,
    proposed_step: f64,
    state: &[f64],
    config: AdaptiveConfig,
    derivative: &mut DerivativeFn<'_>,
) -> Result<AdaptiveStep, SolverError> {
    let config = config.validate()?;
    validate_step(proposed_step)?;
    validate_state(state)?;
    let step = proposed_step.clamp(config.minimum_step, config.maximum_step);
    let full = rk4_step(time, step, state, derivative)?;
    let half = rk4_step(time, step * 0.5, state, derivative)?;
    let refined = rk4_step(time + step * 0.5, step * 0.5, &half, derivative)?;
    let normalized_error = state
        .iter()
        .zip(&full)
        .zip(&refined)
        .map(|((previous, coarse), fine)| {
            let scale = config.absolute_tolerance
                + config.relative_tolerance * previous.abs().max(fine.abs());
            (fine - coarse).abs() / scale
        })
        .fold(0.0, f64::max);
    let accepted = normalized_error <= 1.0;
    let factor = if normalized_error == 0.0 {
        2.0
    } else {
        (0.9 * normalized_error.powf(-0.2)).clamp(0.2, 2.0)
    };
    let next_step = (step * factor).clamp(config.minimum_step, config.maximum_step);
    Ok(AdaptiveStep {
        state: if accepted { refined } else { full },
        accepted_step: if accepted { step } else { 0.0 },
        next_step,
        normalized_error,
    })
}

/// Integrate `y' = f(t, y)` over `[start_time, end_time]` using bounded,
/// step-doubling RK4. Rejected steps retry from the same time/state; retries
/// per attempted step and total attempted steps are explicitly bounded.
///
/// The final proposed step is clipped to the remaining interval. This endpoint
/// step may be smaller than `config.minimum_step`; if it is rejected and cannot
/// be reduced within the configured minimum, integration returns an error.
pub fn integrate_rk4_adaptive(
    start_time: f64,
    end_time: f64,
    initial_step: f64,
    initial_state: &[f64],
    config: AdaptiveConfig,
    max_steps: usize,
    max_retries_per_step: usize,
    derivative: &mut DerivativeFn<'_>,
) -> Result<AdaptiveIntegration, SolverError> {
    let config = config.validate()?;
    validate_state(initial_state)?;
    if !start_time.is_finite() || !end_time.is_finite() || end_time <= start_time {
        return Err(SolverError::InvalidInterval);
    }
    validate_step(initial_step)?;
    if max_steps == 0 {
        return Err(SolverError::StepLimitExceeded(max_steps));
    }

    let mut time = start_time;
    let mut state = initial_state.to_vec();
    let mut proposed_step = initial_step.clamp(config.minimum_step, config.maximum_step);
    let mut samples = vec![IntegrationSample {
        time,
        state: state.clone(),
    }];
    let mut accepted_steps = 0;
    let mut rejected_steps = 0;
    let mut retries_at_time = 0;

    while time < end_time {
        if accepted_steps + rejected_steps >= max_steps {
            return Err(SolverError::StepLimitExceeded(max_steps));
        }
        let remaining = end_time - time;
        let clipped = proposed_step.min(remaining);
        let endpoint_clipped = clipped == remaining;
        if clipped <= 0.0 || time + clipped <= time {
            return Err(SolverError::TimeDidNotAdvance(time));
        }

        // rk4_adaptive_step applies min/max bounds. Permit an endpoint-clipped
        // step below the regular minimum without relaxing normal steps.
        let step_config = if endpoint_clipped && clipped < config.minimum_step {
            AdaptiveConfig {
                minimum_step: clipped,
                maximum_step: clipped,
                ..config
            }
        } else {
            config
        };
        let attempt = rk4_adaptive_step(time, clipped, &state, step_config, derivative)?;
        if attempt.accepted_step > 0.0 {
            let next_time = if endpoint_clipped || clipped == remaining {
                end_time
            } else {
                time + clipped
            };
            time = next_time;
            state = attempt.state;
            samples.push(IntegrationSample {
                time,
                state: state.clone(),
            });
            accepted_steps += 1;
            retries_at_time = 0;
            proposed_step = attempt
                .next_step
                .clamp(config.minimum_step, config.maximum_step);
        } else {
            rejected_steps += 1;
            retries_at_time += 1;
            if retries_at_time > max_retries_per_step {
                return Err(SolverError::RetryLimitExceeded {
                    time,
                    retries: max_retries_per_step,
                });
            }
            if attempt.next_step >= clipped || attempt.next_step < config.minimum_step {
                return Err(SolverError::MinimumStepReached(time));
            }
            proposed_step = attempt.next_step;
        }
    }

    Ok(AdaptiveIntegration {
        samples,
        accepted_steps,
        rejected_steps,
    })
}

fn validate_result(result: &[f64]) -> Result<Vec<f64>, SolverError> {
    result
        .iter()
        .enumerate()
        .find_map(|(index, value)| {
            (!value.is_finite()).then_some(SolverError::NonFiniteResult { index })
        })
        .map_or_else(|| Ok(result.to_vec()), Err)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn euler_integrates_constant_rate() {
        let result = euler_step(0.0, 0.25, &[2.0], &mut |_, _, output| output[0] = 4.0).unwrap();
        assert_eq!(result, vec![3.0]);
    }

    #[test]
    fn rk4_integrates_exponential_growth() {
        let result = rk4_step(0.0, 0.1, &[1.0], &mut |_, state, output| {
            output[0] = state[0]
        })
        .unwrap();
        assert!((result[0] - 1.1051708).abs() < 1e-6);
    }

    #[test]
    fn invalid_step_and_nonfinite_derivative_are_rejected() {
        assert_eq!(
            euler_step(0.0, 0.0, &[1.0], &mut |_, _, output| output[0] = 1.0),
            Err(SolverError::InvalidStep)
        );
        assert_eq!(
            euler_step(0.0, 1.0, &[1.0], &mut |_, _, output| output[0] = f64::NAN),
            Err(SolverError::NonFiniteDerivative { index: 0 })
        );
    }

    #[test]
    fn adaptive_step_accepts_easy_problem_and_grows_step() {
        let config = AdaptiveConfig {
            absolute_tolerance: 1e-6,
            relative_tolerance: 1e-6,
            minimum_step: 0.01,
            maximum_step: 1.0,
        };
        let result = rk4_adaptive_step(0.0, 0.1, &[1.0], config, &mut |_, state, output| {
            output[0] = state[0];
        })
        .unwrap();
        assert!(result.accepted_step > 0.0);
        assert!(result.next_step > result.accepted_step);
        assert!((result.state[0] - 1.1051709).abs() < 1e-6);
    }

    #[test]
    fn adaptive_step_reports_rejection_without_hidden_retry() {
        let config = AdaptiveConfig {
            absolute_tolerance: 1e-14,
            relative_tolerance: 0.0,
            minimum_step: 0.1,
            maximum_step: 1.0,
        };
        let result = rk4_adaptive_step(0.0, 1.0, &[1.0], config, &mut |_, state, output| {
            output[0] = state[0] * state[0] * state[0];
        })
        .unwrap();
        assert_eq!(result.accepted_step, 0.0);
        assert!(result.next_step < 1.0);
        assert!(result.normalized_error > 1.0);
    }

    fn adaptive_config(minimum_step: f64, maximum_step: f64) -> AdaptiveConfig {
        AdaptiveConfig {
            absolute_tolerance: 1e-8,
            relative_tolerance: 1e-8,
            minimum_step,
            maximum_step,
        }
    }

    #[test]
    fn bounded_integrator_returns_samples_and_exact_endpoint() {
        let result = integrate_rk4_adaptive(
            0.0,
            1.0,
            0.2,
            &[1.0],
            adaptive_config(0.01, 0.25),
            100,
            20,
            &mut |_, state, output| output[0] = state[0],
        )
        .unwrap();

        assert_eq!(result.samples.first().unwrap().time, 0.0);
        assert_eq!(result.samples.last().unwrap().time, 1.0);
        assert_eq!(result.samples.len(), result.accepted_steps + 1);
        assert!(result.accepted_steps > 0);
        assert!((result.samples.last().unwrap().state[0] - std::f64::consts::E).abs() < 1e-6);
        assert!(result
            .samples
            .windows(2)
            .all(|pair| pair[1].time > pair[0].time));
    }

    #[test]
    fn bounded_integrator_retries_rejected_steps_from_same_state() {
        let mut seen_times = Vec::new();
        let result = integrate_rk4_adaptive(
            0.0,
            0.1,
            0.1,
            &[1.0],
            AdaptiveConfig {
                absolute_tolerance: 1e-12,
                relative_tolerance: 0.0,
                minimum_step: 1e-5,
                maximum_step: 0.1,
            },
            100,
            20,
            &mut |time, state, output| {
                seen_times.push((time, state[0]));
                output[0] = state[0] * state[0] * state[0];
            },
        )
        .unwrap();

        assert!(result.rejected_steps > 0);
        assert_eq!(result.samples.last().unwrap().time, 0.1);
        assert!(
            seen_times
                .iter()
                .filter(|(time, value)| *time == 0.0 && *value == 1.0)
                .count()
                > 1
        );
    }

    #[test]
    fn bounded_integrator_enforces_retry_and_total_step_bounds() {
        let retry_error = integrate_rk4_adaptive(
            0.0,
            1.0,
            1.0,
            &[1.0],
            AdaptiveConfig {
                absolute_tolerance: 1e-14,
                relative_tolerance: 0.0,
                minimum_step: 0.1,
                maximum_step: 1.0,
            },
            10,
            0,
            &mut |_, state, output| output[0] = state[0].powi(3),
        );
        assert_eq!(
            retry_error,
            Err(SolverError::RetryLimitExceeded {
                time: 0.0,
                retries: 0
            })
        );

        let step_error = integrate_rk4_adaptive(
            0.0,
            1.0,
            0.1,
            &[0.0],
            adaptive_config(0.1, 0.1),
            1,
            2,
            &mut |_, _, output| output[0] = 0.0,
        );
        assert_eq!(step_error, Err(SolverError::StepLimitExceeded(1)));
    }

    #[test]
    fn bounded_integrator_clips_endpoint_below_minimum_step() {
        let result = integrate_rk4_adaptive(
            0.0,
            0.15,
            0.1,
            &[0.0],
            adaptive_config(0.1, 0.1),
            4,
            2,
            &mut |_, _, output| output[0] = 2.0,
        )
        .unwrap();

        assert_eq!(
            result
                .samples
                .iter()
                .map(|sample| sample.time)
                .collect::<Vec<_>>(),
            vec![0.0, 0.1, 0.15]
        );
        assert!((result.samples.last().unwrap().state[0] - 0.3).abs() < f64::EPSILON * 2.0);
    }

    #[test]
    fn bounded_integrator_rejects_invalid_interval_and_reports_unachievable_minimum() {
        assert_eq!(
            integrate_rk4_adaptive(
                1.0,
                1.0,
                0.1,
                &[1.0],
                adaptive_config(0.1, 1.0),
                10,
                2,
                &mut |_, _, out| out[0] = 0.0
            ),
            Err(SolverError::InvalidInterval)
        );
        assert_eq!(
            integrate_rk4_adaptive(
                0.0,
                0.05,
                0.1,
                &[1.0],
                AdaptiveConfig {
                    absolute_tolerance: 1e-14,
                    relative_tolerance: 0.0,
                    minimum_step: 0.1,
                    maximum_step: 1.0
                },
                10,
                5,
                &mut |_, state, out| out[0] = state[0].powi(3),
            ),
            Err(SolverError::MinimumStepReached(0.0))
        );
    }
}
