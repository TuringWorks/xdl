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
   ├── xdl-gnc-telemetry    typed telemetry and JSON-lines export
   ├── xdl-model            typed block/port/signal graph and validation
   └── xdl-sim              deterministic fixed-step/multi-rate scheduler and solvers
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
| `xdl-model` | Serializable block/port/signal graph with connection, bus, subsystem-boundary, and cycle validation |
| `xdl-sim` | Bounded deterministic periodic-task scheduler, reusable Constant/Gain/Sum/Integrator/Derivative/Pid/Mux/Demux/MatrixGain/MatrixTranspose/MatrixVectorProduct/VectorSum/VectorScale/VectorNorm/VectorNormalize/DotProduct/CrossProduct/QuaternionNormalize/QuaternionMultiply/QuaternionConjugate/QuaternionInverse/QuaternionRotate/BusCreator/BusSelector/Saturation/Product/Switch/RateLimiter/Deadzone/LookupTable1D blocks, typed bus values, unit delay, zero-order hold, subsystem runtime, and Euler/RK4 solvers |

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
| `xdl-gnc-simulation` | 16 | Truth and replay primitives, frame-aware constant bias, seeded Gaussian sensor noise, and deterministic Monte Carlo summaries |
| `xdl-gnc-navigation` | 14 | Alpha-beta is a baseline, not a production estimator |
| `xdl-gnc-guidance` | 2 | Waypoint only |
| `xdl-gnc-control` | 6 | Single-axis PID plus state-space and gain-scheduled state feedback |
| `xdl-gnc-actuators` | 3 | First-order scalar actuator only |
| `xdl-gnc-integration` | 4 | Deterministic scenario, seeded noisy scenario, Monte Carlo wrapper, typed replay runner, and ordered telemetry output |
| `xdl-gnc-telemetry` | 3 | Ordered telemetry samples with buffered and streaming JSON-lines export; product-specific sinks remain pending |
| `xdl-model` | 19 | Graph, typed subsystem-boundary contract, block parameters, delay-safe feedback validation, deterministic execution ordering, serializable layout, auto-layout, transactional editor operations, versioned JSON documents, SVG rendering, hit testing, and editor events |
| `xdl-sim` | 19 | Scheduler, parameterized Constant/Gain/Sum/Integrator/Derivative/Pid/Mux/Demux/MatrixGain/MatrixTranspose/MatrixVectorProduct/VectorSum/VectorScale/VectorNorm/VectorNormalize/DotProduct/CrossProduct/QuaternionNormalize/QuaternionMultiply/QuaternionConjugate/QuaternionInverse/QuaternionRotate/BusCreator/BusSelector/Saturation/Product/Switch/RateLimiter/Deadzone/LookupTable1D block runtimes, delay/hold with explicit initial values, matrix-preserving signals, timestamped JSON-lines tracing with cursor replay and tolerance comparison, subsystem runtime, deterministic graph ordering, readiness diagnostics, feedback-safe startup, bounded Euler/RK4 solvers, adaptive RK4 retry, and continuous runtime integration |

Counts from the targeted GNC/model/scheduler test runs on 2026-11-13. Eleven crates, 96 tests, all passing. Clippy `-D warnings` is not yet a repository-wide gate.

## Build and test

```bash
cargo check -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control -p xdl-gnc-actuators -p xdl-gnc-integration -p xdl-gnc-telemetry -p xdl-model -p xdl-sim --all-targets
cargo test  -p xdl-gnc-core -p xdl-gnc-simulation -p xdl-gnc-navigation -p xdl-gnc-guidance -p xdl-gnc-control -p xdl-gnc-actuators -p xdl-gnc-integration -p xdl-gnc-telemetry -p xdl-model -p xdl-sim
```

All eleven GNC/model/simulation crates are `[workspace]` members, so `cargo test --workspace` covers them.

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
