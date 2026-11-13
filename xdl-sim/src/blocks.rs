// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Small reusable scalar blocks for model execution.

use crate::{BlockRuntime, ExecutionError, SignalValue};
use serde_json::Value;
use std::collections::BTreeMap;
use xdl_gnc_control::{PidConfig as GncPidConfig, PidController};
use xdl_gnc_core::{Quaternion, ScalarLimit, Timestamp};
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

/// Selects one of several data inputs using an integral scalar selector.
/// `index_base` is explicit so zero-based and one-based model conventions do
/// not get conflated; selectors outside the configured range are errors.
pub struct MultiportSwitch {
    pub selector: PortId,
    pub inputs: Vec<PortId>,
    pub output: PortId,
    pub index_base: usize,
}

impl MultiportSwitch {
    pub fn new(
        selector: PortId,
        inputs: Vec<PortId>,
        output: PortId,
        index_base: usize,
    ) -> Result<Self, ExecutionError> {
        if inputs.is_empty() || index_base > 1 {
            return Err(ExecutionError::Runtime(
                "multiport switch requires inputs and a zero- or one-based index".to_string(),
            ));
        }
        Ok(Self {
            selector,
            inputs,
            output,
            index_base,
        })
    }
}

impl BlockRuntime for MultiportSwitch {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let selector = scalar_input(inputs, self.selector, "multiport switch selector")?;
        let relative = selector - self.index_base as f64;
        if !relative.is_finite() || relative.fract() != 0.0 || relative < 0.0 {
            return Err(ExecutionError::Runtime(
                "multiport switch selector must be an in-range integer".to_string(),
            ));
        }
        let index = relative as usize;
        let data_port = self.inputs.get(index).ok_or_else(|| {
            ExecutionError::Runtime("multiport switch selector is out of range".to_string())
        })?;
        let value = inputs
            .get(data_port)
            .cloned()
            .ok_or(ExecutionError::MissingInput(*data_port))?;
        if !value.is_finite() {
            return Err(ExecutionError::Runtime(
                "multiport switch input must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, value)]))
    }
}

pub struct RateLimiter {
    pub input: PortId,
    pub output: PortId,
    pub rate_limit: f64,
    pub state: f64,
    last_timestamp: Option<Timestamp>,
}

pub struct Deadzone {
    pub input: PortId,
    pub output: PortId,
    pub lower: f64,
    pub upper: f64,
}

pub struct LookupTable1D {
    pub input: PortId,
    pub output: PortId,
    pub breakpoints: Vec<f64>,
    pub values: Vec<f64>,
}

pub struct Derivative {
    pub input: PortId,
    pub output: PortId,
    previous_value: Option<f64>,
    previous_timestamp: Option<Timestamp>,
}

pub struct Pid {
    pub setpoint: PortId,
    pub measurement: PortId,
    pub output: PortId,
    controller: PidController,
}

pub struct Mux {
    pub inputs: Vec<PortId>,
    pub output: PortId,
}

impl Mux {
    pub fn new(inputs: Vec<PortId>, output: PortId) -> Result<Self, ExecutionError> {
        if inputs.is_empty() {
            return Err(ExecutionError::Runtime(
                "mux requires at least one input".to_string(),
            ));
        }
        Ok(Self { inputs, output })
    }
}

impl BlockRuntime for Mux {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let values = self
            .inputs
            .iter()
            .map(|port| scalar_input(inputs, *port, "mux"))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(values))]))
    }
}

pub struct Demux {
    pub input: PortId,
    pub outputs: Vec<PortId>,
}

impl Demux {
    pub fn new(input: PortId, outputs: Vec<PortId>) -> Result<Self, ExecutionError> {
        if outputs.is_empty() {
            return Err(ExecutionError::Runtime(
                "demux requires at least one output".to_string(),
            ));
        }
        Ok(Self { input, outputs })
    }
}

impl BlockRuntime for Demux {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(values)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "demux input must be a vector".to_string(),
            ));
        };
        if values.len() != self.outputs.len() {
            return Err(ExecutionError::Runtime(
                "demux vector length must match output count".to_string(),
            ));
        }
        Ok(self
            .outputs
            .iter()
            .copied()
            .zip(values.iter().copied())
            .map(|(port, value)| (port, SignalValue::Scalar(value)))
            .collect())
    }
}

pub struct MatrixGain {
    pub input: PortId,
    pub output: PortId,
    pub rows: usize,
    pub columns: usize,
    pub values: Vec<f64>,
}

pub struct MatrixSum {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

pub struct MatrixScale {
    pub matrix: PortId,
    pub scale: PortId,
    pub output: PortId,
}

impl MatrixScale {
    pub const fn new(matrix: PortId, scale: PortId, output: PortId) -> Self {
        Self {
            matrix,
            scale,
            output,
        }
    }
}

impl BlockRuntime for MatrixScale {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Matrix {
            rows,
            columns,
            values,
        }) = inputs.get(&self.matrix)
        else {
            return Err(ExecutionError::Runtime(
                "matrix scale input must be a matrix".to_string(),
            ));
        };
        let scale = scalar_input(inputs, self.scale, "matrix scale")?;
        if *rows == 0
            || *columns == 0
            || values.len() != rows.saturating_mul(*columns)
            || values.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix scale dimensions or values are invalid".to_string(),
            ));
        }
        let scaled = values.iter().map(|value| value * scale).collect::<Vec<_>>();
        if scaled.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "matrix scale output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Matrix {
                rows: *rows,
                columns: *columns,
                values: scaled,
            },
        )]))
    }
}

impl MatrixSum {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for MatrixSum {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Matrix {
            rows: left_rows,
            columns: left_columns,
            values: left_values,
        }) = inputs.get(&self.left)
        else {
            return Err(ExecutionError::Runtime(
                "matrix sum left input must be a matrix".to_string(),
            ));
        };
        let Some(SignalValue::Matrix {
            rows: right_rows,
            columns: right_columns,
            values: right_values,
        }) = inputs.get(&self.right)
        else {
            return Err(ExecutionError::Runtime(
                "matrix sum right input must be a matrix".to_string(),
            ));
        };
        if left_rows != right_rows
            || left_columns != right_columns
            || left_values.len() != left_rows.saturating_mul(*left_columns)
            || right_values.len() != right_rows.saturating_mul(*right_columns)
            || left_values.iter().any(|value| !value.is_finite())
            || right_values.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix sum dimensions or values are invalid".to_string(),
            ));
        }
        let values = left_values
            .iter()
            .zip(right_values)
            .map(|(left, right)| left + right)
            .collect::<Vec<_>>();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "matrix sum output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Matrix {
                rows: *left_rows,
                columns: *left_columns,
                values,
            },
        )]))
    }
}

