---
layout: default
title: GNC Crates
parent: Development
nav_order: 9
---

# GNC Crates

Status: **in development**. Typed Rust libraries for guidance, navigation, and
control with a partial XDL-facing `GNC_*` slice. Unverified on hardware or
flight data.

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
| `xdl-sim` | Bounded deterministic periodic-task scheduler, reusable Constant/Gain/Sum/Integrator/Derivative/Pid/StateSpace/Mux/Demux/MultiportSwitch/MatrixGain/MatrixMatrixProduct/MatrixScale/MatrixSum/MatrixTranspose/MatrixVectorProduct/VectorSum/VectorScale/VectorNorm/VectorNormalize/DotProduct/CrossProduct/QuaternionNormalize/QuaternionMultiply/QuaternionConjugate/QuaternionInverse/QuaternionRotate/BusCreator/BusSelector/Saturation/Product/Switch/RateLimiter/Deadzone/LookupTable1D blocks, typed bus values, unit delay, zero-order hold, subsystem runtime, and Euler/RK4 solvers |

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
| `xdl-gnc-navigation` | 21 | Alpha-beta is a baseline, not a production estimator |
| `xdl-gnc-guidance` | 2 | Waypoint only |
| `xdl-gnc-control` | 6 | Single-axis PID plus state-space and gain-scheduled state feedback |
| `xdl-gnc-actuators` | 3 | First-order scalar actuator only |
| `xdl-gnc-integration` | 9 | Deterministic scenario, seeded noisy scenario, Monte Carlo wrapper, typed replay runner, ordered telemetry output, and vertical-slice replay tests |
| `xdl-gnc-telemetry` | 3 | Ordered telemetry samples with buffered and streaming JSON-lines export; product-specific sinks remain pending |
| `xdl-model` | 26 | Graph, typed subsystem-boundary contract, block parameters, delay-safe feedback validation, deterministic execution ordering, serializable layout, auto-layout, transactional editor operations, versioned JSON documents, SVG rendering, hit testing, editor events, Mars system-model validation, structural SVG rendering, and state/event contracts |
| `xdl-sim` | 25 | Scheduler, parameterized Constant/Gain/Sum/Integrator/Derivative/Pid/StateSpace/Mux/Demux/MultiportSwitch/MatrixGain/MatrixMatrixProduct/MatrixScale/MatrixSum/MatrixTranspose/MatrixVectorProduct/VectorSum/VectorScale/VectorNorm/VectorNormalize/DotProduct/CrossProduct/QuaternionNormalize/QuaternionMultiply/QuaternionConjugate/QuaternionInverse/QuaternionRotate/BusCreator/BusSelector/Saturation/Product/Switch/RateLimiter/Deadzone/LookupTable1D block runtimes, delay/hold with explicit initial values, matrix-preserving signals, timestamped JSON-lines tracing with cursor replay and tolerance comparison, typed model-graph execution, deterministic state-machine runtime, subsystem runtime, deterministic graph ordering, readiness diagnostics, feedback-safe startup, bounded Euler/RK4 solvers, adaptive RK4 retry, and continuous runtime integration |

`xdl-model` also exposes a typed `StateMachine` contract with explicit states,
initial-state validation, deterministic event transitions, and duplicate-event
rejection. It is the semantic foundation for Stateflow/Xcos-style state/event
subsystems; a scheduler-integrated runtime, timed events, guards, and graphical
state-machine editing remain open work.

The XDL standard library currently exposes the explicit-state
`GNC_FLIGHT_READINESS_STEP` binding used by the Mars example. It accepts the
current state, temperature, SOC, and launch request, returning the next state
and readiness flags. The binding is deterministic and has no process-global
state; it is a domain vertical slice, not a general Stateflow language.

Counts from the targeted GNC/model/scheduler test run on 2026-11-13. Ten crates, 120 tests, all passing. The same targeted crate set also passes Clippy
with `-D warnings`; Clippy is not yet a repository-wide gate.

## Standard block library

The public `xdl-sim` block runtimes are intended to be embedded by model
adapters and product applications. The current GNC-oriented mapping is:

| Block family | Product use |
|---|---|
| `Constant`, `Gain`, `Sum`, `Product`, `Saturation`, `Deadzone`, `RateLimiter` | Command shaping, actuator limits, and calibration logic |
| `Integrator`, `Derivative`, `Pid`, `UnitDelay`, `ZeroOrderHold` | Discrete controllers, plant state, and sampled-data loops |
| `Mux`, `Demux`, `VectorSum`, `VectorScale`, `VectorNorm`, `VectorNormalize`, `DotProduct`, `CrossProduct` | Multi-axis guidance, line-of-sight geometry, and vector control |
| `StateSpace`, `MatrixGain`, `MatrixScale`, `MatrixSum`, `MatrixTranspose`, `MatrixVectorProduct`, `MatrixMatrixProduct` | State-space models, covariance transforms, and coordinate maps |
| `QuaternionNormalize`, `QuaternionMultiply`, `QuaternionConjugate`, `QuaternionInverse`, `QuaternionRotate` | Attitude propagation and frame transformations |
| `BusCreator`, `BusSelector`, `LookupTable1D`, `Switch`, `MultiportSwitch` | Structured telemetry, calibration maps, and supervisory modes |

All blocks preserve explicit signal shapes and return execution errors for
missing inputs, incompatible dimensions, non-finite values, or invalid time.

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

Verified on 2026-11-13 with `cargo run -p xdl-cli -- -q
examples/gnc_guidance_control.xdl`: the example exited successfully and
reported the expected distance, desired velocity, PID command, and actuator
status. This verifies the XDL-facing slice only; it does not establish a
closed-loop model, hardware equivalence, or flight readiness.

[`examples/gnc_closed_loop.xdl`](../../examples/gnc_closed_loop.xdl) extends
that slice to ten deterministic control ticks and makes the plant update
explicit: navigation → guidance → PID → actuator → position/velocity update.
Each tick also produces a JSON-lines telemetry record with
`GNC_TELEMETRY_JSON`, and the example now prints each record so the telemetry
is observable and capturable by the test/artifact workflow. This remains an
example-level closed loop, not a validated Simulink-equivalent runtime.

[`examples/mars_helicopter_system_level.xdl`](../../examples/mars_helicopter_system_level.xdl)
is the first domain demonstration for a Mars-helicopter system-level workflow:
it models Mars gravity, solar charging, battery state of charge, heater-driven
thermal readiness, a flight-readiness gate, waypoint guidance, PID control,
actuator dynamics, ground contact, and JSON-lines flight telemetry. It is
executable and covered by `xdl-cli` integration tests.

The plant now includes explicit first-order motor speed, gearbox reduction,
equal-speed contra-rotating rotor state, and a six-cell series battery
abstraction with SOC-dependent voltage, load power, and current. Thrust and
battery behavior are still calibrated reduced-order maps, so these states are
architectural model components, not validated aerodynamic or electrochemical
predictions.

The compatibility target is the [MathWorks Mars Helicopter Simulink-based
system-level design](https://www.mathworks.com/help/aeroblks/mars-helicopter-system-level-design.html),
which combines Aerospace Blockset and Simscape domains for the reference
workflow.

This is deliberately a reduced-order XDL analogue, not a claim of MATLAB
Aerospace Blockset/Simscape parity. It does not yet model electrical or thermal
physical networks, coaxial rotor aerodynamics, gearbox mechanics, flap effects,
3-D attitude, or graphical block-diagram editing. Parameter studies are
available as a separate reduced-order battery-string sweep, while the
physical-network and aerodynamic gaps remain explicit follow-on compatibility
gaps; the example provides a truthful, repeatable vertical slice on which to
implement them.

```bash
cargo run -p xdl-cli -- -q examples/mars_helicopter_system_level.xdl
```

The companion
[`examples/mars_helicopter_battery_sweep.xdl`](../../examples/mars_helicopter_battery_sweep.xdl)
executes a deterministic one-to-four battery-string endurance sweep. It makes
the parameter-study workflow executable in XDL while clearly labeling its
energy-budget model as reduced order.

```bash
cargo run -p xdl-cli -- -q examples/mars_helicopter_battery_sweep.xdl
```

## License

MIT, like the rest of XDL (see `LICENSE`).
