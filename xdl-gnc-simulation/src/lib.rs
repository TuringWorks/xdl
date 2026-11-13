// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic simulation primitives for the XDL GNC vertical slice.
//!
//! This crate intentionally starts with a small constant-acceleration truth model
//! and explicit sensor records. It provides a stable place to add scheduling,
//! disturbances, replay, and Monte Carlo execution without hiding time, frame,
//! unit, or validity semantics inside the estimator or controller.
//!
//! Units are fixed by record type and exposed through [`VectorMeta`]:
//! positions in metres, velocities in m/s, specific force in m/s^2, and
//! angular rates in rad/s.

use serde::{Deserialize, Serialize};
use xdl_gnc_core::{
    DurationNs, FrameId, GncError, Quality, SignalStatus, Timestamp, Unit, Vec3, VectorMeta,
};

/// A point-mass state expressed in `frame`. Position is in metres and velocity
/// in m/s.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct KinematicState {
    pub timestamp: Timestamp,
    pub frame: FrameId,
    pub position: Vec3,
    pub velocity: Vec3,
}

impl KinematicState {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        position: Vec3,
        velocity: Vec3,
    ) -> Result<Self, GncError> {
        if !position.is_finite() {
            return Err(GncError::NonFinite {
                context: "kinematic position".to_string(),
            });
        }
        if !velocity.is_finite() {
            return Err(GncError::NonFinite {
                context: "kinematic velocity".to_string(),
            });
        }
        Ok(Self {
            timestamp,
            frame,
            position,
            velocity,
        })
    }

    pub fn position_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::Meter)
    }

    pub fn velocity_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::MeterPerSecond)
    }
}

/// Constant-acceleration truth model using an explicit fixed-step update.
/// `acceleration` is in m/s^2 in `frame`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ConstantAccelerationModel {
    pub frame: FrameId,
    pub acceleration: Vec3,
}

impl ConstantAccelerationModel {
    pub fn new(frame: FrameId, acceleration: Vec3) -> Result<Self, GncError> {
        if !acceleration.is_finite() {
            return Err(GncError::NonFinite {
                context: "constant acceleration".to_string(),
            });
        }
        Ok(Self {
            frame,
            acceleration,
        })
    }

    pub fn step(&self, state: &KinematicState, dt: DurationNs) -> Result<KinematicState, GncError> {
        if state.frame != self.frame {
            return Err(GncError::InvalidValue {
                context: "constant-acceleration step".to_string(),
                reason: format!(
                    "state frame {:?} does not match model frame {:?}",
                    state.frame, self.frame
                ),
            });
        }
        if !dt.is_positive() {
            return Err(GncError::InvalidTime(
                "simulation step must be positive".to_string(),
            ));
        }
        let seconds = dt.as_seconds();
        let next_position = state.position
            + state.velocity * seconds
            + self.acceleration * (0.5 * seconds * seconds);
        let next_velocity = state.velocity + self.acceleration * seconds;
        let next_timestamp = state.timestamp.checked_add(dt)?;
        KinematicState::new(next_timestamp, self.frame, next_position, next_velocity)
    }
}

/// A deterministic idealized IMU sample. Specific force is in m/s^2 and angular
/// rate in rad/s, both in `frame`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ImuMeasurement {
    pub status: SignalStatus,
    pub frame: FrameId,
    pub specific_force: Vec3,
    pub angular_rate: Vec3,
}

impl ImuMeasurement {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        specific_force: Vec3,
        angular_rate: Vec3,
    ) -> Result<Self, GncError> {
        if !specific_force.is_finite() {
            return Err(GncError::NonFinite {
                context: "IMU specific force".to_string(),
            });
        }
        if !angular_rate.is_finite() {
            return Err(GncError::NonFinite {
                context: "IMU angular rate".to_string(),
            });
        }
        Ok(Self {
            status: SignalStatus::nominal(timestamp),
            frame,
            specific_force,
            angular_rate,
        })
    }

    pub fn specific_force_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::MeterPerSecondSquared)
    }

    pub fn angular_rate_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::RadianPerSecond)
    }
}

