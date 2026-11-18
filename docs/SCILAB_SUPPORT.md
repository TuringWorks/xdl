---
layout: default
title: Scilab Support
parent: Compatibility
nav_order: 4
---

# Scilab Support

XDL has an independent, deliberately small Scilab source front end. Running
`.sci` and `.sce` files selects it by extension:

```sh
xdl example.sce
xdl script.sci
xdl examples/scilab/mars_readiness_gate.sce
```

This is an executable syntax slice, not general Scilab or Xcos compatibility.
It supports multiple limited single-output function definitions in one file; it
does not implement Scilab libraries or cross-file function loading.

## Implemented and verified

- Assignments with expressions accepted by the XDL parser; one statement per
  source line, with an optional trailing semicolon.
- Comma-separated one-dimensional array literals such as `[1, 2, 3]`, and
  rectangular numeric 2D literals such as `[1, 2; 3, 4]`, lowered to nested
  XDL arrays in row order. A CLI fixture executes `disp(A)` and checks the
  displayed nested values and row/column order.
- Scalar two-subscript reads `A(i,j)` for variables previously assigned a
  rectangular numeric 2D literal. Both indices must be positive integer
  literals; they are lowered from Scilab's one-based form to XDL's zero-based
  `A[i-1,j-1]` form. Other function calls are preserved. Bounds are checked by
  XDL at runtime, so out-of-range indices fail during execution.
- Scalar one-subscript reads `v(k)` for variables previously assigned a
  comma-separated one-dimensional numeric vector literal such as
  `v = [10, 20, 30]`. Positive integer scalar expressions are validated at
  runtime by `SCILAB_INDEX` and converted from Scilab's one-based form to XDL's
  zero-based index. Zero, negative, fractional, non-finite, and unrepresentable
  indices fail explicitly; XDL still reports source-array bounds errors.
  Inclusive vector range reads `v(a:b)` and `v(a:s:b)` are also supported when
  the source is such a tracked vector. Start, end, and optional step accept
  scalar expressions; runtime validation requires positive integers, a positive
  step, and bounds within the vector. Direction-mismatch ranges return empty.
  The `SCILAB_INDEX_RANGE` helper applies Scilab's inclusive endpoint semantics
  directly. Ordinary calls such as
  `sum(v)` remain function calls. Negative/zero range components, a different
  number of subscripts, and indexed assignment
  targets are rejected explicitly. General indexing remains unsupported.
- Apostrophe transpose for identifier operands (`A'`) and numeric row-vector
  literals (`[1, 2, 3]'`), lowered to XDL `TRANSPOSE`. The CLI regression
  executes a 1×3 numeric vector transpose and observes the resulting 3×1
  representation. Single- and double-quoted string literals remain strings.
- `%pi`, `%e`, and `%t`/`%T` and `%f`/`%F`, mapped to XDL `!PI`,
  `!E`, `1`, and `0`.
- `//` line comments and `/* ... */` block comments, outside quoted strings.
- `disp(expression)`, lowered to XDL `PRINT`.
- Multiple single-output function definitions in one file, each written with zero to four
  comma-separated inputs, for example `function result = combine(a, b, c, d)` …
  `endfunction`, or with no inputs as `function result = constant_value()` …
  `endfunction`. Function bodies accept straight-line assignments only and must
  assign the declared output. Same-file calls before or after definitions are
  supported; definitions are emitted before script statements. Duplicate
  function names (case-insensitive to match XDL dispatch), multiple outputs,
  more than four inputs, duplicate input names, an input matching the output
  name, nested functions, and function control flow are unsupported and rejected.
- Nested `if ... then`, `elseif ... then`, `else`, and `end`.
- Bounded `for` loops in exactly the forms `for i = start:stop` and
  `for i = start:step:stop`, closed by `end`. Bounds and step must be finite
  signed 32-bit integer literals (decimal and exponent notation are rejected);
  the step must be nonzero. Endpoints are inclusive, and loop variable values
  are preserved without one-based index adjustment. Ranges are limited to
  1,000,000 iterations; a step pointing away from the endpoint produces zero
  iterations. The transpiler lowers these to XDL's inclusive
  `FOR variable = start, stop, step DO BEGIN ... ENDFOR` form. Nested loops and
  combinations with supported conditionals use the innermost `end` to close
  the innermost block. The CLI fixture executes `1:5` and `1:2:5`, verifying
  sums of 15 and 9.
