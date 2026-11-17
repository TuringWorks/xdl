# VibeCoder + XDL Scientific Computing Platform Gap Analysis

**Date:** 2026-11-13
**Scope:** MATLAB, GNU Octave, Scilab/Xcos, and MATLAB/Simulink-class scientific and engineering workflows  
**North star:** make VibeCoder + XDL the most productive, reproducible, extensible, and deployable open scientific-computing platform for research and industry engineering.

## Executive conclusion

The winning strategy is not to clone every MATLAB toolbox or reproduce the entire MathWorks catalog one package at a time. The platform should combine two complementary products:

```text
 VibeCoder: AI-native scientific workbench, editor, notebook, project, review,
            agent, collaboration, provenance, and deployment UX
                              |
 XDL:       open numerical language, arrays, models, solvers, data, plots,
            typed Rust kernels, deterministic simulation, and native embedding
                              |
 Open ecosystem: Python/R/Julia/C/C++/Fortran, files, databases, FMUs,
                 hardware, cloud jobs, containers, and domain libraries
```

XDL already has meaningful breadth: an interpreter and REPL, MATLAB transpiler, 500+ standard-library claims, linear algebra, signal/image/ML functions, dataframe/database work, plotting and 3D, GPU/SIMD experiments, LSP support, scientific-I/O hooks, and emerging `xdl-model`/`xdl-sim`/GNC crates. VibeCoder already supplies the engineering shell: Monaco, LSP, Git, terminal, browser preview, AI agents, diff review, MCP, hooks, checkpoints, artifacts, traces, background jobs, and parallel agents.

The decisive gaps are not “more functions” alone. They are:

1. **Trustworthy numerical contracts:** typed arrays, units, dimensions, sparse/distributed data, rigorous error semantics, and reference validation.
2. **A first-class scientific artifact:** notebooks/runs/models that capture code, data, parameters, environment, seeds, outputs, plots, and provenance.
3. **Model-based engineering:** block/signal graphs, deterministic multi-rate simulation, continuous solvers, FMU exchange, scopes, SIL/PIL/HIL adapters, and code generation.
4. **Production interoperability:** Python/R/Julia/C/C++/Fortran, Arrow/Parquet/Zarr/HDF5/NetCDF/FITS, SQL, FMI, ROS 2, CAN, MQTT, OPC UA, and cloud/HPC execution.
5. **Industrial verification:** requirements-to-test traceability, golden vectors, numerical tolerances, coverage, static checks, fixed-point analysis, reproducible CI, and audit-ready reports.
6. **AI with engineering discipline:** agents that can inspect data and models, run experiments, explain numerical changes, propose diffs, and preserve an immutable run ledger.

The recommended wedge is **AI-native, open, reproducible model-based scientific engineering**, starting with GNC/robotics/autonomy and expanding through reusable numerical and domain packages. Competing head-on with MATLAB’s breadth is a long-term outcome, not the first milestone.

## Competitive capability map

The competitor baseline is based on the current official product descriptions: MATLAB covers matrix/array language, data import, mathematics, graphics, programming, app building, software development, external interfaces, and deployment; MathWorks exposes a large toolbox family across optimization, statistics, signal processing, control, robotics, imaging, code generation, verification, and deployment. Simulink adds block-diagram modeling, libraries, fixed/variable-step solvers, scopes, code generation, SIL/PIL/HIL, and continuous verification. Scilab combines numerical analysis, visualization, algorithm development, deployment, and Xcos for hybrid dynamic-system modeling. Octave is the principal open MATLAB-language compatibility target and ecosystem reference.

### Maturity scale

| Rating | Meaning |
|---|---|
| **A** | Demonstrable, tested, documented, and usable in a real workflow |
| **B** | Core exists, but depth, coverage, UX, or verification is incomplete |
| **C** | Prototype, placeholder, or narrow slice |
| **D** | Planned/absent |
| **E** | Fundamental architectural gap |

### Feature-by-feature matrix

