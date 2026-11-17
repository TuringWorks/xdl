# Real-World MATLAB/Octave Support

This document describes the MATLAB/Octave compatibility subset currently
implemented in the XDL transpiler. It supports selected real-world scripts but
is not MATLAB or Octave language/toolbox parity. Validate each required feature
with an executable test before production use.

## Overview

The XDL MATLAB transpiler supports selected MATLAB constructs and example
workflows used in scientific computing. The examples below demonstrate runtime
paths; they are not a conformance suite and generally lack MATLAB/Octave
reference-output comparisons.

## Features Implemented

### 1. Range Operators in Expressions

**MATLAB Code:**

```matlab
t = (0:L-1)*T;        % Create range from 0 to L-1, multiply by T
x = (1:2:10);         % Create range with step: 1, 3, 5, 7, 9
```

**XDL Equivalent:**

```idl
t = FINDGEN(L) * T
x = (FINDGEN(((10)-(1))/(2) +1) * (2)) + (1)
```

**How It Works:**

- The transpiler detects colon operators `:` inside parentheses
- Converts `(start:end)` to `FINDGEN((end)-(start)+1) + (start)`
- Optimizes `(0:N-1)` to simply `FINDGEN(N)`
- Handles step ranges `(start:step:end)` appropriately

### 2. Array Slicing with Ranges

**MATLAB Code:**

```matlab
arr = (0:19);
slice = arr(1:10);     % Get the first ten elements
subset = arr(5:10);    % Get elements 5 through 10
```

**How It Works:**

- Scalar indexing and vector ranges are translated for known source variables,
  including function parameters. `arr(start:end)` and positive constant-step
  `arr(start:step:end)` map MATLAB's 1-based bounds to XDL's zero-based,
  exclusive-end slices. Vector `end`, `:`, and omitted range bounds are
  supported. Two scalar subscripts on a simple 2D matrix and scalar linear
  reads use MATLAB column-major order in the tested cases. Indexed assignment,
  multidimensional slices, multidimensional `end`, negative steps, and
  logical/index-vector subscripts remain incomplete.
- Automatically converts MATLAB's 1-based indexing to XDL's 0-based indexing
- Handles both numeric indices and variable-based ranges

### 3. Random Number Generation

**MATLAB Code:**

```matlab
r = rand(size(t));         % Uniform random with same size as t
r = randn(size(data));     % Normal random with same size as data
r = rand(10, 20);          % Uniform 10x20 array (falls back to RANDOMU)
```

**XDL Equivalent:**

```idl
r = RANDOMU(1, N_ELEMENTS(t))
r = RANDOMU(1, N_ELEMENTS(data))
r = RANDOMU(1, 10, 20)
```

**How It Works:**

- Special handling for `rand(size(x))` and `randn(size(x))` patterns
- Converts `size(x)` to `N_ELEMENTS(x)` for RANDOMU/RANDOMN
- `rand` maps to a uniform generator; `randn` is not yet a faithful
  normal-distribution implementation and must not be treated as equivalent
- Seed is fixed at 1 for reproducibility

### 4. Statistical Functions

**Supported Functions:**

- `mean(x)` → `MEAN(x)`
- `std(x)` → `STDDEV(x)`
- `min(x)` → `MIN(x)`
- `max(x)` → `MAX(x)`
- `sum(x)` → `TOTAL(x)`
- `median(x)` → `MEDIAN(x)`
- `var(x)` → `VARIANCE(x)`

### 5. Mathematical Functions with Arrays

**Element-wise Operations:**

```matlab
X = A .* B;      % Element-wise multiply
Y = A ./ B;      % Element-wise divide
Z = A .^ B;      % Element-wise power
```

**Array Functions:**

```matlab
y = sin(x);      % Sine (operates on arrays)
y = cos(x);      % Cosine
y = exp(x);      % Exponential
y = sqrt(x);     % Square root
y = abs(x);      % Absolute value
y = log(x);      % Natural logarithm
```

Many mapped math functions work with scalar and array inputs, but broadcasting,
shape rules, and edge cases are still under conformance testing.

### 6. Advanced Plotting

**Line Styles:**

```matlab
plot(x, y, 'b-');     % Blue solid line
plot(x, y, 'r--');    % Red dashed line
plot(x, y, 'g:');     % Green dotted line
```

Some line-style strings are detected but may be ignored because XDL plotting
style coverage is incomplete; ignored styling is a compatibility limitation,
not feature parity.

**Multiple Plots:**

```matlab
figure;
plot(x, y1, 'b-');
hold on;
plot(x, y2, 'r--');
hold off;
title('Multiple Plots');
xlabel('X axis');
ylabel('Y axis');
```

**Tiled Layouts:**

```matlab
tiledlayout(2, 2);
nexttile;
plot(x1, y1);
nexttile;
plot(x2, y2);
```

Converts to tile-specific plot files: `tile1_plot.png`, `tile2_plot.png`, etc.

**3D Plots:**

```matlab
comet3(x, y, z);      % 3D animated comet plot
plot3(x, y, z);       % 3D line plot
```