/// A GNSS position/velocity sample with explicit quality status. Position is in
/// metres and velocity in m/s, both in `frame`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GnssMeasurement {
    pub status: SignalStatus,
    pub frame: FrameId,
    pub position: Vec3,
    pub velocity: Vec3,
}

/// One timestamped replay record. Measurements are optional because real logs
/// commonly contain asynchronous sensor streams and dropouts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ReplayRecord {
    pub timestamp: Timestamp,
    pub imu: Option<ImuMeasurement>,
    pub gnss: Option<GnssMeasurement>,
}

/// A single sensor measurement in an asynchronous replay stream.
///
/// Unlike [`ReplayRecord`], this does not force different sensors to share a
/// frame or a timestamp. IMU data can remain in the body frame while GNSS data
/// remains in the navigation frame, and equal timestamps are ordered by
/// `ReplayEvent::sequence`.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum ReplayMeasurement {
    Imu(ImuMeasurement),
    Gnss(GnssMeasurement),
}

impl ReplayMeasurement {
    pub fn timestamp(self) -> Timestamp {
        match self {
            Self::Imu(measurement) => measurement.status.timestamp,
            Self::Gnss(measurement) => measurement.status.timestamp,
        }
    }

    pub fn frame(self) -> FrameId {
        match self {
            Self::Imu(measurement) => measurement.frame,
            Self::Gnss(measurement) => measurement.frame,
        }
    }
}

/// A deterministic event in an asynchronous sensor replay stream.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub timestamp: Timestamp,
    pub sequence: u64,
    pub measurement: ReplayMeasurement,
}

/// Ordered replay for independent, multi-rate sensor streams.
///
/// Events are ordered lexicographically by `(timestamp, sequence)`. The
/// sequence number makes same-timestamp events deterministic without imposing
/// an artificial ordering between IMU and GNSS clocks.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayEventLog {
    next_sequence: u64,
    events: Vec<ReplayEvent>,
}

impl ReplayEventLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&mut self, measurement: ReplayMeasurement) -> Result<u64, GncError> {
        let sequence = self.next_sequence;
        if sequence == u64::MAX {
            return Err(GncError::InvalidValue {
                context: "replay sequence".to_string(),
                reason: "sequence number overflow".to_string(),
            });
        }
        self.push_event(ReplayEvent {
            timestamp: measurement.timestamp(),
            sequence,
            measurement,
        })?;
        self.next_sequence = sequence + 1;
        Ok(sequence)
    }

    pub fn push_event(&mut self, event: ReplayEvent) -> Result<(), GncError> {
        if event.timestamp != event.measurement.timestamp() {
            return Err(GncError::InvalidTime(
                "replay event timestamp does not match measurement timestamp".to_string(),
            ));
        }
        if let Some(previous) = self.events.last() {
            let previous_key = (previous.timestamp, previous.sequence);
            let event_key = (event.timestamp, event.sequence);
            if event_key <= previous_key {
                return Err(GncError::InvalidTime(
                    "replay events must be ordered by timestamp and sequence".to_string(),
                ));
            }
        }
        self.next_sequence = self.next_sequence.max(event.sequence.saturating_add(1));
        self.events.push(event);
        Ok(())
    }

    pub fn events(&self) -> &[ReplayEvent] {
        &self.events
    }

    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Creates a forward-only cursor for deterministic replay execution.
    pub fn cursor(&self) -> ReplayCursor<'_> {
        ReplayCursor {
            events: &self.events,
            index: 0,
        }
    }
}

/// Forward-only execution cursor over a [`ReplayEventLog`].
///
/// The cursor deliberately has no clock or sleep behavior: callers decide how
/// to schedule events, while this type guarantees that each event is observed
/// once and in the log's deterministic order.
#[derive(Debug, Clone)]
pub struct ReplayCursor<'a> {
    events: &'a [ReplayEvent],
    index: usize,
}

impl<'a> ReplayCursor<'a> {
    pub fn position(&self) -> usize {
        self.index
    }

