// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

// Covariance and Jacobian updates index two fixed-size matrices with the same
// loop variables; the iterator form is harder to audit against the equations.
#![allow(clippy::needless_range_loop)]

//! Small deterministic navigation estimators for the XDL GNC vertical slice.
//!
//! The alpha-beta filter here is intentionally a transparent baseline, not a
//! replacement for an error-state EKF. Its purpose is to establish timestamp,
//! measurement-quality, residual, and replay contracts before adding a more
//! sophisticated estimator.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{GncError, Quality, Quaternion, SignalStatus, Timestamp, Vec3};
use xdl_gnc_simulation::{GnssMeasurement, ImuMeasurement, KinematicState};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct AlphaBetaConfig {
    pub position_gain: f64,
    pub velocity_gain: f64,
}

impl AlphaBetaConfig {
    pub fn new(position_gain: f64, velocity_gain: f64) -> Result<Self, GncError> {
        if !position_gain.is_finite() || !velocity_gain.is_finite() {
            return Err(GncError::NonFinite {
                context: "alpha-beta gains".to_string(),
            });
        }
        if !(0.0..=1.0).contains(&position_gain) || !(0.0..=1.0).contains(&velocity_gain) {
            return Err(GncError::InvalidValue {
                context: "alpha-beta gains".to_string(),
                reason: "gains must be between 0 and 1".to_string(),
            });
        }
        Ok(Self {
            position_gain,
            velocity_gain,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavigationEstimate {
    pub state: KinematicState,
    pub position_residual: Vec3,
    pub velocity_residual: Vec3,
    pub status: SignalStatus,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AlphaBetaFilter {
    config: AlphaBetaConfig,
    state: KinematicState,
    last_position_residual: Vec3,
    last_velocity_residual: Vec3,
}

pub type Covariance6 = [[f64; 6]; 6];
pub type Covariance15 = [[f64; 15]; 15];

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EkfConfig {
    pub acceleration_process_noise: f64,
    pub position_measurement_noise: f64,
    pub velocity_measurement_noise: f64,
}

impl EkfConfig {
    pub fn new(
        acceleration_process_noise: f64,
        position_measurement_noise: f64,
        velocity_measurement_noise: f64,
    ) -> Result<Self, GncError> {
        let values = [
            acceleration_process_noise,
            position_measurement_noise,
            velocity_measurement_noise,
        ];
        if !values.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "EKF noise configuration".to_string(),
            });
        }
        if values.iter().any(|value| *value <= 0.0) {
            return Err(GncError::InvalidValue {
                context: "EKF noise configuration".to_string(),
                reason: "all noise variances must be positive".to_string(),
            });
        }
        Ok(Self {
            acceleration_process_noise,
            position_measurement_noise,
            velocity_measurement_noise,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EkfEstimate {
    pub state: KinematicState,
    pub covariance: Covariance6,
    pub position_innovation: Vec3,
    pub velocity_innovation: Vec3,
    pub status: SignalStatus,
}

/// Fixed-size 3D position/velocity EKF with sequential GNSS updates.
///
/// This is a navigation foundation, not a full inertial error-state filter:
/// attitude and IMU bias states are intentionally deferred until the state,
/// covariance, and replay contracts are independently exercised.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PositionVelocityEkf {
    config: EkfConfig,
    state: KinematicState,
    covariance: Covariance6,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrapdownConfig {
    /// Gravity expressed in the navigation frame, in m/s².
    pub gravity: Vec3,
}

impl StrapdownConfig {
    pub fn new(gravity: Vec3) -> Result<Self, GncError> {
        if !gravity.is_finite() {
            return Err(GncError::NonFinite {
                context: "strapdown gravity".to_string(),
            });
        }
        Ok(Self { gravity })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrapdownState {
    pub timestamp: Timestamp,
    pub position: Vec3,
    pub velocity: Vec3,
    /// Body-to-navigation attitude represented as a unit quaternion.
    pub attitude: Quaternion,
    pub accelerometer_bias: Vec3,
    pub gyroscope_bias: Vec3,
}

impl StrapdownState {
    pub fn new(
        timestamp: Timestamp,
        position: Vec3,
        velocity: Vec3,
        attitude: Quaternion,
        accelerometer_bias: Vec3,
        gyroscope_bias: Vec3,
    ) -> Result<Self, GncError> {
        if !position.is_finite()
            || !velocity.is_finite()
            || !accelerometer_bias.is_finite()
            || !gyroscope_bias.is_finite()
        {
            return Err(GncError::NonFinite {
                context: "strapdown state".to_string(),
            });
        }
        Ok(Self {
            timestamp,
            position,
            velocity,
            attitude: attitude.try_normalize()?,
            accelerometer_bias,
            gyroscope_bias,
        })
    }
}

/// Fixed-step strapdown inertial mechanization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StrapdownMechanization {
    config: StrapdownConfig,
    state: StrapdownState,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ErrorStateEkfConfig {
    pub acceleration_noise: f64,
    pub gyro_noise: f64,
    pub accelerometer_bias_noise: f64,
    pub gyroscope_bias_noise: f64,
    pub position_measurement_noise: f64,
    pub velocity_measurement_noise: f64,
}

impl ErrorStateEkfConfig {
    pub fn new(
        acceleration_noise: f64,
        gyro_noise: f64,
        accelerometer_bias_noise: f64,
        gyroscope_bias_noise: f64,
        position_measurement_noise: f64,
        velocity_measurement_noise: f64,
    ) -> Result<Self, GncError> {
        let values = [
            acceleration_noise,
            gyro_noise,
            accelerometer_bias_noise,
            gyroscope_bias_noise,
            position_measurement_noise,
            velocity_measurement_noise,
        ];
        if !values.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "error-state EKF noise configuration".to_string(),
            });
        }
        if values.iter().any(|value| *value <= 0.0) {
            return Err(GncError::InvalidValue {
                context: "error-state EKF noise configuration".to_string(),
                reason: "all noise variances must be positive".to_string(),
            });
        }
        Ok(Self {
            acceleration_noise,
            gyro_noise,
            accelerometer_bias_noise,
            gyroscope_bias_noise,
            position_measurement_noise,
            velocity_measurement_noise,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ErrorStateEstimate {
    pub nominal: StrapdownState,
    pub covariance: Covariance15,
    pub position_innovation: Vec3,
    pub velocity_innovation: Vec3,
    pub status: SignalStatus,
}

/// Coupled 15-state error-state EKF around a strapdown nominal state.
///
/// Error-state ordering is position, velocity, attitude, accelerometer bias,
/// and gyroscope bias. The nominal state is propagated nonlinearly while the
/// covariance uses a first-order fixed-step linearization.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ErrorStateEkf {
    config: ErrorStateEkfConfig,
    nominal: StrapdownState,
    covariance: Covariance15,
    gravity: Vec3,
}

impl ErrorStateEkf {
    pub fn new(
        config: ErrorStateEkfConfig,
        nominal: StrapdownState,
        covariance: Covariance15,
        gravity: Vec3,
    ) -> Result<Self, GncError> {
        validate_covariance15(covariance)?;
        if !gravity.is_finite() {
            return Err(GncError::NonFinite {
                context: "error-state EKF gravity".to_string(),
            });
        }
        Ok(Self {
            config,
            nominal,
            covariance,
            gravity,
        })
    }

    pub fn nominal(&self) -> StrapdownState {
        self.nominal
    }

    pub fn covariance(&self) -> Covariance15 {
        self.covariance
    }

    pub fn predict(&mut self, measurement: ImuMeasurement) -> Result<(), GncError> {
        let dt = measurement
            .status
            .timestamp
            .duration_since(self.nominal.timestamp)?;
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "error-state EKF timestamps must be strictly increasing".to_string(),
            ));
        }
        if !measurement.status.is_usable() {
            return Err(GncError::InvalidValue {
                context: "error-state EKF IMU measurement".to_string(),
                reason: "measurement quality is not usable".to_string(),
            });
        }

        let seconds = dt.as_seconds();
        let corrected_force = measurement.specific_force - self.nominal.accelerometer_bias;
        let corrected_rate = measurement.angular_rate - self.nominal.gyroscope_bias;
        let old_attitude = self.nominal.attitude;
        let attitude_delta = attitude_delta(corrected_rate, seconds)?;
        let next_attitude = (old_attitude * attitude_delta).try_normalize()?;
        let acceleration_nav = old_attitude.rotate_vector(corrected_force)? + self.gravity;
        self.nominal = StrapdownState::new(
            measurement.status.timestamp,
            self.nominal.position
                + self.nominal.velocity * seconds
                + acceleration_nav * (0.5 * seconds * seconds),
            self.nominal.velocity + acceleration_nav * seconds,
            next_attitude,
            self.nominal.accelerometer_bias,
            self.nominal.gyroscope_bias,
        )?;

        let mut transition = identity15();
        let force_skew = skew(corrected_force);
        let rate_skew = skew(corrected_rate);
        let rotation = rotation_matrix(old_attitude);
        for axis in 0..3 {
            transition[axis][axis + 3] = seconds;
            transition[axis + 3][axis + 9] = -seconds;
            transition[axis + 6][axis + 12] = -seconds;
            for component_index in 0..3 {
                let force_coupling = (0..3)
                    .map(|inner| rotation[axis][inner] * force_skew[inner][component_index])
                    .sum::<f64>();
                transition[axis + 3][component_index + 6] = -seconds * force_coupling;
                transition[axis + 6][component_index + 6] =
                    -seconds * rate_skew[axis][component_index];
            }
        }

        let mut propagated = mat15_mul(
            mat15_mul(transition, self.covariance),
            transpose15(transition),
        );
        for axis in 0..3 {
            propagated[axis + 3][axis + 3] += self.config.acceleration_noise * seconds * seconds;
            propagated[axis + 6][axis + 6] += self.config.gyro_noise * seconds * seconds;
            propagated[axis + 9][axis + 9] += self.config.accelerometer_bias_noise * seconds;
            propagated[axis + 12][axis + 12] += self.config.gyroscope_bias_noise * seconds;
        }
        self.covariance = symmetrize_and_validate15(propagated)?;
        Ok(())
    }

    pub fn update_gnss(
        &mut self,
        measurement: GnssMeasurement,
    ) -> Result<ErrorStateEstimate, GncError> {
        if measurement.status.quality == Quality::Invalid {
            return Err(GncError::InvalidValue {
                context: "error-state EKF GNSS measurement".to_string(),
                reason: "measurement is marked invalid".to_string(),
            });
        }
        if measurement.status.timestamp != self.nominal.timestamp {
            return Err(GncError::InvalidTime(
                "predict error-state EKF to GNSS timestamp before updating".to_string(),
            ));
        }
        let position_innovation = measurement.position - self.nominal.position;
        let velocity_innovation = measurement.velocity - self.nominal.velocity;
        for axis in 0..3 {
            self.scalar_update(
                axis,
                component(position_innovation, axis)?,
                self.config.position_measurement_noise,
            )?;
            self.scalar_update(
                axis + 3,
                component(velocity_innovation, axis)?,
                self.config.velocity_measurement_noise,
            )?;
        }
        Ok(ErrorStateEstimate {
            nominal: self.nominal,
            covariance: self.covariance,
            position_innovation,
            velocity_innovation,
            status: measurement.status,
        })
    }

    fn scalar_update(
        &mut self,
        measurement_index: usize,
        innovation: f64,
        noise: f64,
    ) -> Result<(), GncError> {
        let innovation_variance = self.covariance[measurement_index][measurement_index] + noise;
        if !innovation_variance.is_finite() || innovation_variance <= 0.0 {
            return Err(GncError::InvalidValue {
                context: "error-state EKF innovation variance".to_string(),
                reason: "variance must be finite and positive".to_string(),
            });
        }
        let gain: [f64; 15] = core::array::from_fn(|row| {
            self.covariance[row][measurement_index] / innovation_variance
        });
        let mut error = [0.0; 15];
        for row in 0..15 {
            error[row] = gain[row] * innovation;
        }
        self.apply_error(error)?;
        let prior = self.covariance;
        let mut updated = [[0.0; 15]; 15];
        for row in 0..15 {
            for column in 0..15 {
                updated[row][column] =
                    prior[row][column] - gain[row] * prior[measurement_index][column];
            }
        }
        self.covariance = symmetrize_and_validate15(updated)?;
        Ok(())
    }

    fn apply_error(&mut self, error: [f64; 15]) -> Result<(), GncError> {
        let attitude_error = Vec3::new(error[6], error[7], error[8]);
        let delta = attitude_delta(attitude_error, 1.0)?;
        self.nominal = StrapdownState::new(
            self.nominal.timestamp,
            self.nominal.position + Vec3::new(error[0], error[1], error[2]),
            self.nominal.velocity + Vec3::new(error[3], error[4], error[5]),
            (self.nominal.attitude * delta).try_normalize()?,
            self.nominal.accelerometer_bias + Vec3::new(error[9], error[10], error[11]),
            self.nominal.gyroscope_bias + Vec3::new(error[12], error[13], error[14]),
        )?;
        Ok(())
    }
}

impl StrapdownMechanization {
    pub fn new(config: StrapdownConfig, state: StrapdownState) -> Self {
        Self { config, state }
    }

    pub fn state(&self) -> StrapdownState {
        self.state
    }

    pub fn step(&mut self, measurement: ImuMeasurement) -> Result<StrapdownState, GncError> {
        let dt = measurement
            .status
            .timestamp
            .duration_since(self.state.timestamp)?;
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "IMU timestamps must be strictly increasing".to_string(),
            ));
        }
        if !measurement.status.is_usable() {
            return Err(GncError::InvalidValue {
                context: "strapdown IMU measurement".to_string(),
                reason: "measurement quality is not usable".to_string(),
            });
        }

        let seconds = dt.as_seconds();
        let corrected_acceleration = measurement.specific_force - self.state.accelerometer_bias;
        let corrected_rate = measurement.angular_rate - self.state.gyroscope_bias;
        let attitude_delta = attitude_delta(corrected_rate, seconds)?;
        let next_attitude = (self.state.attitude * attitude_delta).try_normalize()?;
        let acceleration_nav =
            self.state.attitude.rotate_vector(corrected_acceleration)? + self.config.gravity;
        let next_position = self.state.position
            + self.state.velocity * seconds
            + acceleration_nav * (0.5 * seconds * seconds);
        let next_velocity = self.state.velocity + acceleration_nav * seconds;
        self.state = StrapdownState::new(
            measurement.status.timestamp,
            next_position,
            next_velocity,
            next_attitude,
            self.state.accelerometer_bias,
            self.state.gyroscope_bias,
        )?;
        Ok(self.state)
    }
}

fn attitude_delta(body_rate: Vec3, seconds: f64) -> Result<Quaternion, GncError> {
    if !body_rate.is_finite() || !seconds.is_finite() {
        return Err(GncError::NonFinite {
            context: "attitude propagation".to_string(),
        });
    }
    let rotation_vector = body_rate * seconds;
    let angle = rotation_vector.norm();
    if angle <= f64::EPSILON {
        return Ok(Quaternion::IDENTITY);
    }
    Quaternion::from_axis_angle(rotation_vector, angle)
}

impl PositionVelocityEkf {
    pub fn new(
        config: EkfConfig,
        initial_state: KinematicState,
        initial_covariance: Covariance6,
    ) -> Result<Self, GncError> {
        validate_covariance(initial_covariance)?;
        Ok(Self {
            config,
            state: initial_state,
            covariance: initial_covariance,
        })
    }

    pub fn state(&self) -> KinematicState {
        self.state
    }

    pub fn covariance(&self) -> Covariance6 {
        self.covariance
    }

    pub fn predict(&mut self, timestamp: Timestamp, acceleration: Vec3) -> Result<(), GncError> {
        let dt = timestamp.duration_since(self.state.timestamp)?;
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "EKF timestamps must be strictly increasing".to_string(),
            ));
        }
        if !acceleration.is_finite() {
            return Err(GncError::NonFinite {
                context: "EKF acceleration".to_string(),
            });
        }

