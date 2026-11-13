// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Small reusable scalar blocks for model execution.

use crate::{BlockRuntime, ExecutionError, SignalValue};
use serde_json::Value;
use std::collections::BTreeMap;
use xdl_gnc_core::Timestamp;
use xdl_model::PortId;

pub struct Constant {
    pub output: PortId,
    pub value: f64,
}

impl Constant {
    pub fn new(output: PortId, value: f64) -> Result<Self, ExecutionError> {
        if !value.is_finite() {
            return Err(ExecutionError::Runtime(
                "constant value must be finite".to_string(),
            ));
        }
        Ok(Self { output, value })
    }
}

impl BlockRuntime for Constant {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(value) = parameters.get("value") {
            self.value = value.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("constant value parameter must be numeric".to_string())
            })?;
            if !self.value.is_finite() {
                return Err(ExecutionError::Runtime(
                    "constant value parameter must be finite".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        _inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(self.value),
        )]))
    }
}

pub struct Gain {
    pub input: PortId,
    pub output: PortId,
    pub gain: f64,
}

impl Gain {
    pub fn new(input: PortId, output: PortId, gain: f64) -> Result<Self, ExecutionError> {
        if !gain.is_finite() {
            return Err(ExecutionError::Runtime("gain must be finite".to_string()));
        }
        Ok(Self {
            input,
            output,
            gain,
        })
    }
}

impl BlockRuntime for Gain {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(gain) = parameters.get("gain") {
            self.gain = gain.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("gain parameter must be numeric".to_string())
            })?;
            if !self.gain.is_finite() {
                return Err(ExecutionError::Runtime(
                    "gain parameter must be finite".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = scalar_input(inputs, self.input, "gain")?;
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(value * self.gain),
        )]))
    }
}

pub struct Sum {
    pub inputs: Vec<PortId>,
    pub output: PortId,
}

pub struct Saturation {
    pub input: PortId,
    pub output: PortId,
    pub minimum: f64,
    pub maximum: f64,
}

impl Saturation {
    pub fn new(
        input: PortId,
        output: PortId,
        minimum: f64,
        maximum: f64,
    ) -> Result<Self, ExecutionError> {
        validate_limits(minimum, maximum, "saturation")?;
        Ok(Self {
            input,
            output,
            minimum,
            maximum,
        })
    }
}

impl BlockRuntime for Saturation {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(minimum) = parameters.get("minimum") {
            self.minimum = minimum.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("saturation minimum must be numeric".to_string())
            })?;
        }
        if let Some(maximum) = parameters.get("maximum") {
            self.maximum = maximum.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("saturation maximum must be numeric".to_string())
            })?;
        }
        validate_limits(self.minimum, self.maximum, "saturation")
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = scalar_input(inputs, self.input, "saturation")?;
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(value.clamp(self.minimum, self.maximum)),
        )]))
    }
}

pub struct Integrator {
    pub input: PortId,
    pub output: PortId,
    pub state: f64,
    last_timestamp: Option<Timestamp>,
}

impl Integrator {
    pub fn new(input: PortId, output: PortId, initial_state: f64) -> Result<Self, ExecutionError> {
        if !initial_state.is_finite() {
            return Err(ExecutionError::Runtime(
                "integrator initial state must be finite".to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            state: initial_state,
            last_timestamp: None,
        })
    }
}

impl BlockRuntime for Integrator {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(initial_state) = parameters.get("initial_state") {
            self.state = initial_state.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("integrator initial state must be numeric".to_string())
            })?;
            if !self.state.is_finite() {
                return Err(ExecutionError::Runtime(
                    "integrator initial state must be finite".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let input = scalar_input(inputs, self.input, "integrator")?;
        if let Some(previous) = self.last_timestamp {
            let seconds = timestamp
                .duration_since(previous)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?
                .as_seconds();
            self.state += input * seconds;
            if !self.state.is_finite() {
                return Err(ExecutionError::Runtime(
                    "integrator state became non-finite".to_string(),
                ));
            }
        }
        self.last_timestamp = Some(timestamp);
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(self.state),
        )]))
    }
}

