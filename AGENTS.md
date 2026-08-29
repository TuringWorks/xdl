# XDL — Agent Guidelines

XDL is a Rust reimplementation of GNU Data Language (GDL) / IDL: a lexer, parser,
interpreter, and a 754-entry standard library, plus plotting, 3D visualisation,
GPU acceleration, a language server, and an editor extension. Everything below
exists because the surface is wide, the dispatch is stringly-typed, and **most of
the ways to break it produce no compile error at all.**

This file is the authority for all AI coding agents (Claude Code, Codex, Cursor,
…). `CLAUDE.md` is the short entry point; when the two disagree, this file wins.

---

## Product Matrix — know every surface before you change code

| Surface | Crate / path | Kind | What it is |
|---|---|---|---|
| Language core | `xdl-core` | lib | `XdlValue`, `XdlArray`, `Dimension`, `XdlError`/`XdlResult` |
| Lexer + parser | `xdl-parser` | lib | tokens → `ast.rs` (`Statement`, `Expression`) |
| Interpreter | `xdl-interpreter` | lib | tree-walking evaluator, `Context`, method dispatch |
| Runtime | `xdl-runtime` | lib | memory + GC scaffolding |
| Standard library | `xdl-stdlib` | lib | **37 k lines**; `lib.rs` is one giant `match` on the uppercased name |
| CLI + REPL | `xdl-cli` | bin `xdl` | `xdl file.xdl`, `-e`, `-i`, `parse`, `test` |
| C FFI | `xdl-ffi` | lib | `#[no_mangle]` embedding surface + GSL/HDF5/NetCDF wrappers |
| Desktop GUI | `xdl-gui` | bin | egui/eframe plot + image windows |
| 2D charts | `xdl-charts` | lib | ECharts HTML generation |
| Chart viewer | `xdl-chart-viewer` | Tauri app | interactive viewer; own `tauri.conf.json` |
| Desktop viewer | `xdl-desktop-viewer` | Tauri lib | **outside the workspace — nothing depends on it** |
| 3D viz | `xdl-viz3d`, `-web`, `-threejs` | lib | wgpu shaders, HTTP server, Three.js templates |
| GPU / SIMD | `xdl-amp` | lib | 13 backends: Metal, MPS, MLX, CoreML, CUDA, cuDNN, ROCm, Vulkan, OpenCL, DirectX, DirectML, ONNX, SIMD |
| DataFrames | `xdl-dataframe` | lib | Polars-backed |
| Database | `xdl-database` | lib | SQL access from XDL |
| Language server | `xdl-lsp` | bin | completion, hover, goto, symbols, semantic tokens, diagnostics |
| VS Code extension | `vscode-xdl` | TS + TextMate | `syntaxes/xdl.tmLanguage.json`, `src/extension.ts` |
| MATLAB transpiler | `xdl-matlab` | lib | `.m` → XDL; **not in `[workspace] members`**, auto-included as a path dep |
| Docs site | `docs/` | Jekyll | just-the-docs, published to GitHub Pages |

**`[workspace] members` lists 17 crates; `cargo metadata` reports 18.** Cargo
auto-includes `xdl-matlab` because `xdl-cli` and `xdl-gui` depend on it by path,
so `--workspace` *does* cover it — do not "fix" the members list on the
assumption that it doesn't.

**`xdl-desktop-viewer` is genuinely outside.** Nothing depends on it, so nothing
compiles it. It is a Tauri crate using `xdl-core`-adjacent types; after a core
type or AST change, check it by hand:

```bash
cargo check --manifest-path xdl-desktop-viewer/Cargo.toml
```

**`--all-features` is not free.** It enables `scientific-io` (netcdf + hdf5),
`gis` (proj), and `python` (PyO3), all of which need system libraries. On a
machine without them the build fails for reasons unrelated to your change. Check
the feature sets you actually touched:

```bash
cargo check --workspace                                  # default: dataframes + datascience
cargo check -p xdl-stdlib --features rustpython
cargo check -p xdl-stdlib --features scientific-io       # needs libnetcdf + libhdf5
```

---

## Change-Surface Cookbook — when I change X, I also need to touch Y

Every row below is a place where skipping a step produces **a silent capability
gap, not a build error.**

### Adding a built-in function

An XDL function is reachable only if every one of these agrees. The compiler
checks exactly one of them.

| # | File | What to add | Skipping it means |
|---|---|---|---|
| 1 | `xdl-stdlib/src/<module>.rs` | the implementation, `fn f(args: &[XdlValue]) -> XdlResult<XdlValue>` | — |
| 2 | `xdl-stdlib/src/lib.rs` → `call_function_with_keywords` | `"NAME" => module::f(args),` | **the function does not exist** at runtime |
| 3 | `xdl-lsp/src/symbols.rs` → `builtin_functions` | name, doc, return type, signature | no completion, no hover — invisible in every editor |
| 4 | `vscode-xdl/syntaxes/xdl.tmLanguage.json` | the name in the builtin-function pattern | unhighlighted, reads as a user identifier |
| 5 | `docs/` reference page | signature + example | undiscoverable |
| 6 | `xdl-stdlib/tests/` or `tests/*.xdl` | a test that calls it from XDL source | untested |

**The match arm is uppercased.** `call_function_with_keywords` matches on
`name.to_uppercase().as_str()`. **An arm whose literal is not fully uppercase is
dead code that never fires and never warns.** Grep before you commit:

```bash
grep -nE '^\s*"[^"]*[a-z][^"]*" =>' xdl-stdlib/src/lib.rs   # must print nothing
```

**Functions and procedures are separate registries.** `call_function*` returns a
value; `call_procedure*` is invoked statement-style (`PLOT, x, y`). Registering
in the wrong one gives "unknown function" at the call the user actually wrote.

**Keyword arguments arrive separately.** The `_with_keywords` variants take
`&HashMap<String, XdlValue>`; the bare `call_function` / `call_procedure` pass an
empty map. If your function honours keywords, it must be wired to the
`_with_keywords` arm, or the keywords silently vanish.

### Adding a language construct (new statement, operator, or literal)

```
xdl-parser/src/lexer.rs      new token
        ▼
xdl-parser/src/ast.rs        new Statement / Expression variant
        ▼
xdl-parser/src/parser.rs     the production that builds it
        ▼
xdl-interpreter/src/evaluator.rs   evaluation  ← compiler catches a missing arm
        ▼
xdl-lsp/src/semantic_tokens.rs     highlighting classification
xdl-lsp/src/completion.rs          keyword list
vscode-xdl/syntaxes/*.json         TextMate rule
        ▼
tests/*.xdl + docs/core/
```