| Capability family | MATLAB / Octave / Scilab / Simulink expectation | XDL today | VibeCoder today | Target combination | Gap / priority |
|---|---|---|---|---|---|
| Matrix language and expressions | Mature matrix/array syntax, broadcasting, indexing, scripts/functions | XDL/IDL language and arrays; MATLAB transpiler is basic-to-moderate | Monaco, agent editing, LSP | MATLAB-compatible surface plus safer XDL-native mode | **B → A; P0**: array literals, indexing, broadcasting, shape diagnostics, compatibility tests |
| Functions, modules, packages, OOP | Scripts, functions, classes, packages, namespaces | User functions/procedures; limited MATLAB class/package support | Workspace/project editing and Git | Package manager, modules, typed APIs, generated docs | **C → B; P1** |
| Interactive console / REPL | Rich command window, history, workspace inspection | CLI/REPL exists | Integrated terminal and agent context | Scientific console with tables, plots, variable inspector, rich errors | **B → A; P0** |
| Notebook / literate computing | Live Editor, Jupyter integrations, executable reports | No clear first-class notebook artifact | Browser panel, editor, artifacts | `.xnb`/`.xdlbook` notebook with code, results, plots, markdown, run metadata | **D → A; P0** |
| Debugging and profiling | Breakpoints, stepping, workspace, profiler | Basic interpreter diagnostics; no complete debugger contract | Agent traces and terminal; Monaco foundation | Source debugger, array inspector, time profiler, memory profiler, agent-assisted diagnosis | **C → A; P1** |
| Core math | Linear algebra, calculus, transforms, special functions | Strong breadth claims; nalgebra and stdlib foundations | UX only | Validated BLAS/LAPACK/FFT/special-function layer with documented numerical behavior | **B → A; P0** |
| Sparse and structured arrays | Sparse matrices, decompositions, banded/structured operations | Not a demonstrated canonical type | None | Sparse, diagonal, banded, lazy, chunked, memory-mapped arrays | **D/E → A; P0** |
| GPU / accelerator compute | GPU arrays, CUDA, distributed/parallel computing | `xdl-amp` with multiple backend experiments | Background jobs and agents | Capability-checked CPU/GPU execution with correctness parity and provenance | **C → B; P1** |
| Automatic differentiation | Deep learning and optimization workflows increasingly depend on AD | No first-class AD contract found | Agents can generate code, not differentiate it | Forward/reverse AD, Jacobians, sensitivities, differentiable simulation | **D → A; P1** |
| Symbolic math | Symbolic variables, simplification, calculus, code generation | Not a demonstrated subsystem | AI can assist explanations | Symbolic package or integration with SymPy/Sage, with explicit exact/float boundaries | **D → B; P2** |
| Statistics | Descriptive stats, distributions, hypothesis tests, regression | Mean/variance/fit and ML functions; depth uneven | Experiment management can be added | Tested statistics/distributions package and result objects | **B/C → A; P1** |
| Optimization | Local/global, constrained, least squares, integer, nonlinear | Some ML/linalg; no complete optimization product | Agents can run sweeps | Optimization API with gradients, constraints, diagnostics, parallel sweeps | **C → A; P1** |
| Signal processing | Filters, spectral analysis, wavelets, time-frequency, DSP | Signal functions and FFT foundations | Data/workflow UX | Validated DSP package, streaming signals, spectrogram/scopes, hardware adapters | **B → A; P1** |
| Image / computer vision | Image processing, CV, medical imaging, acquisition | Image-processing functions and 3D foundation | Browser/asset/artifact UX | Typed image types, pipelines, GPU kernels, dataset/annotation support | **B/C → A; P1** |
| Dataframes / tabular data | Tables, joins, groups, import/export, large data | Polars dataframe crate and database crate | Project/terminal/agent workflow | DataFrame + Arrow/Parquet/SQL + lazy plans + visual data explorer | **B → A; P0** |
| Data formats | CSV, MAT, HDF5, NetCDF, FITS, images, audio, spreadsheets | CSV/JSON/Parquet/DB; FITS/HDF5/NetCDF partly placeholder/feature gated | Files, browser, agent tools | Production readers/writers with schema, chunking, metadata, streaming, round-trip tests | **C → A; P0** |
| Scientific visualization | Publication 2D/3D, layouts, annotations, animation, export | Plotting, ECharts, 3D, GUI; backend depth varies | Browser preview and artifact browser | Grammar-independent plotting API, interactive views, SVG/PDF/PNG/WebGPU, reproducible themes | **B → A; P0** |
| App/UI building | App Designer, GUIs, deployment | Widgets/dialogs mostly placeholders or GUI-specific | Strong desktop UI shell and extension system | Scientific app panels, plot-linked controls, forms, dashboards, shareable apps | **C → A; P1** |
| Project/package management | MATLAB projects, paths, add-ons, toolbox packaging | Cargo workspace, docs, scripts | Multi-workspace, Git, extensions | `xdl.toml`, lockfile, package registry, environment capture, domain bundles | **B → A; P0** |
| Language tooling | Editor, completion, diagnostics, refactor, help | XDL LSP + VS Code extension | Monaco, LSP client, command palette | Native XDL language experience inside VibeCoder with symbol-aware agent context | **B → A; P0** |
| AI assistance | MATLAB Copilot/agent integrations are now product features | No integrated product agent | Strong multi-provider agent loop, MCP, hooks, memory, traces | Scientific agent that can plan/run/compare/verify and cite artifacts | **B → A; P0** |
| Reproducibility | Scripts, projects, Live Editor, CI, artifacts | Versioned manifest with source hashes, timing, stdout/failure artifacts, and local replay; numerical/clean-machine evidence incomplete | Traces/checkpoints/artifacts | Content-addressed run ledger: code/data/model/config/env/seed/output hashes | **C → A; P0** |
| Version control / review | Git integrations, project diffing | Git repository and scripts | Mature Git/diff/agent review surface | Numerical diff review: data, plots, model graphs, metrics, not only text | **B → A; P1** |
| Continuous integration | MATLAB Test, projects, code quality, coverage | Recursive `.xdl` runner with manifest, source/artifact hashes, diagnostics, and local replay; golden/numerical/timeout gates incomplete | Hooks/jobs/agent orchestration | `xdl test`, golden vectors, property tests, benchmark gates, report artifacts | **C → A; P0** |
| Control design | PID, state space, LQR/LQG, frequency response, robust/MPC/fuzzy | GNC crates and linalg; no complete control toolbox | Workflow shell | Control toolbox with typed systems, trim, linearization, tuning, margins, MPC | **C → A; P1** |
| Model identification | Parameter estimation and system identification | Not a demonstrated complete package | Experiment orchestration possible | Identification pipelines with uncertainty and validation datasets | **D → B; P2** |
| Dynamic simulation | ODE/DAE solvers, events, hybrid systems | `xdl-sim` emerging, fixed-step/multi-rate intent | Run orchestration | Deterministic discrete + continuous + hybrid engine | **C → A; P0** |
| Block diagrams | Simulink/Xcos graphical/programmatic models | `xdl-model` typed graph seed; no full editor | Canvas/UI platform exists but not scientific model editor | Versioned block graph, visual editor, code/API parity | **C/D → A; P0** |
| Physical modeling | Simscape-style mechanical/electrical/thermal/fluids | GNC domain crates only | None | Component/connector/units model library and FMI integration | **D/E → B; P2** |
| State machines/events | Stateflow/SimEvents/Xcos hybrid systems | Language control flow only | Agent/IDE shell | Explicit state/event subsystem with deterministic semantics | **D → A; P1** |
| FMI / co-simulation | Simulink FMU, Xcos FMI, plant integration | Not first-class | MCP can host adapters | FMU 1/2/3 import/export and co-simulation runner | **D → B; P1** |
| Code generation | C/C++/CUDA/HDL/PLC and embedded deployment | Rust/C FFI prototypes; no model codegen contract | Build/terminal/agent infrastructure | Rust first, C ABI second, C/CUDA/WASM/embedded profiles later | **C/D → A; P0** |
| SIL/PIL/HIL | Mature model-to-code validation and real-time workflows | Not present as a product | Jobs/terminal/device hooks | Same model/test vectors across interpreter, native, target, and HIL | **D → B; P1** |
| Requirements / traceability | Requirements Toolbox, test linkage, reports | No first-class traceability | Session traces and artifacts | Requirements IDs linked to model blocks, tests, runs, results, approvals | **D → A; P1** |
| Safety / compliance | DO-178, ISO 26262, IEC, MISRA/Polyspace ecosystem | No claims; Rust safety foundation only | Audit traces and checkpoints | Evidence-oriented workflows; never claim certification without qualification | **D/E → B; P2** |
| Robotics/autonomy | ROS, navigation, sensor fusion, mapping, simulation | GNC crates and visualization seed | MCP/device-agent potential | ROS 2, bags, frames, filters, planning, perception, scenario replay | **C → A; P1** |
| Domain toolboxes | Aerospace, automotive, RF, finance, biology, energy, etc. | Early GNC focus | Extension/plugin system | Installable domain packs, shared core APIs, community registry | **D → B; P2** |
| Distributed/HPC/cloud | Parallel Server, clusters, cloud, batch | Rayon/GPU experiments; no unified job model | Background jobs, manager view | Reproducible local/remote jobs with artifacts and cost/resource metadata | **C → A; P1** |
| Deployment | Apps, web, services, compiled artifacts | CLI/GUI/FFI; native Rust path | Tauri desktop, extensions | One-click desktop, web, container, service, WASM, embedded export | **B/C → A; P1** |
| Education/community | Courses, examples, package ecosystem, support | Docs/examples; small ecosystem | AI onboarding and project workflows | Open examples, executable lessons, benchmark corpus, support/community loop | **C → A; P1** |

