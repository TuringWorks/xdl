---
layout: default
title: GNC Crates
parent: Development
nav_order: 9
---

# GNC Crates

Status: **in development**. Typed Rust libraries for guidance, navigation, and
control. Not yet exposed as XDL built-ins. Unverified on hardware or flight data.

## Layering

```
xdl-gnc-core         types, errors, time, frames, limits, signal status
   ├── xdl-gnc-simulation   truth model, sensor records, replay log
   ├── xdl-gnc-navigation   alpha-beta filter, position/velocity EKF, strapdown, error-state EKF
   ├── xdl-gnc-guidance     waypoint guidance
   └── xdl-gnc-control      bounded PID
```

Rules:

- No crate depends on the XDL interpreter or on `XdlValue`. Bindings belong at the XDL boundary, not here.
- Every step takes an explicit `Timestamp` / `DurationNs`. No hidden clocks.
- Non-finite input is an error (`GncError::NonFinite`), not a silent value.
- Signal validity travels with the value as `SignalStatus` (quality + `StepEvent`s), not as a sentinel.

## Crates

| Crate | Provides |
|---|---|
| `xdl-gnc-core` | `Vec3`, `Quaternion`, `FrameId`, `Pose`, `Transform`, `ScalarLimit`, `Timestamp`, `DurationNs`, `Quality`, `SignalStatus`, `StepResult`, `StepEvent`, `GncError` |
| `xdl-gnc-simulation` | `KinematicState`, `ConstantAccelerationModel` (fixed-step truth), `ImuMeasurement`, `GnssMeasurement`, `ReplayLog` (ordered, rejects out-of-order records) |
| `xdl-gnc-navigation` | `AlphaBetaFilter` (baseline), `PositionVelocityEkf` (6-state), `StrapdownMechanization`, `ErrorStateEkf` (15-state covariance, IMU predict, GNSS update) |
| `xdl-gnc-guidance` | `WaypointGuidance` → `GuidanceCommand` |
| `xdl-gnc-control` | `PidController` (integral clamp, anti-windup on saturation, output limit) → `PidOutput` |

### Step events

`StepEvent` tags a `StepResult` with why it is degraded or notable. Variants:

| Event | Emitted by today |
|---|---|
| `TargetReached` | `xdl-gnc-guidance` (waypoint within acceptance radius) |
| `Saturated` | `xdl-gnc-control` (output clamped by limit) |
| `RateLimited`, `StaleInput`, `DegradedInput`, `InvalidInput`, `NumericalWarning` | defined, not yet emitted |

## Status by crate

| Crate | Unit tests | Known limits |
|---|---|---|
| `xdl-gnc-core` | 4 | Foundation only |
| `xdl-gnc-simulation` | 4 | Constant-acceleration truth only; no disturbances, scheduling, or Monte Carlo |
| `xdl-gnc-navigation` | 9 | Alpha-beta is a baseline, not a production estimator |
| `xdl-gnc-guidance` | 2 | Waypoint only |
| `xdl-gnc-control` | 2 | Single-axis PID |

Counts from `cargo test -p <crate>` on 2026-11-12. Five crates, 21 tests, all passing.

## Build and test

```bash
cargo check -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control --all-targets
cargo test  -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control
```

All five crates are `[workspace]` members, so `cargo test --workspace` covers them.

## License

The source headers say `GPL-2.0-only`. The repository's license is itself
inconsistent (`Cargo.toml` GPL-2.0, `LICENSE` MIT). Resolve that before
publishing these crates separately.