    pub fn is_finished(&self) -> bool {
        self.index == self.events.len()
    }

    pub fn peek(&self) -> Option<&'a ReplayEvent> {
        self.events.get(self.index)
    }

    pub fn advance(&mut self) -> Option<&'a ReplayEvent> {
        let event = self.peek()?;
        self.index += 1;
        Some(event)
    }

    /// Consumes all events through `timestamp`, preserving event order.
    pub fn next_through(&mut self, timestamp: Timestamp) -> Vec<&'a ReplayEvent> {
        let start = self.index;
        while self
            .peek()
            .is_some_and(|event| event.timestamp <= timestamp)
        {
            self.index += 1;
        }
        self.events[start..self.index].iter().collect()
    }
}

/// Returns a copy of a replay stream with every `interval`th GNSS event
/// removed. The returned log is renumbered deterministically, while IMU events
/// and their relative ordering are preserved.
pub fn drop_every_nth_gnss(
    log: &ReplayEventLog,
    interval: usize,
) -> Result<(ReplayEventLog, usize), GncError> {
    if interval == 0 {
        return Err(GncError::InvalidValue {
            context: "GNSS dropout interval".to_string(),
            reason: "interval must be positive".to_string(),
        });
    }

    let mut output = ReplayEventLog::new();
    let mut gnss_count = 0;
    let mut dropped = 0;
    for event in log.events() {
        if matches!(event.measurement, ReplayMeasurement::Gnss(_)) {
            gnss_count += 1;
            if gnss_count % interval == 0 {
                dropped += 1;
                continue;
            }
        }
        output.push(event.measurement)?;
    }
    Ok((output, dropped))
}

/// Ordered, deterministic sensor replay log. All records in one log share a
/// single frame, fixed by the first record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ReplayLog {
    frame: Option<FrameId>,
    records: Vec<ReplayRecord>,
}

impl ReplayLog {
    pub fn new() -> Self {
        Self::default()
    }

    /// The frame all records in this log are expressed in, once one is pushed.
    pub fn frame(&self) -> Option<&FrameId> {
        self.frame.as_ref()
    }

    pub fn push(&mut self, record: ReplayRecord) -> Result<(), GncError> {
        if let Some(previous) = self.records.last() {
            if record.timestamp <= previous.timestamp {
                return Err(GncError::InvalidTime(
                    "replay timestamps must be strictly increasing".to_string(),
                ));
            }
        }
        if let Some(imu) = &record.imu {
            if imu.status.timestamp != record.timestamp {
                return Err(GncError::InvalidTime(
                    "IMU timestamp does not match replay timestamp".to_string(),
                ));
            }
        }
        if let Some(gnss) = &record.gnss {
            if gnss.status.timestamp != record.timestamp {
                return Err(GncError::InvalidTime(
                    "GNSS timestamp does not match replay timestamp".to_string(),
                ));
            }
        }
        let record_frames = record
            .imu
            .iter()
            .map(|imu| &imu.frame)
            .chain(record.gnss.iter().map(|gnss| &gnss.frame));
        for frame in record_frames {
            match &self.frame {
                None => self.frame = Some(*frame),
                Some(expected) if expected == frame => {}
                Some(expected) => {
                    return Err(GncError::InvalidValue {
                        context: "replay record".to_string(),
                        reason: format!("frame {frame:?} does not match log frame {expected:?}"),
                    });
                }
            }
        }
        self.records.push(record);
        Ok(())
    }

    pub fn records(&self) -> &[ReplayRecord] {
        &self.records
    }

    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl GnssMeasurement {
    pub fn new(
        timestamp: Timestamp,
        frame: FrameId,
        position: Vec3,
        velocity: Vec3,
    ) -> Result<Self, GncError> {
        if !position.is_finite() {
            return Err(GncError::NonFinite {
                context: "GNSS position".to_string(),
            });
        }
        if !velocity.is_finite() {
            return Err(GncError::NonFinite {
                context: "GNSS velocity".to_string(),
            });
        }
        Ok(Self {
            status: SignalStatus::nominal(timestamp),
            frame,
            position,
            velocity,
        })
    }

    pub fn position_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::Meter)
    }

    pub fn velocity_meta(&self) -> VectorMeta {
        VectorMeta::new(self.frame, Unit::MeterPerSecond)
    }

    pub fn with_quality(mut self, quality: Quality) -> Self {
        self.status.quality = quality;
        self
    }
}