        let seconds = dt.as_seconds();
        self.state = KinematicState::new(
            timestamp,
            self.state.frame,
            self.state.position
                + self.state.velocity * seconds
                + acceleration * (0.5 * seconds * seconds),
            self.state.velocity + acceleration * seconds,
        )?;

        let mut predicted = [[0.0; 6]; 6];
        for row in 0..6 {
            for column in 0..6 {
                let mut value = self.covariance[row][column];
                if row < 3 {
                    value += seconds * self.covariance[row + 3][column];
                }
                if column < 3 {
                    value += seconds * self.covariance[row][column + 3];
                }
                if row < 3 && column < 3 {
                    value += seconds * seconds * self.covariance[row + 3][column + 3];
                }
                predicted[row][column] = value;
            }
        }

        let q = self.config.acceleration_process_noise;
        let q_position = 0.25 * seconds.powi(4) * q;
        let q_cross = 0.5 * seconds.powi(3) * q;
        let q_velocity = seconds * seconds * q;
        for axis in 0..3 {
            predicted[axis][axis] += q_position;
            predicted[axis][axis + 3] += q_cross;
            predicted[axis + 3][axis] += q_cross;
            predicted[axis + 3][axis + 3] += q_velocity;
        }
        self.covariance = symmetrize_and_validate(predicted)?;
        Ok(())
    }

    pub fn update_gnss(&mut self, measurement: GnssMeasurement) -> Result<EkfEstimate, GncError> {
        if measurement.status.quality == Quality::Invalid {
            return Err(GncError::InvalidValue {
                context: "EKF GNSS measurement".to_string(),
                reason: "measurement is marked invalid".to_string(),
            });
        }
        if measurement.status.timestamp != self.state.timestamp {
            return Err(GncError::InvalidTime(
                "predict EKF to GNSS timestamp before applying measurement".to_string(),
            ));
        }

        let position_innovation = measurement.position - self.state.position;
        let velocity_innovation = measurement.velocity - self.state.velocity;
        for axis in 0..3 {
            self.scalar_update(
                axis,
                component(position_innovation, axis)?,
                self.config.position_measurement_noise,
            )?;
            self.scalar_update(
                axis + 3,
                component(velocity_innovation, axis)?,
                self.config.velocity_measurement_noise,
            )?;
        }

        Ok(EkfEstimate {
            state: self.state,
            covariance: self.covariance,
            position_innovation,
            velocity_innovation,
            status: measurement.status,
        })
    }

    fn scalar_update(
        &mut self,
        measurement_index: usize,
        innovation: f64,
        noise: f64,
    ) -> Result<(), GncError> {
        let innovation_variance = self.covariance[measurement_index][measurement_index] + noise;
        if !innovation_variance.is_finite() || innovation_variance <= 0.0 {
            return Err(GncError::InvalidValue {
                context: "EKF innovation variance".to_string(),
                reason: "variance must be finite and positive".to_string(),
            });
        }

        let gain: [f64; 6] = core::array::from_fn(|row| {
            self.covariance[row][measurement_index] / innovation_variance
        });
        let mut corrected = self.state;
        let correction = Vec3::new(
            gain[0] * innovation,
            gain[1] * innovation,
            gain[2] * innovation,
        );
        let velocity_correction = Vec3::new(
            gain[3] * innovation,
            gain[4] * innovation,
            gain[5] * innovation,
        );
        corrected.position = corrected.position + correction;
        corrected.velocity = corrected.velocity + velocity_correction;
        self.state = KinematicState::new(
            corrected.timestamp,
            self.state.frame,
            corrected.position,
            corrected.velocity,
        )?;

        let prior = self.covariance;
        let mut corrected_covariance = [[0.0; 6]; 6];
        for row in 0..6 {
            for column in 0..6 {
                corrected_covariance[row][column] =
                    prior[row][column] - gain[row] * prior[measurement_index][column];
            }
        }
        self.covariance = symmetrize_and_validate(corrected_covariance)?;
        Ok(())
    }
}