pub struct Product {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

impl Product {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for Product {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = scalar_input(inputs, self.left, "product")?
            * scalar_input(inputs, self.right, "product")?;
        if !value.is_finite() {
            return Err(ExecutionError::Runtime(
                "product output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(value))]))
    }
}

pub struct Switch {
    pub control: PortId,
    pub true_input: PortId,
    pub false_input: PortId,
    pub output: PortId,
    pub threshold: f64,
}

pub struct RateLimiter {
    pub input: PortId,
    pub output: PortId,
    pub rate_limit: f64,
    pub state: f64,
    last_timestamp: Option<Timestamp>,
}

impl RateLimiter {
    pub fn new(
        input: PortId,
        output: PortId,
        rate_limit: f64,
        initial_state: f64,
    ) -> Result<Self, ExecutionError> {
        if !rate_limit.is_finite() || rate_limit < 0.0 || !initial_state.is_finite() {
            return Err(ExecutionError::Runtime(
                "rate limiter requires a finite non-negative rate and finite initial state"
                    .to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            rate_limit,
            state: initial_state,
            last_timestamp: None,
        })
    }
}

impl BlockRuntime for RateLimiter {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(rate_limit) = parameters.get("rate_limit") {
            self.rate_limit = rate_limit.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("rate limiter rate must be numeric".to_string())
            })?;
        }
        if let Some(initial_state) = parameters.get("initial_state") {
            self.state = initial_state.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("rate limiter initial state must be numeric".to_string())
            })?;
        }
        if !self.rate_limit.is_finite() || self.rate_limit < 0.0 || !self.state.is_finite() {
            return Err(ExecutionError::Runtime(
                "rate limiter requires a finite non-negative rate and finite state".to_string(),
            ));
        }
        Ok(())
    }

    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let target = scalar_input(inputs, self.input, "rate limiter")?;
        if let Some(previous) = self.last_timestamp {
            let seconds = timestamp
                .duration_since(previous)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?
                .as_seconds();
            let maximum_delta = self.rate_limit * seconds;
            self.state += (target - self.state).clamp(-maximum_delta, maximum_delta);
        }
        self.last_timestamp = Some(timestamp);
        if !self.state.is_finite() {
            return Err(ExecutionError::Runtime(
                "rate limiter output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(self.state),
        )]))
    }
}

impl Switch {
    pub fn new(
        control: PortId,
        true_input: PortId,
        false_input: PortId,
        output: PortId,
        threshold: f64,
    ) -> Result<Self, ExecutionError> {
        if !threshold.is_finite() {
            return Err(ExecutionError::Runtime(
                "switch threshold must be finite".to_string(),
            ));
        }
        Ok(Self {
            control,
            true_input,
            false_input,
            output,
            threshold,
        })
    }
}

impl BlockRuntime for Switch {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(threshold) = parameters.get("threshold") {
            self.threshold = threshold.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("switch threshold must be numeric".to_string())
            })?;
            if !self.threshold.is_finite() {
                return Err(ExecutionError::Runtime(
                    "switch threshold must be finite".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let control = scalar_input(inputs, self.control, "switch")?;
        let selected = if control >= self.threshold {
            self.true_input
        } else {
            self.false_input
        };
        let value = scalar_input(inputs, selected, "switch")?;
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(value))]))
    }
}

impl Sum {
    pub fn new(inputs: Vec<PortId>, output: PortId) -> Result<Self, ExecutionError> {
        if inputs.is_empty() {
            return Err(ExecutionError::Runtime(
                "sum requires at least one input".to_string(),
            ));
        }
        Ok(Self { inputs, output })
    }
}