## What “preeminent” should mean

The platform should win on dimensions where an open Rust + AI-native system can be structurally better than a mature proprietary suite:

- **Reproducibility by default:** every run has a manifest, hashes, seeds, environment, and exact inputs.
- **Agentic experimentation:** an agent can form a hypothesis, edit a branch, run a parameter sweep, compare metrics and plots, explain the change, and present a reviewable diff.
- **One semantic model, many execution targets:** XDL interpretation, native Rust, generated C, WASM, GPU, and HIL share the same typed model and test vectors.
- **Open interoperability:** no walled garden around files, models, APIs, packages, or deployment.
- **Industry-grade determinism:** fixed-step scheduling, explicit units/frames/rates, numerical tolerances, failure modes, and traceability are first-class.
- **Fast path from idea to product:** notebook/script → tested package → model → native library/service/embedded artifact.
- **Excellent collaboration:** scientific changes are reviewable as code, model graph diffs, data summaries, plot diffs, and test evidence.
- **Domain packs instead of monolith:** aerospace/GNC, robotics, energy, signal processing, imaging, finance, and biology can evolve independently on stable core contracts.

## Gap diagnosis: the important architectural issues

### 1. The numerical core needs a canonical typed data model

Dynamic `XdlValue` is appropriate at the language boundary, but industry workflows need explicit types for:

- scalar and array element type;
- shape, rank, strides, layout, and memory ownership;
- sparse/diagonal/banded/chunked/memory-mapped representation;
- units and dimensional analysis;
- coordinate frames and reference systems;
- sample rate, timestamp, validity, and provenance;
- missing, NaN, Inf, overflow, saturation, and uncertainty;
- CPU/GPU/device placement.

Create a `xdl-data`/`xdl-numerics` contract that can be consumed by the interpreter, dataframe layer, plots, model engine, FFI, and code generator. Keep dynamic conversion at edges and make the typed representation canonical for important kernels.

### 2. XDL needs a real scientific execution artifact

A source file is not enough for serious engineering. Define a versioned artifact containing:

```text
project/
  xdl.toml                 # dependencies, profiles, targets
  model.xdlmodel           # optional typed block/signal graph
  notebooks/*.xdlbook      # executable cells and narrative
  runs/<run-id>/manifest.json
  runs/<run-id>/inputs/    # content-addressed or referenced inputs
  runs/<run-id>/results/   # tables, arrays, plots, logs
  tests/                   # golden vectors and properties
  reports/                 # HTML/PDF/JSON evidence
```

VibeCoder should make this artifact visible and reviewable. The agent must never silently overwrite the only copy of a model, result, or dataset.

### 3. Model-based design is a separate product surface

More built-ins will not create a Simulink competitor. The missing semantic contract is:

```text
typed BlockGraph + SignalGraph
  -> structural validation
  -> deterministic scheduler / solver
  -> scopes and run artifacts
  -> test / requirements evidence
  -> native code generation / HIL adapters
```

The existing `xdl-model` and `xdl-sim` crates are the correct direction. Stabilize their schema and execution semantics before building a beautiful editor. The visual editor should be a client of the model graph, not the source of truth.

### 4. Compatibility must be measured, not advertised

The project currently has useful MATLAB compatibility documentation, but it also records important limitations such as array literals/indexing, anonymous functions, classes, file I/O, and incomplete plotting semantics. Treat compatibility as a conformance program:

