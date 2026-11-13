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
   ├── xdl-gnc-control      bounded PID, state-space, gain scheduling
   ├── xdl-gnc-actuators    first-order actuator dynamics, command/rate limits
   ├── xdl-gnc-integration  deterministic end-to-end scenario contracts
   └── xdl-gnc-telemetry    typed telemetry and JSON-lines export
```

Rules:

- No crate depends on the XDL interpreter or on `XdlValue`. Bindings belong at the XDL boundary, not here.
- Every step takes an explicit `Timestamp` / `DurationNs`. No hidden clocks.
- Non-finite input is an error (`GncError::NonFinite`), not a silent value.
- Every value carries its frame. Estimators take the frame from their initial state and reject measurements in another frame.
- Signal validity travels with the value as `SignalStatus` (quality + `StepEvent`s), not as a sentinel.

## Crates

| Crate | Provides |
|---|---|
| `xdl-gnc-core` | `Vec3`, `Quaternion`, `FrameId`, `Pose`, `Transform`, `ScalarLimit`, `Timestamp`, `DurationNs`, `Quality`, `SignalStatus`, `StepResult`, `StepEvent`, `GncError` |
| `xdl-gnc-simulation` | `KinematicState`, `ConstantAccelerationModel` (fixed-step truth), `ImuMeasurement`, `GnssMeasurement`, `ReplayLog` (legacy synchronized records), `ReplayEventLog` (multi-rate typed events), `ReplayCursor` (deterministic dispatch), GNSS dropout injector |
| `xdl-gnc-navigation` | `AlphaBetaFilter` (baseline), `PositionVelocityEkf` (6-state), `StrapdownMechanization`, `ErrorStateEkf` (15-state covariance, IMU predict, GNSS update) |
| `xdl-gnc-guidance` | `WaypointGuidance` → `GuidanceCommand` |
| `xdl-gnc-control` | `PidController` (integral clamp, anti-windup on saturation, output limit), discrete state-space, linear gain scheduling |
| `xdl-gnc-actuators` | First-order actuator with command and rate limits; emits saturation/rate-limit events |
| `xdl-gnc-integration` | Closed-loop deterministic truth → sensors → EKF → guidance → PID → actuator contract test |
| `xdl-gnc-telemetry` | Ordered telemetry samples with finite-value validation and JSON-lines import/export |

### Step events

`StepEvent` tags a `StepResult` with why it is degraded or notable. Variants:

| Event | Emitted by today |
|---|---|
| `TargetReached` | `xdl-gnc-guidance` (waypoint within acceptance radius) |
| `Saturated` | `xdl-gnc-control` (output clamped by limit) |
| `RateLimited` | `xdl-gnc-actuators` |
| `StaleInput`, `DegradedInput`, `InvalidInput`, `NumericalWarning` | defined, not yet emitted uniformly |

## Status by crate

| Crate | Unit tests | Known limits |
|---|---|---|
| `xdl-gnc-core` | 9 | Foundation only |
| `xdl-gnc-simulation` | 13 | Truth and replay primitives; deterministic GNSS dropout only; no disturbances or Monte Carlo |
| `xdl-gnc-navigation` | 14 | Alpha-beta is a baseline, not a production estimator |
| `xdl-gnc-guidance` | 2 | Waypoint only |
| `xdl-gnc-control` | 6 | Single-axis PID plus state-space and gain-scheduled state feedback |
| `xdl-gnc-actuators` | 3 | First-order scalar actuator only |
| `xdl-gnc-integration` | 3 | Deterministic scenario plus typed replay runner; telemetry wiring remains separate |
| `xdl-gnc-telemetry` | 2 | JSON-lines export; product-specific sinks remain pending |

Counts from the targeted GNC test run on 2026-11-12. Nine crates, 52 tests, all passing. Clippy `-D warnings` is not yet a repository-wide gate.

## Build and test

```bash
cargo check -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control -p xdl-gnc-actuators -p xdl-gnc-integration -p xdl-gnc-telemetry --all-targets
cargo test  -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control -p xdl-gnc-actuators -p xdl-gnc-integration -p xdl-gnc-telemetry
```

All nine GNC crates are `[workspace]` members, so `cargo test --workspace` covers them.

## Runnable XDL example

The current XDL-facing slice is demonstrated by
[`examples/gnc_guidance_control.xdl`](../../examples/gnc_guidance_control.xdl):

```bash
cargo run -p xdl-cli -- -q examples/gnc_guidance_control.xdl
```

It creates a vector, computes waypoint guidance, performs an explicit-state PID
step, and applies a first-order actuator update with saturation/rate-limit
reporting.

[`examples/gnc_closed_loop.xdl`](../../examples/gnc_closed_loop.xdl) extends
that slice to ten deterministic control ticks and makes the plant update
explicit: navigation → guidance → PID → actuator → position/velocity update.
Each tick also produces a JSON-lines telemetry record with
`GNC_TELEMETRY_JSON`.

## License

MIT, like the rest of XDL (see `LICENSE`).