fn component(value: Vec3, index: usize) -> Result<f64, GncError> {
    match index {
        0 => Ok(value.x),
        1 => Ok(value.y),
        2 => Ok(value.z),
        _ => Err(GncError::InvalidValue {
            context: "EKF measurement component".to_string(),
            reason: format!("component index {index} is outside Vec3"),
        }),
    }
}

fn validate_covariance(covariance: Covariance6) -> Result<(), GncError> {
    for row in covariance {
        if !row.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "EKF covariance".to_string(),
            });
        }
    }
    for index in 0..6 {
        if covariance[index][index] < 0.0 {
            return Err(GncError::InvalidValue {
                context: "EKF covariance".to_string(),
                reason: "diagonal variances cannot be negative".to_string(),
            });
        }
    }
    Ok(())
}

fn symmetrize_and_validate(covariance: Covariance6) -> Result<Covariance6, GncError> {
    let mut result = covariance;
    for row in 0..6 {
        for column in row..6 {
            let value = 0.5 * (covariance[row][column] + covariance[column][row]);
            result[row][column] = value;
            result[column][row] = value;
        }
    }
    validate_covariance(result)?;
    Ok(result)
}

fn identity15() -> Covariance15 {
    let mut identity = [[0.0; 15]; 15];
    for index in 0..15 {
        identity[index][index] = 1.0;
    }
    identity
}

