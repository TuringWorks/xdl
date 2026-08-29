# XDL — Claude Code Guidelines

See **[AGENTS.md](./AGENTS.md)** for the full product matrix, change-surface
cookbook, functional-style rules, and verification playbook that apply to all AI
coding agents. Pay special attention to the
[Change-Surface Cookbook](./AGENTS.md#change-surface-cookbook--when-i-change-x-i-also-need-to-touch-y)
— XDL's built-in dispatch is a 754-arm `match` on a string, and **almost every
way to break it produces no compile error.**

See **[SOUL.md](./SOUL.md)** for what XDL is for and why the compatibility
contract is not negotiable.

---

## Answer Style — dense, compact, caveman

Chat replies, status updates, findings. **Not** code, comments, commit messages,
or `docs/` — those stay full prose.

- Very short sentences. No filler, no pleasantries.
- Core keywords only. Symbols over words: `→`, `=`, `≥`.
- No long explanation unless asked. Tables > paragraphs.

Cut words, never caveats. Measured numbers and "unverified" labels survive
compression — `added SMOOTH → dispatch + lsp. echarts backend untested.` is
compact and honest; `works now` is neither.

Full rule: [AGENTS.md → Answer Style](./AGENTS.md#answer-style--dense-compact-caveman).

---

## Quick Reference

### Build and run

```bash
cargo build --release -p xdl-cli          # the `xdl` binary
./target/release/xdl script.xdl           # run a program
./target/release/xdl -e 'print, findgen(5)'
./target/release/xdl -i                   # REPL

cargo check --workspace                   # what the edit hook runs (18 crates)
cargo test --workspace --no-fail-fast     # ALWAYS --no-fail-fast
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

### Workspace membership is not what `Cargo.toml` says

`[workspace] members` lists 17 crates; `cargo metadata` reports **18**. Cargo
auto-includes `xdl-matlab` because `xdl-cli` and `xdl-gui` depend on it by path,
so `--workspace` covers its lib. It does not cover `xdl-matlab/tests/` — run
`cargo test -p xdl-matlab` after an AST change.

**`xdl-desktop-viewer` is genuinely outside**: nothing depends on it, so nothing
compiles it. After a core-type or AST change:

```bash
cargo check --manifest-path xdl-desktop-viewer/Cargo.toml
```

**`--all-features` is not free.** It pulls in `scientific-io` (netcdf + hdf5),
`gis` (proj), and `python` (PyO3) — system libraries that may not be installed.
Check the feature sets you actually touched instead of reaching for
`--all-features` and reading its failure as your bug.

### Adding a built-in function — six files, one compiler check

| # | File | Skipping it means |
|---|---|---|
| 1 | `xdl-stdlib/src/<module>.rs` — the implementation | — |
| 2 | `xdl-stdlib/src/lib.rs` — `"NAME" => module::f(args),` | **the function does not exist** |
| 3 | `xdl-lsp/src/symbols.rs` — `builtin_functions` entry | no completion, no hover |
| 4 | `vscode-xdl/syntaxes/xdl.tmLanguage.json` | unhighlighted |
| 5 | `docs/` reference page | undiscoverable |
| 6 | `tests/*.xdl` calling it from XDL source | untested |

Only #2 is checked by anything, and only for type errors. **The match literal
must be fully uppercase** — dispatch runs `name.to_uppercase()`, so a lowercase
arm is dead code that never warns:

```bash
grep -nE '^\s*"[^"]*[a-z][^"]*" =>' xdl-stdlib/src/lib.rs   # must print nothing
scripts/check-builtin-parity.sh                             # stdlib ↔ LSP ↔ grammar
```

**Functions and procedures are separate registries.** `call_function*` returns a
value; `call_procedure*` is statement-style (`PLOT, x, y`). Wrong registry =
"unknown function" at the call the user actually wrote. Keywords only reach a
function wired to the `_with_keywords` arm.

Use the **`add-builtin` skill** (`.claude/skills/add-builtin/SKILL.md`) — it has
the ordered walk-through and the parity checks.

### Functional style & idioms — Rust and TypeScript

Write new code and refactor existing code toward a functional style: **pure
functions over immutable data, effects pushed to the edges, total error
handling.** An interpreter is the ideal shape for it — the core is
source → AST → values, and only the edges (IO, plotting, GPU, REPL) have effects.

- **Rust**: iterator combinators (`map`/`filter_map`/`fold`/`collect`) over index
  loops; `let` (not `let mut`) by default; `?` + `map_err`/`ok_or_else` over
  `match` pyramids; **no `.unwrap()`/`.expect()`/`panic!` in library, interpreter,
  or stdlib paths** — a panic there kills the user's whole session; borrow /
  `Arc` / `Cow` instead of `.clone()` on an `XdlValue` (it can own a whole array);
  enums + exhaustive `match` over `bool` flags and stringly-typed status;
  `rayon` for large side-effect-free element work, measured.
- **Never `_ =>` on a match over an AST node or a domain enum.** That match being
  exhaustive is the only automated help this codebase gets when a variant is added.
- **`.lock().unwrap()` is a panic on poisoning** — one panicking thread makes
  every later access panic. Use `.lock().unwrap_or_else(|e| e.into_inner())`.
- **TypeScript (`vscode-xdl`)**: `const` and immutable updates; `map`/`filter`/
  `reduce` over loops; discriminated unions with a `never`-exhaustive switch;
  `unknown` + narrowing, never `any`.
- **Use the language's idiom, not the pattern's name.** Most GoF patterns are a
  language feature here: sum type → `enum` + `match`; strategy → a function value;
  RAII → `Drop`; newtype → `struct FileUnit(i32)`. Refactor toward a pattern when
  the smell is there (a growing type-tag match, a boolean parameter selecting
  behaviour), never because the pattern is admired.
- **Check for an existing helper before writing one.** Errors → `XdlError` +
  `XdlErrorContext::context` (`xdl-core/src/error.rs`); shape/stride logic →
  `xdl_core::dimension::Dimension`; type promotion → `xdl_core::types`; 2D/3D
  plotting → `xdl_stdlib::graphics::{plot2d, plot3d}`; colours →
  `graphics::state::{Color, ColorTable}`; ECharts options → `xdl_charts::echarts`.
- **Refactors must be behaviour-preserving and test-covered.** In an
  IDL-compatible language "observable output" includes the printed formatting of a
  number and the exact error text. Pin behaviour with a test first, then refactor.
  Keep style sweeps in their own commit.

Full guidance, idiom table, refactor triggers and shared-helper table:
[AGENTS.md → Functional Style & Safe Refactoring](./AGENTS.md#functional-style--safe-refactoring).

### Panic debt — where it is

**248 `.unwrap()`/`.expect()`/`panic!` sites in `xdl-stdlib`, 131 in `xdl-amp`**,
plus 34 in `xdl-core`, 29 in `xdl-interpreter`, 20 in `xdl-parser`. A panic inside
a stdlib function kills the user's session and loses their workspace. Do not add
the next one; remove one on the way past. The highest-value class to fix first is
`sort_by(|a, b| a.partial_cmp(b).unwrap())` — it panics on any `NaN`, which is a
value IDL programs compute with deliberately.

### IDL semantics — the compatibility contract

Where Rust's natural behaviour and IDL's documented behaviour differ, **IDL
wins**, with a comment saying so.

- Names are **case-insensitive** — anything keyed by the source spelling is a bug.
- Arrays are **0-based**; index order matters. Pin it with a non-square fixture —
  a 3×3 test array proves nothing about a transpose.
- **Type promotion follows IDL's table**, not Rust's. `BYTE` arithmetic wraps;
  integer division truncates. Do not "fix" these toward Rust.
- **`NaN` is a value, not an error channel.** Failure is `XdlError`. Do not
  silently drop `NaN` from a reduction — propagate it or be told to skip.
- Keyword arguments are named, optional, order-independent; `/KEY` sets it to 1;
  absent ≠ zero.
- **Error message text is part of the surface** — users porting IDL code grep it.

Full rules: [AGENTS.md → IDL Semantics](./AGENTS.md#idl-semantics--the-compatibility-contract).

### Numerical honesty, performance, verification — what a green build misses

**A green build proves almost nothing, and a good number proves less.**

1. **Never substitute a plausible default for missing or failed numeric data.**
   `unwrap_or(0.0)` on a failed computation puts a real-looking number into an
   array, and the mean, the fit, and the plotted line are then silently wrong with
   nothing on screen looking odd. A blank invites a question; a plausible value
   does not. A clamp that binds — a truncated range, a fit that stopped at
   `max_iter` — must say so, not report its bound as the answer.
2. **Measure → attribute from the call tree → fix → re-measure like-for-like →
   verify the output is still correct.** Always on `--release` (`[profile.dev]` is
   `opt-level = 0`). Win order: per-element interpretation overhead → `XdlValue`
   cloning → re-compiled regexes and rebuilt tables → boundary conversions → only
   then algorithms and allocation. A speedup from a computation no longer
   happening is the easiest regression to celebrate.
3. **`cargo check` clean is the floor, not evidence.** Run a real `.xdl` program,
   read stderr, and open the rendered plot. Say which surfaces you exercised and
   which you did not — feature-gated modules and GPU backends you do not have are
   unverified no matter how green the build.
4. **Never size an allocation from unvalidated input, and check *before*
   allocating.** `MAKE_ARRAY` dimensions, FITS/HDF5 record counts, `READU` lengths
   and FFI size arguments all cross a trust boundary. On stable Rust an allocation
   failure **aborts** — there is no `Err` to handle, so the input bound *is* the
   error handling. `as usize` is not validation: a negative `i32` becomes
   ~`usize::MAX`.

Full playbooks: [Numerical Honesty](./AGENTS.md#numerical-honesty--a-result-that-cannot-be-wrong-is-not-a-result) ·
[Performance](./AGENTS.md#performance--measure-attribute-verify) ·
[Verification](./AGENTS.md#verification--a-green-build-proves-nothing) ·
[Traps a Build Cannot Catch](./AGENTS.md#traps-a-build-cannot-catch) ·
[The Craft Checklist](./AGENTS.md#the-craft-checklist).

### The dispatch/LSP/grammar triple drifts silently

The list of built-in names lives in four places and nothing fails when one gains
an entry and another does not: `xdl-stdlib/src/lib.rs` (what runs),
`xdl-lsp/src/symbols.rs` (what editors advertise),
`vscode-xdl/syntaxes/xdl.tmLanguage.json` (what is highlighted), and `docs/`.

Advertised-but-not-implemented is the bad direction — the user gets completion
for a function that does not exist.

```bash
scripts/check-builtin-parity.sh    # prints both directions of the drift
```

### Plot backends — both branches must exist

`SET_PLOT_BACKEND` selects `PlotBackend::XDLPlot` (plotters → PNG) or
`PlotBackend::ECharts` (HTML). A new plot type that implements only one silently
does nothing for users on the other. If a backend genuinely cannot render it,
return an `XdlError` naming the backend — never a no-op.

`PLOT_BACKEND` and `GRAPHICS_STATE` are process-global mutexes. See
[Test Isolation](./AGENTS.md#test-isolation--shared-state-is-the-top-cause-of-flaky-here).

### Test isolation — and `--no-fail-fast`

Cargo runs tests on parallel threads in one process. Every "flaky" test of this
shape is shared state, not timing.

- **`PLOT_BACKEND` / `GRAPHICS_STATE`**: never mutate in a test without
  serialising through a poison-tolerant `static LOCK: Mutex<()>`. Prefer a pure
  function that takes the backend as a parameter.
- **Output files**: write into a `tempfile::TempDir`, never the repo root or a
  fixed `/tmp` path — fixed paths collide *across processes*.
- **macOS symlinks**: `/var` → `/private/var`, so canonicalise before comparing
  OS-reported paths.

**`cargo test` stops at the first failing binary** — always run
`cargo test --workspace --no-fail-fast` before trusting a failure count.

### Visual changes — the design system is mandatory

Four backends render the same data: plotters (PNG), ECharts (HTML), Three.js
(HTML), egui (native). The shared contract is
**[`design-system/`](./design-system/README.md)**.

**In a plotting tool a colour is a value**, so colour drift is a correctness bug.
XDL has **three unrelated colormap registries** today —
`graphics/state.rs::load_table` (numbers, falls back to grayscale silently),
`xdl-viz3d-threejs/colormaps.rs` (names, falls back to viridis silently), and an
11-stop ramp hard-coded in `xdl-charts/echarts.rs`. `LOADCT, 5` therefore
produces a grayscale figure the user believes is table 5. **Do not add a fourth
registry; do not add another silent fallback** — an unknown colormap is an
`XdlError` naming what was asked for.

- Never a hex literal in a template; **34 remain** across `echarts.rs` (23),
  `xdl-chart-viewer/src/main.rs` (6), `viz3d-web/template.rs` (4),
  `viz3d-threejs/templates.rs` (1). Replace one when you touch its file.
- UI colour adapts to the theme; **data colour never does**.
- Colormap family must match the data shape; diverging maps centre on the
  meaningful zero, not the range midpoint.
- Every figure: labelled axes **with units**, and a colourbar wherever colour
  encodes a value. Square data pixels for images — a stretched `TVSCL` is a wrong
  figure.
- Before calling it done: render it, open it, check both themes, use a
  **non-square** input so a transpose shows up.

Full rules: [AGENTS.md → Design System](./AGENTS.md#design-system--mandatory-for-every-visual-surface) ·
[design-system/foundations/color.md](./design-system/foundations/color.md).

---

## Cross-cutting change checklist (quick — full list in AGENTS.md)

| Type of change | Surfaces to touch |
|---|---|
| New built-in function/procedure | `xdl-stdlib/src/<mod>.rs` → dispatch arm in `lib.rs` → `xdl-lsp/src/symbols.rs` → `vscode-xdl/syntaxes/xdl.tmLanguage.json` → `docs/` → `tests/*.xdl` |
| New language construct | `xdl-parser/src/lexer.rs` → `ast.rs` → `parser.rs` → `xdl-interpreter/src/evaluator.rs` → `xdl-lsp/{semantic_tokens,completion}.rs` → TextMate grammar → `xdl-matlab` (outside the workspace) → `tests/*.xdl` + `docs/core/` |
| New plot / chart type | `graphics_procs.rs` (+ dispatch arm) → **both** `PlotBackend` branches → `graphics/plot2d.rs`\|`plot3d.rs` → `xdl-charts/src/echarts.rs` → `xdl-gui/src/plot_window.rs` → `docs/graphics/` + a rendered PNG in `docs/examples_images/` |
| New `xdl-amp` GPU backend | module → `Backend` variant → `dispatch.rs` arm → a capability probe that **fails closed** → feature gate → a CPU-reference test within tolerance |
| New feature-gated module | `Cargo.toml` feature → `#[cfg]` on the module **and** its dispatch arms → an error naming the missing feature (never "unknown function") → check the build **with and without** the feature |
| New CLI subcommand | `xdl-cli/src/main.rs` `Commands` → handler → `repl.rs` if it should work interactively → `xdl-cli/tests/cli_tests.rs` → `docs/getting-started/` → README |
| Version bump | `Cargo.toml` `[workspace.package]` → `xdl-chart-viewer/tauri.conf.json` → `vscode-xdl/package.json` → `CHANGELOG.md` → `docs/`. Run `scripts/check-version-sync.sh` |
| Core type change (`XdlValue`, `XdlError`, AST) | the workspace, **plus** `cargo test -p xdl-matlab` (its tests are not in `--workspace --all-targets`) and `cargo check --manifest-path xdl-desktop-viewer/Cargo.toml` |

---

## Known repo-level defects — fix on the way past, don't re-derive

These are measured, not guessed. Each has a check that stops it getting worse.

| Defect | Where | Ratcheted by |
|---|---|---|
| **License stated three ways** — needs a maintainer decision, not a script | `Cargo.toml` says `GPL-2.0`; `LICENSE` and the README badge say MIT | `scripts/check-version-sync.sh` (warns) |
| **Panic debt** — a panic here kills the user's session | 248 `xdl-stdlib`, 131 `xdl-amp`, 34 `xdl-core`, 29 `xdl-interpreter`, 20 `xdl-parser`, 14 `xdl-dataframe` | `.github/workflows/security.yml` (counts may fall, never rise) |
| **Three disagreeing colormap registries**, two with silent fallbacks — `LOADCT, 5` returns grayscale | `graphics/state.rs`, `viz3d-threejs/colormaps.rs`, `charts/echarts.rs` | [design-system/foundations/color.md](./design-system/foundations/color.md) |
| **34 hard-coded hex colours** | `echarts.rs` 23, `chart-viewer/main.rs` 6, `viz3d-web/template.rs` 4, `viz3d-threejs/templates.rs` 1 | design-system rule 1 |
| **3 built-ins advertised by the LSP but not dispatched** | `READ_ASCII`, `READ_BINARY`, `READ_CSV` | `scripts/check-builtin-parity.sh` + `scripts/known-drift.txt` |
| **605 dispatched built-ins missing from the LSP** — no completion, no hover | `xdl-lsp/src/symbols.rs` covers 86 of 688 | same check (warns) |
| **The `.xdl` suite**: 12 of 47 scripts pass, 2 never terminate | `tests/` | `tests/passing.txt` + `scripts/run-xdl-suite.sh` |
| **Dead source files a search will hit** | `xdl-stdlib/src/charting_procs_broken.rs`, `charting_procs.rs.bak`, `xdl-gui/src/gui.rs.bak`, `xdl-parser/src/lexer_old.rs` | — |
| **Generated data committed next to source** | `xdl-dataframe/*.csv`, `*.png` | — |

If you fix one, fix it completely and say so — a half-removed `.bak` is worse
than the original.