pub struct MatrixTranspose {
    pub input: PortId,
    pub output: PortId,
}

pub struct MatrixMatrixProduct {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

impl MatrixMatrixProduct {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for MatrixMatrixProduct {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Matrix {
            rows: left_rows,
            columns: left_columns,
            values: left_values,
        }) = inputs.get(&self.left)
        else {
            return Err(ExecutionError::Runtime(
                "matrix product left input must be a matrix".to_string(),
            ));
        };
        let Some(SignalValue::Matrix {
            rows: right_rows,
            columns: right_columns,
            values: right_values,
        }) = inputs.get(&self.right)
        else {
            return Err(ExecutionError::Runtime(
                "matrix product right input must be a matrix".to_string(),
            ));
        };
        if *left_rows == 0
            || *left_columns == 0
            || *right_rows == 0
            || *right_columns == 0
            || *left_columns != *right_rows
            || left_values.len() != left_rows.saturating_mul(*left_columns)
            || right_values.len() != right_rows.saturating_mul(*right_columns)
            || left_values.iter().any(|value| !value.is_finite())
            || right_values.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix product dimensions or values are invalid".to_string(),
            ));
        }
        let values = (0..*left_rows)
            .flat_map(|row| {
                (0..*right_columns).map(move |column| {
                    (0..*left_columns)
                        .map(|inner| {
                            left_values[row * *left_columns + inner]
                                * right_values[inner * *right_columns + column]
                        })
                        .sum::<f64>()
                })
            })
            .collect::<Vec<_>>();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "matrix product output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Matrix {
                rows: *left_rows,
                columns: *right_columns,
                values,
            },
        )]))
    }
}

impl MatrixTranspose {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for MatrixTranspose {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Matrix {
            rows,
            columns,
            values,
        }) = inputs.get(&self.input)
        else {
            return Err(ExecutionError::Runtime(
                "matrix transpose input must be a matrix".to_string(),
            ));
        };
        if *rows == 0
            || *columns == 0
            || values.len() != rows.saturating_mul(*columns)
            || values.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix transpose dimensions or values are invalid".to_string(),
            ));
        }
        let transposed = (0..*columns)
            .flat_map(|column| (0..*rows).map(move |row| values[row * *columns + column]))
            .collect::<Vec<_>>();
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Matrix {
                rows: *columns,
                columns: *rows,
                values: transposed,
            },
        )]))
    }
}

pub struct MatrixVectorProduct {
    pub matrix: PortId,
    pub vector: PortId,
    pub output: PortId,
}

impl MatrixVectorProduct {
    pub const fn new(matrix: PortId, vector: PortId, output: PortId) -> Self {
        Self {
            matrix,
            vector,
            output,
        }
    }
}

impl BlockRuntime for MatrixVectorProduct {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Matrix {
            rows,
            columns,
            values,
        }) = inputs.get(&self.matrix)
        else {
            return Err(ExecutionError::Runtime(
                "matrix-vector product matrix input must be a matrix".to_string(),
            ));
        };
        let Some(SignalValue::Vector(vector)) = inputs.get(&self.vector) else {
            return Err(ExecutionError::Runtime(
                "matrix-vector product vector input must be a vector".to_string(),
            ));
        };
        if *rows == 0
            || *columns == 0
            || vector.len() != *columns
            || values.len() != rows.saturating_mul(*columns)
            || values.iter().any(|value| !value.is_finite())
            || vector.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix-vector product dimensions or values are invalid".to_string(),
            ));
        }
        let result = values
            .chunks_exact(*columns)
            .map(|row| row.iter().zip(vector).map(|(a, b)| a * b).sum::<f64>())
            .collect::<Vec<_>>();
        if result.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "matrix-vector product output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(result))]))
    }
}

/// Discrete state-space block with row-major A/B/C/D matrices.
///
/// The output is computed from the current state before the state is advanced:
/// `y[k] = C*x[k] + D*u[k]`, `x[k+1] = A*x[k] + B*u[k]`.
pub struct StateSpaceConfig {
    pub states: usize,
    pub inputs: usize,
    pub outputs: usize,
    pub a: Vec<f64>,
    pub b: Vec<f64>,
    pub c: Vec<f64>,
    pub d: Vec<f64>,
    pub initial_state: Vec<f64>,
}

pub struct StateSpace {
    pub input: PortId,
    pub output: PortId,
    states: usize,
    inputs: usize,
    outputs: usize,
    a: Vec<f64>,
    b: Vec<f64>,
    c: Vec<f64>,
    d: Vec<f64>,
    state: Vec<f64>,
}

impl StateSpace {
    pub fn new(
        input: PortId,
        output: PortId,
        config: StateSpaceConfig,
    ) -> Result<Self, ExecutionError> {
        let StateSpaceConfig {
            states,
            inputs,
            outputs,
            a,
            b,
            c,
            d,
            initial_state,
        } = config;
        if states == 0 || inputs == 0 || outputs == 0 {
            return Err(ExecutionError::Runtime(
                "state-space dimensions must be positive".to_string(),
            ));
        }
        if !valid_matrix(&a, states, states)
            || !valid_matrix(&b, states, inputs)
            || !valid_matrix(&c, outputs, states)
            || !valid_matrix(&d, outputs, inputs)
            || initial_state.len() != states
            || initial_state.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "state-space dimensions or values are invalid".to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            states,
            inputs,
            outputs,
            a,
            b,
            c,
            d,
            state: initial_state,
        })
    }
}

impl BlockRuntime for StateSpace {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(input)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "state-space input must be a vector".to_string(),
            ));
        };
        if input.len() != self.inputs || input.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "state-space input dimensions or values are invalid".to_string(),
            ));
        }
        let output = matrix_vector_sum(
            &self.c,
            self.outputs,
            self.states,
            &self.state,
            &self.d,
            input,
        );
        let next_state = matrix_vector_sum(
            &self.a,
            self.states,
            self.states,
            &self.state,
            &self.b,
            input,
        );
        if output.iter().any(|value| !value.is_finite())
            || next_state.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "state-space output must be finite".to_string(),
            ));
        }
        self.state = next_state;
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(output))]))
    }
}