- parser corpus by syntax feature;
- function-by-function mapping table;
- behavioral tests against MATLAB/Octave where licensing permits and against independent references otherwise;
- numerical tolerance policy by function family;
- source-location mapping for transpiler errors;
- “unsupported” diagnostics that name the exact gap and suggest a migration path;
- no claim of support when an implementation is a placeholder.

The same rule applies to scientific I/O and GPU backends: capability probes must fail closed and report what actually ran.

## Product architecture

### Layers

| Layer | Responsibility | Primary home |
|---|---|---|
| Interaction | Editor, notebook, plots, model canvas, inspectors, command palette | VibeCoder |
| AI orchestration | Planning, tools, experiment loops, reviews, memory, policy, MCP | VibeCoder / VibeCLI |
| Project/artifacts | Manifest, run ledger, data lineage, reports, package UX | Shared protocol; VibeCoder UI |
| Language | XDL parser, interpreter, MATLAB/Octave compatibility, LSP | XDL |
| Canonical data | Arrays, tables, units, frames, signals, sparse/chunked data | XDL crates |
| Numerical kernels | BLAS/LAPACK/FFT, solvers, optimization, AD, DSP, image, ML | XDL/Rust crates |
| Model execution | Block graph, scheduler, solver, event semantics, scopes | `xdl-model`, `xdl-sim`, future crates |
| Domain libraries | GNC, robotics, control, energy, imaging, RF, etc. | XDL domain crates |
| Integration | Python/R/Julia/C/C++/Fortran, FMI, ROS, SQL, files, hardware | XDL adapters + VibeCoder tools |
| Deployment | Native libraries, services, WASM, GPU, embedded, containers | XDL build/codegen + VibeCoder workflows |

### Core contracts to freeze early

1. **Array and table ABI:** layout, ownership, dtype, missingness, device, serialization.
2. **Units and frames:** conversion, validation, and error reporting.
3. **Signal contract:** timestamp, rate, validity, uncertainty, frame, latency, and ordering.
4. **Model schema:** stable IDs, hierarchy, ports, parameters, sample times, events, versions.
5. **Run manifest:** source/model/data/config/environment/seed/toolchain hashes.
6. **Error contract:** structured diagnostics with source spans, machine codes, and remediation.
7. **Plugin contract:** versioned domain packages and capability discovery.
8. **Execution contract:** interpreted/native/generated equivalence and defined tolerance classes.

## Prioritized roadmap

### P0 — trust and daily usability (0–6 months)

Goal: a scientist can install the platform, import a real dataset, write MATLAB-like code, see a plot, run a test, and reproduce the result.

- Establish `xdl.toml`, lockfile, package identity, profiles, and environment capture.
- Implement first-class notebook/run artifacts in VibeCoder.
- Finish core MATLAB/Octave syntax: literals, colon/range/indexing, broadcasting, anonymous functions, structs/cells where feasible, `try/catch`, file I/O, plotting properties.
- Replace compatibility claims with a generated function/syntax coverage report,
  then add runtime and numerical reference evidence to corpus entries.
- Promote scientific I/O from placeholders to tested readers/writers; prioritize HDF5, NetCDF, FITS, MAT, Arrow/Parquet, Zarr, CSV.
- [partial] Implement `xdl test` with unit, script, golden-vector, property,
  and numerical-tolerance modes. Recursive script execution, manifests,
  hashes, artifacts, and local replay exist; golden/property/numerical modes
  remain open.
- Build numerical diff views: array summary, histogram/delta, plot image diff, table diff, model graph diff.
- Stabilize array shape/type diagnostics and error spans.
- Create the first 20 end-to-end examples: signal analysis, imaging, tabular science, GNC, Monte Carlo, and a model simulation.
- Add a VibeCoder XDL workspace mode: run/inspect variables, plots, diagnostics, tests, and artifacts from one panel.

**P0 exit criteria:** 50 real-world MATLAB/Octave scripts run or fail with actionable diagnostics; 10 GB-class chunked dataset workflow; every result has a replayable manifest; CI can reproduce a reference run on a clean machine.

### P1 — model-based scientific engineering (6–18 months)

- Finish typed `BlockGraph`/`SignalGraph` and schema versioning.
- Deterministic fixed-step and multi-rate scheduler with rate transitions, delays, events, and trace logs.
- Continuous solvers: Euler/RK2/RK4/RK45 plus stiff-solver interface and failure diagnostics.
- Scopes, signal logging, replay, synchronized cursors, and decimation.
- Unit/frame/rate validation and explicit algebraic-loop diagnostics.
- Control package: LTI systems, state-space, transfer functions, PID, LQR/LQG, trim, linearization, frequency response, margins, and basic MPC.
- FMU import/export and co-simulation runner.
- Rust-native execution and C ABI for model kernels; generated Rust for deterministic profiles.
- Scripting and visual model parity: every canvas action serializes to the same graph API.
- VibeCoder model canvas, parameter inspector, scope panel, trace explorer, and model-aware agent tools.

**P1 exit criteria:** a closed-loop GNC/robotics model can be authored by API and GUI, run deterministically, replayed, tested against golden vectors, inspected in VibeCoder, and exported as a native library.

### P2 — industry support and domain expansion (18–36 months)

- SIL/PIL/HIL adapters and real-time clock policies.
- Requirements links, coverage, test harnesses, review approvals, and evidence reports.
- Fixed-point and quantization analysis.
- Python/R/Julia zero-copy or low-copy interop; C/C++/Fortran ABI and bindgen workflow.
- ROS 2, CAN, MQTT, OPC UA, DAQ, and instrument integrations.
- Distributed/HPC job execution with resource/cost metadata.
- Domain packs: aerospace/GNC, robotics, energy, signal/RF, imaging/medical, finance, and computational biology.
- C/C++/CUDA/WASM code generation; embedded profiles and deployment templates.
- Registry, signed packages, reproducible builds, and organization policy controls.
- Collaborative review of models, numerical artifacts, and requirements evidence.