impl BlockRuntime for Sum {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = self.inputs.iter().try_fold(0.0, |total, port| {
            scalar_input(inputs, *port, "sum").map(|value| total + value)
        })?;
        if !value.is_finite() {
            return Err(ExecutionError::Runtime(
                "sum output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(value))]))
    }
}

fn scalar_input(
    inputs: &BTreeMap<PortId, SignalValue>,
    port: PortId,
    block: &str,
) -> Result<f64, ExecutionError> {
    match inputs.get(&port) {
        Some(SignalValue::Scalar(value)) if value.is_finite() => Ok(*value),
        Some(_) => Err(ExecutionError::Runtime(format!(
            "{block} input {port:?} is not a finite scalar"
        ))),
        None => Err(ExecutionError::MissingInput(port)),
    }
}

fn validate_limits(minimum: f64, maximum: f64, block: &str) -> Result<(), ExecutionError> {
    if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
        return Err(ExecutionError::Runtime(format!(
            "{block} limits must be finite and minimum must not exceed maximum"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn standard_scalar_blocks_execute_and_accept_model_parameters() {
        let input = PortId::new(1);
        let output = PortId::new(2);
        let mut gain = Gain::new(input, output, 2.0).expect("gain");
        gain.configure(&BTreeMap::from([(String::from("gain"), Value::from(3.0))]))
            .expect("gain parameter");
        let result = gain
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Scalar(4.0))]),
            )
            .expect("gain step");
        assert_eq!(result.get(&output), Some(&SignalValue::Scalar(12.0)));

        let sum_output = PortId::new(3);
        let mut sum = Sum::new(vec![input, output], sum_output).expect("sum");
        let result = sum
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Scalar(4.0)),
                    (output, SignalValue::Scalar(12.0)),
                ]),
            )
            .expect("sum step");
        assert_eq!(result.get(&sum_output), Some(&SignalValue::Scalar(16.0)));
    }

    #[test]
    fn standard_blocks_reject_invalid_configuration_and_inputs() {
        assert!(Gain::new(PortId::new(1), PortId::new(2), f64::NAN).is_err());
        assert!(Sum::new(Vec::new(), PortId::new(2)).is_err());
        let mut constant = Constant::new(PortId::new(2), 1.0).expect("constant");
        assert!(constant
            .configure(&BTreeMap::from([(
                String::from("value"),
                Value::from("bad")
            )]))
            .is_err());
        assert!(Saturation::new(PortId::new(1), PortId::new(2), 1.0, -1.0).is_err());
        assert!(Integrator::new(PortId::new(1), PortId::new(2), f64::INFINITY).is_err());
    }

    #[test]
    fn dynamic_blocks_integrate_and_clamp_with_explicit_time() {
        let input = PortId::new(1);
        let output = PortId::new(2);
        let mut integrator = Integrator::new(input, output, 1.0).expect("integrator");
        let inputs = BTreeMap::from([(input, SignalValue::Scalar(2.0))]);
        assert_eq!(
            integrator
                .step(Timestamp::ZERO, &inputs)
                .expect("initial step")[&output],
            SignalValue::Scalar(1.0)
        );
        assert_eq!(
            integrator
                .step(Timestamp::from_nanoseconds(500_000_000), &inputs)
                .expect("integrated step")[&output],
            SignalValue::Scalar(2.0)
        );

        let mut saturation = Saturation::new(input, output, -1.0, 1.0).expect("saturation");
        assert_eq!(
            saturation
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(3.0))])
                )
                .expect("saturation step")[&output],
            SignalValue::Scalar(1.0)
        );

        let product_output = PortId::new(4);
        let mut product = Product::new(input, output, product_output);
        assert_eq!(
            product
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([
                        (input, SignalValue::Scalar(2.0)),
                        (output, SignalValue::Scalar(3.0)),
                    ]),
                )
                .expect("product step")[&product_output],
            SignalValue::Scalar(6.0)
        );

        let switch_output = PortId::new(5);
        let mut switch =
            Switch::new(input, output, product_output, switch_output, 0.5).expect("switch");
        assert_eq!(
            switch
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([
                        (input, SignalValue::Scalar(1.0)),
                        (output, SignalValue::Scalar(3.0)),
                        (product_output, SignalValue::Scalar(6.0)),
                    ]),
                )
                .expect("switch step")[&switch_output],
            SignalValue::Scalar(3.0)
        );

        let mut limiter = RateLimiter::new(input, output, 2.0, 0.0).expect("rate limiter");
        limiter
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Scalar(5.0))]),
            )
            .expect("initial rate limiter step");
        assert_eq!(
            limiter
                .step(
                    Timestamp::from_nanoseconds(500_000_000),
                    &BTreeMap::from([(input, SignalValue::Scalar(5.0))]),
                )
                .expect("limited rate limiter step")[&output],
            SignalValue::Scalar(1.0)
        );
    }
}
