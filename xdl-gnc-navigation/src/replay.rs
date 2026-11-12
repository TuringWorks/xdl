// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Deterministic replay of a recorded sensor log through an estimator.

use crate::{AlphaBetaFilter, NavigationEstimate};
use xdl_gnc_core::{GncError, Vec3};
use xdl_gnc_simulation::{ReplayLog, ReplayRecord};

/// Runs `filter` over every record in `log`, in recorded order.
///
/// `acceleration` supplies the prediction input for each record, so the caller
/// chooses how IMU or model data map to acceleration. The filter's frame must
/// match the log's frame; a mismatch is an error, not a silent reinterpretation.
/// The first failing record stops the run and its error is returned.
///
/// The run is a pure function of `(filter, log, acceleration)`: repeating it
/// with the same inputs yields the same estimates.
pub fn replay_alpha_beta<F>(
    mut filter: AlphaBetaFilter,
    log: &ReplayLog,
    mut acceleration: F,
) -> Result<Vec<NavigationEstimate>, GncError>
where
    F: FnMut(&ReplayRecord) -> Vec3,
{
    if let Some(log_frame) = log.frame() {
        let filter_frame = filter.state().frame;
        if filter_frame != *log_frame {
            return Err(GncError::InvalidValue {
                context: "replay".to_string(),
                reason: format!(
                    "estimator frame {filter_frame:?} does not match log frame {log_frame:?}"
                ),
            });
        }
    }
    log.records()
        .iter()
        .map(|record| {
            let input = acceleration(record);
            filter.step(record.timestamp, input, record.gnss)
        })
        .collect()
}