**P2 exit criteria:** selected industrial pilots can move from experiment to reviewed package to deployed native service/device artifact without changing numerical semantics or losing traceability.

## VibeCoder-specific product backlog

VibeCoder should become a **scientific engineering cockpit**, not merely a code editor with a chatbot.

### Must-have panels and commands

- Scientific workspace: project, environment, datasets, models, runs, reports.
- Variables/data inspector: shape, dtype, units, NaN/Inf, range, memory, device, provenance.
- Plot pane: linked cursors, multiple axes, annotations, export, image diff, theme lock.
- Notebook pane: executable cells with run status and captured outputs.
- Model canvas: blocks, ports, wires, subsystem hierarchy, diagnostics, scopes.
- Experiment runner: parameters, sweeps, seeds, parallelism, cancellation, result ranking.
- Test explorer: unit, script, golden, property, model, SIL/PIL/HIL.
- Artifact explorer: manifest, input hashes, environment, toolchain, output lineage.
- Agent mode: “explain this failure,” “run a controlled sweep,” “compare these runs,” “find the first numerical divergence,” “generate a test,” and “prepare a reviewable patch.”

### Scientific agent guardrails

Agents must:

- state the hypothesis and planned experiment before expensive runs;
- show the exact files, data, model, and parameters used;
- use a disposable branch/checkpoint for mutations;
- preserve random seeds and report nondeterminism;
- distinguish measured results from inference;
- never replace failed numeric values with plausible defaults;
- fail closed when a requested GPU, file format, or solver is unavailable;
- attach code, data, plot, metric, and environment diffs to the result;
- require explicit approval for external writes, hardware actions, or destructive data operations.

## MATLAB / Octave compatibility strategy

Compatibility should be a funnel, not a promise that every language feature maps perfectly:

1. **Native execution:** common MATLAB/Octave scripts run directly through a compatibility front-end.
2. **Automatic migration:** transpiler produces XDL plus source maps, warnings, and a compatibility report.
3. **Semantic bridge:** unsupported features call Python/Rust/Octave adapters when safe and configured.
4. **Native rewrite:** hot or production paths migrate to typed XDL/Rust APIs with equivalence tests.

Prioritize by real-world frequency:

- array literals, colon operator, 1-based indexing translation;
- elementwise vs matrix operators;
- broadcasting and implicit expansion;
- structs, cells, tables, strings, logicals, sparse matrices;
- scripts/functions/nested functions/anonymous functions;
- file I/O and MAT compatibility;
- plotting, handles, layouts, annotations, and export;
- `try/catch`, warnings, `nargin`, `varargin`, name-value options;
- object/class syntax only after the data and function substrate is solid.

Do not map MATLAB `randn` to a uniform generator or silently ignore plot styles. Such shortcuts create scientifically wrong output while looking successful.

## Scilab/Xcos and Simulink strategy

Scilab/Xcos validates that an open platform can combine numerical programming, visualization, deployment, and a hybrid dynamic-systems modeler. Simulink demonstrates that the industrial moat is the model lifecycle: authoring, libraries, solvers, code generation, test, traceability, and hardware integration.

XDL should target **semantic interoperability**, not file-format imitation:

- import/export through FMI wherever possible;
- a documented XDL model schema with stable IDs and explicit semantics;
- adapters for Modelica/Functional Mock-up Units and common block libraries;
- import of useful data and test vectors from MATLAB/Scilab/CSV/JSON/Parquet;
- conversion tools that produce a migration report and identify unsupported behavior;
- visual and programmatic authoring over the same graph API.

The competitive differentiator is an AI agent that understands the model graph, can propose a subsystem change, run controlled simulations, find numerical regressions, and prepare the evidence package for review.

### Present-state audit: XDL model and simulation lifecycle

This section distinguishes capabilities visible in the current `xdl-model`,
`xdl-sim`, and GNC crate APIs and their documentation from product-level parity.
“Implemented” means a concrete, usable library capability exists in the stated
scope; it does not imply complete Simulink/Xcos behavior. “Scaffolded” means
there is a useful contract or narrow vertical slice, with substantial lifecycle
or integration work still missing. “Absent” means no corresponding capability
was found in these crates or their current GNC documentation.