pub struct VectorSum {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

pub struct VectorScale {
    pub vector: PortId,
    pub scale: PortId,
    pub output: PortId,
}

impl VectorScale {
    pub const fn new(vector: PortId, scale: PortId, output: PortId) -> Self {
        Self {
            vector,
            scale,
            output,
        }
    }
}

impl BlockRuntime for VectorScale {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(vector)) = inputs.get(&self.vector) else {
            return Err(ExecutionError::Runtime(
                "vector scale input must be a vector".to_string(),
            ));
        };
        let scale = scalar_input(inputs, self.scale, "vector scale")?;
        let values = vector.iter().map(|value| value * scale).collect::<Vec<_>>();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "vector scale output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(values))]))
    }
}

pub struct VectorNorm {
    pub input: PortId,
    pub output: PortId,
}

pub struct VectorNormalize {
    pub input: PortId,
    pub output: PortId,
}

impl VectorNormalize {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for VectorNormalize {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(vector)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "vector normalize input must be a vector".to_string(),
            ));
        };
        let norm = vector.iter().try_fold(0.0_f64, |norm, value| {
            if !value.is_finite() {
                Err(ExecutionError::Runtime(
                    "vector normalize input must be finite".to_string(),
                ))
            } else {
                Ok(norm.hypot(*value))
            }
        })?;
        if norm == 0.0 {
            return Err(ExecutionError::Runtime(
                "cannot normalize a zero-length vector".to_string(),
            ));
        }
        let normalized = vector.iter().map(|value| value / norm).collect::<Vec<_>>();
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(normalized),
        )]))
    }
}

pub struct BusCreator {
    pub inputs: BTreeMap<String, PortId>,
    pub output: PortId,
}

impl BusCreator {
    pub fn new(inputs: BTreeMap<String, PortId>, output: PortId) -> Result<Self, ExecutionError> {
        if inputs.is_empty() || inputs.keys().any(|name| name.trim().is_empty()) {
            return Err(ExecutionError::Runtime(
                "bus creator requires named inputs".to_string(),
            ));
        }
        Ok(Self { inputs, output })
    }
}

impl BlockRuntime for BusCreator {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let fields = self
            .inputs
            .iter()
            .map(|(name, port)| {
                inputs
                    .get(port)
                    .cloned()
                    .map(|value| (name.clone(), value))
                    .ok_or(ExecutionError::MissingInput(*port))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        Ok(BTreeMap::from([(self.output, SignalValue::Bus(fields))]))
    }
}

pub struct BusSelector {
    pub input: PortId,
    pub outputs: BTreeMap<String, PortId>,
}

impl BusSelector {
    pub fn new(input: PortId, outputs: BTreeMap<String, PortId>) -> Result<Self, ExecutionError> {
        if outputs.is_empty() || outputs.keys().any(|name| name.trim().is_empty()) {
            return Err(ExecutionError::Runtime(
                "bus selector requires named outputs".to_string(),
            ));
        }
        Ok(Self { input, outputs })
    }
}

impl BlockRuntime for BusSelector {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Bus(fields)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "bus selector input must be a bus".to_string(),
            ));
        };
        self.outputs
            .iter()
            .map(|(name, port)| {
                fields
                    .get(name)
                    .cloned()
                    .map(|value| (*port, value))
                    .ok_or_else(|| {
                        ExecutionError::Runtime(format!("bus field {name} is not present"))
                    })
            })
            .collect()
    }
}

pub struct CrossProduct {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

pub struct QuaternionNormalize {
    pub input: PortId,
    pub output: PortId,
}

impl QuaternionNormalize {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for QuaternionNormalize {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let [w, x, y, z] = vector4_input(inputs, self.input, "quaternion normalize")?;
        let quaternion = Quaternion::new(w, x, y, z);
        let normalized = quaternion
            .try_normalize()
            .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(vec![normalized.w, normalized.x, normalized.y, normalized.z]),
        )]))
    }
}

pub struct QuaternionMultiply {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

impl QuaternionMultiply {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for QuaternionMultiply {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let [left_w, left_x, left_y, left_z] =
            vector4_input(inputs, self.left, "quaternion multiply")?;
        let [right_w, right_x, right_y, right_z] =
            vector4_input(inputs, self.right, "quaternion multiply")?;
        let left = Quaternion::new(left_w, left_x, left_y, left_z);
        let right = Quaternion::new(right_w, right_x, right_y, right_z);
        let product = left * right;
        if !product.w.is_finite()
            || !product.x.is_finite()
            || !product.y.is_finite()
            || !product.z.is_finite()
        {
            return Err(ExecutionError::Runtime(
                "quaternion product must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(vec![product.w, product.x, product.y, product.z]),
        )]))
    }
}

pub struct QuaternionRotate {
    pub quaternion: PortId,
    pub vector: PortId,
    pub output: PortId,
}

impl QuaternionRotate {
    pub const fn new(quaternion: PortId, vector: PortId, output: PortId) -> Self {
        Self {
            quaternion,
            vector,
            output,
        }
    }
}

impl BlockRuntime for QuaternionRotate {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let [w, x, y, z] = vector4_input(inputs, self.quaternion, "quaternion rotate")?;
        let [vx, vy, vz] = vector3_input(inputs, self.vector, "quaternion rotate")?;
        let rotated = Quaternion::new(w, x, y, z)
            .rotate_vector(xdl_gnc_core::Vec3::new(vx, vy, vz))
            .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(vec![rotated.x, rotated.y, rotated.z]),
        )]))
    }
}

pub struct QuaternionConjugate {
    pub input: PortId,
    pub output: PortId,
}

pub struct QuaternionInverse {
    pub input: PortId,
    pub output: PortId,
}

impl QuaternionInverse {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for QuaternionInverse {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let [w, x, y, z] = vector4_input(inputs, self.input, "quaternion inverse")?;
        let quaternion = Quaternion::new(w, x, y, z);
        let norm_squared = quaternion.norm_squared();
        if !norm_squared.is_finite() || norm_squared <= f64::EPSILON {
            return Err(ExecutionError::Runtime(
                "quaternion inverse requires a non-zero finite quaternion".to_string(),
            ));
        }
        let inverse = quaternion.conjugate();
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(vec![
                inverse.w / norm_squared,
                inverse.x / norm_squared,
                inverse.y / norm_squared,
                inverse.z / norm_squared,
            ]),
        )]))
    }
}

