// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic scheduling primitives for XDL model execution.
//!
//! This crate schedules work; it does not sleep, read a wall clock, or execute
//! blocks. A future executor can consume [`ScheduledEvent`] values and apply
//! block semantics without changing ordering or timing behavior.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;
use xdl_gnc_core::{DurationNs, GncError, Timestamp};
use xdl_model::{BlockId, Model, ModelError, PortId, SignalShape, Subsystem};

pub mod blocks;
pub use blocks::{
    BusCreator, BusSelector, Constant, CrossProduct, Deadzone, Demux, Derivative, DotProduct, Gain,
    Integrator, LookupTable1D, MatrixGain, MatrixTranspose, MatrixVectorProduct, Mux, Pid,
    PidBlockConfig, Product, QuaternionConjugate, QuaternionInverse, QuaternionMultiply,
    QuaternionNormalize, QuaternionRotate, RateLimiter, Saturation, Sum, Switch, VectorNorm,
    VectorNormalize, VectorScale, VectorSum,
};
pub mod solver;
pub use solver::{
    euler_step, rk4_adaptive_step, rk4_step, AdaptiveConfig, AdaptiveStep, DerivativeFn,
    SolverError,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct TaskId(u32);

impl TaskId {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SchedulerError {
    #[error("invalid scheduler time: {0}")]
    InvalidTime(String),
    #[error("scheduler task count or event count exceeded its bound")]
    EventBoundExceeded,
    #[error("task identifier space exhausted")]
    IdExhausted,
    #[error("task {0:?} does not exist")]
    UnknownTask(TaskId),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum SignalValue {
    Scalar(f64),
    Vector(Vec<f64>),
    Matrix {
        rows: usize,
        columns: usize,
        values: Vec<f64>,
    },
    Bus(BTreeMap<String, SignalValue>),
}

impl SignalValue {
    fn is_finite(&self) -> bool {
        match self {
            Self::Scalar(value) => value.is_finite(),
            Self::Vector(values) => values.iter().all(|value| value.is_finite()),
            Self::Matrix {
                rows,
                columns,
                values,
            } => {
                values.len() == rows.saturating_mul(*columns)
                    && values.iter().all(|value| value.is_finite())
            }
            Self::Bus(fields) => fields.values().all(Self::is_finite),
        }
    }

    fn matches_shape(&self, shape: &SignalShape) -> bool {
        match (self, shape) {
            (Self::Scalar(_), SignalShape::Scalar) => true,
            (Self::Vector(values), SignalShape::Vector(length)) => values.len() == *length,
            (
                Self::Matrix {
                    rows: value_rows,
                    columns: value_columns,
                    values,
                },
                SignalShape::Matrix { rows, columns },
            ) => {
                *value_rows == *rows
                    && *value_columns == *columns
                    && values.len() == rows.saturating_mul(*columns)
            }
            (Self::Bus(values), SignalShape::Bus(fields)) => {
                values.len() == fields.len()
                    && fields.iter().all(|field| {
                        values
                            .get(&field.name)
                            .is_some_and(|value| value.matches_shape(&field.shape))
                    })
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExecutionError {
    #[error("model error: {0}")]
    Model(#[from] ModelError),
    #[error("subsystem error: {0}")]
    Subsystem(String),
    #[error("scheduler error: {0}")]
    Scheduler(#[from] SchedulerError),
    #[error("task {0:?} is not registered with a runtime block")]
    UnregisteredTask(TaskId),
    #[error("block {0:?} has no registered runtime task")]
    UnregisteredBlock(BlockId),
    #[error("required input port {0:?} has no signal")]
    MissingInput(PortId),
    #[error("block {block:?} emitted an invalid value for port {port:?}")]
    InvalidOutput { block: BlockId, port: PortId },
    #[error("trace update metadata does not match the owning model block")]
    TraceMetadataMismatch,
    #[error("block runtime failed: {0}")]
    Runtime(String),
}

/// Runtime implementation for one graph block. Implementations should be
/// deterministic and avoid wall-clock or blocking I/O.
pub trait BlockRuntime {
    fn configure(
        &mut self,
        _parameters: &BTreeMap<String, serde_json::Value>,
    ) -> Result<(), ExecutionError> {
        Ok(())
    }

    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError>;
}

/// Continuous block contract. The runtime owns the physical state while the
/// engine owns scheduling and input routing.
pub trait ContinuousBlockRuntime {
    fn derivative(
        &mut self,
        time: f64,
        state: &[f64],
        inputs: &BTreeMap<PortId, SignalValue>,
        derivative: &mut [f64],
    );

    fn outputs(
        &mut self,
        time: f64,
        state: &[f64],
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError>;
}

/// Adapts a continuous block to the scheduled block interface.
pub struct AdaptiveContinuousRuntime {
    runtime: Box<dyn ContinuousBlockRuntime>,
    state: Vec<f64>,
    config: AdaptiveConfig,
    last_timestamp: Option<Timestamp>,
}

impl AdaptiveContinuousRuntime {
    pub fn new(
        runtime: Box<dyn ContinuousBlockRuntime>,
        initial_state: Vec<f64>,
        config: AdaptiveConfig,
    ) -> Result<Self, ExecutionError> {
        config
            .validate()
            .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
        if initial_state.iter().any(|value| !value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "continuous initial state must be finite".to_string(),
            ));
        }
        Ok(Self {
            runtime,
            state: initial_state,
            config,
            last_timestamp: None,
        })
    }
}

impl BlockRuntime for AdaptiveContinuousRuntime {
    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        if let Some(previous) = self.last_timestamp {
            let duration = timestamp
                .duration_since(previous)
                .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
            if duration.as_nanoseconds() < 0 {
                return Err(ExecutionError::Runtime(
                    "continuous runtime timestamp moved backwards".to_string(),
                ));
            }
            if duration.as_nanoseconds() > 0 {
                let time = previous.as_seconds();
                let total_step = duration.as_seconds();
                let mut elapsed = 0.0;
                let mut proposed_step = total_step.min(self.config.maximum_step);
                let mut rejected_steps = 0;
                while elapsed < total_step {
                    let remaining = total_step - elapsed;
                    let config = AdaptiveConfig {
                        minimum_step: self.config.minimum_step.min(remaining),
                        ..self.config
                    };
                    let runtime = &mut self.runtime;
                    let result = rk4_adaptive_step(
                        time + elapsed,
                        proposed_step.min(remaining),
                        &self.state,
                        config,
                        &mut |sample_time, sample_state, derivative| {
                            runtime.derivative(sample_time, sample_state, inputs, derivative);
                        },
                    )
                    .map_err(|error| ExecutionError::Runtime(error.to_string()))?;
                    if result.accepted_step == 0.0 {
                        rejected_steps += 1;
                        if rejected_steps > 64 {
                            return Err(ExecutionError::Runtime(
                                "continuous solver exceeded its retry bound".to_string(),
                            ));
                        }
                        if result.next_step >= proposed_step.min(remaining) {
                            return Err(ExecutionError::Runtime(format!(
                                "continuous step rejected at minimum step with normalized error {}",
                                result.normalized_error
                            )));
                        }
                        proposed_step = result.next_step;
                        continue;
                    }
                    elapsed += result.accepted_step;
                    self.state = result.state;
                    proposed_step = result.next_step;
                }
            }
        }
        self.last_timestamp = Some(timestamp);
        self.runtime
            .outputs(timestamp.as_seconds(), &self.state, inputs)
    }
}

/// One-sample delay. The first execution emits `initial`; each later execution
/// emits the previous input and stores the current input for the next tick.
pub struct UnitDelay {
    pub input: PortId,
    pub output: PortId,
    state: SignalValue,
    initialized: bool,
}

impl UnitDelay {
    pub fn new(
        input: PortId,
        output: PortId,
        initial: SignalValue,
    ) -> Result<Self, ExecutionError> {
        if !initial.is_finite() {
            return Err(ExecutionError::Runtime(
                "unit-delay initial value must be finite".to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            state: initial,
            initialized: false,
        })
    }
}

impl BlockRuntime for UnitDelay {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        let output = BTreeMap::from([(self.output, self.state.clone())]);
        if let Some(input) = inputs.get(&self.input).cloned() {
            self.state = input;
            self.initialized = true;
        } else if self.initialized {
            return Err(ExecutionError::MissingInput(self.input));
        }
        Ok(output)
    }
}

/// Explicit zero-order hold. It emits the most recently sampled input without
/// interpolation; the scheduler controls when this block is executed.
pub struct ZeroOrderHold {
    pub input: PortId,
    pub output: PortId,
    state: SignalValue,
}

impl ZeroOrderHold {
    pub fn new(
        input: PortId,
        output: PortId,
        initial: SignalValue,
    ) -> Result<Self, ExecutionError> {
        if !initial.is_finite() {
            return Err(ExecutionError::Runtime(
                "zero-order-hold initial value must be finite".to_string(),
            ));
        }
        Ok(Self {
            input,
            output,
            state: initial,
        })
    }
}

impl BlockRuntime for ZeroOrderHold {
    fn step(
        &mut self,
        _timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        if let Some(value) = inputs.get(&self.input).cloned() {
            self.state = value;
        }
        Ok(BTreeMap::from([(self.output, self.state.clone())]))
    }
}

struct RegisteredBlock {
    block: BlockId,
    runtime: Box<dyn BlockRuntime>,
}

pub struct ExecutionEngine {
    pub model: Model,
    pub scheduler: Scheduler,
    registered: BTreeMap<TaskId, RegisteredBlock>,
    signals: BTreeMap<PortId, SignalValue>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExecutionReport {
    pub events: Vec<ScheduledEvent>,
    pub signal_updates: usize,
    pub updates: Vec<SignalUpdate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SignalUpdate {
    pub timestamp: Timestamp,
    pub task: TaskId,
    pub block: BlockId,
    pub port: PortId,
    pub value: SignalValue,
}

impl ExecutionReport {
    pub fn to_json_lines(&self) -> Result<String, ExecutionError> {
        self.updates
            .iter()
            .map(|update| {
                serde_json::to_string(update)
                    .map_err(|error| ExecutionError::Runtime(error.to_string()))
            })
            .collect::<Result<Vec<_>, _>>()
            .map(|lines| lines.join("\n"))
    }

    pub fn parse_json_lines(lines: &str) -> Result<Vec<SignalUpdate>, ExecutionError> {
        let mut updates = Vec::new();
        for (line_number, line) in lines.lines().enumerate() {
            if line.trim().is_empty() {
                continue;
            }
            let update: SignalUpdate = serde_json::from_str(line).map_err(|error| {
                ExecutionError::Runtime(format!("invalid trace line {}: {error}", line_number + 1))
            })?;
            if updates
                .last()
                .is_some_and(|previous: &SignalUpdate| update.timestamp < previous.timestamp)
            {
                return Err(ExecutionError::Runtime(format!(
                    "trace timestamp moved backwards at line {}",
                    line_number + 1
                )));
            }
            updates.push(update);
        }
        Ok(updates)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TraceCursor {
    updates: Vec<SignalUpdate>,
    position: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TraceCompareError {
    #[error("trace comparison tolerances must be finite and non-negative")]
    InvalidTolerance,
    #[error("trace lengths differ: expected {expected}, actual {actual}")]
    LengthMismatch { expected: usize, actual: usize },
    #[error("trace metadata differs at update {index}")]
    MetadataMismatch { index: usize },
    #[error("trace values differ at update {index} ({path})")]
    ValueMismatch { index: usize, path: String },
}

pub fn compare_traces(
    expected: &[SignalUpdate],
    actual: &[SignalUpdate],
    absolute_tolerance: f64,
    relative_tolerance: f64,
) -> Result<(), TraceCompareError> {
    if !absolute_tolerance.is_finite()
        || absolute_tolerance < 0.0
        || !relative_tolerance.is_finite()
        || relative_tolerance < 0.0
    {
        return Err(TraceCompareError::InvalidTolerance);
    }
    if expected.len() != actual.len() {
        return Err(TraceCompareError::LengthMismatch {
            expected: expected.len(),
            actual: actual.len(),
        });
    }
    for (index, (expected, actual)) in expected.iter().zip(actual).enumerate() {
        if expected.timestamp != actual.timestamp
            || expected.task != actual.task
            || expected.block != actual.block
            || expected.port != actual.port
        {
            return Err(TraceCompareError::MetadataMismatch { index });
        }
        compare_signal_values(
            &expected.value,
            &actual.value,
            absolute_tolerance,
            relative_tolerance,
            &index.to_string(),
        )
        .map_err(|path| TraceCompareError::ValueMismatch { index, path })?;
    }
    Ok(())
}

fn compare_signal_values(
    expected: &SignalValue,
    actual: &SignalValue,
    absolute_tolerance: f64,
    relative_tolerance: f64,
    path: &str,
) -> Result<(), String> {
    match (expected, actual) {
        (SignalValue::Scalar(expected), SignalValue::Scalar(actual)) => {
            let bound = absolute_tolerance + relative_tolerance * expected.abs().max(actual.abs());
            if (expected - actual).abs() <= bound {
                Ok(())
            } else {
                Err(path.to_string())
            }
        }
        (SignalValue::Vector(expected), SignalValue::Vector(actual)) => {
            if expected.len() != actual.len() {
                return Err(format!("{path}.length"));
            }
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .try_for_each(|(index, (expected, actual))| {
                    compare_signal_values(
                        &SignalValue::Scalar(*expected),
                        &SignalValue::Scalar(*actual),
                        absolute_tolerance,
                        relative_tolerance,
                        &format!("{path}[{index}]"),
                    )
                })
        }
        (
            SignalValue::Matrix {
                rows: expected_rows,
                columns: expected_columns,
                values: expected,
            },
            SignalValue::Matrix {
                rows: actual_rows,
                columns: actual_columns,
                values: actual,
            },
        ) => {
            if expected_rows != actual_rows || expected_columns != actual_columns {
                return Err(format!("{path}.shape"));
            }
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .try_for_each(|(index, (expected, actual))| {
                    compare_signal_values(
                        &SignalValue::Scalar(*expected),
                        &SignalValue::Scalar(*actual),
                        absolute_tolerance,
                        relative_tolerance,
                        &format!("{path}[{index}]"),
                    )
                })
        }
        (SignalValue::Bus(expected), SignalValue::Bus(actual)) => {
            if expected.keys().ne(actual.keys()) {
                return Err(format!("{path}.fields"));
            }
            expected.iter().try_for_each(|(name, expected)| {
                compare_signal_values(
                    expected,
                    &actual[name],
                    absolute_tolerance,
                    relative_tolerance,
                    &format!("{path}.{name}"),
                )
            })
        }
        _ => Err(format!("{path}.shape")),
    }
}

impl TraceCursor {
    pub fn new(updates: Vec<SignalUpdate>) -> Result<Self, ExecutionError> {
        if updates.iter().any(|update| !update.value.is_finite()) {
            return Err(ExecutionError::Runtime(
                "trace contains a non-finite signal value".to_string(),
            ));
        }
        if updates
            .windows(2)
            .any(|pair| pair[1].timestamp < pair[0].timestamp)
        {
            return Err(ExecutionError::Runtime(
                "trace timestamps must be monotonic".to_string(),
            ));
        }
        Ok(Self {
            updates,
            position: 0,
        })
    }

    pub fn from_json_lines(lines: &str) -> Result<Self, ExecutionError> {
        Self::new(ExecutionReport::parse_json_lines(lines)?)
    }

    pub fn peek(&self) -> Option<&SignalUpdate> {
        self.updates.get(self.position)
    }

    pub fn next_update(&mut self) -> Option<&SignalUpdate> {
        let update = self.updates.get(self.position);
        if update.is_some() {
            self.position += 1;
        }
        update
    }

    pub fn is_exhausted(&self) -> bool {
        self.position == self.updates.len()
    }

    pub fn reset(&mut self) {
        self.position = 0;
    }

    pub fn updates(&self) -> &[SignalUpdate] {
        &self.updates
    }
}

impl ExecutionEngine {
    pub fn new(model: Model, scheduler: Scheduler) -> Result<Self, ExecutionError> {
        model.validate()?;
        Ok(Self {
            model,
            scheduler,
            registered: BTreeMap::new(),
            signals: BTreeMap::new(),
        })
    }

    pub fn register_task(
        &mut self,
        task: TaskId,
        block: BlockId,
        runtime: Box<dyn BlockRuntime>,
    ) -> Result<(), ExecutionError> {
        if !self.scheduler.has_task(task) {
            return Err(ExecutionError::Scheduler(SchedulerError::UnknownTask(task)));
        }
        let parameters = &self.model.block(block)?.parameters;
        let mut runtime = runtime;
        runtime.configure(parameters)?;
        self.registered
            .insert(task, RegisteredBlock { block, runtime });
        Ok(())
    }

    pub fn signal(&self, port: PortId) -> Option<&SignalValue> {
        self.signals.get(&port)
    }

    pub fn signal_snapshot(&self) -> BTreeMap<PortId, SignalValue> {
        self.signals.clone()
    }

    pub fn restore_signal_snapshot(
        &mut self,
        snapshot: BTreeMap<PortId, SignalValue>,
    ) -> Result<(), ExecutionError> {
        for (port, value) in &snapshot {
            let definition = self.model.port(*port)?;
            if !value.matches_shape(&definition.signal.shape) || !value.is_finite() {
                return Err(ExecutionError::InvalidOutput {
                    block: self.model.block_for_port(*port)?,
                    port: *port,
                });
            }
        }
        self.signals = snapshot;
        Ok(())
    }

    pub fn validate_ready(&self) -> Result<(), ExecutionError> {
        for block in self.model.blocks.keys().copied() {
            if !self
                .registered
                .values()
                .any(|registered| registered.block == block)
            {
                return Err(ExecutionError::UnregisteredBlock(block));
            }
        }
        Ok(())
    }

    pub fn inject_signal(
        &mut self,
        port: PortId,
        value: SignalValue,
    ) -> Result<(), ExecutionError> {
        let definition = self.model.port(port)?;
        if definition.direction != xdl_model::PortDirection::Input
            || !value.matches_shape(&definition.signal.shape)
            || !value.is_finite()
        {
            return Err(ExecutionError::InvalidOutput {
                block: self.model.block_for_port(port)?,
                port,
            });
        }
        self.signals.insert(port, value);
        Ok(())
    }

    pub fn apply_trace_update(&mut self, update: &SignalUpdate) -> Result<(), ExecutionError> {
        let definition = self.model.port(update.port)?;
        let block = self.model.block_for_port(update.port)?;
        if block != update.block {
            return Err(ExecutionError::TraceMetadataMismatch);
        }
        if definition.direction != xdl_model::PortDirection::Output
            || !update.value.matches_shape(&definition.signal.shape)
            || !update.value.is_finite()
        {
            return Err(ExecutionError::InvalidOutput {
                block,
                port: update.port,
            });
        }
        self.signals.insert(update.port, update.value.clone());
        for connection in self
            .model
            .connections
            .iter()
            .filter(|connection| connection.output == update.port)
        {
            self.signals.insert(connection.input, update.value.clone());
        }
        Ok(())
    }

    pub fn apply_trace_until(
        &mut self,
        cursor: &mut TraceCursor,
        timestamp: Timestamp,
    ) -> Result<usize, ExecutionError> {
        let mut applied = 0;
        while cursor
            .peek()
            .is_some_and(|update| update.timestamp <= timestamp)
        {
            let update = cursor
                .next_update()
                .ok_or_else(|| {
                    ExecutionError::Runtime("trace cursor advanced unexpectedly".to_string())
                })?
                .clone();
            self.apply_trace_update(&update)?;
            applied += 1;
        }
        Ok(applied)
    }

    pub fn advance_to(&mut self, timestamp: Timestamp) -> Result<ExecutionReport, ExecutionError> {
        self.validate_ready()?;
        let mut events = self.scheduler.advance_to(timestamp)?;
        let order = self.model.execution_order()?;
        let ranks: BTreeMap<_, _> = order
            .into_iter()
            .enumerate()
            .map(|(rank, block)| (block, rank))
            .collect();
        events.sort_by_key(|event| {
            let rank = self
                .registered
                .get(&event.task)
                .and_then(|registered| ranks.get(&registered.block).copied())
                .unwrap_or(usize::MAX);
            (
                event.timestamp,
                rank,
                event.priority,
                event.task,
                event.sequence,
            )
        });
        let mut signal_updates = 0;
        let mut updates = Vec::new();
        for event in &events {
            let registered = self
                .registered
                .get_mut(&event.task)
                .ok_or(ExecutionError::UnregisteredTask(event.task))?;
            let block = registered.block;
            let mut inputs = BTreeMap::new();
            for port in self.model.block(block)?.ports.values() {
                if port.direction == xdl_model::PortDirection::Input {
                    let value = self
                        .signals
                        .get(&port.id)
                        .cloned()
                        .ok_or(ExecutionError::MissingInput(port.id))?;
                    inputs.insert(port.id, value);
                }
            }
            let outputs = registered.runtime.step(event.timestamp, &inputs)?;
            for (port, value) in outputs {
                let definition = self.model.port(port)?;
                if self.model.block_for_port(port)? != block
                    || !value.matches_shape(&definition.signal.shape)
                    || !value.is_finite()
                {
                    return Err(ExecutionError::InvalidOutput { block, port });
                }
                self.signals.insert(port, value.clone());
                for connection in self
                    .model
                    .connections
                    .iter()
                    .filter(|connection| connection.output == port)
                {
                    self.signals.insert(connection.input, value.clone());
                }
                signal_updates += 1;
                updates.push(SignalUpdate {
                    timestamp: event.timestamp,
                    task: event.task,
                    block,
                    port,
                    value,
                });
            }
        }
        Ok(ExecutionReport {
            events,
            signal_updates,
            updates,
        })
    }
}

/// Executes a subsystem as one block in a parent graph.
pub struct SubsystemRuntime {
    engine: ExecutionEngine,
    input_bindings: Vec<xdl_model::BoundaryBinding>,
    output_bindings: Vec<xdl_model::BoundaryBinding>,
}

impl SubsystemRuntime {
    pub fn new(subsystem: Subsystem, scheduler: Scheduler) -> Result<Self, ExecutionError> {
        subsystem
            .validate()
            .map_err(|error| ExecutionError::Subsystem(error.to_string()))?;
        let engine = ExecutionEngine::new(subsystem.model, scheduler)?;
        Ok(Self {
            engine,
            input_bindings: subsystem.input_bindings,
            output_bindings: subsystem.output_bindings,
        })
    }

    pub fn engine_mut(&mut self) -> &mut ExecutionEngine {
        &mut self.engine
    }
}

impl BlockRuntime for SubsystemRuntime {
    fn step(
        &mut self,
        timestamp: Timestamp,
        inputs: &BTreeMap<PortId, SignalValue>,
    ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
        for binding in &self.input_bindings {
            let value = inputs
                .get(&binding.boundary)
                .cloned()
                .ok_or(ExecutionError::MissingInput(binding.boundary))?;
            self.engine.inject_signal(binding.internal, value)?;
        }
        self.engine.advance_to(timestamp)?;
        self.output_bindings
            .iter()
            .map(|binding| {
                self.engine
                    .signal(binding.internal)
                    .cloned()
                    .map(|value| (binding.boundary, value))
                    .ok_or(ExecutionError::MissingInput(binding.internal))
            })
            .collect()
    }
}

impl From<GncError> for SchedulerError {
    fn from(error: GncError) -> Self {
        Self::InvalidTime(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchedulerConfig {
    pub base_period: DurationNs,
    pub max_events_per_advance: usize,
}

impl SchedulerConfig {
    pub fn new(
        base_period: DurationNs,
        max_events_per_advance: usize,
    ) -> Result<Self, SchedulerError> {
        if !base_period.is_positive() {
            return Err(SchedulerError::InvalidTime(
                "base period must be positive".to_string(),
            ));
        }
        if max_events_per_advance == 0 {
            return Err(SchedulerError::EventBoundExceeded);
        }
        Ok(Self {
            base_period,
            max_events_per_advance,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeriodicTask {
    pub id: TaskId,
    pub name: String,
    pub period: DurationNs,
    pub phase: DurationNs,
    pub priority: u32,
    next_release: Timestamp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScheduledEvent {
    pub timestamp: Timestamp,
    pub task: TaskId,
    pub priority: u32,
    pub sequence: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scheduler {
    config: SchedulerConfig,
    current_time: Timestamp,
    next_task: u32,
    next_sequence: u64,
    tasks: BTreeMap<TaskId, PeriodicTask>,
}

impl Scheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            current_time: Timestamp::ZERO,
            next_task: 0,
            next_sequence: 0,
            tasks: BTreeMap::new(),
        }
    }

    pub fn add_periodic_task(
        &mut self,
        name: impl Into<String>,
        period: DurationNs,
        phase: DurationNs,
        priority: u32,
    ) -> Result<TaskId, SchedulerError> {
        if !period.is_positive() || phase.as_nanoseconds() < 0 {
            return Err(SchedulerError::InvalidTime(
                "task period must be positive and phase non-negative".to_string(),
            ));
        }
        if phase >= period {
            return Err(SchedulerError::InvalidTime(
                "task phase must be smaller than its period".to_string(),
            ));
        }
        if period.as_nanoseconds() % self.config.base_period.as_nanoseconds() != 0 {
            return Err(SchedulerError::InvalidTime(
                "task period must be an integer multiple of the base period".to_string(),
            ));
        }
        let id = TaskId(self.next_task);
        self.next_task = self
            .next_task
            .checked_add(1)
            .ok_or(SchedulerError::IdExhausted)?;
        let next_release = Timestamp::ZERO.checked_add(phase)?;
        self.tasks.insert(
            id,
            PeriodicTask {
                id,
                name: name.into(),
                period,
                phase,
                priority,
                next_release,
            },
        );
        Ok(id)
    }

    pub fn current_time(&self) -> Timestamp {
        self.current_time
    }

    pub fn tasks(&self) -> impl Iterator<Item = &PeriodicTask> {
        self.tasks.values()
    }

    pub fn has_task(&self, task: TaskId) -> bool {
        self.tasks.contains_key(&task)
    }

    /// Advances to `timestamp` and returns every task release at or before it.
    /// Same-time releases are sorted by priority, then task ID, and receive a
    /// monotonically increasing sequence number.
    pub fn advance_to(
        &mut self,
        timestamp: Timestamp,
    ) -> Result<Vec<ScheduledEvent>, SchedulerError> {
        if timestamp < self.current_time {
            return Err(SchedulerError::InvalidTime(
                "scheduler time cannot move backwards".to_string(),
            ));
        }
        let mut pending = Vec::new();
        for task in self.tasks.values_mut() {
            while task.next_release <= timestamp {
                pending.push((task.next_release, task.priority, task.id));
                task.next_release = task.next_release.checked_add(task.period)?;
                if pending.len() > self.config.max_events_per_advance {
                    return Err(SchedulerError::EventBoundExceeded);
                }
            }
        }
        pending.sort_by_key(|(release, priority, task)| (*release, *priority, *task));
        let events = pending
            .into_iter()
            .map(|(release, priority, task)| {
                let sequence = self.next_sequence;
                self.next_sequence = self
                    .next_sequence
                    .checked_add(1)
                    .ok_or(SchedulerError::EventBoundExceeded)?;
                Ok(ScheduledEvent {
                    timestamp: release,
                    task,
                    priority,
                    sequence,
                })
            })
            .collect::<Result<Vec<_>, SchedulerError>>()?;
        self.current_time = timestamp;
        Ok(events)
    }

    pub fn reset(&mut self) -> Result<(), SchedulerError> {
        self.current_time = Timestamp::ZERO;
        self.next_sequence = 0;
        for task in self.tasks.values_mut() {
            task.next_release = Timestamp::ZERO.checked_add(task.phase)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn duration(nanoseconds: i64) -> DurationNs {
        DurationNs::from_nanoseconds(nanoseconds)
    }

    #[test]
    fn scheduler_orders_multi_rate_same_time_events_deterministically() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::new(duration(10), 32).expect("valid scheduler config"));
        let slow = scheduler
            .add_periodic_task("slow", duration(20), DurationNs::ZERO, 10)
            .expect("task");
        let fast = scheduler
            .add_periodic_task("fast", duration(10), DurationNs::ZERO, 1)
            .expect("task");

        let events = scheduler
            .advance_to(Timestamp::from_nanoseconds(20))
            .expect("advance");
        assert_eq!(events.len(), 5);
        assert_eq!(events[0].task, fast);
        assert_eq!(events[1].task, slow);
        assert_eq!(events[2].task, fast);
        assert_eq!(events[3].task, fast);
        assert_eq!(events[4].task, slow);
        assert!(events
            .windows(2)
            .all(|pair| pair[0].sequence < pair[1].sequence));
    }

    #[test]
    fn scheduler_rejects_invalid_rates_and_backwards_time() {
        let config = SchedulerConfig::new(duration(10), 8).expect("valid config");
        let mut scheduler = Scheduler::new(config);
        assert!(scheduler
            .add_periodic_task("bad", duration(15), DurationNs::ZERO, 0)
            .is_err());
        scheduler
            .add_periodic_task("good", duration(10), DurationNs::ZERO, 0)
            .expect("task");
        scheduler
            .advance_to(Timestamp::from_nanoseconds(10))
            .expect("advance");
        assert!(scheduler
            .advance_to(Timestamp::from_nanoseconds(9))
            .is_err());
    }

    #[test]
    fn scheduler_enforces_event_bound() {
        let mut scheduler =
            Scheduler::new(SchedulerConfig::new(duration(1), 2).expect("valid scheduler config"));
        scheduler
            .add_periodic_task("fast", duration(1), DurationNs::ZERO, 0)
            .expect("task");
        assert!(matches!(
            scheduler.advance_to(Timestamp::from_nanoseconds(3)),
            Err(SchedulerError::EventBoundExceeded)
        ));
    }

    struct Constant {
        output: PortId,
        value: f64,
    }

    impl BlockRuntime for Constant {
        fn configure(
            &mut self,
            parameters: &BTreeMap<String, serde_json::Value>,
        ) -> Result<(), ExecutionError> {
            if let Some(value) = parameters.get("value") {
                self.value = value.as_f64().ok_or_else(|| {
                    ExecutionError::Runtime("constant value parameter must be numeric".to_string())
                })?;
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

    struct Gain {
        input: PortId,
        output: PortId,
    }

    impl BlockRuntime for Gain {
        fn step(
            &mut self,
            _timestamp: Timestamp,
            inputs: &BTreeMap<PortId, SignalValue>,
        ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
            let Some(SignalValue::Scalar(value)) = inputs.get(&self.input) else {
                return Err(ExecutionError::Runtime(
                    "gain input is not scalar".to_string(),
                ));
            };
            Ok(BTreeMap::from([(
                self.output,
                SignalValue::Scalar(value * 2.0),
            )]))
        }
    }

    struct Decay {
        output: PortId,
    }

    impl ContinuousBlockRuntime for Decay {
        fn derivative(
            &mut self,
            _time: f64,
            state: &[f64],
            _inputs: &BTreeMap<PortId, SignalValue>,
            derivative: &mut [f64],
        ) {
            derivative[0] = -state[0];
        }

        fn outputs(
            &mut self,
            _time: f64,
            state: &[f64],
            _inputs: &BTreeMap<PortId, SignalValue>,
        ) -> Result<BTreeMap<PortId, SignalValue>, ExecutionError> {
            Ok(BTreeMap::from([(
                self.output,
                SignalValue::Scalar(state[0]),
            )]))
        }
    }

    #[test]
    fn adaptive_continuous_runtime_integrates_state_through_engine() {
        let mut model = Model::new("decay");
        let block = model.add_block("decay").expect("block");
        let output = model
            .add_port(
                block,
                "out",
                xdl_model::PortDirection::Output,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        let config = SchedulerConfig::new(duration(100_000_000), 8).expect("scheduler");
        let mut scheduler = Scheduler::new(config);
        let task = scheduler
            .add_periodic_task("decay", duration(100_000_000), DurationNs::ZERO, 0)
            .expect("task");
        let mut engine = ExecutionEngine::new(model, scheduler).expect("engine");
        engine
            .register_task(
                task,
                block,
                Box::new(
                    AdaptiveContinuousRuntime::new(
                        Box::new(Decay { output }),
                        vec![1.0],
                        AdaptiveConfig {
                            absolute_tolerance: 1e-10,
                            relative_tolerance: 1e-10,
                            minimum_step: 1e-4,
                            maximum_step: 0.2,
                        },
                    )
                    .expect("continuous runtime"),
                ),
            )
            .expect("registration");
        engine.advance_to(Timestamp::ZERO).expect("initial output");
        engine
            .advance_to(Timestamp::from_nanoseconds(100_000_000))
            .expect("integrated output");
        let SignalValue::Scalar(value) = engine.signal(output).expect("output") else {
            panic!("expected scalar output");
        };
        assert!((*value - (-0.1_f64).exp()).abs() < 1e-5);
    }

    #[test]
    fn execution_engine_routes_signals_in_scheduler_order() {
        let mut model = Model::new("gain");
        let source = model.add_block("source").expect("block");
        let gain = model.add_block("gain").expect("block");
        let source_output = model
            .add_port(
                source,
                "out",
                xdl_model::PortDirection::Output,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        let gain_input = model
            .add_port(
                gain,
                "in",
                xdl_model::PortDirection::Input,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        let gain_output = model
            .add_port(
                gain,
                "out",
                xdl_model::PortDirection::Output,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        model
            .connect(source_output, gain_input)
            .expect("connection");
        model
            .set_block_parameter(source, "value", serde_json::json!(4.0))
            .expect("source parameter");

        let config = SchedulerConfig::new(duration(10), 8).expect("config");
        let mut scheduler = Scheduler::new(config);
        let source_task = scheduler
            .add_periodic_task("source", duration(10), DurationNs::ZERO, 1)
            .expect("task");
        let gain_task = scheduler
            .add_periodic_task("gain", duration(10), DurationNs::ZERO, 0)
            .expect("task");
        let mut engine = ExecutionEngine::new(model, scheduler).expect("engine");
        engine
            .register_task(
                source_task,
                source,
                Box::new(Constant {
                    output: source_output,
                    value: 3.0,
                }),
            )
            .expect("source runtime");
        engine
            .register_task(
                gain_task,
                gain,
                Box::new(Gain {
                    input: gain_input,
                    output: gain_output,
                }),
            )
            .expect("gain runtime");

        let report = engine.advance_to(Timestamp::ZERO).expect("execute tick");
        assert_eq!(report.events.len(), 2);
        assert_eq!(report.signal_updates, 2);
        assert_eq!(report.updates.len(), 2);
        assert_eq!(report.updates[0].task, source_task);
        assert_eq!(report.updates[0].port, source_output);
        assert_eq!(report.updates[1].task, gain_task);
        assert_eq!(report.updates[1].value, SignalValue::Scalar(8.0));
        let trace = report.to_json_lines().expect("json trace");
        assert_eq!(trace.lines().count(), 2);
        assert!(trace.contains("\"port\""));
        let decoded = ExecutionReport::parse_json_lines(&trace).expect("parse trace");
        assert_eq!(decoded, report.updates);
        assert!(ExecutionReport::parse_json_lines("not json").is_err());
        let mut cursor = TraceCursor::from_json_lines(&trace).expect("trace cursor");
        assert_eq!(cursor.peek(), Some(&report.updates[0]));
        assert_eq!(cursor.next_update(), Some(&report.updates[0]));
        assert!(!cursor.is_exhausted());
        cursor.reset();
        assert_eq!(cursor.updates(), report.updates.as_slice());
        assert_eq!(
            engine
                .apply_trace_until(&mut cursor, Timestamp::ZERO)
                .expect("apply trace through time zero"),
            2
        );
        assert!(cursor.is_exhausted());
        engine
            .apply_trace_update(&report.updates[1])
            .expect("apply replay update");
        compare_traces(&report.updates, &report.updates, 0.0, 0.0).expect("equal traces");
        let mut changed = report.updates.clone();
        changed[1].value = SignalValue::Scalar(8.1);
        assert!(compare_traces(&report.updates, &changed, 1e-3, 0.0).is_err());
        compare_traces(&report.updates, &changed, 0.2, 0.0).expect("tolerant comparison");
        let mut misattributed = report.updates[1].clone();
        misattributed.block = BlockId::new(999);
        assert_eq!(
            engine.apply_trace_update(&misattributed),
            Err(ExecutionError::TraceMetadataMismatch)
        );
        let snapshot = engine.signal_snapshot();
        engine
            .restore_signal_snapshot(BTreeMap::new())
            .expect("clear snapshot");
        assert_eq!(engine.signal(gain_output), None);
        engine
            .restore_signal_snapshot(snapshot)
            .expect("restore snapshot");
        assert_eq!(engine.signal(gain_output), Some(&SignalValue::Scalar(8.0)));
    }

    #[test]
    fn execution_engine_reports_missing_runtime_before_advance() {
        let mut model = Model::new("incomplete");
        let block = model.add_block("unimplemented").expect("block");
        let scheduler =
            Scheduler::new(SchedulerConfig::new(duration(10), 4).expect("scheduler config"));
        let mut engine = ExecutionEngine::new(model, scheduler).expect("engine");
        assert_eq!(
            engine.advance_to(Timestamp::ZERO),
            Err(ExecutionError::UnregisteredBlock(block))
        );
    }

    #[test]
    fn matrix_signal_preserves_shape_and_rejects_wrong_dimensions() {
        let mut model = Model::new("matrix");
        let block = model.add_block("matrix_sink").expect("block");
        let input = model
            .add_port(
                block,
                "in",
                xdl_model::PortDirection::Input,
                xdl_model::SignalSpec {
                    shape: xdl_model::SignalShape::Matrix {
                        rows: 2,
                        columns: 2,
                    },
                    unit: None,
                    frame: None,
                    sample_period: None,
                },
            )
            .expect("input");
        let scheduler =
            Scheduler::new(SchedulerConfig::new(duration(10), 4).expect("scheduler config"));
        let mut engine = ExecutionEngine::new(model, scheduler).expect("engine");
        let matrix = SignalValue::Matrix {
            rows: 2,
            columns: 2,
            values: vec![1.0, 2.0, 3.0, 4.0],
        };
        engine
            .inject_signal(input, matrix.clone())
            .expect("matrix input");
        assert_eq!(engine.signal(input), Some(&matrix));
        assert!(engine
            .inject_signal(
                input,
                SignalValue::Matrix {
                    rows: 1,
                    columns: 4,
                    values: vec![1.0, 2.0, 3.0, 4.0],
                },
            )
            .is_err());
    }

    #[test]
    fn unit_delay_emits_initial_then_previous_input() {
        let input = PortId::new(1);
        let output = PortId::new(2);
        let mut delay = UnitDelay::new(input, output, SignalValue::Scalar(0.0)).expect("delay");
        let first = delay
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(input, SignalValue::Scalar(3.0))]),
            )
            .expect("first step");
        let second = delay
            .step(
                Timestamp::from_nanoseconds(1),
                &BTreeMap::from([(input, SignalValue::Scalar(5.0))]),
            )
            .expect("second step");
        assert_eq!(first.get(&output), Some(&SignalValue::Scalar(0.0)));
        assert_eq!(second.get(&output), Some(&SignalValue::Scalar(3.0)));
        assert_eq!(
            delay.step(Timestamp::from_nanoseconds(2), &BTreeMap::new()),
            Err(ExecutionError::MissingInput(input))
        );
    }

    #[test]
    fn unit_delay_can_emit_initial_state_before_feedback_input_exists() {
        let input = PortId::new(3);
        let output = PortId::new(4);
        let mut delay = UnitDelay::new(input, output, SignalValue::Scalar(7.0)).expect("delay");
        let first = delay
            .step(Timestamp::ZERO, &BTreeMap::new())
            .expect("initial state");
        assert_eq!(first.get(&output), Some(&SignalValue::Scalar(7.0)));
    }

    #[test]
    fn zero_order_hold_starts_with_initial_value_then_holds_last_sample() {
        let input = PortId::new(5);
        let output = PortId::new(6);
        let mut hold = ZeroOrderHold::new(input, output, SignalValue::Scalar(2.0)).expect("hold");
        let first = hold
            .step(Timestamp::ZERO, &BTreeMap::new())
            .expect("initial");
        assert_eq!(first.get(&output), Some(&SignalValue::Scalar(2.0)));
        let second = hold
            .step(
                Timestamp::from_nanoseconds(1),
                &BTreeMap::from([(input, SignalValue::Scalar(9.0))]),
            )
            .expect("sample");
        assert_eq!(second.get(&output), Some(&SignalValue::Scalar(9.0)));
        let third = hold
            .step(Timestamp::from_nanoseconds(2), &BTreeMap::new())
            .expect("held sample");
        assert_eq!(third.get(&output), Some(&SignalValue::Scalar(9.0)));
    }

    #[test]
    fn subsystem_runtime_maps_boundary_inputs_and_outputs() {
        let mut internal = Model::new("inner");
        let block = internal.add_block("gain").expect("block");
        let internal_input = internal
            .add_port(
                block,
                "in",
                xdl_model::PortDirection::Input,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        let internal_output = internal
            .add_port(
                block,
                "out",
                xdl_model::PortDirection::Output,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("port");
        let mut subsystem =
            xdl_model::Subsystem::new(xdl_model::SubsystemId::new(2), "gain_subsystem", internal);
        let boundary_input = subsystem
            .add_boundary_port(
                "in",
                xdl_model::PortDirection::Input,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("boundary input");
        let boundary_output = subsystem
            .add_boundary_port(
                "out",
                xdl_model::PortDirection::Output,
                xdl_model::SignalSpec::scalar(),
            )
            .expect("boundary output");
        subsystem
            .bind_input(boundary_input, internal_input)
            .expect("input binding");
        subsystem
            .bind_output(boundary_output, internal_output)
            .expect("output binding");
        let scheduler = Scheduler::new(SchedulerConfig::new(duration(10), 8).expect("config"));
        let mut runtime = SubsystemRuntime::new(subsystem, scheduler).expect("runtime");
        let task = runtime
            .engine_mut()
            .scheduler
            .add_periodic_task("gain", duration(10), DurationNs::ZERO, 0)
            .expect("task");
        runtime
            .engine_mut()
            .register_task(
                task,
                block,
                Box::new(Gain {
                    input: internal_input,
                    output: internal_output,
                }),
            )
            .expect("register internal block");
        let output = runtime
            .step(
                Timestamp::ZERO,
                &BTreeMap::from([(boundary_input, SignalValue::Scalar(4.0))]),
            )
            .expect("subsystem step");
        assert_eq!(
            output.get(&boundary_output),
            Some(&SignalValue::Scalar(8.0))
        );
    }
}
