// SPDX-License-Identifier: MIT
// Copyright (C) 2026 Ravindra Boddipalli

//! End-to-end vertical slice: truth → GNSS log with a dropout → replay → estimate.

use xdl_gnc_core::{DurationNs, FrameId, GncError, Quality, StepEvent, Timestamp, Vec3};
use xdl_gnc_navigation::{replay_alpha_beta, AlphaBetaConfig, AlphaBetaFilter, NavigationEstimate};
use xdl_gnc_simulation::{
    ConstantAccelerationModel, GnssMeasurement, KinematicState, ReplayLog, ReplayRecord,
};

const STEPS: u32 = 50;
const DROPOUT: std::ops::RangeInclusive<u32> = 20..=24;
const ACCEL: Vec3 = Vec3::new(0.5, -0.25, 0.0);

fn body() -> FrameId {
    FrameId::Body
}

/// Truth trajectory and a replay log. GNSS is missing during `DROPOUT`.
fn build_log() -> (ReplayLog, Vec<KinematicState>) {
    let model = ConstantAccelerationModel::new(body(), ACCEL).expect("finite acceleration");
    let dt = DurationNs::from_seconds(0.1).expect("finite step");
    let mut truth = KinematicState::new(
        Timestamp::ZERO,
        body(),
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
    )
    .expect("finite state");
    let mut truth_track = vec![truth];
    // The estimator starts at t = 0 (its initial state), so the log begins at
    // the first step after it; a record at t = 0 would be a zero-length predict.
    let mut log = ReplayLog::new();
    for step in 1..=STEPS {
        truth = model.step(&truth, dt).expect("positive step");
        let gnss = if DROPOUT.contains(&step) {
            None
        } else {
            Some(
                GnssMeasurement::new(truth.timestamp, body(), truth.position, truth.velocity)
                    .expect("finite GNSS"),
            )
        };
        log.push(ReplayRecord {
            timestamp: truth.timestamp,
            imu: None,
            gnss,
        })
        .expect("strictly increasing record");
        truth_track.push(truth);
    }
    (log, truth_track)
}

fn run(log: &ReplayLog) -> Result<Vec<NavigationEstimate>, GncError> {
    let initial = KinematicState::new(
        Timestamp::ZERO,
        body(),
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
    )
    .expect("finite state");
    let filter = AlphaBetaFilter::new(AlphaBetaConfig::new(0.5, 0.25).expect("gains"), initial);
    replay_alpha_beta(filter, log, |_| ACCEL)
}

#[test]
fn replay_runner_produces_identical_runs() {
    let (log, _) = build_log();
    let first = run(&log).expect("first run");
    let second = run(&log).expect("second run");
    assert_eq!(first, second);
}

#[test]
fn vertical_slice_tracks_truth_with_gnss_dropout() {
    let (log, truth) = build_log();
    let estimates = run(&log).expect("replay");
    assert_eq!(estimates.len(), truth.len() - 1);

    for (index, estimate) in estimates.iter().enumerate() {
        let step = index as u32 + 1;
        let expected = &truth[index + 1];
        assert_eq!(estimate.state.frame, body(), "frame survives replay");
        assert_eq!(estimate.state.timestamp, expected.timestamp);

        let error = (estimate.state.position - expected.position).norm();
        assert!(
            error < 1e-9,
            "step {step}: position error {error} exceeds tolerance"
        );

        if DROPOUT.contains(&step) {
            assert_eq!(estimate.status.quality, Quality::Degraded, "step {step}");
            assert_eq!(estimate.position_residual, None, "step {step}");
            assert_eq!(estimate.velocity_residual, None, "step {step}");
            assert!(
                estimate.events.is_empty(),
                "missing GNSS is not an input event"
            );
        } else {
            assert_eq!(estimate.status.quality, Quality::Nominal, "step {step}");
            assert!(estimate.position_residual.is_some(), "step {step}");
            assert!(estimate.events.is_empty(), "step {step}");
        }
    }
}

#[test]
fn vertical_slice_rejects_frame_mismatch() {
    let (log, _) = build_log();
    let initial = KinematicState::new(Timestamp::ZERO, FrameId::Inertial, Vec3::ZERO, Vec3::ZERO)
        .expect("finite state");
    let filter = AlphaBetaFilter::new(AlphaBetaConfig::new(0.5, 0.25).expect("gains"), initial);
    assert!(replay_alpha_beta(filter, &log, |_| ACCEL).is_err());
}

#[test]
fn vertical_slice_rejects_stale_gnss_record_and_reports_event_tag() {
    let mut log = ReplayLog::new();
    let t1 = Timestamp::from_seconds(0.1).expect("finite timestamp");
    let stale = GnssMeasurement::new(t1, body(), Vec3::ZERO, Vec3::ZERO)
        .expect("finite GNSS")
        .with_quality(Quality::Stale);
    log.push(ReplayRecord {
        timestamp: t1,
        imu: None,
        gnss: Some(stale),
    })
    .expect("log accepts the record; the estimator decides");

    let result = run(&log);
    assert!(result.is_err(), "stale GNSS must stop the replay");

    let degraded = GnssMeasurement::new(t1, body(), Vec3::ZERO, Vec3::ZERO)
        .expect("finite GNSS")
        .with_quality(Quality::Degraded);
    let mut log = ReplayLog::new();
    log.push(ReplayRecord {
        timestamp: t1,
        imu: None,
        gnss: Some(degraded),
    })
    .expect("record");
    let estimates = run(&log).expect("degraded input is accepted");
    assert_eq!(estimates[0].events, vec![StepEvent::DegradedInput]);
}

#[test]
fn batch_replay_matches_serial_runs_in_job_order() {
    use xdl_gnc_navigation::{replay_alpha_beta_batch, ReplayJob};
    let (log, _) = build_log();
    let mut short_log = ReplayLog::new();
    for record in log.records().iter().take(5) {
        short_log.push(record.clone()).expect("ordered");
    }
    let initial = KinematicState::new(
        Timestamp::ZERO,
        body(),
        Vec3::ZERO,
        Vec3::new(1.0, 0.0, 0.0),
    )
    .expect("finite state");
    let make = || AlphaBetaFilter::new(AlphaBetaConfig::new(0.5, 0.25).expect("gains"), initial);
    let jobs = [
        ReplayJob {
            filter: make(),
            log: &log,
        },
        ReplayJob {
            filter: make(),
            log: &short_log,
        },
        ReplayJob {
            filter: make(),
            log: &log,
        },
    ];
    let batch = replay_alpha_beta_batch(&jobs, |_| ACCEL);
    assert_eq!(batch.len(), 3);
    assert_eq!(batch[0], run(&log));
    assert_eq!(batch[1].as_ref().map(Vec::len).ok(), Some(5));
    assert_eq!(batch[2], run(&log));
}