/// Deterministic, frame-aware constant sensor bias injection for simulation
/// and replay tests. It does not model random noise or time-varying drift.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SensorBias {
    pub frame: FrameId,
    pub imu_specific_force: Vec3,
    pub imu_angular_rate: Vec3,
    pub gnss_position: Vec3,
    pub gnss_velocity: Vec3,
}

impl SensorBias {
    pub fn zero(frame: FrameId) -> Self {
        Self {
            frame,
            imu_specific_force: Vec3::ZERO,
            imu_angular_rate: Vec3::ZERO,
            gnss_position: Vec3::ZERO,
            gnss_velocity: Vec3::ZERO,
        }
    }

    pub fn validate(self) -> Result<Self, GncError> {
        if [
            self.imu_specific_force,
            self.imu_angular_rate,
            self.gnss_position,
            self.gnss_velocity,
        ]
        .into_iter()
        .any(|value| !value.is_finite())
        {
            return Err(GncError::NonFinite {
                context: "sensor bias".to_string(),
            });
        }
        Ok(self)
    }

    pub fn apply_imu(&self, measurement: &ImuMeasurement) -> Result<ImuMeasurement, GncError> {
        if measurement.frame != self.frame {
            return Err(GncError::InvalidValue {
                context: "IMU sensor bias".to_string(),
                reason: "measurement frame does not match bias frame".to_string(),
            });
        }
        let mut biased = ImuMeasurement::new(
            measurement.status.timestamp,
            measurement.frame,
            measurement.specific_force + self.imu_specific_force,
            measurement.angular_rate + self.imu_angular_rate,
        )?;
        biased.status = measurement.status;
        Ok(biased)
    }