fn transpose15(matrix: Covariance15) -> Covariance15 {
    let mut transposed = [[0.0; 15]; 15];
    for row in 0..15 {
        for column in 0..15 {
            transposed[row][column] = matrix[column][row];
        }
    }
    transposed
}

fn mat15_mul(left: Covariance15, right: Covariance15) -> Covariance15 {
    let mut result = [[0.0; 15]; 15];
    for row in 0..15 {
        for column in 0..15 {
            result[row][column] = (0..15)
                .map(|index| left[row][index] * right[index][column])
                .sum();
        }
    }
    result
}

fn skew(value: Vec3) -> [[f64; 3]; 3] {
    [
        [0.0, -value.z, value.y],
        [value.z, 0.0, -value.x],
        [-value.y, value.x, 0.0],
    ]
}

fn rotation_matrix(q: Quaternion) -> [[f64; 3]; 3] {
    let w = q.w;
    let x = q.x;
    let y = q.y;
    let z = q.z;
    [
        [
            1.0 - 2.0 * (y * y + z * z),
            2.0 * (x * y - z * w),
            2.0 * (x * z + y * w),
        ],
        [
            2.0 * (x * y + z * w),
            1.0 - 2.0 * (x * x + z * z),
            2.0 * (y * z - x * w),
        ],
        [
            2.0 * (x * z - y * w),
            2.0 * (y * z + x * w),
            1.0 - 2.0 * (x * x + y * y),
        ],
    ]
}