Adding an AST **variant** is the one change in this repo the compiler helps with:
a non-exhaustive `match` in the evaluator is an error. **Do not add a `_ =>`
catch-all to an AST match** — it converts that help into a silent no-op.

`xdl-matlab` also pattern-matches the AST. It is a workspace member via the path
dependency, so `--workspace` covers its lib — but not `xdl-matlab/tests/`. Run
`cargo test -p xdl-matlab` after an AST change.

### Adding a plot / chart type

| Layer | File |
|---|---|
| Procedure entry | `xdl-stdlib/src/graphics_procs.rs` + arm in `lib.rs` `call_procedure_with_keywords` |
| Backend dispatch | `graphics_procs.rs` — `match backend { PlotBackend::XDLPlot => …, PlotBackend::ECharts => … }` |
| plotters impl | `xdl-stdlib/src/graphics/plot2d.rs` / `plot3d.rs` |
| ECharts impl | `xdl-charts/src/echarts.rs` + `templates.rs` |
| GUI window | `xdl-gui/src/plot_window.rs` (register via `register_gui_plot_callback`) |
| 3D | `xdl-viz3d*` |
| Docs | `docs/graphics/`, `docs/getting-started/graphics-quickstart.md`, a rendered PNG in `docs/examples_images/` |

**`SET_PLOT_BACKEND` is real and both branches must exist.** A new plot type that
implements only `XDLPlot` silently does nothing for a user on `ECHARTS`. If a
backend genuinely cannot render it, return an `XdlError` naming the backend — do
not fall through to a no-op or to the other backend.