impl QuaternionConjugate {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for QuaternionConjugate {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let [w, x, y, z] = vector4_input(inputs, self.input, "quaternion conjugate")?;
        let conjugate = Quaternion::new(w, x, y, z).conjugate();
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Vector(vec![conjugate.w, conjugate.x, conjugate.y, conjugate.z]),
        )]))
    }
}

impl CrossProduct {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for CrossProduct {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let left = vector3_input(inputs, self.left, "cross product")?;
        let right = vector3_input(inputs, self.right, "cross product")?;
        let value = vec![
            left[1] * right[2] - left[2] * right[1],
            left[2] * right[0] - left[0] * right[2],
            left[0] * right[1] - left[1] * right[0],
        ];
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(value))]))
    }
}

pub struct DotProduct {
    pub left: PortId,
    pub right: PortId,
    pub output: PortId,
}

impl DotProduct {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for DotProduct {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(left)) = inputs.get(&self.left) else {
            return Err(ExecutionError::Runtime(
                "dot product left input must be a vector".to_string(),
            ));
        };
        let Some(SignalValue::Vector(right)) = inputs.get(&self.right) else {
            return Err(ExecutionError::Runtime(
                "dot product right input must be a vector".to_string(),
            ));
        };
        if left.len() != right.len() {
            return Err(ExecutionError::Runtime(
                "dot product inputs must have equal length".to_string(),
            ));
        }
        let value = left
            .iter()
            .zip(right)
            .try_fold(0.0, |total, (left, right)| {
                if !left.is_finite() || !right.is_finite() {
                    Err(ExecutionError::Runtime(
                        "dot product inputs must be finite".to_string(),
                    ))
                } else {
                    Ok(total + left * right)
                }
            })?;
        if !value.is_finite() {
            return Err(ExecutionError::Runtime(
                "dot product output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(value))]))
    }
}

impl VectorNorm {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self { input, output }
    }
}

impl BlockRuntime for VectorNorm {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(vector)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "vector norm input must be a vector".to_string(),
            ));
        };
        let norm = vector.iter().try_fold(0.0_f64, |norm, value| {
            if !value.is_finite() {
                Err(ExecutionError::Runtime(
                    "vector norm input must be finite".to_string(),
                ))
            } else {
                Ok(norm.hypot(*value))
            }
        })?;
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(norm))]))
    }
}

impl VectorSum {
    pub const fn new(left: PortId, right: PortId, output: PortId) -> Self {
        Self {
            left,
            right,
            output,
        }
    }
}

impl BlockRuntime for VectorSum {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(left)) = inputs.get(&self.left) else {
            return Err(ExecutionError::Runtime(
                "vector sum left input must be a vector".to_string(),
            ));
        };
        let Some(SignalValue::Vector(right)) = inputs.get(&self.right) else {
            return Err(ExecutionError::Runtime(
                "vector sum right input must be a vector".to_string(),
            ));
        };
        if left.len() != right.len() {
            return Err(ExecutionError::Runtime(
                "vector sum inputs must have equal length".to_string(),
            ));
        }
        let values = left
            .iter()
            .zip(right)
            .map(|(left, right)| left + right)
            .collect::<Vec<_>>();
        if values.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "vector sum output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(values))]))
    }
}

impl MatrixGain {
    pub fn new(
        input: PortId,
        output: PortId,
        rows: usize,
        columns: usize,
        values: Vec<f64>,
    ) -> Result<Self, ExecutionError> {
        if rows == 0
            || columns == 0
            || values.len() != rows.saturating_mul(columns)
            || values.iter().any(|value| !value.is_finite())
        {
            return Err(ExecutionError::Runtime(
                "matrix gain requires non-zero dimensions and a finite matrix of matching size"
                    .to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            rows,
            columns,
            values,
        })
    }
}

impl BlockRuntime for MatrixGain {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let Some(SignalValue::Vector(vector)) = inputs.get(&self.input) else {
            return Err(ExecutionError::Runtime(
                "matrix gain input must be a vector".to_string(),
            ));
        };
        if vector.len() != self.columns {
            return Err(ExecutionError::Runtime(
                "matrix gain input length must match matrix columns".to_string(),
            ));
        }
        let result = self
            .values
            .chunks_exact(self.columns)
            .map(|row| row.iter().zip(vector).map(|(a, b)| a * b).sum::<f64>())
            .collect::<Vec<_>>();
        if result.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "matrix gain output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(self.output, SignalValue::Vector(result))]))
    }
}

pub struct PidBlockConfig {
    pub proportional_gain: f64,
    pub integral_gain: f64,
    pub derivative_gain: f64,
    pub output_minimum: f64,
    pub output_maximum: f64,
    pub integral_minimum: f64,
    pub integral_maximum: f64,
}

impl Pid {
    pub fn new(
        setpoint: PortId,
        measurement: PortId,
        output: PortId,
        block_config: PidBlockConfig,
    ) -> Result<Self, ExecutionError> {
        let output_limit =
            ScalarLimit::new(block_config.output_minimum, block_config.output_maximum)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        let integral_limit =
            ScalarLimit::new(block_config.integral_minimum, block_config.integral_maximum)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        let config = GncPidConfig::new(
            block_config.proportional_gain,
            block_config.integral_gain,
            block_config.derivative_gain,
            output_limit,
            integral_limit,
        )
        .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        Ok(Self {
            setpoint,
            measurement,
            output,
            controller: PidController::new(config),
        })
    }
}

impl BlockRuntime for Pid {
    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let setpoint = scalar_input(inputs, self.setpoint, "PID")?;
        let measurement = scalar_input(inputs, self.measurement, "PID")?;
        let command = self
            .controller
            .step(timestamp, setpoint, measurement)
            .map_err(|error| ExecutionError::Runtime(error.to_string()))?
            .value
            .command;
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(command),
        )]))
    }
}

impl Derivative {
    pub const fn new(input: PortId, output: PortId) -> Self {
        Self {
            input,
            output,
            previous_value: None,
            previous_timestamp: None,
        }
    }
}

