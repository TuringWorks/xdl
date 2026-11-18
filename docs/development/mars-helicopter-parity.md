---
layout: default
title: Mars Helicopter Parity
parent: Development
nav_order: 10
---

# Mars Helicopter Parity Matrix

Reference target: [MathWorks Mars Helicopter Simulink-based system-level
design](https://www.mathworks.com/help/aeroblks/mars-helicopter-system-level-design.html).

This matrix is deliberately conservative. “Reduced-order” means the behavior is
represented by an executable approximation with explicit parameters; it does
not mean the result is physically equivalent to Aerospace Blockset or Simscape.

| Reference capability | XDL status | Evidence | Remaining work |
|---|---|---|---|
| Mars gravity and vertical plant | Reduced-order | `examples/mars_helicopter_system_level.xdl`; executable CLI test | 3-D translational and rotational dynamics, validated parameters |
| Solar input and battery SOC | Reduced-order | XDL example final SOC and battery-string sweep | Solar-cell electrical behavior, charging limits, cell imbalance, and coupling to flight-load simulation |
| Battery-pack variants | Partial, fixed configuration | Six-cell aggregate state and `SixCellBatteryPack` model block | The reference switches between six-cell detailed and one-cell simplified pack variants; XDL has no variant selection or equivalent parameterized pack model |
| Six-cell thermal battery | Reduced-order | Aggregate battery temperature and SOC in the XDL example | Per-cell state, cell-to-cell heat transfer, enclosure heat loss, and instrumented central cell |
| Command dashboard | Not implemented | No interactive in-run command surface is provided by the XDL example | Sliders/switches/knob for altitude, flight preparation, and simulation pacing |
| Heater and thermal readiness | Reduced-order | `GNC_FLIGHT_READINESS_STEP`; hysteresis tests | Thermal conduction/radiation network and calibrated heater plant |
| Flight-readiness state chart | Partial, executable | A focused `xdl-sim` integration test runs a chart block in `ExecutionEngine`, with connected battery-temperature/SOC sources and launch/takeoff events driving `COLD → READY → FLYING`; the separate `.xdl` example also prints the same readiness trace | The integrated graph is a readiness subsystem, not the complete Mars plant; host signal connections are manual, and undo/history, automatic signal-source creation, and full Stateflow parity remain open |
| Flight controller | Partial, executable | `GNC_WAYPOINT`, one `GNC_PID_STEP`, and `GNC_ACTUATOR_STEP` in the vertical example | Reference uses nested rotor-speed, climb-rate, and altitude PID loops; XDL lacks those cascaded loops, tuning UI, and linearization |
| Motor and gearboxes | Reduced-order | Motor speed, gear ratio, upper/lower rotor states in XDL example | Mechanical drivetrain, torque/load coupling, validated gearbox data |
| Contra-rotating coaxial rotors | Reduced-order | `CoaxialRotorGearbox` model block and dual rotor state | Blade aerodynamics, induced flow, flap effects, rotor interference |
| Ground contact / landing protection | Reduced-order | Explicit altitude floor in XDL example | Contact forces, bounce/friction, terrain geometry |
| Telemetry and replay | Partial, executable | `GNC_TELEMETRY_JSON`, `xdl test --manifest`, `xdl replay`; simulation writes `mars_helicopter_results.csv` | Typed battery/thermal channels in telemetry, clean-machine/reference replay |
| Altitude and temperature result plots | Partial, executable | Headless XDLPlot path runs the simulation, reads its numeric CSV, and writes separate PNGs for altitude and aggregate battery temperature | Reference plots six cell temperatures alongside altitude; XDL lacks per-cell thermal traces, the combined subplot/legend workflow, general CSV parsing, and visual comparison |
| Battery-string flight-duration sweep | Reduced-order, executable | `examples/mars_helicopter_battery_sweep.xdl` recomputes battery mass, vehicle mass, Mars weight, lift feasibility, and SOC-based flight duration for one to six parallel strings; CLI test covers takeoff and no-takeoff configurations | Reference re-simulates the closed-loop flight plant and plots each trajectory and flight duration; XDL sweep still uses an illustrative lift threshold and energy budget without a coupled controller, per-configuration altitude trace, or plots |
| Simulink/Simscape physical network | Not implemented | Explicitly excluded from current example | Physical ports, solver domains, units, initialization, conservation |

## Verification commands

```bash
cargo run -p xdl-cli -- -q examples/mars_helicopter_system_level.xdl
cargo run -p xdl-cli -- -q examples/mars_helicopter_battery_sweep.xdl
cargo run -p xdl-cli -- --headless -q examples/mars_helicopter_system_level.xdl
cargo run -p xdl-cli -- --headless -q examples/mars_helicopter_results.xdl
cargo test -p xdl-cli --test cli_tests
cargo test -p xdl-model
cargo test -p xdl-sim
```

The current exit criterion is not “looks like Simulink.” It is that every row
has an executable claim, an evidence path, and an explicit next compatibility
step.
