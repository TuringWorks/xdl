---
name: add-builtin
description: How to add or change an XDL built-in function or procedure — the six files that must agree, why only one of them is compiler-checked, the uppercase-dispatch trap, function-vs-procedure registries, keyword arguments, feature gating, and the parity checks that catch the drift. Use when adding a built-in, changing a signature, adding a keyword, or debugging "unknown function" for something that exists.
---

# Adding / changing an XDL built-in

XDL's built-in dispatch is a **754-arm `match` on a `String`** in
`xdl-stdlib/src/lib.rs`. Nothing verifies that the implementation, the dispatch,
the language server, the editor grammar, and the docs agree. **Five of the six
steps below produce no compile error when skipped** — they produce a function
that is unreachable, invisible, or advertised-but-absent.

## The six files, in order

| # | File | Add | Skipping it means |
|---|---|---|---|
| 1 | `xdl-stdlib/src/<module>.rs` | `pub fn name(args: &[XdlValue]) -> XdlResult<XdlValue>` | — |
| 2 | `xdl-stdlib/src/lib.rs` | `"NAME" => module::name(args),` in the right registry | **the function does not exist** at runtime |
| 3 | `xdl-lsp/src/symbols.rs` | a `builtin_functions` entry: name, doc, return type, signature | no completion, no hover — invisible in every editor |
| 4 | `vscode-xdl/syntaxes/xdl.tmLanguage.json` | the name in the builtin pattern | unhighlighted; reads as a user identifier |
| 5 | `docs/` (the matching reference page) | signature + a worked example | undiscoverable |
| 6 | `tests/*.xdl` or `xdl-stdlib/tests/` | a script that calls it and asserts the value | untested |

Only #2 is checked by the compiler, and only for type errors.

## Step 1 — the implementation

Put it in the module that owns the category: `math.rs`, `array.rs`,
`statistics.rs`, `string.rs`, `io.rs`, `linalg.rs`, `signal.rs`, `image.rs`,
`system.rs`, `matlab_compat.rs`, `complex.rs`, `data_structures.rs`. Graphics
implementations go under `graphics/` with a thin wrapper in `graphics_procs.rs`.

```rust
/// SMOOTH(array, width) — boxcar-average an array.
///
/// IDL semantics: the edges are returned unsmoothed unless /EDGE_TRUNCATE.
pub fn smooth(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let arr = args
        .first()
        .ok_or_else(|| XdlError::RuntimeError("SMOOTH: expected an array".into()))?;
    // ...
}
```

Rules that apply here and are easy to get wrong:

- **Return `XdlResult`, never panic.** A panic inside a built-in kills the user's
  whole session and loses their loaded data. No `.unwrap()`, no `.expect()`, no
  indexing that can go out of range. `args.first()` / `args.get(n)`, not `args[n]`.
- **`NaN` is a value, not an error channel.** IDL programs compute with `NaN`
  deliberately. Failure is `XdlError`; a data gap is `NaN`. And
  `sort_by(|a, b| a.partial_cmp(b).unwrap())` **panics** on `NaN` — use
  `total_cmp` or handle the `None`.
- **Never substitute a plausible default** for a value you could not compute.
  `unwrap_or(0.0)` puts a real-looking number into the user's array and every
  derived mean, fit, and plot is then silently wrong.
- **Validate any length before allocating from it.** A dimension argument comes
  straight from a user script; `vec![0.0; n]` **aborts** the process on a bad `n`,
  and on stable Rust there is no `Err` to catch. Bound it first.
- **Match IDL's behaviour, not Rust's** — 0-based indexing, IDL's type-promotion
  table, integer division truncating. Where they differ, IDL wins, with a comment.

## Step 2 — the dispatch arm

`xdl-stdlib/src/lib.rs`. **Pick the right registry:**

| Called as | Registry | Example |
|---|---|---|
| `y = SMOOTH(x, 3)` | `call_function_with_keywords` | a value-returning function |
| `PLOT, x, y` | `call_procedure_with_keywords` | a statement-style procedure |

Registering in the wrong one gives "unknown function" at the call the user
actually wrote, which reads like a typo and is not.

**The literal must be fully uppercase.** Dispatch is
`match name.to_uppercase().as_str()`. An arm containing a lowercase letter can
never fire and the compiler will not warn:

```bash
grep -nE '^[[:space:]]*"[^"]*[a-z][^"]*"[[:space:]]*=>' xdl-stdlib/src/lib.rs
# must print nothing
```

The PostToolUse hook runs this automatically after any edit to `lib.rs`.

**Keywords arrive separately.** The `_with_keywords` variants take
`&HashMap<String, XdlValue>`; the bare `call_function` / `call_procedure` pass an
empty map. If your built-in honours keywords, its arm must forward them:

```rust
"SMOOTH" => array::smooth_kw(args, keywords),
```

Otherwise every `/EDGE_TRUNCATE` a user writes is silently discarded — the worst
kind of bug, because the user believes it took effect. **A keyword you accept and
ignore is worse than one you reject by name.**

**Aliases:** IDL has several. Use `|` in one arm rather than duplicating the body:

```rust
"ALOG" | "LOG" => math::alog(args),
```

## Step 3 — the language server

`xdl-lsp/src/symbols.rs`. Built-ins are registered as `(name, doc, return_type,
signature)` tuples grouped by category (`math_funcs`, `array_funcs`,
`stat_funcs`, `string_funcs`, `io_funcs`, then `procedures`), chained together and
inserted into `builtin_functions`. Add your entry to the matching group:

```rust
(
    "SMOOTH",
    "smooth(array, width) - Boxcar-average an array",
    "ARRAY",
    "SMOOTH(array, width [, /EDGE_TRUNCATE])",
),
```

This one table feeds completion (`completion.rs`) and hover (`hover.rs`).

**The dangerous direction is advertised-but-not-implemented**: a name here with
no dispatch arm gives the user completion for a function that does not exist.

## Step 4 — the editor grammar

`vscode-xdl/syntaxes/xdl.tmLanguage.json`. Add the name to the built-in-function
pattern so it highlights as a built-in rather than a user identifier. This is
cosmetic but it is the fastest signal a user has that they typed a real name.

## Step 5 — the docs

The matching page under `docs/` (`docs/core/`, `docs/graphics/`, `docs/advanced/`,
or the relevant reference index). Signature, argument meanings, keywords, and a
runnable example. `docs/` is Jekyll (just-the-docs) published to GitHub Pages —
run `scripts/check-docs-links.sh` if you add cross-references.

## Step 6 — the test

**Test at the boundary**, in XDL, the way a user calls it — not the Rust function
in isolation. A Rust unit test reproduces the function's own assumptions and will
happily pass while step 2 is missing.

```idl
; tests/stdlib/smooth_test.xdl
a = [1.0, 2.0, 10.0, 2.0, 1.0]
s = smooth(a, 3)
print, s
```

- Pick fixture values no fallback could coincide with. If the failure mode
  returns `0.0`, do not test with data whose answer is `0.0`.
- Use a **non-square** array for anything shape-related; a 3×3 fixture proves
  nothing about a transpose.
- **Watch the test fail before you count it as done.** Comment out the dispatch
  arm, confirm it goes red for the right reason, restore it. Say so in the commit.

## Feature-gated built-ins

Gates: `python`, `rustpython`, `dataframes` (Polars), `ml` (Linfa). The module is
`#[cfg(feature = "…")]`, and so must its dispatch arm be — but **the name must
still be recognised with the feature off**, returning an error that names the
missing feature:

```rust
#[cfg(feature = "dataframes")]
"DF_READ_CSV" => polars_df::read_csv(args),
#[cfg(not(feature = "dataframes"))]
"DF_READ_CSV" => Err(XdlError::RuntimeError(
    "DF_READ_CSV requires XDL built with --features dataframes".into(),
)),
```

"unknown function" sends the user hunting for a typo they did not make.

## Graphics procedures — both backends

`SET_PLOT_BACKEND` selects `PlotBackend::XDLPlot` (plotters → PNG) or
`PlotBackend::ECharts` (HTML). A new plot procedure implements **both branches**
in `graphics_procs.rs`. If a backend genuinely cannot render it, return an
`XdlError` naming the backend — never a silent no-op, and never a fall-through to
the other backend, which produces a figure the user did not ask for.

## Verify before you call it done

```bash
cargo build --release -p xdl-cli
./target/release/xdl -e 'print, SMOOTH([1.0,2.0,10.0,2.0,1.0], 3)'   # it must actually run
cargo test --workspace --no-fail-fast
scripts/check-builtin-parity.sh          # stdlib dispatch <-> LSP symbol table
```

`scripts/check-builtin-parity.sh` prints both directions of the drift:
implemented-but-not-advertised (users cannot discover it) and
advertised-but-not-implemented (users get completion for nothing).

Report what you ran and what you did not: which features were enabled, which plot
backend you exercised, whether you opened the rendered output.

## Debugging "unknown function" for something that exists

In order of likelihood:

1. The arm is in `call_procedure*` but the user called it as a function (or the
   reverse).
2. The arm's literal is not fully uppercase → it is dead code.
3. The module is feature-gated and the build does not have the feature.
4. The arm was added to a `match` that already has an earlier arm for the same
   name — the first wins, silently.