impl BlockRuntime for Derivative {
    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = scalar_input(inputs, self.input, "derivative")?;
        let derivative = match (self.previous_value, self.previous_timestamp) {
            (Some(previous_value), Some(previous_timestamp)) => {
                let seconds = timestamp
                    .duration_since(previous_timestamp)
                    .map_err(|error| ExecutionError::Runtime(error.to_string()))?
                    .as_seconds();
                if seconds <= 0.0 {
                    return Err(ExecutionError::Runtime(
                        "derivative timestamps must increase".to_string(),
                    ));
                }
                (value - previous_value) / seconds
            }
            _ => 0.0,
        };
        self.previous_value = Some(value);
        self.previous_timestamp = Some(timestamp);
        if !derivative.is_finite() {
            return Err(ExecutionError::Runtime(
                "derivative output must be finite".to_string(),
            ));
        }
        Ok(BTreeMap::from([(
            self.output,
            SignalValue::Scalar(derivative),
        )]))
    }
}

impl LookupTable1D {
    pub fn new(
        input: PortId,
        output: PortId,
        breakpoints: Vec<f64>,
        values: Vec<f64>,
    ) -> Result<Self, ExecutionError> {
        validate_lookup_table(&breakpoints, &values)?;
        Ok(Self {
            input,
            output,
            breakpoints,
            values,
        })
    }
}

impl BlockRuntime for LookupTable1D {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(breakpoints) = parameters.get("breakpoints") {
            self.breakpoints = finite_array_parameter(breakpoints, "lookup breakpoints")?;
        }
        if let Some(values) = parameters.get("values") {
            self.values = finite_array_parameter(values, "lookup values")?;
        }
        validate_lookup_table(&self.breakpoints, &self.values)
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let input = scalar_input(inputs, self.input, "lookup")?;
        let value = lookup_linear(&self.breakpoints, &self.values, input);
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(value))]))
    }
}

impl Deadzone {
    pub fn new(
        input: PortId,
        output: PortId,
        lower: f64,
        upper: f64,
    ) -> Result<Self, ExecutionError> {
        validate_deadzone(lower, upper)?;
        Ok(Self {
            input,
            output,
            lower,
            upper,
        })
    }
}

impl BlockRuntime for Deadzone {
    fn configure(&mut self, parameters: &BTreeMap<String, Value>) -> Result<(), ExecutionError> {
        if let Some(lower) = parameters.get("lower") {
            self.lower = lower.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("deadzone lower limit must be numeric".to_string())
            })?;
        }
        if let Some(upper) = parameters.get("upper") {
            self.upper = upper.as_f64().ok_or_else(|| {
                ExecutionError::Runtime("deadzone upper limit must be numeric".to_string())
            })?;
        }
        validate_deadzone(self.lower, self.upper)
    }

    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let value = scalar_input(inputs, self.input, "deadzone")?;
        let output = if value > self.upper {
            value - self.upper
        } else if value < self.lower {
            value - self.lower
        } else {
            0.0
        };
        Ok(BTreeMap::from([(self.output, SignalValue::Scalar(output))]))
    }
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

fn vector3_input(
    inputs: &BTreeMap<PortId, SignalValue>,
    port: PortId,
    block: &str,
) -> Result<[f64; 3], ExecutionError> {
    let Some(SignalValue::Vector(values)) = inputs.get(&port) else {
        return Err(ExecutionError::Runtime(format!(
            "{block} input {port:?} must be a vector"
        )));
    };
    let values: [f64; 3] = values.as_slice().try_into().map_err(|_| {
        ExecutionError::Runtime(format!("{block} input {port:?} must have length 3"))
    })?;
    if values.iter().any(|value| !value.is_finite()) {
        return Err(ExecutionError::Runtime(format!(
            "{block} input {port:?} must be finite"
        )));
    }
    Ok(values)
}

fn vector4_input(
    inputs: &BTreeMap<PortId, SignalValue>,
    port: PortId,
    block: &str,
) -> Result<[f64; 4], ExecutionError> {
    let Some(SignalValue::Vector(values)) = inputs.get(&port) else {
        return Err(ExecutionError::Runtime(format!(
            "{block} input {port:?} must be a vector"
        )));
    };
    let values: [f64; 4] = values.as_slice().try_into().map_err(|_| {
        ExecutionError::Runtime(format!("{block} input {port:?} must have length 4"))
    })?;
    if values.iter().any(|value| !value.is_finite()) {
        return Err(ExecutionError::Runtime(format!(
            "{block} input {port:?} must be finite"
        )));
    }
    Ok(values)
}

fn validate_limits(minimum: f64, maximum: f64, block: &str) -> Result<(), ExecutionError> {
    if !minimum.is_finite() || !maximum.is_finite() || minimum > maximum {
        return Err(ExecutionError::Runtime(format!(
            "{block} limits must be finite and minimum must not exceed maximum"
        )));
    }
    Ok(())
}

fn validate_deadzone(lower: f64, upper: f64) -> Result<(), ExecutionError> {
    if !lower.is_finite() || !upper.is_finite() || lower > 0.0 || upper < 0.0 || lower > upper {
        return Err(ExecutionError::Runtime(
            "deadzone limits must be finite, ordered, and contain zero".to_string(),
        ));
    }
    Ok(())
}

fn validate_lookup_table(breakpoints: &[f64], values: &[f64]) -> Result<(), ExecutionError> {
    if breakpoints.len() < 2
        || breakpoints.len() != values.len()
        || breakpoints.iter().any(|value| !value.is_finite())
        || values.iter().any(|value| !value.is_finite())
        || breakpoints.windows(2).any(|pair| pair[0] >= pair[1])
    {
        return Err(ExecutionError::Runtime(
            "lookup table requires equal finite arrays with at least two strictly increasing breakpoints"
                .to_string(),
        ));
    }
    Ok(())
}

fn finite_array_parameter(value: &Value, name: &str) -> Result<Vec<f64>, ExecutionError> {
    value
        .as_array()
        .ok_or_else(|| ExecutionError::Runtime(format!("{name} parameter must be an array")))?
        .iter()
        .map(|value| {
            let number = value.as_f64().ok_or_else(|| {
                ExecutionError::Runtime(format!("{name} parameter must be numeric"))
            })?;
            if !number.is_finite() {
                return Err(ExecutionError::Runtime(format!(
                    "{name} parameter must contain finite values"
                )));
            }
            Ok(number)
        })
        .collect()
}