fn validate_covariance15(covariance: Covariance15) -> Result<(), GncError> {
    for row in covariance {
        if !row.iter().all(|value| value.is_finite()) {
            return Err(GncError::NonFinite {
                context: "error-state EKF covariance".to_string(),
            });
        }
    }
    for index in 0..15 {
        if covariance[index][index] < 0.0 {
            return Err(GncError::InvalidValue {
                context: "error-state EKF covariance".to_string(),
                reason: "diagonal variances cannot be negative".to_string(),
            });
        }
    }
    Ok(())
}

fn symmetrize_and_validate15(covariance: Covariance15) -> Result<Covariance15, GncError> {
    let mut result = covariance;
    for row in 0..15 {
        for column in row..15 {
            let value = 0.5 * (covariance[row][column] + covariance[column][row]);
            result[row][column] = value;
            result[column][row] = value;
        }
    }
    validate_covariance15(result)?;
    Ok(result)
}

impl AlphaBetaFilter {
    pub fn new(config: AlphaBetaConfig, initial_state: KinematicState) -> Self {
        Self {
            config,
            state: initial_state,
            last_position_residual: Vec3::ZERO,
            last_velocity_residual: Vec3::ZERO,
        }
    }

    pub fn state(&self) -> KinematicState {
        self.state
    }

