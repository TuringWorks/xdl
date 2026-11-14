// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic fixed-step continuous-time integration primitives.

use thiserror::Error;

const MAX_STATE_ELEMENTS: usize = 1_000_000;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
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
}
