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
| Solar input and battery SOC | Reduced-order | XDL example final SOC and battery-string sweep | Electrothermal battery model, charging limits, cell imbalance |
| Six-cell battery pack | Reduced-order | `SixCellBatteryPack` in `examples/mars_helicopter_system_level.model.json` | Electrical network and validated cell/string behavior |
| Heater and thermal readiness | Reduced-order | `GNC_FLIGHT_READINESS_STEP`; hysteresis tests | Thermal conduction/radiation network and calibrated heater plant |
| Flight-readiness state chart | Partial, executable | `GNC_FLIGHT_READINESS_STEP`; `COLD → READY → FLYING` trace | General state/event runtime, timed events, guards, graphical editor |
| Flight controller | Partial, executable | `GNC_WAYPOINT`, `GNC_PID_STEP`, `GNC_ACTUATOR_STEP` | Multi-axis attitude/position controller, tuning and linearization |
| Motor and gearboxes | Reduced-order | Motor speed, gear ratio, upper/lower rotor states in XDL example | Mechanical drivetrain, torque/load coupling, validated gearbox data |
| Contra-rotating coaxial rotors | Reduced-order | `CoaxialRotorGearbox` model block and dual rotor state | Blade aerodynamics, induced flow, flap effects, rotor interference |
| Ground contact / landing protection | Reduced-order | Explicit altitude floor in XDL example | Contact forces, bounce/friction, terrain geometry |
| Telemetry and replay | Partial, executable | `GNC_TELEMETRY_JSON`, `xdl test --manifest`, `xdl replay`; simulation writes `mars_helicopter_results.csv` | Typed battery/thermal channels in telemetry, clean-machine/reference replay |
| Altitude and temperature result plots | Partial, executable | Simulation CSV loaded by `examples/mars_helicopter_results.xdl`; separate altitude/temperature PNGs | Visual comparison against reference and manifest-linked plot artifact capture |
| Battery-string flight-duration sweep | Reduced-order, executable | `examples/mars_helicopter_battery_sweep.xdl` | Couple sweep to full plant, parameter artifacts, ranking and plots |
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