| Lifecycle area | Present evidence | Status | Parity gap |
|---|---|---|---|
| Block-diagram model representation | `xdl-model` defines serializable blocks, ports, signal shapes/metadata, connections, buses, subsystem boundaries, parameters, graph validation, execution ordering, and cycle checks. The Mars fixture is validated as a typed graph. | **Implemented foundation** | This is a graph/schema library, not broad Simulink/Xcos block-library, model-import, or model-authoring parity. |
| Diagram editing and display | `ModelDocument` has versioned JSON serialization, validation, layout, hit testing, selection/move/reconnect editor events, deterministic auto-layout, and structural SVG output. `StateChartDocument` adds layout, state/transition-label movement, hit testing, JSON persistence, and SVG. The FLTK state-chart editor exposes state/event/timed-transition creation and editing, deletion, initial-state selection, open/save, runtime-parameter export, selection, and drag controls. | **Scaffolded native editor** | The interactive surface is limited to state charts. A general block-model canvas, undo/history, subsystem navigation, block palette, richer annotations, and VibeCoder integration are not established. SVG and canvas rendering are early structural UX. |
| Subsystems and state/event models | Typed subsystem boundary binding, named guarded transitions, a periodic-scheduler-registered state-machine runtime, serialized state-chart documents, and a native GUI canvas for editing state/event/timed transitions exist. The host provides the active guard set; the GUI exports runtime block parameters. | **Scaffolded** | No general Stateflow/Xcos hybrid language, guard-expression evaluator, or direct execution-engine integration from the authored chart is established. |
| Discrete and multi-rate execution | `xdl-sim` has a deterministic periodic-task scheduler with explicit timestamps, phase/priority ordering, event bounds, and periods constrained to integer multiples of a base period. An execution engine routes typed signal values through registered runtimes; delay, zero-order-hold, and subsystem runtimes exist. | **Implemented narrow runtime** | The documented/runtime contract is a bounded scheduler and block runtime, not complete Simulink rate-transition, tasking, data-store, initialization, enable/trigger, or execution-order semantics. |
| Block library | Reusable scalar, vector, matrix, quaternion, bus, switch, lookup, control, and GNC-oriented block runtimes are present, including Constant, Gain, Sum, Integrator, Derivative, PID, StateSpace, delays, and rate limiting. | **Implemented subset** | The set is not a comprehensive, versioned Simulink/Xcos library; broad domain, physical-network, and compatibility semantics remain unverified. |
| Numerical solvers | `xdl-sim::solver` provides bounded explicit Euler and classical RK4 steps plus RK4 step-doubling with an error estimate and explicit retry response. Continuous runtime integration consumes adaptive RK4; diagnostics reject invalid/non-finite states and bound retries. | **Implemented subset** | No evidence here for stiff ODE methods, DAE solvers, zero-crossing/event localization, solver configuration parity, robust variable-step production behavior, or a comparative solver-conformance suite. Do not describe this as full Simulink solver parity. |
| Model execution and examples | A typed model-graph execution path, continuous runtime adapter, deterministic GNC integration examples, and an executable Mars system-level XDL scenario are documented. The Mars example is expressly described as a reduced-order analogue, with unvalidated thrust/battery maps and omitted physical/aerodynamic effects. | **Scaffolded vertical slices** | The Mars XDL scenario and the JSON graph fixture are evidence of related workflows, not proof that the graph fixture executes that full scenario or that the physical model is validated. |
| Simulation traces and replay | `ExecutionReport` serializes signal updates as JSON Lines; `TraceCursor` replays them, and `compare_traces` checks metadata and values under absolute/relative tolerances. GNC replay and telemetry crates provide additional deterministic, typed replay/export slices. | **Implemented narrow verification utility** | This is not a full run/artifact lifecycle: no demonstrated model coverage, requirements traceability, test campaign/reporting, clean-machine end-to-end model replay contract, or audit approval workflow. |
| Code generation and deployment | The inspected `xdl-model` and `xdl-sim` APIs expose model representation, runtime execution, and solver functions. The GNC crate guide describes these as typed Rust libraries and does not document a model compiler/code generator. | **Absent** | No model-to-Rust/C/C++/CUDA/WASM/PLC/HDL generation, generated artifact validation, target profile, or generated-vs-interpreted equivalence contract is established. Existing XDL C FFI or ordinary Rust compilation is not model code generation. |
| SIL, PIL, HIL, and real-time verification | No such target runners or hardware integration are documented in the audited model/simulation/GNC surfaces. The GNC guide states hardware and flight data are unverified; the Mars example disclaims flight readiness. | **Absent** | No software-, processor-, or hardware-in-the-loop lifecycle, real-time scheduling guarantee, target trace capture, or equivalent-result evidence. |
| Requirements, coverage, and assurance | Unit/contract tests, deterministic scenario/replay checks, and tolerance-based trace comparison exist as engineering evidence. | **Scaffolded verification** | Requirements-to-model/test links, model or decision coverage, formal analysis, safety-case generation, certification evidence, and production acceptance gates are not established. |

**Current parity judgment:** XDL has an implemented typed graph and useful
simulation primitives, plus narrow editor, solver, replay, and GNC vertical
slices. The integrated block-diagram lifecycle remains **scaffolded**; model
code generation and SIL/PIL/HIL are **absent** in the audited surfaces. This is
meaningful groundwork, not a Simulink or Xcos competitor today. The GNC
documentation likewise labels the crates in development and explicitly says
they are unverified on hardware or flight data.

**Evidence locations:** [`xdl-model` API](../xdl-model/src/lib.rs),
[`xdl-model` editor and document API](../xdl-model/src/editor.rs),
[`xdl-sim` runtime and scheduler](../xdl-sim/src/lib.rs),
[`xdl-sim` block library](../xdl-sim/src/blocks.rs),
[`xdl-sim` solvers](../xdl-sim/src/solver.rs),
[GNC crate status and limitations](development/gnc-crates.md), and the
[Mars model fixture checks](../xdl-model/tests/mars_system_model.rs). The test
counts in the verification table below are the recorded 2026-11-13 checkpoint;
this documentation audit did not rerun tests.

## Verification and quality gates

Every major capability must have four layers of evidence:

| Layer | Example |
|---|---|
| Semantic | parser/function/model behavior is pinned by tests |
| Numerical | reference values, tolerances, NaN/Inf/overflow cases |
| Workflow | clean-machine run through CLI and VibeCoder |
| Artifact | manifest, inputs, outputs, plots, logs, and provenance are replayable |

Required benchmark/reference suites:

- MATLAB/Octave syntax and function corpus;
- NumPy/SciPy/PyTorch interop comparisons;
- BLAS/LAPACK/FFT and special-function reference vectors;
- HDF5/NetCDF/FITS/MAT/Arrow/Parquet round trips;
- solver convergence, event ordering, stiffness, and determinism;
- GPU versus CPU bit/tolerance comparisons;
- model interpreted/native/generated equivalence;
- plot snapshots and image diffs;
- memory and throughput benchmarks on representative array sizes;
- GNC/robotics scenario replays with seeded noise and fault injection.