- A limited precondition `while` block with the header `while expression`,
  optionally followed by `do` or `then` on that same line, and terminated by
  Scilab `end`. The condition is translated and reevaluated before every body
  iteration. Nested `if`, `for`, and `while` blocks use the innermost `end`.
  Only one statement per source line is accepted; inline comma/semicolon
  bodies, `while ... else`, and `endwhile` are outside this subset. The XDL
  runtime does not cap `while` iterations, so termination depends on the
  program's condition and body.
- Comparison translation for `==`, `~=`, `<`, `<=`, `>`, and `>=`.
- Explicit errors for unsupported percent forms, malformed strings, unclosed
  comments/conditionals, and statements outside this subset.
- CLI routing tests show `.sci` and `.sce` use Scilab transpilation, `.m`
  remains on MATLAB transpilation, and `.xdl` remains native XDL.
- [Mars readiness-gate example](../examples/scilab/mars_readiness_gate.sce)
  demonstrates nested conditionals and `disp` in the supported subset; it is
  a small logic example, not a flight dynamics simulation.
- The VS Code extension recognizes `.sci`/`.sce` as Scilab files and offers
  its run-current-file command; the XDL language server does not analyze
  Scilab syntax.

## Scope and limitations

The front end does not implement general Scilab function or loop syntax. Each
function has one output, zero to four distinct comma-separated inputs, and a
straight-line assignment body. A file may contain multiple such function
definitions, but duplicate function names are rejected case-insensitively.
Function libraries and calls across files are unsupported. Duplicate/malformed
parameter lists and an input that reuses the output name are rejected. Loops inside
functions are unsupported. Loop support is
limited to the bounded literal `for` forms and the limited precondition `while`
form above. A `while` loop can run indefinitely if its
condition never becomes false. It also does not
implement the complete matrix language, multiple statements on one line, line
continuation, the full set of percent-prefixed constants, graphics, file I/O,
toolboxes, or Xcos models. Matrix construction is limited to rectangular
numeric 2D literals; general matrix concatenation and full indexing semantics
are not implemented. Matrix reads support scalar two-subscript access on a
tracked rectangular numeric matrix literal with positive integer literal
indices. Tracked vectors support scalar one-subscript reads with a positive
integer literal or scalar expression; runtime validation rejects nonpositive,
fractional, non-finite, and unrepresentable values. Vectors also support
inclusive `v(a:b)` and `v(a:s:b)` reads with positive integer literal
components. These lower to XDL's zero-based, end-exclusive range form,
preserving Scilab's inclusive endpoint. Negative-step ranges, vector-valued
index expressions, indexed writes, and other matrix indexing forms remain
unsupported. Transpose lowering is limited to
an identifier or a numeric row-vector literal; this does not establish complex conjugate-transpose
semantics or general matrix transpose compatibility. XDL parsing and runtime
semantics still determine whether an expression in the accepted syntax subset
can execute. A successful transpilation alone is not proof of Scilab behavioral
compatibility. For-loop bounds and steps must be integer literals: expressions
or variables, decimal/exponent forms, values outside `i32`, zero steps, and
ranges exceeding the iteration cap are rejected.

Scilab uses `//` and `/* ... */` comments; percent-prefixed names include
constants and other special values. The lexer is separate from the MATLAB
lexer so MATLAB's `%` comments keep their existing meaning. See the
[Scilab comment reference](https://help.scilab.org/docs/2024.1.0/en_US/comments.html).
The accepted `while` header and `end` termination follow the documented Scilab
forms; inline bodies and the optional `else` variant are intentionally not
transpiled ([Scilab `while` reference](https://help.scilab.org/docs/2025.1.0/en_US/while.html)).

## Verification

Verified 2026-11-17:

```sh
cargo test -p xdl-scilab
cargo test -p xdl-cli --test scilab_file_routing
cargo test -p xdl-cli --test scilab_dynamic_indexing
cargo test --lib -p xdl-stdlib scilab_compat
```

The transpiler suite has 27 passing tests, including multiple same-file
single-output function
lowering and rejection cases, rectangular matrix validation, bounded apostrophe
disambiguation, one- and two-subscript scalar read lowering, scalar-expression
index lowering with runtime validation, literal and runtime-valued inclusive
vector range lowering, inclusive literal loops with
nested blocks, direction mismatch, range caps, and nested `while` lowering. The
eleven-test routing suite executes both Scilab extensions, checks MATLAB/native-XDL
routing, verifies scalar and inclusive/stepped vector range values and element
counts, matrix read values and 3×1 transpose, executes inclusive `1:5` and `1:2:5` loops whose
sums are 15 and 9, checks a direction-mismatched range that performs zero
iterations, calls a single-output function before its definition in the same
file, and executes a four-input function with the expected result. The separate
`scilab_function_file` CLI test composes two same-file helpers and checks the
result.