The backend lives in `static PLOT_BACKEND: Mutex<PlotBackend>` — process-global.
See [Test Isolation](#test-isolation--shared-state-is-the-top-cause-of-flaky-here).

### Adding an `xdl-amp` GPU backend

`xdl-amp` has 13 backend modules behind `dispatch.rs`. A new one needs: the
module, the `Backend` enum variant, the `dispatch.rs` arm, a **capability probe
that fails closed** (an unavailable backend must report unavailable, never fall
back silently to SIMD while claiming GPU), the feature gate in `Cargo.toml`, and
a CPU-reference test asserting the GPU result matches within tolerance.

A GPU path that silently degrades to CPU is the fastest way to ship a "10× GPU
speedup" that measured the CPU. See [Verification](#verification--a-green-build-proves-nothing).

### Adding a feature-gated module

Current gates: `python` (PyO3), `rustpython`, `dataframes` (Polars), `ml` (Linfa).
A `#[cfg(feature = "…")]` module needs a **matching cfg on its dispatch arms in
`lib.rs`**, or the default build fails to compile the arm; and the *function must
still be nameable* with the feature off — return an error saying "XDL was built
without the `dataframes` feature", never "unknown function". The user cannot tell
those apart, and the second sends them hunting for a typo.

Always run `cargo check --workspace --all-features` **and** the default build.

### Version bump

Version currently disagrees across three files. Bump all of them together:

| File | Field |
|---|---|
| `Cargo.toml` | `[workspace.package] version` — every crate inherits via `version.workspace = true` |
| `xdl-chart-viewer/tauri.conf.json` | `version` |
| `vscode-xdl/package.json` | `version` |
| `CHANGELOG.md` | the entry |
| `docs/` | any quoted version in install/quickstart output |

Run `scripts/check-version-sync.sh` — it fails when they drift.

Do **not** bump historical statements ("added in 0.1.0") or minimum-version
claims in compatibility docs.

### Adding a CLI subcommand

`xdl-cli/src/main.rs` (`Commands` enum) → the handler → `xdl-cli/tests/cli_tests.rs`
→ `docs/getting-started/` → the README usage block. The REPL (`repl.rs`) has its
own command table; a subcommand that should also work interactively needs both.

---

## Rules for Agents

### DO

- **Write in a functional style and refactor toward it** — pure functions over
  immutable data, effects at the edges, total error handling. See
  [Functional Style & Safe Refactoring](#functional-style--safe-refactoring).
- **Use the language's own idiom**, not a pattern's name — `enum` + exhaustive
  `match`, iterator chains, `Drop`, newtypes. See
  [the idiom table](#reach-for-the-languages-idiom-not-the-patterns-name).
- **Return `XdlResult` and propagate with `?`.** `XdlError` already has the
  variants; `XdlErrorContext` (`xdl-core/src/error.rs`) turns an `Option` or a
  foreign `Result` into one with a message.
- **Walk the [Change-Surface Cookbook](#change-surface-cookbook--when-i-change-x-i-also-need-to-touch-y) row** for the kind of change you are making.
- **Preserve IDL/GDL semantics.** When Rust's natural behaviour and IDL's
  documented behaviour differ, IDL wins, and a comment says so. See
  [IDL Semantics](#idl-semantics--the-compatibility-contract).
- **Run the XDL programs, not only `cargo test`.** `tests/*.xdl` is where the
  language is actually exercised.
- **Measure before optimising, attribute from the call tree**, re-measure
  like-for-like, then confirm the plot still renders. See
  [Performance](#performance--measure-attribute-verify).
- **Explain cross-file changes with an ASCII diagram before prose.** See
  [Explaining Changes](#explaining-changes--diagrams-before-prose).
- **Say what you verified and what you did not** — which crates, which features,
  which backend, whether you looked at the rendered output.

### DO NOT

- **`.unwrap()` / `.expect()` / `panic!` on a value that can legitimately fail**
  in library, interpreter, or stdlib code. A panic inside a stdlib function kills
  the user's whole session and loses their workspace. There are **248 such sites
  in `xdl-stdlib` and 131 in `xdl-amp`** — do not add the next one, and remove one
  on the way past. Panics are for tests and provably-infallible invariants with a
  comment proving it.
- **Add a `_ =>` catch-all to a match on an AST node or a domain enum.** That
  match being exhaustive is the only automated help this codebase gets.
- **Register a function in `lib.rs` and stop there.** Steps 3–6 of the cookbook
  are what make it reachable by a user in an editor.
- **Write a lowercase or mixed-case literal in the dispatch match.** It is dead.
- **Substitute a plausible default for missing or failed numeric data** —
  `unwrap_or(0.0)` on a failed computation, silently returning `NaN` as if it were
  a result, a clamped index reported as a value. See
  [Numerical Honesty](#numerical-honesty--a-result-that-cannot-be-wrong-is-not-a-result).
- **Claim a change works because `cargo check` is clean.** It is the floor, and it
  does not even cover `xdl-matlab` or `xdl-desktop-viewer`. See
  [Verification](#verification--a-green-build-proves-nothing).
- **Optimise something you have not measured**, or report a speedup without
  saying whether the output is still correct.
- **Leave a new `.bak`, `_old`, or `_broken` file in the tree.** There are already
  four (`charting_procs_broken.rs`, `charting_procs.rs.bak`, `gui.rs.bak`,
  `lexer_old.rs`); they cost every reader a "which one is live?" detour. Delete
  what git already remembers.
- **Hard-code a colour in an HTML/chart template.** See
  [Design System](#design-system--mandatory-for-every-visual-surface).

---

## Functional Style & Safe Refactoring

An interpreter is the ideal shape for functional code: the core is a
transformation from source to AST to values, and only the edges (IO, plotting,
GPU, the REPL) have effects. Write new code that way, and leave what you touch a
little more functional than you found it — as long as the refactor is
behaviour-preserving and covered by a test.

**Guiding principle:** separate *computation* (pure, deterministic, testable)
from *effects* (file IO, drawing, GPU dispatch, the graphics state mutex). A
stdlib function that both computes an array and writes a PNG is two functions
wearing a trenchcoat — and only one of them can be unit-tested.

### The four rules, in priority order

1. **Pure by default.** Output depends only on input. Push IO, clocks, RNG, and
   mutation to the edges; keep the core a tree of pure transforms.
2. **Total functions.** Handle every case. Model absence and failure in the type
   — `XdlResult` / `Option`, never a sentinel value.
3. **Illegal states unrepresentable.** Encode invariants in types so the compiler
   rejects bad states before runtime.
4. **Immutability by default.** Reserve mutation for a locally-owned accumulator
   inside an otherwise-pure function.

The payoff is testability: a pure core can be tested without a window, a GPU, or
a temp file — which is why the hardest logic belongs there. In this repo that
means: **language semantics belong in a pure function; `main.rs`, `repl.rs`,
`graphics_procs.rs`, and the AMP backends are transport.**

**Where to stop being pure:** measured hot paths. An array kernel, the lexer's
inner loop, or a colormap fill may use indexed loops and reused buffers — with a
comment saying a profiler said so, and a link to the measurement. *Functional by
default; imperative where measured.*

### Reach for the language's idiom, not the pattern's name

Most GoF patterns are already a language feature here. Using the feature is
shorter, faster, and reads as native.

| Intent | Rust | TypeScript (`vscode-xdl`) |
|---|---|---|
| Sum type / visitor | `enum` + exhaustive `match` | discriminated union + `switch` with a `never` default |
| Strategy | trait object, or an `fn` value | a function parameter |
| Scope guard / RAII | `Drop` | `try…finally` |
| Builder | typestate builder, or `..Default::default()` | options object |
| Iterator pipeline | `Iterator` adapters | `map`/`filter`/`reduce`/`flatMap` |
| Newtype | `struct FileUnit(i32)` | branded type |
| Absence / failure | `Option` / `XdlResult` + `?` | `T \| undefined`, result-shaped union |
| Interpreter dispatch | `match` on an `enum`, not on a `String` | — |

**Refactor toward a pattern when the smell is there** — a growing `match` on a
type tag, a `bool` parameter that selects behaviour, construction logic sprawling
across call sites. **Never because the pattern is admired.** A `Factory` that
produces one type is indirection with no payer.

### Rust

- **Prefer iterator combinators over manual loops.** `map` / `filter` /
  `filter_map` / `fold` / `try_fold` / `collect` / `partition` express intent and
  eliminate the off-by-one and index-out-of-bounds classes entirely — which
  matters more here than in most codebases, because array indexing *is* the
  domain. Reach for a `for` loop only when the body has early-exit side effects
  that read worse as a chain.
- **Total error handling — no panics in library, interpreter, or stdlib paths.**
  Return `XdlResult<T>` and propagate with `?`. Replace `match` pyramids with
  `?`, `map_err`, `and_then`, `ok_or_else`, `unwrap_or_else`. Use
  `XdlErrorContext::context()` (`xdl-core/src/error.rs`) to attach a message to an
  `Option` or a foreign error in one call.
- **Immutable by default.** Start every binding as `let`; add `mut` only when the
  compiler forces it. Prefer building a new value (`Vec::from_iter`, struct update
  `..old`) over mutating in place.
- **Borrow, don't clone.** Take `&str` / `&[XdlValue]` / `&XdlArray` (or
  `impl AsRef<str>`, `Cow<'_, str>`) rather than owned `String` / `Vec<T>`. Share
  large arrays with `Arc` rather than deep-cloning. **Every `.clone()` of an
  `XdlValue` inside a loop is a refactor candidate** — an `XdlValue` can be a
  whole N-dimensional array, so a clone in an element loop is quadratic.
- **Return `impl Iterator` instead of allocating a `Vec`** when the caller just
  iterates. Avoid `.collect::<Vec<_>>()` immediately followed by another iteration.
- **Make illegal states unrepresentable.** Enums over `bool` flags and
  stringly-typed status; newtypes (`struct FileUnit(i32)`, `struct Lun(u8)`) over
  bare integers; `NonZeroUsize` where the domain forbids zero. Exhaustive `match`
  (no `_` on domain enums) so a new variant is a compile error.
- **Use the map APIs that avoid double lookups:** `entry(k).or_insert_with(..)`,
  `.retain(..)`. Replace an O(n²) nested-loop membership test with a `HashSet`
  built once — `WHERE`, `UNIQ`, and `SORT`-adjacent helpers are where this bites.
- **Parallelise independent, side-effect-free element work with `rayon`**
  (already a workspace dependency) where the array is large enough to pay for it —
  and measure, because `par_iter` on a 100-element array is slower than `iter`.
- **Never block a `Mutex` across an await or a long computation.** `PLOT_BACKEND`
  and `GRAPHICS_STATE` are process-global mutexes; read the value out and drop the
  guard before doing work.
- **`.lock().unwrap()` is a panic on poisoning.** One panicking thread makes every
  later access panic. Use `.lock().unwrap_or_else(|e| e.into_inner())`.

### Before you write a helper, check whether it already exists

The largest source of duplication here is a helper written a second time because
the first was in another crate.

| You need | Use | Never |
|---|---|---|
| An error with a message | `XdlError::*` + `XdlErrorContext::context` (`xdl-core/src/error.rs`) | a `String` error or `anyhow` inside stdlib |
| Array shape / rank logic | `xdl-core::dimension::Dimension` | recomputing strides inline |
| Element-type promotion | the promotion helpers in `xdl-core::types` | an ad-hoc `match` on the pair of types |
| A typed value out of `&[XdlValue]` | the argument-extraction helpers in the stdlib module you're in | `args[0].clone()` then `if let` |
| 2D/3D plotting | `xdl_stdlib::graphics::{plot2d, plot3d}` | new plotters code in a procedure wrapper |
| A colour or colour table | `xdl_stdlib::graphics::state::{Color, ColorTable}` | a hex literal |
| An ECharts option blob | `xdl-charts::echarts` | hand-built JSON in a `format!` |

If you genuinely need a new shared primitive, it belongs in `xdl-core` — it is
the only crate everything else depends on.

### TypeScript (`vscode-xdl`)

`const` over `let`; never mutate a value you did not create. `map`/`filter`/
`reduce` over loops. No `any` — use `unknown` plus a narrowing guard. Model
variants as discriminated unions and `switch` exhaustively with a `never`
default. Keep VS Code API calls (the effects) at the edge and the transforms
pure and unit-testable.

### Refactor triggers (safe, high-value — do these when you see them)

| Smell | Refactor | Wins |
|---|---|---|
| `.unwrap()`/`.expect()` off a fallible value in non-test code | `?` + `XdlResult`, or `ok_or_else` | safety — no session-killing panic |
| `.lock().unwrap()` | `.lock().unwrap_or_else(\|e\| e.into_inner())` | one panic no longer poisons every later call |
| `.clone()` on an `XdlValue` inside a loop | borrow, `Arc`, or `Cow` | speed — an `XdlValue` can be a whole array |
| Nested loop doing a membership test | build a `HashSet` once | O(n²) → O(n) |
| `let mut v = Vec::new(); for … { v.push(…) }` | `iter().map(…).collect()` / `filter_map` | clarity + safety |
| `match` pyramid on `Result`/`Option` | `?`, `map_err`, `and_then` | clarity |
| `bool` flag pair encoding a state | an enum with an exhaustive `match` | safety |
| `_ =>` on an AST or domain enum | list the variants | the compiler starts helping again |
| Byte-slicing a user string (`&s[..4]`) | operate on `.chars()` | safety — panics on any multi-byte input |
| `Vec::new()` then `push` a known number of times | `Vec::with_capacity(n)` / `collect` | one allocation instead of log₂n |
| Owned `String` / `Vec<T>` parameter that is only read | `&str` / `&[T]` / `impl AsRef<str>` | no copy at every call site |
| A helper you are about to write | check the table above first | less code, no drift |

### Allocation discipline

Apply this **after** cadence and structure — see [Performance](#performance--measure-attribute-verify).

**Do not allocate.** Reuse instead of recreating; hoist containers out of loops;
`with_capacity`/`reserve` when the size is known; `const`/`static` tables (colour
tables, function-name lists) over lazily-built ones.

**Do not copy.** View types at boundaries (`&str`, `&[T]`, `Cow<'_, str>`). Borrow;
clone when you need *ownership*, not to quiet the borrow checker. Move for
transfers; store indices instead of duplicating elements.

**Lay it out for the cache.** Flat over pointer-chasing. Order struct fields to
cut padding and group hot fields. This is an array language — a contiguous `Vec<f64>`
with computed strides beats a nested `Vec<Vec<f64>>` by more than any algorithmic
change you are likely to make.

**Do less work.** Fast path first, rare handling out of line. Precompute at
construction what would be recomputed per call. Hoist loop invariants. Cheap
reject before an expensive check.

#### A `Regex::new` outside a `LazyLock` is a hot-path mistake

The lexer, the diagnostics provider, and the MATLAB transpiler all run regexes
per line or per token. `Regex::new` compiles the pattern *every call*. Hoist into
a `static LazyLock<Regex>`. Before adding one:

```bash
grep -rn 'Regex::new' --include='*.rs' . | grep -v static | grep -v './target'
```

#### Never size an allocation from unvalidated input

This is a **security** rule, not a performance one, and it does not wait for a
measurement. XDL programs are user input, and so are the files they read.

```rust
// WRONG — a bogus header kills the process before a byte of data arrives.
let n: usize = header.parse()?;
let mut buf = vec![0u8; n];

// RIGHT — bound first.
if n == 0 || n > MAX_ELEMENTS {
    return Err(XdlError::RuntimeError(format!("invalid element count {n}")));
}
let mut buf = vec![0u8; n];
```

Applies to every length crossing a trust boundary: `MAKE_ARRAY` / `FLTARR`
dimensions from a script, a record count from a FITS/HDF5/NetCDF header, a
`READU` length, a size argument from the FFI surface, a buffer size from a GPU
kernel descriptor.

- **`as usize` is not validation.** A negative `i32` becomes a number near
  `usize::MAX`. Use `usize::try_from(n).map_err(…)?`.
- **Check before, not after.** Reading and *then* checking the length bounds the
  disk, not the memory — the OOM already happened.
- **Clamp a knob, reject a request.** A plot's DPI should `clamp`; an array
  dimension claiming 10¹⁵ elements should be an `XdlError`.
- **On stable Rust you cannot handle allocation failure.** `vec![0.0; n]` aborts
  — no `Err`, no unwinding, the process dies. **The bound on the input *is* the
  error handling.** Treat "where does this size come from?" as part of reviewing
  every `with_capacity` / `vec![_; n]`.

### Discipline for refactors

- **Behaviour-preserving only.** A speed or safety refactor must not change
  observable output — and in an IDL-compatible language, "observable output"
  includes the printed formatting of a number and the exact error text. If no
  test pins the behaviour, add one *first*, then refactor under it.
- **Let the hooks gate you.** Every `.rs` edit runs `cargo check` on the owning
  crate (see `.claude/settings.json`). A refactor is not done until it is clean —
  and `xdl-matlab` / `xdl-desktop-viewer` are checked separately.
- **One concern per commit.** Do not fold a style sweep into a feature change; it
  makes review and `git bisect` painful. Mechanical FP refactors get their own commit.
- **Do not refactor what you cannot test or measure.** For a "this is faster"
  claim on a kernel, prefer a `criterion` bench (`criterion` is already a
  workspace dependency) over intuition.

---

## IDL Semantics — the compatibility contract

XDL exists to run IDL and GDL code. Where Rust's natural behaviour and IDL's
documented behaviour differ, **IDL wins**, and the code says why. These are the
places the difference is easy to miss:

- **Names are case-insensitive.** The dispatch uppercases; so must anything else
  that compares a function, procedure, keyword, or variable name. A `HashMap`
  keyed by the raw source spelling is a bug waiting for a user who types `plot`.
- **Arrays are 0-based and column-major-ish in their index order.** Getting this
  wrong produces a transposed image that looks plausible. Pin it with a
  non-square test array — a 3×3 fixture proves nothing.
- **Type promotion follows IDL's table, not Rust's.** `BYTE + BYTE` wraps in IDL;
  integer division truncates; `!VALUES.F_NAN` is a real value a program can
  compute with. Do not "fix" one of these to match Rust.
- **`!`-prefixed system variables** are a namespace of their own
  (`xdl-lsp/src/symbols.rs` has the list). A new one must be added to the
  interpreter's context *and* to the LSP table, or editors will flag valid code.
- **Keyword arguments are named, optional, and order-independent**, and an absent
  keyword is not the same as a keyword set to zero. `/KEYWORD` sets it to 1.
- **Error messages are part of the surface.** A user porting IDL code greps them.
  Change one only deliberately.

When you cannot determine the IDL behaviour, say so and pick the conservative
option — an error — rather than inventing a semantic. A wrong-but-quiet answer in
a data-analysis language propagates into someone's published figure.

---

## Performance — measure, attribute, verify

**A green build proves almost nothing, and a good number proves less.** Every
performance change follows the same five steps, in order:

```
1. MEASURE      a number, not a hunch. CPU, memory, or wall-clock?
2. ATTRIBUTE    profile. Read the CALL TREE, not the leaf histogram.
3. FIX          the cause you attributed, not the symptom you noticed.
4. RE-MEASURE   same protocol, like-for-like.
5. VERIFY       the output is still correct — the numbers and the picture.
```

### Step 1 — measure before you form a theory

```bash
/usr/bin/time -l ./target/release/xdl script.xdl     # macOS: wall, user, peak RSS
hyperfine './target/release/xdl script.xdl'          # if installed
cargo bench                                          # criterion, for kernels
```

Always measure `--release`. `[profile.dev]` is `opt-level = 0`, so a debug
measurement of an array kernel is off by an order of magnitude and will send you
optimising the wrong thing.

### Step 2 — attribute, and read the call tree

| Surface | Symptom | Attribution |
|---|---|---|
| Interpreter / stdlib | wall-clock | `cargo flamegraph`, `samply` |
| macOS, any binary | RSS, CPU | `sample <pid> 10`, `vmmap --summary`, `heap <pid>` |
| Allocation churn | RSS growth | `dhat`, or a counting allocator in a bench target |
| GPU (`xdl-amp`) | throughput | the backend's own profiler (Metal System Trace, Nsight) |
| Chart viewer / web viz | frame time | DevTools Performance → call tree |

A flat "hottest functions" list is a list of **symptoms** — it says what is
expensive, never *who asked for it*. Walk down from the entry point.

### Step 3 — where the wins are, in yield order

1. **Interpretation overhead per element.** The single biggest structural cost in
   a tree-walking interpreter is doing per-element work through the generic value
   path. An array operation that dispatches on `XdlValue` once and then runs a
   typed loop beats one that dispatches per element — routinely by 10–100×. Look
   here first, always.
2. **Cloning `XdlValue`.** An `XdlValue` may own a whole array. A clone in an
   argument-marshalling path costs the array. `grep -n '\.clone()' ` in the
   evaluator and the stdlib entry points.
3. **Re-parsing and re-compiling.** A regex, a format string, or a colour table
   rebuilt per call. Hoist into `LazyLock`.
4. **Redundant conversions at the boundary.** `f32`↔`f64` and layout conversions
   between the array core, plotters, ECharts, and the GPU backends. Convert once.
5. **Only now:** algorithms, containers, allocation → [Allocation discipline](#allocation-discipline).

### Step 4 — re-measure like-for-like

Same script, same input size, same build profile, same warm-up, or you are
comparing runs rather than changes. Write the protocol down. If a number is
noisy across runs, quote the **stable extreme** and say the metric is noisy.

### Step 5 — verify the result

**A speedup that comes from a computation no longer happening is the easiest way
to ship a regression while celebrating it.** In this repo that failure mode is
specific and cheap to check:

- The array still has the same values — compare against the pre-change output,
  element-wise, not by eye.
- The plot still renders, and renders the same thing. Open the PNG.
- A GPU path still ran on the GPU and did not fall back to SIMD.
- NaN/Inf handling did not change — a fast path that skips the NaN check is
  faster and wrong.

### Knowing when to stop

Optimising something you have not measured as a problem is churn that reads as
progress. If you fix a latent issue because the fix is small, **say plainly that
it is not a measured win.** Record what you deliberately left alone, and why.

---

## Numerical Honesty — a result that cannot be wrong is not a result

These are correctness bugs no compiler, linter, or type check will catch, because
the code is valid and the output is plausible. In a data-analysis language they
are the *worst* class of bug: the output goes into someone's paper.

### A fallback default is a data-integrity bug

`unwrap_or(0.0)` on a failed computation reads like ordinary defensive coding. It
puts a real-looking number into an array, and everything downstream — a mean, a
fit, a plotted line — is then silently wrong with nothing on screen looking odd.
**A blank invites a question; a plausible value does not.**

**Ask what the default asserts.** `unwrap_or(0)` on a count asserts "none" —
usually true. `unwrap_or(0.0)` on a measurement asserts "the value is zero", which
is a claim about the world nobody checked. `unwrap_or_default()` on a config
struct asserts "the user configured nothing", which is a different statement from
"we failed to read the config".

Absent data stays absent: `NaN` where IDL uses `NaN`, an `XdlError` where the
operation genuinely failed, and **refuse to derive from it** rather than
propagating a fabricated number.

### NaN is a value, not an error channel

IDL programs compute with `NaN` deliberately. So:

- **Do not use `NaN` to signal failure.** A caller cannot distinguish "the data
  had a gap" from "the function broke". Failure is `XdlError`.
- **Do not silently drop `NaN`** from a reduction. `MEAN` over an array with a gap
  must either propagate `NaN` or be explicitly told to skip — never quietly skip
  and report a mean over a different population than the user thinks.
- **`NaN != NaN`.** A `HashMap`, a `sort`, a `dedup`, or an equality-based test
  over floats containing `NaN` does not behave the way the code reads. `sort_by`
  with `partial_cmp().unwrap()` **panics** on `NaN` — a live instance of the panic
  debt above.

### A clamp is not a value

A guard rail that silently substitutes its own bound converts "this is broken"
into "this has an opinion". A clamped array index, a truncated plot range, an
iteration cap in a fit, a saturated integer conversion — **if the bound binds,
say so**, in the return type and in what the user sees. A fit that stopped at
`max_iter` did not converge; reporting its last value as *the* result is a lie
with error bars.

**Detection habit:** when a number looks wrong, check whether it is *exactly* a
bound, a default, or a round multiple of another number on screen.

### Decorative parameters invite false confidence

**If a parameter can be removed without changing any output, it is not modelling
anything.** A keyword the procedure accepts and ignores is worse than an
unsupported keyword: the user believes it took effect. Either implement it or
reject it by name. When a tolerance or an iteration cap was picked from a handful
of examples, record that it is provisional rather than derived.

### Units and bases do not survive multiplication

Wherever two numbers from different sources meet in one expression, check they
share a basis — radians vs degrees, 0-based vs 1-based indices, bytes vs
elements, pixels vs data coordinates, `f32` vs `f64` precision. Prefer deriving
through a ratio the source also supplies, which fixes every affected case rather
than the one that was noticed. Two of three sources agreeing is not validation;
it is how the bug hid.

---

## Verification — a green build proves nothing

The PostToolUse hook runs `cargo check` on the crate you edited. **Treat a clean
hook as the floor, not as evidence.** Ranked by what each check actually catches:

| Check | What it catches |
|---|---|
| `cargo check` / build | signature mismatches — rarely the bug you shipped |
| `cargo clippy -D warnings` | real lint debt, but nothing about semantics |
| **run an `.xdl` program and read stderr** | dispatch gaps, wiring errors, panics |
| **look at the rendered plot** | wrong data, wrong axes, empty output |
| **reconcile one number against an external reference** | unit, promotion, and default bugs |
| **the feature sets you touched, plus `xdl-desktop-viewer`** | what the default `--workspace` build silently skipped |

### Unexercised code is unverified code

A stdlib function is only exercised when an XDL program **calls** it; an LSP
provider only when an editor **requests** it; a GPU backend only when the
hardware **is present**. Anything behind a feature flag you did not enable, or a
backend you do not have, is unverified no matter how green the build is.

**Say explicitly which surfaces you exercised and which you did not.** "Builds
clean, ran `tests/test_plot.xdl`, did not exercise the ECharts backend or any GPU
path" is a report. "Tests pass" is not.

### The minimum honest check for a stdlib change

```bash
cargo build --release -p xdl-cli
./target/release/xdl -e 'print, YOUR_FUNCTION(findgen(10))'
cargo test --workspace --no-fail-fast
```

If it plots, open the PNG. If it is feature-gated, build with the feature *and*
without it.

### Prove the artifact carries the change

Building is not shipping. A stale binary on `PATH`, an installed `xdl` from a
previous build, or a Tauri bundle with a staged copy of an old sidecar all look
exactly like a logic bug.

```bash
which xdl                                          # is this the one you built?
strings ./target/release/xdl | grep -c NEW_SYMBOL  # 0 means stale
```

### Reporting

State plainly what you ran, what you saw, and what you did not check. "Tests
pass" without naming the command is not a report. If a step was skipped, say so.

---

## Traps a Build Cannot Catch

### An affordance is a claim

A function name in the docs, a keyword in the completion list, or a menu entry in
the viewer asserts that the system does the thing. The gap between "the surface
offers it" and "the code does it" is where the worst surprises live.

**This repo's specific version:** the stdlib dispatch is a `match` on a `String`.
**Nothing fails when a name appears in the LSP table, the TextMate grammar, or
the docs but not in `lib.rs`** — the user gets completion for a function that does
not exist. And nothing fails in the other direction either: a function
implemented and dispatched but absent from `symbols.rs` is invisible to everyone
using an editor.

**Detection habit** — enumerate both sides and compare:

```bash
# names the interpreter can actually dispatch
grep -oE '^\s*"[A-Z0-9_]+" =>' xdl-stdlib/src/lib.rs | tr -d ' "=>' | sort -u > /tmp/dispatch.txt
# names the LSP advertises
grep -oE '^\s+"[A-Z0-9_]+",' xdl-lsp/src/symbols.rs | tr -d ' ",' | sort -u > /tmp/lsp.txt
comm -13 /tmp/dispatch.txt /tmp/lsp.txt   # advertised but not implemented — the bad direction
comm -23 /tmp/dispatch.txt /tmp/lsp.txt   # implemented but not discoverable
```

Run it after any stdlib change. `scripts/check-builtin-parity.sh` wraps it.

### Duplicated contracts drift silently

The list of built-in names exists in four places: `xdl-stdlib/src/lib.rs`,
`xdl-lsp/src/symbols.rs`, `vscode-xdl/syntaxes/xdl.tmLanguage.json`, and `docs/`.
Nothing fails when one gains an entry and another does not. That is exactly why
the [Cookbook](#change-surface-cookbook--when-i-change-x-i-also-need-to-touch-y)
exists. If you deliberately skip a row, write that down.

### Porting copies conventions, not just code

When a feature moves between backends (plotters → ECharts, CPU → GPU, XDL →
MATLAB transpiler), the *defaults* travel differently from the logic. A ported
plot can look right and be driven by a different default range, colour table, or
sample count — neither implementation wrong in isolation. **Diff the inputs, not
just the render:** run both backends on the same script and compare.

### A stale `.bak` is a fork

`charting_procs_broken.rs`, `charting_procs.rs.bak`, `gui.rs.bak`, `lexer_old.rs`.
Each one is a plausible-looking file that a search will hit and a reader may
edit. Grep results from a dead file cost real debugging time. Delete them; git
remembers.

### "An error appeared after my change" ≠ "my change caused it"

Check provenance before assuming causation, and say which it was. A pre-existing
bug surfaced by your change is worth fixing and worth *labelling* as
pre-existing — otherwise the next reader learns the wrong lesson.

---

## Answer Style — dense, compact, caveman

Default output register for every agent working in this repo. Applies to chat
replies, status updates, and findings — **not** to code, code comments, commit
messages, or `docs/` prose, which stay in full prose.

- Very short sentences.
- Drop filler and pleasantries. No "I'll now…", "Great question", "Let me…".
- Core keywords and meaningful words only.
- Symbols over words: `→` (leads to / then), `=` (is), `<` `>` `≥`.
- No long explanation unless asked.
- Tables and lists over paragraphs.

```
bad:   I went ahead and added the function to the standard library, and it turns
       out the editor wasn't picking it up because of the symbol table.
good:  added SMOOTH → stdlib dispatch. not in lsp symbols.rs → no completion.
```

**Dense ≠ vague, and short never buys itself accuracy.** Every rule in
[Numerical Honesty](#numerical-honesty--a-result-that-cannot-be-wrong-is-not-a-result)
and [Verification](#verification--a-green-build-proves-nothing) still binds.
Keep: measured numbers, "unverified" labels, and what was *not* checked. Cut
words, never caveats — `ran tests/test_plot.xdl. echarts backend untested.` is
compact *and* honest. `works now` is neither.

---

## Explaining Changes — diagrams before prose

When a change crosses file boundaries, adds a dispatch layer, or shifts how a
value flows between crates, **lead with an ASCII diagram, not a paragraph.**
Diagrams make invariants visible that prose hides: who calls whom, where state
lives, which boxes are new.

### When to draw one

- A new value flow (source → lexer → AST → evaluator → stdlib → backend)
- A new module that adds dispatch (a backend, a plot type, a provider of any kind)
- A cross-crate contract (an `xdl-core` type used by `xdl-desktop-viewer`, which nothing compiles)
- Anything touching the [Product Matrix](#product-matrix--know-every-surface-before-you-change-code)
  or the [Cookbook](#change-surface-cookbook--when-i-change-x-i-also-need-to-touch-y)

### How to draw one

- ASCII box-drawing renders cleanly in terminals and GitHub markdown.
- Label boxes with the **file path or module name** — a reader should be able to
  grep the label.
- Show flow direction at every hop (`→`, `▼`).
- Add a short legend when boxes differ in kind (new vs existing, sync vs async).
- Follow it with a concrete walk-through of one call — do not make the reviewer
  simulate it mentally.
- Two small focused diagrams beat one that shows everything.

Prose alone is fine for single-file edits and scoped bug fixes. The rule kicks in
when a reviewer needs to know **where something lives** or **how a value travels**.

---

## Codebase Layout

```
xdl-core/src/
├── types.rs        ← XdlValue, type promotion
├── array.rs        ← XdlArray
├── dimension.rs    ← Dimension, strides, rank
└── error.rs        ← XdlError, XdlResult, XdlErrorContext

xdl-parser/src/
├── lexer.rs        ← tokens          (lexer_old.rs is dead — delete)
├── ast.rs          ← Statement, Expression
└── parser.rs       ← productions

xdl-interpreter/src/
├── evaluator.rs    ← the tree-walk; dispatches into xdl-stdlib
├── context.rs      ← variables, system variables, scopes
└── methods.rs      ← object/struct method dispatch

xdl-stdlib/src/
├── lib.rs          ← THE DISPATCH: call_function*/call_procedure* — 754 arms
├── math.rs array.rs statistics.rs string.rs io.rs linalg.rs signal.rs image.rs …
├── graphics/       ← plot2d.rs, plot3d.rs, state.rs (Color, ColorTable, GRAPHICS_STATE)
├── graphics_procs.rs ← procedure wrappers + PlotBackend dispatch
└── charting_procs.rs ← ECharts procedures   (…_broken.rs, …rs.bak are dead — delete)

xdl-lsp/src/
├── symbols.rs      ← builtin_functions + system_variables tables  ← keep in sync with stdlib
├── completion.rs hover.rs goto.rs semantic_tokens.rs diagnostics.rs

xdl-amp/src/
├── dispatch.rs     ← backend selection
└── metal.rs mps.rs mlx.rs coreml.rs cuda.rs cudnn.rs rocm.rs vulkan.rs opencl.rs
    directx.rs directml.rs onnx.rs simd_ops.rs

vscode-xdl/
├── syntaxes/xdl.tmLanguage.json  ← keep builtin list in sync with stdlib
└── src/extension.ts              ← launches xdl-lsp

xdl-matlab/          ← .m → XDL transpiler; pattern-matches xdl-parser's AST.
                       Not in [workspace] members, but auto-included as a path
                       dependency of xdl-cli and xdl-gui, so --workspace covers it.

Outside the workspace — nothing depends on it, nothing compiles it:
xdl-desktop-viewer/  ← check with --manifest-path after a core-type change
```

---

## Design System — mandatory for every visual surface

XDL renders the same data through **four independent backends**: plotters (PNG,
`xdl-stdlib/src/graphics/`), ECharts (HTML, `xdl-charts/`), Three.js / WebGPU
(HTML, `xdl-viz3d-*`), and egui (native, `xdl-gui/`). The shared contract lives in
**[`design-system/`](./design-system/README.md)** and is not optional.

| File | Read when |
|---|---|
| [`design-system/README.md`](./design-system/README.md) | Always — the ten rules and the per-backend table |
| [`design-system/foundations/color.md`](./design-system/foundations/color.md) | Any colour decision, and every colormap change |
| [`design-system/foundations/layout.md`](./design-system/foundations/layout.md) | Type, spacing, figure proportions, axes, legends |
| [`design-system/components/figure.md`](./design-system/components/figure.md) | Building any HTML chart or viewer page |
| [`design-system/tokens.css`](./design-system/tokens.css) | Looking up a CSS custom property |

### The part that is a correctness bug, not a style bug

**In a scientific plotting tool a colour is a value.** Two backends disagreeing
about a colormap are giving two answers to the same question. XDL currently
defines colormaps in three unrelated places:

| Where | Keyed by | Unknown request → |
|---|---|---|
| `xdl-stdlib/src/graphics/state.rs` `ColorTable::load_table` | numbers 0, 1, 2, 3, 13 | **grayscale, silently** |
| `xdl-viz3d-threejs/src/colormaps.rs` `generate_colormap` | names `VIRIDIS PLASMA INFERNO TURBO RAINBOW GRAYSCALE` | **viridis, silently** |
| `xdl-charts/src/echarts.rs` | nothing — an 11-stop ramp hard-coded inline | n/a |

So `LOADCT, 5` produces a grayscale figure the user believes is table 5 — the
[fallback-default data-integrity bug](#a-fallback-default-is-a-data-integrity-bug)
exactly. **Do not add a fourth registry, and do not add another silent fallback:**
an unknown table number or name is an `XdlError` naming what was asked for.

### Hard rules

1. **Never write a hex colour** in a template, chart option, or shader. UI colour
   comes from `tokens.css`; data colour comes from a colormap. **34 hard-coded hex
   literals** remain across `xdl-charts/src/echarts.rs` (23),
   `xdl-chart-viewer/src/main.rs` (6), `xdl-viz3d-web/src/template.rs` (4), and
   `xdl-viz3d-threejs/src/templates.rs` (1). Replace one when you touch its file.
2. **UI colour adapts to the theme; data colour never does.** A reader must get
   the same value off a colourbar in light and dark mode.
3. **Match the colormap family to the data shape** — sequential, diverging
   (centred on the meaningful zero, not the range midpoint), cyclic, categorical.
   Perceptually uniform by default; rainbow stays for IDL compatibility and is
   never the default for anything new.
4. **Every figure carries labelled axes with units and, where colour encodes a
   value, a colourbar.** `(dimensionless)` is a unit; a blank is unfinished.
5. **Square data pixels for images.** A stretched `TVSCL` is a wrong figure, the
   same class of error as a wrong colormap.
6. **Spacing is `--space-1 … --space-8`**; interactive elements are
   `<button>`/`<a>`; icons are inline SVG with `currentColor` — no emoji, no
   `▶ ▼ ×`.
7. **Every HTML surface honours `prefers-color-scheme`**, with `body` painting an
   explicit background.
8. **`tokens.css` and the Rust palette constants are two copies of one palette.**
   A PNG has no CSS, so plotters and egui read constants. Change both together —
   same [duplicated-contract trap](#duplicated-contracts-drift-silently) as the
   built-in name lists.

### Before claiming a visual change is done

Render it. Open the PNG or the HTML. Check light **and** dark. Use a
**non-square** input so a transposed axis shows up. Resize, and confirm the data
kept its aspect ratio. Say what you looked at.

---

## Testing

Three layers, and the middle one is where this repo's real coverage lives:

| Layer | Where | Runs with |
|---|---|---|
| Rust unit tests | `#[test]` in each crate | `cargo test --workspace --no-fail-fast` |
| **XDL language tests** | `tests/*.xdl`, `tests/stdlib/` | `./target/release/xdl tests/<file>.xdl`, `tests/test_all.sh` |
| Integration | `xdl-cli/tests/`, `xdl-stdlib/tests/`, `xdl-chart-viewer/tests/` | `cargo test` |

There are ~284 `#[test]` functions against 37 000 lines of stdlib. **Assume a
stdlib function is untested until you have seen its test.**

### Test the joins, not just the parts

**The bugs that reach users here are not inside functions. They are between
them** — on seams where two sides agree by convention and nothing verifies it:

| Join | Held together by | What `rustc` sees |
|---|---|---|
| XDL source → `lib.rs` dispatch | an uppercased string literal | nothing — a typo is "unknown function" at runtime |
| dispatch → `xdl-lsp/symbols.rs` | two hand-maintained lists | nothing |
| dispatch → `tmLanguage.json` | a third hand-maintained list | nothing |
| `xdl-core` types → `xdl-desktop-viewer` | a crate nothing depends on | nothing — it is never compiled |
| plot procedure → `PlotBackend` arm | a `match` the author may have half-filled | nothing if a `_ =>` exists |

A unit test written from inside a function reproduces that function's
assumptions. So when a change crosses a boundary, **write the test at the
boundary**: an `.xdl` script that calls the function the way a user would, and an
assertion on the value it prints — not on the Rust function's return.

Pick fixture values no fallback could coincide with. If the failure mode returns
`0.0`, do not test with data whose answer is `0.0`.

### A test that has never failed is a claim, not a check

**Before a test counts as done, watch it fail.** Revert the fix, or mutate the
thing it guards, and confirm it goes red *for the right reason*. State it in the
commit: what you reverted, and what the test said when it broke. "Verified to
fail before passing" is evidence; "added tests" is not.

### `cargo test` stops at the first failing binary

A single failure hides every later target, so a suite can look like "1 failure"
while carrying a dozen. **Always `cargo test --workspace --no-fail-fast`** before
believing a count.

---

## Test Isolation — shared state is the top cause of "flaky" here

Cargo runs tests on parallel threads in one process. Anything process-global is
shared between them.

| Shared thing | Symptom | Rule |
|---|---|---|
| `PLOT_BACKEND` (`static Mutex<PlotBackend>`) | a *different* plot test fails each run | Never call `SET_PLOT_BACKEND` in a test without serialising. Prefer a pure function taking the backend as a parameter |
| `GRAPHICS_STATE` (`static`, `graphics/state.rs`) | colour/window state leaks between tests | same — serialise with a poison-tolerant `static LOCK: Mutex<()>` |
| `std::env::set_var` | a different test fails each run | prefer a function taking the value; otherwise serialise |
| Fixed `/tmp` or CWD-relative output paths | passes alone, fails in a full run — collides **across processes** | `tempfile::TempDir`, or suffix with `std::process::id()` |
| Files written into the repo root | `*.png` and `*.csv` appear in `git status` after a test run | write into a `TempDir` |
| macOS symlinked temp dirs | macOS-only failures | canonicalise — `/var` → `/private/var`, so OS-reported paths never equal `dir.join(..)` |
| A poisoned `Mutex` | one panic makes every later test panic | `.lock().unwrap_or_else(\|e\| e.into_inner())` |

Note the current working tree already carries generated `.csv` and `.png` files
committed next to source (`xdl-dataframe/`). That is this failure mode, already
shipped.

---

## The Craft Checklist

Short enough to actually run. Copy the relevant block into the PR description.

**Before optimising**
- [ ] A measurement, not a hunch — and taken on a `--release` build
- [ ] Attributed from the **call tree**, not a leaf list
- [ ] Baseline recorded, with the exact protocol written down

**While optimising**
- [ ] Per-element interpretation overhead and `XdlValue` cloning checked *before* micro-work
- [ ] Nothing "fixed" that was not measured as a problem

**After optimising**
- [ ] Re-measured like-for-like, same script, same profile
- [ ] **The output is still correct** — values compared element-wise, plot opened
- [ ] Noisy numbers reported at their stable extreme, with the noise stated

**Before calling a change shipped**
- [ ] `cargo test --workspace --no-fail-fast`
- [ ] `cargo check --workspace` (and the specific feature sets you touched)
- [ ] `cargo check --manifest-path xdl-desktop-viewer/Cargo.toml` if core types moved
- [ ] Ran a real `.xdl` program through `./target/release/xdl` and read stderr
- [ ] Walked the [Cookbook](#change-surface-cookbook--when-i-change-x-i-also-need-to-touch-y) row for this change type
- [ ] `scripts/check-builtin-parity.sh` clean (stdlib ↔ LSP ↔ grammar)
- [ ] Reconciled at least one number against an external reference (IDL docs, NumPy, a hand calculation)
- [ ] Opened the rendered plot if anything visual changed

**Recording it**
- [ ] The number before and after
- [ ] The cause, not just the change
- [ ] What you deliberately left alone, and why
- [ ] What is still **untested**, marked as such