fn lookup_linear(breakpoints: &[f64], values: &[f64], input: f64) -> f64 {
    if input <= breakpoints[0] {
        return values[0];
    }
    if input >= breakpoints[breakpoints.len() - 1] {
        return values[values.len() - 1];
    }
    let upper = breakpoints.partition_point(|breakpoint| *breakpoint < input);
    let lower = upper - 1;
    let fraction = (input - breakpoints[lower]) / (breakpoints[upper] - breakpoints[lower]);
    values[lower] + fraction * (values[upper] - values[lower])
}

fn valid_matrix(values: &[f64], rows: usize, columns: usize) -> bool {
    rows > 0
        && columns > 0
        && values.len() == rows.saturating_mul(columns)
        && values.iter().all(|value| value.is_finite())
}

fn matrix_vector_sum(
    left_matrix: &[f64],
    rows: usize,
    left_columns: usize,
    left_vector: &[f64],
    right_matrix: &[f64],
    right_vector: &[f64],
) -> Vec<f64> {
    (0..rows)
        .map(|row| {
            let left = (0..left_columns)
                .map(|column| left_matrix[row * left_columns + column] * left_vector[column])
                .sum::<f64>();
            let right = (0..right_vector.len())
                .map(|column| {
                    right_matrix[row * right_vector.len() + column] * right_vector[column]
                })
                .sum::<f64>();
            left + right
        })
        .collect()
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
        assert!(Deadzone::new(PortId::new(1), PortId::new(2), 1.0, 2.0).is_err());
        assert!(LookupTable1D::new(
            PortId::new(1),
            PortId::new(2),
            vec![0.0, 0.0],
            vec![1.0, 2.0]
        )
        .is_err());
        assert!(Pid::new(
            PortId::new(1),
            PortId::new(2),
            PortId::new(3),
            PidBlockConfig {
                proportional_gain: 1.0,
                integral_gain: 0.0,
                derivative_gain: 0.0,
                output_minimum: 1.0,
                output_maximum: -1.0,
                integral_minimum: -1.0,
                integral_maximum: 1.0,
            },
        )
        .is_err());
        assert!(Mux::new(Vec::new(), PortId::new(2)).is_err());
        assert!(Demux::new(PortId::new(1), Vec::new()).is_err());
        assert!(MultiportSwitch::new(PortId::new(1), Vec::new(), PortId::new(2), 0).is_err());
        assert!(
            MultiportSwitch::new(PortId::new(1), vec![PortId::new(2)], PortId::new(3), 2,).is_err()
        );
        assert!(MatrixGain::new(PortId::new(1), PortId::new(2), 2, 2, vec![1.0]).is_err());
        assert!(StateSpace::new(
            PortId::new(1),
            PortId::new(2),
            StateSpaceConfig {
                a: vec![1.0],
                b: vec![1.0],
                c: vec![1.0],
                d: vec![0.0],
                initial_state: vec![0.0],
                states: 2,
                inputs: 1,
                outputs: 1,
            },
        )
        .is_err());
        let mut matrix_vector =
            MatrixVectorProduct::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(matrix_vector
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
        assert!(BusCreator::new(BTreeMap::new(), PortId::new(2)).is_err());
        assert!(BusSelector::new(PortId::new(1), BTreeMap::new()).is_err());
        let mut cross = CrossProduct::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(cross
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Vector(vec![1.0, 2.0]))])
            )
            .is_err());
        let mut matrix_vector =
            MatrixVectorProduct::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(matrix_vector
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
        let mut transpose = MatrixTranspose::new(PortId::new(1), PortId::new(2));
        assert!(transpose
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
        let mut matrix_product =
            MatrixMatrixProduct::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(matrix_product
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
        let mut matrix_sum = MatrixSum::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(matrix_sum
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
        let mut matrix_scale = MatrixScale::new(PortId::new(1), PortId::new(2), PortId::new(3));
        assert!(matrix_scale
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(1), SignalValue::Scalar(1.0))])
            )
            .is_err());
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

        let selector = PortId::new(7);
        let first = PortId::new(8);
        let second = PortId::new(9);
        let multiport_output = PortId::new(10);
        let mut multiport =
            MultiportSwitch::new(selector, vec![first, second], multiport_output, 1)
                .expect("multiport switch");
        assert_eq!(
            multiport
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([
                        (selector, SignalValue::Scalar(2.0)),
                        (first, SignalValue::Scalar(3.0)),
                        (second, SignalValue::Scalar(5.0)),
                    ]),
                )
                .expect("one-based multiport switch")[&multiport_output],
            SignalValue::Scalar(5.0)
        );
        assert!(multiport
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (selector, SignalValue::Scalar(1.5)),
                    (first, SignalValue::Scalar(3.0)),
                    (second, SignalValue::Scalar(5.0)),
                ]),
            )
            .is_err());

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

        let mut deadzone = Deadzone::new(input, output, -1.0, 1.0).expect("deadzone");
        assert_eq!(
            deadzone
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(0.5))]),
                )
                .expect("deadzone interior")[&output],
            SignalValue::Scalar(0.0)
        );
        assert_eq!(
            deadzone
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(3.0))]),
                )
                .expect("deadzone exterior")[&output],
            SignalValue::Scalar(2.0)
        );

        let mut lookup = LookupTable1D::new(
            input,
            output,
            vec![0.0, 10.0, 20.0],
            vec![0.0, 100.0, 200.0],
        )
        .expect("lookup table");
        assert_eq!(
            lookup
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(15.0))]),
                )
                .expect("interpolated lookup")[&output],
            SignalValue::Scalar(150.0)
        );
        assert_eq!(
            lookup
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(-1.0))]),
                )
                .expect("clamped lookup")[&output],
            SignalValue::Scalar(0.0)
        );

        let mut derivative = Derivative::new(input, output);
        assert_eq!(
            derivative
                .step(
                    Timestamp::ZERO,
                    &BTreeMap::from([(input, SignalValue::Scalar(1.0))]),
                )
                .expect("initial derivative")[&output],
            SignalValue::Scalar(0.0)
        );
        assert_eq!(
            derivative
                .step(
                    Timestamp::from_nanoseconds(500_000_000),
                    &BTreeMap::from([(input, SignalValue::Scalar(2.0))]),
                )
                .expect("derivative step")[&output],
            SignalValue::Scalar(2.0)
        );

        let mut pid = Pid::new(
            input,
            output,
            PortId::new(6),
            PidBlockConfig {
                proportional_gain: 2.0,
                integral_gain: 0.0,
                derivative_gain: 0.0,
                output_minimum: -10.0,
                output_maximum: 10.0,
                integral_minimum: -5.0,
                integral_maximum: 5.0,
            },
        )
        .expect("PID block");
        assert_eq!(
            pid.step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Scalar(3.0)),
                    (output, SignalValue::Scalar(1.0)),
                ]),
            )
            .expect("PID step")[&PortId::new(6)],
            SignalValue::Scalar(4.0)
        );

        let mux_output = PortId::new(7);
        let mut mux = Mux::new(vec![input, output], mux_output).expect("mux");
        let vector = mux
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Scalar(2.0)),
                    (output, SignalValue::Scalar(3.0)),
                ]),
            )
            .expect("mux step");
        assert_eq!(
            vector.get(&mux_output),
            Some(&SignalValue::Vector(vec![2.0, 3.0]))
        );

        let mut demux =
            Demux::new(mux_output, vec![PortId::new(8), PortId::new(9)]).expect("demux");
        let outputs = demux.step(Timestamp::ZERO, &vector).expect("demux step");
        assert_eq!(
            outputs.get(&PortId::new(8)),
            Some(&SignalValue::Scalar(2.0))
        );
        assert_eq!(
            outputs.get(&PortId::new(9)),
            Some(&SignalValue::Scalar(3.0))
        );

        let mut matrix_gain =
            MatrixGain::new(mux_output, PortId::new(10), 2, 2, vec![1.0, 2.0, 3.0, 4.0])
                .expect("matrix gain");
        let matrix_output = matrix_gain
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(mux_output, SignalValue::Vector(vec![2.0, 3.0]))]),
            )
            .expect("matrix gain step");
        assert_eq!(
            matrix_output.get(&PortId::new(10)),
            Some(&SignalValue::Vector(vec![8.0, 18.0]))
        );

        let mut dynamic_matrix =
            MatrixVectorProduct::new(PortId::new(25), mux_output, PortId::new(26));
        let dynamic_output = dynamic_matrix
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (
                        PortId::new(25),
                        SignalValue::Matrix {
                            rows: 2,
                            columns: 2,
                            values: vec![1.0, 2.0, 3.0, 4.0],
                        },
                    ),
                    (mux_output, SignalValue::Vector(vec![2.0, 3.0])),
                ]),
            )
            .expect("dynamic matrix-vector product");
        assert_eq!(
            dynamic_output.get(&PortId::new(26)),
            Some(&SignalValue::Vector(vec![8.0, 18.0]))
        );

        let mut transpose = MatrixTranspose::new(PortId::new(25), PortId::new(27));
        let transposed = transpose
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(
                    PortId::new(25),
                    SignalValue::Matrix {
                        rows: 2,
                        columns: 3,
                        values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                    },
                )]),
            )
            .expect("matrix transpose");
        assert_eq!(
            transposed.get(&PortId::new(27)),
            Some(&SignalValue::Matrix {
                rows: 3,
                columns: 2,
                values: vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0],
            })
        );

        let mut matrix_product =
            MatrixMatrixProduct::new(PortId::new(25), PortId::new(27), PortId::new(28));
        let product = matrix_product
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (
                        PortId::new(25),
                        SignalValue::Matrix {
                            rows: 2,
                            columns: 3,
                            values: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0],
                        },
                    ),
                    (
                        PortId::new(27),
                        SignalValue::Matrix {
                            rows: 3,
                            columns: 2,
                            values: vec![1.0, 4.0, 2.0, 5.0, 3.0, 6.0],
                        },
                    ),
                ]),
            )
            .expect("matrix product");
        assert_eq!(
            product.get(&PortId::new(28)),
            Some(&SignalValue::Matrix {
                rows: 2,
                columns: 2,
                values: vec![14.0, 32.0, 32.0, 77.0],
            })
        );

        let mut matrix_sum = MatrixSum::new(PortId::new(25), PortId::new(27), PortId::new(29));
        let sum = matrix_sum
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (
                        PortId::new(25),
                        SignalValue::Matrix {
                            rows: 2,
                            columns: 2,
                            values: vec![1.0, 2.0, 3.0, 4.0],
                        },
                    ),
                    (
                        PortId::new(27),
                        SignalValue::Matrix {
                            rows: 2,
                            columns: 2,
                            values: vec![5.0, 6.0, 7.0, 8.0],
                        },
                    ),
                ]),
            )
            .expect("matrix sum");
        assert_eq!(
            sum.get(&PortId::new(29)),
            Some(&SignalValue::Matrix {
                rows: 2,
                columns: 2,
                values: vec![6.0, 8.0, 10.0, 12.0],
            })
        );

        let mut matrix_scale = MatrixScale::new(PortId::new(25), input, PortId::new(30));
        let scaled_matrix = matrix_scale
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (
                        PortId::new(25),
                        SignalValue::Matrix {
                            rows: 2,
                            columns: 2,
                            values: vec![1.0, 2.0, 3.0, 4.0],
                        },
                    ),
                    (input, SignalValue::Scalar(2.0)),
                ]),
            )
            .expect("matrix scale");
        assert_eq!(
            scaled_matrix.get(&PortId::new(30)),
            Some(&SignalValue::Matrix {
                rows: 2,
                columns: 2,
                values: vec![2.0, 4.0, 6.0, 8.0],
            })
        );

        let mut vector_sum = VectorSum::new(mux_output, PortId::new(10), PortId::new(11));
        let vector_sum_output = vector_sum
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (mux_output, SignalValue::Vector(vec![2.0, 3.0])),
                    (PortId::new(10), SignalValue::Vector(vec![8.0, 18.0])),
                ]),
            )
            .expect("vector sum step");
        assert_eq!(
            vector_sum_output.get(&PortId::new(11)),
            Some(&SignalValue::Vector(vec![10.0, 21.0]))
        );

        let mut scale = VectorScale::new(mux_output, input, PortId::new(12));
        let scaled = scale
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (mux_output, SignalValue::Vector(vec![2.0, 3.0])),
                    (input, SignalValue::Scalar(2.0)),
                ]),
            )
            .expect("vector scale step");
        assert_eq!(
            scaled.get(&PortId::new(12)),
            Some(&SignalValue::Vector(vec![4.0, 6.0]))
        );

        let mut norm = VectorNorm::new(PortId::new(12), PortId::new(13));
        let norm_output = norm
            .step(Timestamp::ZERO, &scaled)
            .expect("vector norm step");
        assert_eq!(
            norm_output.get(&PortId::new(13)),
            Some(&SignalValue::Scalar(52.0_f64.sqrt()))
        );

        let mut normalize = VectorNormalize::new(PortId::new(12), PortId::new(18));
        let normalized = normalize
            .step(Timestamp::ZERO, &scaled)
            .expect("vector normalize step");
        assert_eq!(
            normalized.get(&PortId::new(18)),
            Some(&SignalValue::Vector(vec![
                4.0 / 52.0_f64.sqrt(),
                6.0 / 52.0_f64.sqrt()
            ]))
        );
        assert!(normalize
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(PortId::new(12), SignalValue::Vector(vec![0.0, 0.0]))])
            )
            .is_err());

        let bus_output = PortId::new(14);
        let mut creator = BusCreator::new(
            BTreeMap::from([
                (String::from("position"), input),
                (String::from("velocity"), output),
            ]),
            bus_output,
        )
        .expect("bus creator");
        let bus = creator
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Scalar(2.0)),
                    (output, SignalValue::Scalar(3.0)),
                ]),
            )
            .expect("bus creator step");
        let mut selector = BusSelector::new(
            bus_output,
            BTreeMap::from([
                (String::from("position"), PortId::new(15)),
                (String::from("velocity"), PortId::new(16)),
            ]),
        )
        .expect("bus selector");
        let selected = selector
            .step(Timestamp::ZERO, &bus)
            .expect("bus selector step");
        assert_eq!(
            selected.get(&PortId::new(15)),
            Some(&SignalValue::Scalar(2.0))
        );
        assert_eq!(
            selected.get(&PortId::new(16)),
            Some(&SignalValue::Scalar(3.0))
        );

        let mut cross = CrossProduct::new(input, output, PortId::new(17));
        let cross_output = cross
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Vector(vec![1.0, 0.0, 0.0])),
                    (output, SignalValue::Vector(vec![0.0, 1.0, 0.0])),
                ]),
            )
            .expect("cross product step");
        assert_eq!(
            cross_output.get(&PortId::new(17)),
            Some(&SignalValue::Vector(vec![0.0, 0.0, 1.0]))
        );

        let mut dot = DotProduct::new(input, output, PortId::new(23));
        let dot_output = dot
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Vector(vec![1.0, 2.0, 3.0])),
                    (output, SignalValue::Vector(vec![4.0, 5.0, 6.0])),
                ]),
            )
            .expect("dot product step");
        assert_eq!(
            dot_output.get(&PortId::new(23)),
            Some(&SignalValue::Scalar(32.0))
        );

        let mut quaternion_normalize = QuaternionNormalize::new(input, PortId::new(19));
        assert!(quaternion_normalize
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Vector(vec![0.0, 0.0, 0.0, 0.0]))])
            )
            .is_err());

        let mut quaternion_multiply = QuaternionMultiply::new(input, output, PortId::new(20));
        let quaternion_output = quaternion_multiply
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (input, SignalValue::Vector(vec![1.0, 0.0, 0.0, 0.0])),
                    (output, SignalValue::Vector(vec![0.0, 1.0, 0.0, 0.0])),
                ]),
            )
            .expect("quaternion multiply step");
        assert_eq!(
            quaternion_output.get(&PortId::new(20)),
            Some(&SignalValue::Vector(vec![0.0, 1.0, 0.0, 0.0]))
        );

        let mut quaternion_rotate = QuaternionRotate::new(input, output, PortId::new(21));
        let rotated = quaternion_rotate
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([
                    (
                        input,
                        SignalValue::Vector(vec![
                            (std::f64::consts::FRAC_PI_4).cos(),
                            0.0,
                            0.0,
                            (std::f64::consts::FRAC_PI_4).sin(),
                        ]),
                    ),
                    (output, SignalValue::Vector(vec![1.0, 0.0, 0.0])),
                ]),
            )
            .expect("quaternion rotate step");
        let SignalValue::Vector(rotated) = rotated.get(&PortId::new(21)).expect("rotated vector")
        else {
            panic!("quaternion rotation must return a vector");
        };
        assert!((rotated[0]).abs() < 1.0e-12);
        assert!((rotated[1] - 1.0).abs() < 1.0e-12);
        assert!((rotated[2]).abs() < 1.0e-12);

        let mut conjugate = QuaternionConjugate::new(input, PortId::new(22));
        let conjugated = conjugate
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Vector(vec![1.0, 2.0, 3.0, 4.0]))]),
            )
            .expect("quaternion conjugate step");
        assert_eq!(
            conjugated.get(&PortId::new(22)),
            Some(&SignalValue::Vector(vec![1.0, -2.0, -3.0, -4.0]))
        );

        let mut inverse = QuaternionInverse::new(input, PortId::new(24));
        let inverted = inverse
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Vector(vec![2.0, 0.0, 0.0, 0.0]))]),
            )
            .expect("quaternion inverse step");
        assert_eq!(
            inverted.get(&PortId::new(24)),
            Some(&SignalValue::Vector(vec![0.5, 0.0, 0.0, 0.0]))
        );
    }

    #[test]
    fn state_space_emits_current_output_before_advancing_state() {
        let input = PortId::new(40);
        let output = PortId::new(41);
        let mut state_space = StateSpace::new(
            input,
            output,
            StateSpaceConfig {
                a: vec![1.0],
                b: vec![1.0],
                c: vec![1.0],
                d: vec![0.0],
                initial_state: vec![0.0],
                states: 1,
                inputs: 1,
                outputs: 1,
            },
        )
        .expect("state-space block");
        let inputs = BTreeMap::from([(input, SignalValue::Vector(vec![2.0]))]);

        assert_eq!(
            state_space
                .step(Timestamp::ZERO, &inputs)
                .expect("first state-space step")[&output],
            SignalValue::Vector(vec![0.0])
        );
        assert_eq!(
            state_space
                .step(Timestamp::from_nanoseconds(1), &inputs)
                .expect("second state-space step")[&output],
            SignalValue::Vector(vec![2.0])
        );
    }
}