Quality gates should reject:

- unsupported features silently becoming no-ops;
- placeholder scientific-I/O paths in production profiles;
- GPU claims when execution fell back to CPU;
- changed error text or numerical behavior without a compatibility note;
- nondeterministic results without a declared source and policy;
- artifact runs missing source/data/config/toolchain hashes.

## Operating model and ecosystem

### Packaging and governance

- Keep the numerical core permissively licensed where possible and preserve required upstream licenses.
- Publish stable extension interfaces and semantic versioning.
- Maintain a public compatibility dashboard generated from tests.
- Accept domain packs independently from the core release cadence.
- Provide long-term support releases for industrial users.
- Publish security advisories, provenance/SBOM data, and reproducible build instructions.

### Community flywheel

1. Open examples solve real problems and produce artifacts.
2. Artifacts become regression tests and documentation.
3. Domain experts contribute packages and reference data.
4. VibeCoder agents make the contribution loop easier.
5. Compatibility dashboards and benchmark results build trust.
6. Industrial pilots fund deeper integrations and verification.

### Commercial support opportunities

- enterprise support and LTS;
- validated domain packs;
- managed HPC/cloud runners;
- qualification/evidence packages;
- integration services for legacy MATLAB/Simulink estates;
- private package registries and policy controls;
- training, migration, and model-review services.

## Measures of success

Track product outcomes, not just crate counts:

- percentage of top 1,000 MATLAB/Octave functions with behavioral tests;
- percentage of real-world compatibility corpus passing;
- time from dataset import to first valid plot;
- time from script to reproducible run artifact;
- time from model edit to verified regression result;
- interpreted/native/generated numerical equivalence rate;
- clean-machine replay success rate;
- median agent experiment success rate without manual repair;
- number of domain packages and active contributors;
- benchmark throughput and peak memory against MATLAB/Octave/Python baselines;
- number of industrial workflows with requirements-to-result traceability;
- defect escape rate for numerical, I/O, and reproducibility issues.

## Immediate next actions

1. Create a `scientific-platform` milestone spanning VibeCoder and XDL.
2. Freeze the data, units, signal, model, run-manifest, and error contracts.
3. Build the P0 compatibility/conformance harness before adding another large batch of built-ins.
4. Implement `xdl test` and a replayable run manifest.
5. Replace or clearly gate scientific-I/O placeholders.
6. Ship the first VibeCoder scientific workspace with variable inspection, plots, tests, and artifacts.
7. Make `xdl-model`/`xdl-sim` the canonical path for the first closed-loop GNC demonstrator.
8. Add a real MATLAB/Octave migration corpus and publish the pass/fail matrix.
9. Develop one end-to-end industry story—GNC/robotics is the strongest initial candidate—from data ingestion through model simulation, test evidence, and native deployment.
10. Reassess every quarter using the capability matrix and measured user outcomes.

The next bounded MATLAB bridges identified from the current code are:

- Extend two-/three-output `find` to N-D and vector cases with MATLAB reference
  fixtures. The verified rectangular 2D subset supports `[row,col,linear]`
  outputs and positive integer `k` with `'first'`/`'last'` selection.
- Extend numeric `readmatrix` beyond the verified headerless rectangular CSV
  subset, using explicit options and reference fixtures before adding header,
  missing-value, or mixed-type behavior.
- Add broader plot labels and graphics-state support, including dynamic labels,
  legends, axis/grid controls, and multiple axes.

Completed since the prior checkpoint: literal `title`, `xlabel`, and `ylabel`
calls immediately following a supported `plot(x,y)` are lowered into XDL plot
keywords. Unsupported annotation forms now fail explicitly rather than being
silently discarded. Two-output `find(A,k,direction)` now forwards the optional
limit and direction to its runtime helpers and has CLI fixture coverage for
first/last selection. Three-output `[row,col,linear]=find(A,k,direction)` is
now supported for rectangular 2D numeric matrices and covered by runtime,
transpiler, and CLI checks. Acceptance for remaining increments needs
end-to-end `.m` fixtures and reference outputs for each behavior.

## Verified local baseline

The table records the 2026-11-13 checkpoint; MATLAB, CLI, interpreter, model,
simulation, standard-library, and LSP rows were rerun on 2026-11-16 for the
compatibility and state-machine increments. Other crate counts remain the
original snapshot and have not been rechecked here.

The current repository checkpoint provides executable evidence for the first
model-based foundation:

| Surface | Verification | Result | Interpretation |
|---|---|---:|---|
| MATLAB transpiler | `cargo test -p xdl-matlab` | 50 passed (verified 2026-11-16) | Tested syntax/function subset, including vector `end`/omitted bounds, 2D scalar and matrix range subscripts, scalar and block indexed-assignment lowering, one-/two-output `find` with bounded first/last selection, single-/multiple-output `size`, named-array `A.'`, negative array literals, and literal plot-label lowering; not MATLAB/Octave parity |
| Interpreter | `cargo test -p xdl-interpreter` | 41 passed (verified 2026-11-16) | Includes rectangular 2D range reads with row/column shape preservation, ragged-matrix rejection, and shape-checked two-axis block writes; not broad MATLAB indexing parity |
| Typed model graph/editor | `cargo test -p xdl-model` | 32 passed (verified 2026-11-16) | Schema, typed connections, validation, layout, editor serialization, state-chart document layout/JSON/SVG/hit testing, validated Mars system-model fixture, structural SVG rendering, and state/event contracts with named guards |
| Native state-chart UI | `cargo check -p xdl-gui` | Passed (verified 2026-11-16) | FLTK canvas with state/event/timer create/edit/delete, initial-state selection, JSON open/save, runtime-parameter export, selection, and move controls compiles; no automated GUI interaction or visual QA was run |
| Deterministic simulation | `cargo test -p xdl-sim` | 28 passed (verified 2026-11-16) | Scheduler, multi-rate ordering, delays, state-space and multiport-switch blocks, typed model-graph execution, scheduler-registered timed state-machine dispatch with host-guarded transitions, continuous integration, and solver error paths foundation |
| Standard library | `cargo test -p xdl-stdlib` | 132 unit + 29 integration passed (verified 2026-11-16) | Full default-feature stdlib suite, including numeric CSV import, one-/two-output `find` with optional limits/direction, malformed/ragged/empty CSV rejection, matrix transpose, and elementwise/scalar-expanded `atan2` |
| Artifact contracts | `cargo test -p xdl-artifacts` | 7 passed | Versioned manifests, bundled JSON schemas, compatibility statuses, artifact hashes, and fail-closed validation |
| Standard-library `atan2` | `cargo test -p xdl-stdlib atan2_tests` | 8 passed (verified 2026-11-16) | Same-shape vector, nested-matrix, and shaped-array elementwise results, scalar expansion, quadrant and signed-zero values, malformed-shape and mismatch rejection; complex and broader singleton-dimension expansion are not established |
| LSP built-ins | `cargo test -p xdl-lsp` | 6 passed (verified 2026-11-16) | Includes case-insensitive ATAN metadata plus MATLAB_READMATRIX and MATLAB_FIND_ROWS/COLUMNS help; not full MATLAB editor integration |
| CLI evidence/replay | `cargo test -p xdl-cli --test cli_tests` | 35 passed (verified 2026-11-16) | Includes MATLAB matrix range reads and scalar/block writes, `find` orientation/limits/directions, nonconjugating transpose, elementwise/scalar-expanded `atan2`, array information, Mars simulation-to-CSV-to-headless-plot, GNC examples, manifests, and replay |
| MATLAB CSV-to-plot | `cargo test -p xdl-cli --test matlab_readmatrix` | 1 passed (verified 2026-11-16) | Imports headerless numeric CSV with `readmatrix`, reads dimensions and values, selects columns, and confirms native PNG output; import options remain unsupported |
| MATLAB two-output `find` | `cargo test -p xdl-cli --test matlab_find_outputs` | 1 passed (verified 2026-11-16) | Verifies one-based row/column subscripts, column-major order, optional `k` with `'first'`/`'last'`, and empty output shapes in `.m` execution |

These tests are useful exit-criteria evidence, but they do not establish
production readiness. The next validation layer must add real XDL programs,
reference numerical vectors, clean-machine replay, artifact manifests, and
cross-target equivalence tests.

The current MATLAB compatibility surface also has known semantic mismatches
that passing tests above do not cover:

- Single-output `size(A)`, `size(A, dim)`, and `[d1,...,dn] = size(A)` for a
  known array variable use `MATLAB_SIZE` and `MATLAB_SIZE_TAIL`. General
  expression/object forms remain unverified.
- `find(A)`, `find(A,k)`, and `find(A,k,'first'|'last')` route through
  `MATLAB_FIND` for tested numeric vectors and rectangular 2D matrices,
  returning one-based indices and preserving row-vector orientation. Multiple
  outputs and broader type/order conformance remain open.
- `atan2(y,x)` is mapped to two-argument `ATAN` and tested elementwise for
  equal-shape numeric vectors, shaped arrays, and rectangular nested matrices,
  including numeric scalar expansion. Complex values and general implicit
  expansion remain unverified.
- Nonconjugating transpose `A.'` is executable for directly named numeric
  vectors and rectangular matrices. Conjugating transpose, compound-expression
  transpose, and broader XDL array-shape preservation remain open.
- Scalar read `A(k)` routes through `MATLAB_INDEX`; transpiler and CLI tests
  verify matrix column-major order and `end`. Scalar vector assignment and
  two-subscript matrix assignment have a CLI execution case. Positive-step
  row/column matrix range writes support scalar expansion or a matching-length
  numeric vector when the other subscript is scalar. Two-axis block writes
  support scalar expansion or a rectangular source with matching shape.
  Rectangular numeric 2D
  range reads `A(i,j:k)`, `A(i:j,k)`, `A(:,k)`, and `A(i,:)` have interpreter
  and CLI coverage with row/column shape preserved. Nested range sources,
  single-subscript matrix writes, and vector-valued subscript results remain
  open; the existing
  `A(i,j)` path is independently tested.

These are explicit parity gaps, not covered capabilities. Their implementation
needs dedicated MATLAB/Octave reference cases before any compatibility claim.

## Sources and local evidence

Official competitor references:

- [MATLAB documentation and capabilities](https://www.mathworks.com/help/matlab/index.html)
- [MATLAB and Simulink product family](https://www.mathworks.com/products.html)
- [Simulink documentation](https://www.mathworks.com/help/simulink/index.html)
- [Simulink product overview](https://www.mathworks.com/products/simulink.html)
- [Scilab features](https://www.scilab.org/features)
- [Scilab software and Xcos overview](https://www.scilab.org/software)

Local evidence inspected:

- XDL `README.md`, `TODO.md`, MATLAB compatibility docs, and the crate inventory.
- XDL `xdl-model` and `xdl-sim` package manifests.
- VibeCoder `README.md`, which documents Monaco, LSP, agent loop, diff review, Git, terminal, MCP, hooks, checkpoints, artifacts, traces, background jobs, and parallel-agent management.

This document is a product strategy and gap map, not a certification claim. Safety-critical, regulated, or flight-critical use requires separate qualification, independent verification, and domain-specific evidence.