Converts to `PLOT3D` with appropriate filenames.

### 7. Constants

**Built-in Constants:**

- `pi` → `!PI`
- `e` → `!E`

Example:

```matlab
x = linspace(0, 2*pi, 100);
y = sin(x);
```

Transpiles correctly with `pi` mapped to `!PI`.

`interp1(x, y, xi)` supports piecewise-linear interpolation; the optional
`'nearest'` method is also supported. Sample locations must be strictly
monotonic (ascending or descending), and queries outside their range return
`NaN`. XDL chooses the next input sample on an equal-distance nearest tie;
descending-axis tie behavior has not been cross-checked against MATLAB/Octave.
Other interpolation methods and extrapolation options are not supported.

## Representative MATLAB/Octave Workflows

These snippets show syntax and runtime paths that XDL is targeting. They are
not a claim of full MATLAB/Octave equivalence; this page does not publish
reference-output comparisons for the complete workflows below.

### Example 1: Noisy Signal Analysis

```matlab
% Generate sample data
n = 100;
t = (0:n-1) * 0.1;
data = sin(2*pi*0.5*t) + 0.5*randn(size(t));

% Basic statistics
data_mean = mean(data);
data_std = std(data);

% Plot
figure;
plot(t, data, 'b-');
title('Noisy Sine Wave');
```

**Status:** The listed constructs have partial support. Validate numerical
outputs and plot properties against the source MATLAB/Octave workflow before
relying on a migrated result.

### Example 2: Multi-panel Plots

```matlab
tiledlayout(2, 2);

nexttile;
x1 = (0:99) * 0.1;
plot(x1, sin(x1));
title('Sine');

nexttile;
x2 = (0:99) * 0.1;
plot(x2, cos(x2));
title('Cosine');
```

**Status:** Tile selection and separate plot-file generation are supported in
the current transpiler path; visual layout parity has not been established.

### Example 3: Array Operations

```matlab
x = (0:99) * 0.1;      % Range with scaling
y = sin(x);             % Function on array
z = y(10:50);           % Array slicing
```

**Status:** Range generation, array expressions, and the documented vector
slice subset are supported. The combined example has no published reference
output comparison.

## Limitations and Future Work

### Currently Not Supported

1. **General Multiple-Output Assignment:**

   ```matlab
   [X, Y] = some_function(x);  % General multiple-output assignment is unsupported
   ```

2. **Anonymous Functions:**

   ```matlab
   f = @(x) x^2 + 2*x + 1;  % Not supported
   ```

3. **ODE Solvers:**

   ```matlab
   [t, y] = ode45(@func, [0 10], [1 1 1]);  % Not supported
   ```

4. **FFT and Advanced Signal Processing:**

   ```matlab
   Y = fft(X);  % FFT function exists but complex number handling needs work
   ```

5. **Surface/Mesh Plots and Grid Parity:**

   ```matlab
   surf(X, Y, Z);  % Needs surface plotting support
   meshgrid(...);  % A mapping exists; full MATLAB output and shape parity is unverified
   ```

6. **Matrix Semantics and Linear Algebra:**
   - MATLAB column-major linear matrix indexing and multidimensional slices
   - Matrix multiplication and full MATLAB shape semantics
   - `inv`/`det` names are mapped, but MATLAB numerical and shape conformance is not established; broader eigenvalue workflows are not verified

### Next Compatibility Work

- Implement and verify MATLAB `size` separately from XDL's IDL-style `SIZE`
  metadata function, including the single-dimension form.
- Add MATLAB column-major linear indexing and multidimensional slices without
  changing native XDL indexing semantics.
- Add general multiple-output, cell-array, and anonymous-function support with
  explicit diagnostics where the runtime cannot represent MATLAB behavior.
- Establish reference-vector comparisons for `randn`, FFT/complex operations,
  plotting properties, and mapped linear-algebra functions.
- Define multi-file function lookup and path behavior for `.m` projects.

## Usage Examples

### Running a MATLAB File

```bash
xdl my_matlab_script.m
```

The transpiler automatically detects `.m` files and transpiles them to XDL before execution.

### Example Session

```bash
$ cat example.m
% Simple analysis
x = (0:99) * 0.01;
y = sin(2*pi*5*x);
plot(x, y);

$ xdl example.m
INFO: Detected MATLAB .m file, transpiling to XDL
PLOT: Rendering 100 points to xdl_plot.png
  Plot saved to 'xdl_plot.png'
```

## Example Execution

Run the current sample script:

```bash
xdl examples/matlab/01_simple_math.m
```

This demonstrates selected range, array, mathematical, random, and plotting
paths. It does not compare numerical results or rendered plots against MATLAB.

## Current Evidence

The CLI suite exercises a focused subset of MATLAB transpilation, including
vector indexing, selected 2D scalar subscripts, and `length`/`numel` on simple
values. The sample workflows above are illustrative. Suitability for a real
project must be established with that project's syntax, numerical reference
outputs, file dependencies, and plot comparisons.