    pub fn predict(&mut self, timestamp: Timestamp, acceleration: Vec3) -> Result<(), GncError> {
        let dt = timestamp.duration_since(self.state.timestamp)?;
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "navigation timestamps must be strictly increasing".to_string(),
            ));
        }
        if !acceleration.is_finite() {
            return Err(GncError::NonFinite {
                context: "navigation acceleration".to_string(),
            });
        }
        let seconds = dt.as_seconds();
        self.state = KinematicState::new(
            timestamp,
            self.state.frame,
            self.state.position
                + self.state.velocity * seconds
                + acceleration * (0.5 * seconds * seconds),
            self.state.velocity + acceleration * seconds,
        )?;
        Ok(())
    }

    pub fn update_gnss(
        &mut self,
        measurement: GnssMeasurement,
    ) -> Result<NavigationEstimate, GncError> {
        if measurement.status.quality == Quality::Invalid {
            return Err(GncError::InvalidValue {
                context: "GNSS measurement".to_string(),
                reason: "measurement is marked invalid".to_string(),
            });
        }
        if measurement.status.timestamp < self.state.timestamp {
            return Err(GncError::InvalidTime(
                "GNSS measurement is older than estimator state".to_string(),
            ));
        }
        if measurement.status.timestamp > self.state.timestamp {
            return Err(GncError::InvalidTime(
                "predict to GNSS timestamp before applying measurement".to_string(),
            ));
        }

        let position_residual = measurement.position - self.state.position;
        let velocity_residual = measurement.velocity - self.state.velocity;
        self.state = KinematicState::new(
            self.state.timestamp,
            self.state.frame,
            self.state.position + position_residual * self.config.position_gain,
            self.state.velocity + velocity_residual * self.config.velocity_gain,
        )?;
        self.last_position_residual = position_residual;
        self.last_velocity_residual = velocity_residual;

        Ok(NavigationEstimate {
            state: self.state,
            position_residual,
            velocity_residual,
            status: measurement.status,
        })
    }

    pub fn step(
        &mut self,
        timestamp: Timestamp,
        acceleration: Vec3,
        gnss: Option<GnssMeasurement>,
    ) -> Result<NavigationEstimate, GncError> {
        self.predict(timestamp, acceleration)?;
        match gnss {
            Some(measurement) => self.update_gnss(measurement),
            None => Ok(NavigationEstimate {
                state: self.state,
                position_residual: self.last_position_residual,
                velocity_residual: self.last_velocity_residual,
                status: SignalStatus {
                    quality: Quality::Degraded,
                    timestamp,
                },
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdl_gnc_core::FrameId;

    #[test]
    fn alpha_beta_update_reduces_position_error() {
        let config = AlphaBetaConfig::new(0.5, 0.25).expect("valid gains");
        let initial = KinematicState::new(
            Timestamp::ZERO,
            FrameId::Inertial,
            Vec3::ZERO,
            Vec3::new(1.0, 0.0, 0.0),
        )
        .expect("finite state");
        let mut filter = AlphaBetaFilter::new(config, initial);
        let timestamp = Timestamp::from_seconds(1.0).expect("finite timestamp");
        let measurement = GnssMeasurement::new(
            timestamp,
            FrameId::Inertial,
            Vec3::new(1.2, 0.0, 0.0),
            Vec3::new(1.1, 0.0, 0.0),
        )
        .expect("finite GNSS measurement");
        let estimate = filter
            .step(timestamp, Vec3::ZERO, Some(measurement))
            .expect("valid estimator step");

        assert!((estimate.position_residual.x - 0.2).abs() < 1e-12);
        assert!((estimate.state.position.x - 1.1).abs() < 1e-12);
        assert!((estimate.state.velocity.x - 1.025).abs() < 1e-12);
    }

    #[test]
    fn missing_gnss_is_explicitly_degraded() {
        let config = AlphaBetaConfig::new(0.5, 0.25).expect("valid gains");
        let initial =
            KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
                .expect("finite state");
        let mut filter = AlphaBetaFilter::new(config, initial);
        let estimate = filter
            .step(
                Timestamp::from_seconds(1.0).expect("finite timestamp"),
                Vec3::ZERO,
                None,
            )
            .expect("valid prediction");
        assert_eq!(estimate.status.quality, Quality::Degraded);
    }

    #[test]
    fn future_gnss_requires_prediction_first() {
        let config = AlphaBetaConfig::new(0.5, 0.25).expect("valid gains");
        let initial =
            KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
                .expect("finite state");
        let mut filter = AlphaBetaFilter::new(config, initial);
        let measurement = GnssMeasurement::new(
            Timestamp::from_seconds(1.0).expect("finite timestamp"),
            FrameId::Inertial,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite GNSS measurement");
        assert!(filter.update_gnss(measurement).is_err());
    }

    fn diagonal_covariance(value: f64) -> Covariance6 {
        let mut covariance = [[0.0; 6]; 6];
        for index in 0..6 {
            covariance[index][index] = value;
        }
        covariance
    }

    #[test]
    fn ekf_update_reduces_position_variance() {
        let config = EkfConfig::new(0.01, 1.0, 1.0).expect("valid noise");
        let initial =
            KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
                .expect("finite state");
        let mut filter = PositionVelocityEkf::new(config, initial, diagonal_covariance(4.0))
            .expect("valid covariance");
        filter
            .predict(
                Timestamp::from_seconds(1.0).expect("finite timestamp"),
                Vec3::ZERO,
            )
            .expect("valid prediction");
        let before = filter.covariance()[0][0];
        let measurement = GnssMeasurement::new(
            filter.state().timestamp,
            FrameId::Inertial,
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::new(1.0, 0.0, 0.0),
        )
        .expect("finite GNSS measurement");
        let estimate = filter.update_gnss(measurement).expect("valid update");
        assert!(estimate.covariance[0][0] < before);
        assert!(estimate.position_innovation.x > 0.0);
    }

    #[test]
    fn ekf_rejects_negative_covariance() {
        let config = EkfConfig::new(0.01, 1.0, 1.0).expect("valid noise");
        let initial =
            KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
                .expect("finite state");
        let mut covariance = [[0.0; 6]; 6];
        covariance[0][0] = -1.0;
        assert!(PositionVelocityEkf::new(config, initial, covariance).is_err());
    }

    #[test]
    fn strapdown_stationary_specific_force_cancels_gravity() {
        let config = StrapdownConfig::new(Vec3::new(0.0, 0.0, -9.81)).expect("finite gravity");
        let state = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite strapdown state");
        let mut mechanization = StrapdownMechanization::new(config, state);
        let measurement = ImuMeasurement::new(
            Timestamp::from_seconds(1.0).expect("finite timestamp"),
            FrameId::Inertial,
            Vec3::new(0.0, 0.0, 9.81),
            Vec3::ZERO,
        )
        .expect("finite IMU measurement");
        let next = mechanization.step(measurement).expect("valid IMU step");
        assert!(next.position.norm() < 1e-12);
        assert!(next.velocity.norm() < 1e-12);
    }

    #[test]
    fn strapdown_propagates_yaw_rate() {
        let config = StrapdownConfig::new(Vec3::ZERO).expect("finite gravity");
        let state = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite strapdown state");
        let mut mechanization = StrapdownMechanization::new(config, state);
        let measurement = ImuMeasurement::new(
            Timestamp::from_seconds(1.0).expect("finite timestamp"),
            FrameId::Inertial,
            Vec3::ZERO,
            Vec3::new(0.0, 0.0, core::f64::consts::FRAC_PI_2),
        )
        .expect("finite IMU measurement");
        let next = mechanization.step(measurement).expect("valid IMU step");
        let rotated = next
            .attitude
            .rotate_vector(Vec3::new(1.0, 0.0, 0.0))
            .expect("valid attitude");
        assert!(rotated.x.abs() < 1e-12);
        assert!((rotated.y - 1.0).abs() < 1e-12);
    }

    fn diagonal_covariance15(value: f64) -> Covariance15 {
        let mut covariance = [[0.0; 15]; 15];
        for index in 0..15 {
            covariance[index][index] = value;
        }
        covariance
    }

    #[test]
    fn error_state_ekf_updates_nominal_and_reduces_covariance() {
        let config = ErrorStateEkfConfig::new(0.01, 0.01, 0.001, 0.001, 1.0, 1.0)
            .expect("valid EKF configuration");
        let nominal = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite nominal state");
        let mut filter = ErrorStateEkf::new(
            config,
            nominal,
            diagonal_covariance15(4.0),
            Vec3::new(0.0, 0.0, -9.81),
        )
        .expect("valid covariance");
        let timestamp = Timestamp::from_seconds(1.0).expect("finite timestamp");
        let imu = ImuMeasurement::new(
            timestamp,
            FrameId::Inertial,
            Vec3::new(0.0, 0.0, 9.81),
            Vec3::ZERO,
        )
        .expect("finite IMU measurement");
        filter.predict(imu).expect("valid prediction");
        let before = filter.covariance()[0][0];
        let gnss = GnssMeasurement::new(
            timestamp,
            FrameId::Inertial,
            Vec3::new(2.0, 0.0, 0.0),
            Vec3::ZERO,
        )
        .expect("finite GNSS measurement");
        let estimate = filter.update_gnss(gnss).expect("valid update");
        assert!(estimate.position_innovation.x > 0.0);
        assert!(estimate.nominal.position.x > 0.0);
        assert!(estimate.covariance[0][0] < before);
    }

    #[test]
    fn error_state_ekf_rejects_unusable_imu() {
        let config = ErrorStateEkfConfig::new(0.01, 0.01, 0.001, 0.001, 1.0, 1.0)
            .expect("valid EKF configuration");
        let nominal = StrapdownState::new(
            Timestamp::ZERO,
            Vec3::ZERO,
            Vec3::ZERO,
            Quaternion::IDENTITY,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite nominal state");
        let mut filter =
            ErrorStateEkf::new(config, nominal, diagonal_covariance15(1.0), Vec3::ZERO)
                .expect("valid covariance");
        let imu = ImuMeasurement::new(
            Timestamp::from_seconds(1.0).expect("finite timestamp"),
            FrameId::Inertial,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite IMU measurement");
        let unusable = ImuMeasurement {
            status: SignalStatus {
                quality: Quality::Invalid,
                timestamp: imu.status.timestamp,
            },
            ..imu
        };
        assert!(filter.predict(unusable).is_err());
    }
}