    pub fn apply_gnss(&self, measurement: &GnssMeasurement) -> Result<GnssMeasurement, GncError> {
        if measurement.frame != self.frame {
            return Err(GncError::InvalidValue {
                context: "GNSS sensor bias".to_string(),
                reason: "measurement frame does not match bias frame".to_string(),
            });
        }
        let mut biased = GnssMeasurement::new(
            measurement.status.timestamp,
            measurement.frame,
            measurement.position + self.gnss_position,
            measurement.velocity + self.gnss_velocity,
        )?;
        biased.status = measurement.status;
        Ok(biased)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use xdl_gnc_core::Quality;

    fn body() -> FrameId {
        FrameId::Body
    }

    #[test]
    fn constant_acceleration_step_is_deterministic() {
        let model = ConstantAccelerationModel::new(body(), Vec3::new(1.0, 0.0, 0.0))
            .expect("finite acceleration");
        let state = KinematicState::new(
            Timestamp::ZERO,
            body(),
            Vec3::ZERO,
            Vec3::new(10.0, 0.0, 0.0),
        )
        .expect("finite state");
        let next = model
            .step(
                &state,
                DurationNs::from_seconds(2.0).expect("finite duration"),
            )
            .expect("positive step");

        assert_eq!(next.timestamp.as_nanoseconds(), 2_000_000_000);
        assert_eq!(next.position, Vec3::new(22.0, 0.0, 0.0));
        assert_eq!(next.velocity, Vec3::new(12.0, 0.0, 0.0));
        assert_eq!(next.frame, body());
    }

    #[test]
    fn simulation_rejects_zero_step() {
        let model =
            ConstantAccelerationModel::new(body(), Vec3::ZERO).expect("finite acceleration");
        let state = KinematicState::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        assert!(model.step(&state, DurationNs::ZERO).is_err());
    }

    #[test]
    fn model_rejects_state_in_a_different_frame() {
        let model =
            ConstantAccelerationModel::new(body(), Vec3::ZERO).expect("finite acceleration");
        let state = KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        let dt = DurationNs::from_seconds(1.0).expect("finite duration");
        assert!(model.step(&state, dt).is_err());
    }

    #[test]
    fn state_metadata_reports_units_for_frame() {
        let state = KinematicState::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite state");
        assert!(state
            .position_meta()
            .require(&body(), Unit::Meter, "position")
            .is_ok());
        assert!(state
            .velocity_meta()
            .require(&body(), Unit::MeterPerSecond, "velocity")
            .is_ok());
    }

    #[test]
    fn sensor_bias_is_deterministic_and_frame_aware() {
        let timestamp = Timestamp::from_nanoseconds(10);
        let imu = ImuMeasurement::new(
            timestamp,
            body(),
            Vec3::new(1.0, 2.0, 3.0),
            Vec3::new(0.1, 0.2, 0.3),
        )
        .expect("IMU");
        let mut bias = SensorBias::zero(body());
        bias.imu_specific_force = Vec3::new(0.5, 0.0, 0.0);
        bias.imu_angular_rate = Vec3::new(0.0, 0.1, 0.0);
        let biased = bias.apply_imu(&imu).expect("biased IMU");
        assert_eq!(biased.specific_force, Vec3::new(1.5, 2.0, 3.0));
        assert!((biased.angular_rate.y - 0.3).abs() < 1e-12);
        let wrong_frame = SensorBias::zero(FrameId::Inertial);
        assert!(wrong_frame.apply_imu(&imu).is_err());
    }

    #[test]
    fn sensor_records_reject_non_finite_values() {
        assert!(ImuMeasurement::new(
            Timestamp::ZERO,
            body(),
            Vec3::new(f64::NAN, 0.0, 0.0),
            Vec3::ZERO,
        )
        .is_err());
        assert!(GnssMeasurement::new(
            Timestamp::ZERO,
            body(),
            Vec3::ZERO,
            Vec3::new(0.0, f64::INFINITY, 0.0),
        )
        .is_err());
    }

    #[test]
    fn imu_metadata_carries_frame_and_rate_units() {
        let imu = ImuMeasurement::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite IMU");
        assert!(imu
            .specific_force_meta()
            .require(&body(), Unit::MeterPerSecondSquared, "specific force")
            .is_ok());
        assert!(imu
            .angular_rate_meta()
            .require(&body(), Unit::RadianPerSecond, "angular rate")
            .is_ok());
    }

    #[test]
    fn degraded_status_is_usable_but_stale_status_is_not() {
        let mut gnss = GnssMeasurement::new(Timestamp::ZERO, body(), Vec3::ZERO, Vec3::ZERO)
            .expect("finite GNSS");
        gnss = gnss.with_quality(Quality::Degraded);
        assert!(gnss.status.is_usable());
        gnss = gnss.with_quality(Quality::Stale);
        assert!(!gnss.status.is_usable());
    }

    #[test]
    fn replay_log_requires_strictly_increasing_timestamps() {
        let mut log = ReplayLog::new();
        let record = ReplayRecord {
            timestamp: Timestamp::from_nanoseconds(1),
            imu: None,
            gnss: None,
        };
        log.push(record.clone()).expect("first record");
        assert!(log.push(record).is_err());
        assert_eq!(log.records().len(), 1);
    }

    #[test]
    fn replay_log_rejects_mixed_frames() {
        let mut log = ReplayLog::new();
        let t0 = Timestamp::from_nanoseconds(1);
        let t1 = Timestamp::from_nanoseconds(2);
        let body_gnss = GnssMeasurement::new(t0, body(), Vec3::ZERO, Vec3::ZERO).expect("GNSS");
        log.push(ReplayRecord {
            timestamp: t0,
            imu: None,
            gnss: Some(body_gnss),
        })
        .expect("first frame fixes log frame");
        assert_eq!(log.frame(), Some(&body()));

        let inertial_gnss =
            GnssMeasurement::new(t1, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO).expect("GNSS");
        let mismatched = log.push(ReplayRecord {
            timestamp: t1,
            imu: None,
            gnss: Some(inertial_gnss),
        });
        assert!(mismatched.is_err());
        assert_eq!(log.records().len(), 1);
    }

    #[test]
    fn event_log_allows_multi_rate_frames_and_equal_timestamps() {
        let mut log = ReplayEventLog::new();
        let timestamp = Timestamp::from_nanoseconds(1_000_000);
        let imu =
            ImuMeasurement::new(timestamp, body(), Vec3::ZERO, Vec3::ZERO).expect("finite IMU");
        let gnss = GnssMeasurement::new(timestamp, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
            .expect("finite GNSS");

        assert_eq!(log.push(ReplayMeasurement::Imu(imu)), Ok(0));
        assert_eq!(log.push(ReplayMeasurement::Gnss(gnss)), Ok(1));
        assert_eq!(log.events().len(), 2);
        assert_eq!(log.events()[0].measurement.frame(), body());
        assert_eq!(log.events()[1].measurement.frame(), FrameId::Inertial);
    }

    #[test]
    fn event_log_rejects_timestamp_mismatch_and_out_of_order_sequence() {
        let mut log = ReplayEventLog::new();
        let measurement = GnssMeasurement::new(
            Timestamp::from_nanoseconds(2),
            FrameId::Inertial,
            Vec3::ZERO,
            Vec3::ZERO,
        )
        .expect("finite GNSS");
        assert!(log
            .push_event(ReplayEvent {
                timestamp: Timestamp::from_nanoseconds(1),
                sequence: 0,
                measurement: ReplayMeasurement::Gnss(measurement),
            })
            .is_err());

        log.push(ReplayMeasurement::Gnss(measurement))
            .expect("first event");
        assert!(log
            .push_event(ReplayEvent {
                timestamp: measurement.status.timestamp,
                sequence: 0,
                measurement: ReplayMeasurement::Gnss(measurement),
            })
            .is_err());
    }

    #[test]
    fn replay_cursor_dispatches_events_through_a_control_tick() {
        let mut log = ReplayEventLog::new();
        for nanoseconds in [1, 2, 2, 3] {
            let measurement = GnssMeasurement::new(
                Timestamp::from_nanoseconds(nanoseconds),
                FrameId::Inertial,
                Vec3::ZERO,
                Vec3::ZERO,
            )
            .expect("finite GNSS");
            log.push(ReplayMeasurement::Gnss(measurement))
                .expect("ordered event");
        }

        let mut cursor = log.cursor();
        assert_eq!(cursor.position(), 0);
        assert_eq!(cursor.next_through(Timestamp::from_nanoseconds(2)).len(), 3);
        assert_eq!(cursor.position(), 3);
        assert_eq!(
            cursor.peek().map(|event| event.timestamp),
            Some(Timestamp::from_nanoseconds(3))
        );
        assert_eq!(cursor.advance().map(|event| event.sequence), Some(3));
        assert!(cursor.is_finished());
        assert!(cursor.advance().is_none());
    }

    #[test]
    fn deterministic_gnss_dropout_preserves_imu_events() {
        let mut log = ReplayEventLog::new();
        for index in 1..=4 {
            let timestamp = Timestamp::from_nanoseconds(index);
            let imu =
                ImuMeasurement::new(timestamp, body(), Vec3::ZERO, Vec3::ZERO).expect("finite IMU");
            let gnss = GnssMeasurement::new(timestamp, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
                .expect("finite GNSS");
            log.push(ReplayMeasurement::Imu(imu)).expect("ordered IMU");
            log.push(ReplayMeasurement::Gnss(gnss))
                .expect("ordered GNSS");
        }

        let (dropped, count) = drop_every_nth_gnss(&log, 2).expect("valid dropout interval");
        assert_eq!(count, 2);
        assert_eq!(dropped.events().len(), 6);
        assert_eq!(
            dropped
                .events()
                .iter()
                .filter(|event| matches!(event.measurement, ReplayMeasurement::Imu(_)))
                .count(),
            4
        );
        assert!(drop_every_nth_gnss(&log, 0).is_err());
    }
}
