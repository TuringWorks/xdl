// SPDX-License-Identifier: GPL-2.0-only
// Copyright (C) 2026 Ravindra Boddipalli

//! Typed foundations for reusable guidance, navigation, and control libraries.
//!
//! This crate deliberately does not depend on the XDL interpreter or dynamic
//! [`xdl_core::XdlValue`] representation. It is intended to be usable by
//! native products first, with XDL bindings added at the boundary.

mod error;
mod frame;
mod limit;
mod math;
mod signal;
mod time;

pub use error::GncError;
pub use frame::{FrameId, Pose, Transform};
pub use limit::{LimitResult, ScalarLimit};
pub use math::{Quaternion, Vec3};
pub use signal::{Quality, SignalStatus, StepEvent, StepResult};
pub use time::{DurationNs, Timestamp};
