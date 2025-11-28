# XDL Complete Function Reference

**Version**: 1.0
**Date**: November 2025
**Total Functions**: 150+

---

## Table of Contents

1. [Mathematical Functions](#mathematical-functions)
2. [Array Functions](#array-functions)
3. [Statistical Functions](#statistical-functions)
4. [String Functions](#string-functions)
5. [I/O Functions](#io-functions)
6. [Graphics Procedures](#graphics-procedures)
7. [Machine Learning Functions](#machine-learning-functions)
8. [DataFrame Functions](#dataframe-functions)
9. [Python Integration](#python-integration)
10. [Linear Algebra](#linear-algebra)
11. [Signal Processing](#signal-processing)
12. [Image Processing](#image-processing)
13. [System Functions](#system-functions)

---

## Mathematical Functions

### Trigonometric

| Function | Description | Example |
|----------|-------------|---------|
| `SIN(x)` | Sine | `y = SIN(!PI/2)` |
| `COS(x)` | Cosine | `y = COS(0)` |
| `TAN(x)` | Tangent | `y = TAN(!PI/4)` |
| `ASIN(x)` | Arc sine | `y = ASIN(1)` |
| `ACOS(x)` | Arc cosine | `y = ACOS(0)` |
| `ATAN(x)` | Arc tangent | `y = ATAN(1)` |

### Exponential & Logarithmic

| Function | Description | Example |
|----------|-------------|---------|
| `EXP(x)` | Exponential (e^x) | `y = EXP(1)` |
| `ALOG(x)` | Natural log | `y = ALOG(2.718)` |
| `ALOG10(x)` | Base-10 log | `y = ALOG10(100)` |
| `SQRT(x)` | Square root | `y = SQRT(16)` |

### Rounding & Conversion

| Function | Description | Example |
|----------|-------------|---------|
| `ABS(x)` | Absolute value | `y = ABS(-5)` |
| `FLOOR(x)` | Floor | `y = FLOOR(3.7)` |
| `CEIL(x)` | Ceiling | `y = CEIL(3.2)` |
| `ROUND(x)` | Round | `y = ROUND(3.5)` |
| `FIX(x)` | Truncate to integer | `y = FIX(3.9)` |

### Special Functions

| Function | Description | Example |
|----------|-------------|---------|
| `FFT(x)` | Fast Fourier Transform | `f = FFT(signal)` |
| `RANDOMU(seed, n)` | Uniform random | `r = RANDOMU(seed, 100)` |
| `RANDOMN(seed, n)` | Normal random | `r = RANDOMN(seed, 100)` |
| `MESHGRID(x, y)` | Create mesh grid | `[xx, yy] = MESHGRID(x, y)` |

---

## Array Functions

### Array Generation

| Function | Description | Example |
|----------|-------------|---------|
| `FINDGEN(n)` | Float indices [0, n-1] | `x = FINDGEN(100)` |
| `INDGEN(n)` | Integer indices | `x = INDGEN(100)` |
| `DINDGEN(n)` | Double indices | `x = DINDGEN(100)` |
| `BINDGEN(n)` | Byte indices | `x = BINDGEN(256)` |
| `LINDGEN(n)` | Long indices | `x = LINDGEN(100)` |
| `FLTARR(dims)` | Float array (zeros) | `a = FLTARR(10, 10)` |
| `DBLARR(dims)` | Double array (zeros) | `a = DBLARR(10, 10)` |
| `INTARR(dims)` | Integer array | `a = INTARR(10)` |
| `BYTARR(dims)` | Byte array | `a = BYTARR(256)` |
| `LONARR(dims)` | Long array | `a = LONARR(100)` |
| `STRARR(dims)` | String array | `s = STRARR(10)` |

### Array Manipulation

| Function | Description | Example |
|----------|-------------|---------|
| `N_ELEMENTS(x)` | Number of elements | `n = N_ELEMENTS(arr)` |
| `WHERE(condition)` | Find indices | `idx = WHERE(arr GT 0)` |
| `REFORM(arr, dims)` | Reshape array | `b = REFORM(a, 5, 20)` |
| `TRANSPOSE(arr)` | Transpose | `b = TRANSPOSE(a)` |
| `REVERSE(arr)` | Reverse array | `b = REVERSE(a)` |
| `SORT(arr)` | Sort indices | `idx = SORT(arr)` |

### Array Statistics

| Function | Description | Example |
|----------|-------------|---------|
| `MIN(arr)` | Minimum value | `m = MIN(arr)` |
| `MAX(arr)` | Maximum value | `m = MAX(arr)` |
| `MEAN(arr)` | Mean | `m = MEAN(arr)` |
| `TOTAL(arr)` | Sum | `s = TOTAL(arr)` |

### Moving Averages

| Function | Description | Example |
|----------|-------------|---------|
| `SMOOTH(arr, w)` | Boxcar smooth | `s = SMOOTH(arr, 5)` |
| `MOVING_AVERAGE(arr, w)` | Simple MA | `ma = MOVING_AVERAGE(arr, 10)` |
| `WMA(arr, w)` | Weighted MA | `wma = WMA(arr, 10)` |
| `EMA(arr, w)` | Exponential MA | `ema = EMA(arr, 10)` |
| `CUMULATIVE_AVERAGE(arr)` | Cumulative avg | `ca = CUMULATIVE_AVERAGE(arr)` |

---

## Statistical Functions

| Function | Description | Example |
|----------|-------------|---------|
| `VARIANCE(arr)` | Variance | `v = VARIANCE(data)` |
| `STDDEV(arr)` | Standard deviation | `s = STDDEV(data)` |
| `MEDIAN(arr)` | Median | `m = MEDIAN(data)` |
| `MOMENT(arr)` | Statistical moments | `mom = MOMENT(data)` |
| `MEANABSDEV(arr)` | Mean absolute deviation | `mad = MEANABSDEV(data)` |
| `SKEWNESS(arr)` | Skewness | `sk = SKEWNESS(data)` |
| `KURTOSIS(arr)` | Kurtosis | `k = KURTOSIS(data)` |
| `GAUSS_PDF(x, mu, sigma)` | Gaussian PDF | `p = GAUSS_PDF(x, 0, 1)` |
| `T_PDF(x, df)` | Student's t PDF | `p = T_PDF(x, 10)` |
| `CHISQR_PDF(x, df)` | Chi-squared PDF | `p = CHISQR_PDF(x, 5)` |

---

## String Functions

| Function | Description | Example |
|----------|-------------|---------|
| `STRLEN(s)` | String length | `n = STRLEN('hello')` |
| `STRPOS(s, sub)` | Find substring | `pos = STRPOS(s, 'abc')` |
| `STRMID(s, start, len)` | Substring | `sub = STRMID(s, 0, 5)` |
| `STRUPCASE(s)` | Uppercase | `u = STRUPCASE(s)` |
| `STRLOWCASE(s)` | Lowercase | `l = STRLOWCASE(s)` |
| `STRING(x)` | Convert to string | `s = STRING(123)` |
| `STRTRIM(s, flag)` | Trim whitespace | `t = STRTRIM(s, 2)` |

---

## I/O Functions

| Function/Procedure | Description | Example |
|--------------------|-------------|---------|
| `PRINT, args` | Print to console | `PRINT, 'Hello'` |
| `OPENR, unit, file` | Open for reading | `OPENR, 1, 'data.txt'` |
| `OPENW, unit, file` | Open for writing | `OPENW, 2, 'out.txt'` |
| `READF, unit, var` | Read formatted | `READF, 1, data` |
| `WRITEF, unit, data` | Write formatted | `WRITEF, 2, data` |
| `CLOSE, unit` | Close file | `CLOSE, 1` |
| `GET_LUN` | Get free unit | `lun = GET_LUN()` |
| `FREE_LUN, unit` | Free unit | `FREE_LUN, lun` |
| `FILEPATH(file)` | Get full path | `p = FILEPATH('data.txt')` |
| `READ_JPEG(file)` | Read JPEG image | `img = READ_JPEG('photo.jpg')` |

---

## Graphics Procedures

### Basic Plotting

| Procedure | Description |
|-----------|-------------|
| `PLOT, y` | 2D line plot |
| `PLOT, x, y` | 2D line plot with x |
| `OPLOT, x, y` | Overplot on existing |
| `PLOTS, x, y` | Draw polyline |
| `XYOUTS, x, y, text` | Draw text |
| `AXIS` | Draw axis |

### 3D Plotting

| Procedure | Description |
|-----------|-------------|
| `SURFACE, z` | 3D surface plot |
| `SHADE_SURF, z` | Shaded surface |
| `CONTOUR, z` | Contour plot |
| `PLOT3D, x, y, z` | 3D line plot |
| `ISOSURFACE` | Isosurface rendering |

### Image Display

| Procedure | Description |
|-----------|-------------|
| `TV, img` | Display image |
| `TVSCL, img` | Display scaled image |
| `IMAGE_DISPLAY, img` | Enhanced image display |

### Window Management

| Procedure | Description |
|-----------|-------------|
| `WINDOW, n` | Create window |
| `WSET, n` | Set active window |
| `WDELETE, n` | Delete window |
| `ERASE` | Clear window |
| `DEVICE` | Device settings |

### Charting (ECharts)

| Procedure | Description |
|-----------|-------------|
| `CHART_PLOT` | Interactive line chart |
| `CHART_SCATTER` | Scatter chart |
| `CHART_BAR` | Bar chart |
| `CHART_CONTOUR` | Contour chart |
| `SURFACE3D` | 3D surface chart |
| `SCATTER3D` | 3D scatter chart |

---

## Machine Learning Functions

### XDLML Functions (50+)

See [ML Complete Reference](ML_COMPLETE_REFERENCE.md) for full documentation.

**Categories:**
- Data utilities: `XDLML_PARTITION`, `XDLML_SHUFFLE`
- Normalizers: `XDLML_*_NORMALIZER`
- Activation functions: `XDLMLAF_*`
- Loss functions: `XDLMLLF_*`
- Optimizers: `XDLMLOPT_*`
- Neural networks: `XDLML_FEEDFORWARDNEURALNETWORK`
- SVM: `XDLML_SUPPORTVECTORMACHINE*`
- Cross-validation: `XDLML_KFOLD`, `XDLML_STRATIFIEDKFOLD`

### Linfa ML Functions (Native Rust)

See [Linfa ML Reference](LINFA_ML_REFERENCE.md) for full documentation.

| Function | Description |
|----------|-------------|
| `ML_KMEANS_FIT` | Train K-Means |
| `ML_KMEANS_PREDICT` | Predict clusters |
| `ML_LINEAR_FIT` | Train linear regression |
| `ML_LINEAR_PREDICT` | Predict values |
| `ML_PCA_FIT` | Fit PCA |
| `ML_PCA_TRANSFORM` | Transform data |
| `ML_ACCURACY` | Classification accuracy |
| `ML_MSE` | Mean squared error |
| `ML_R2_SCORE` | R² coefficient |

---

## DataFrame Functions (Polars)

See [DataFrame Reference](DATAFRAMES_REFERENCE.md) for full documentation.

| Function | Description |
|----------|-------------|
| `DF_READ_CSV` | Read CSV file |
| `DF_READ_PARQUET` | Read Parquet file |
| `DF_READ_JSON` | Read JSON file |
| `DF_CREATE` | Create from arrays |
| `DF_WRITE_CSV` | Write to CSV |
| `DF_WRITE_PARQUET` | Write to Parquet |
| `DF_HEAD` | First N rows |
| `DF_TAIL` | Last N rows |
| `DF_SELECT` | Select columns |
| `DF_FILTER` | Filter rows |
| `DF_SORT` | Sort by column |
| `DF_GROUPBY` | Group and aggregate |
| `DF_JOIN` | Join DataFrames |
| `DF_SHAPE` | Get dimensions |
| `DF_COLUMNS` | Get column names |
| `DF_TO_ARRAY` | Convert to XDL array |
| `DF_DROP` | Free memory |

---

## Python Integration

See [Python Integration](PYTHON_INTEGRATION.md) for full documentation.

| Function | Description |
|----------|-------------|
| `PYTHON_IMPORT(module)` | Import Python module |
| `PYTHON_CALL(mod, func, args)` | Call Python function |
| `PYTHON_CALL_KW(mod, func, args, kwargs)` | Call with keywords |

---

## Linear Algebra

| Function | Description |
|----------|-------------|
| `IDENTITY(n)` | Identity matrix |
| `INVERT(matrix)` | Matrix inverse |
| `DETERM(matrix)` | Determinant |
| `TRANSPOSE(matrix)` | Transpose |
| `CROSSP(a, b)` | Cross product |
| `DOTP(a, b)` | Dot product |
| `NORM(arr)` | Vector norm |
| `DIAGONAL(matrix)` | Diagonal elements |
| `TRACE(matrix)` | Matrix trace |
| `SVDC(matrix)` | SVD decomposition |
| `LA_EIGENVAL(matrix)` | Eigenvalues |
| `LUDC(matrix)` | LU decomposition |
| `LUSOL(lu, b)` | Solve LU system |

---

## Signal Processing

| Function | Description |
|----------|-------------|
| `A_CORRELATE(x, lag)` | Auto-correlation |
| `C_CORRELATE(x, y)` | Cross-correlation |
| `DIGITAL_FILTER(x, coef)` | Digital filter |
| `HILBERT(x)` | Hilbert transform |
| `MEDIAN_FILTER(x, w)` | Median filter |

---

## Image Processing

| Function | Description |
|----------|-------------|
| `CONVOL(img, kernel)` | Convolution |
| `DILATE(img, struct)` | Morphological dilation |
| `ERODE(img, struct)` | Morphological erosion |
| `SOBEL(img)` | Sobel edge detection |
| `ROBERTS(img)` | Roberts cross |
| `PREWITT(img)` | Prewitt operator |
| `GAUSSIAN_FILTER(img, sigma)` | Gaussian blur |
| `THRESHOLD(img, val)` | Binary threshold |

---

## System Functions

| Function/Procedure | Description |
|--------------------|-------------|
| `HELP` | Display help |
| `CD, path` | Change directory |
| `SPAWN, cmd` | Execute system command |
| `WAIT, seconds` | Pause execution |
| `STOP` | Stop execution |
| `CATCH, var` | Error handling |

---

## System Variables

| Variable | Description |
|----------|-------------|
| `!PI` | Pi (float) |
| `!DPI` | Pi (double) |
| `!E` | Euler's number |
| `!RADEG` | Radians to degrees |
| `!DTOR` | Degrees to radians |
| `!NULL` | Null value |
| `!TRUE` | Boolean true |
| `!FALSE` | Boolean false |
| `!VERSION` | XDL version |
| `!DIR` | Installation directory |
| `!PATH` | Search path |

---

## Keywords

### Control Flow

```idl
IF condition THEN statement
IF condition THEN BEGIN ... END ELSE BEGIN ... END

FOR var = start, end DO statement
FOR var = start, end, step DO BEGIN ... END

WHILE condition DO statement
WHILE condition DO BEGIN ... END

REPEAT statement UNTIL condition
REPEAT BEGIN ... END UNTIL condition

FOREACH item, collection DO statement

SWITCH expression OF
  value1: statement
  value2: BEGIN ... END
  ELSE: default_statement
ENDSWITCH

CASE expression OF
  value1: statement
  value2: statement
  ELSE: default
ENDCASE

BREAK
CONTINUE
RETURN [, value]
```

### Definitions

```idl
FUNCTION name, param1, param2, KEYWORD=keyword
  ; function body
  RETURN, result
ENDFUNCTION

PRO name, param1, param2, KEYWORD=keyword
  ; procedure body
ENDPRO
```

### Operators

| Type | Operators |
|------|-----------|
| Arithmetic | `+`, `-`, `*`, `/`, `^`, `MOD` |
| Comparison | `EQ`, `NE`, `LT`, `GT`, `LE`, `GE` |
| Logical | `AND`, `OR`, `NOT`, `XOR` |
| Assignment | `=`, `+=`, `-=`, `*=`, `/=` |
| Matrix | `#` (multiply), `##` (transpose multiply) |
| Pointer | `->` |
| Range | `:` |

---

## Feature Flags

| Flag | Description | Functions |
|------|-------------|-----------|
| `python` | Python integration | `PYTHON_*` |
| `dataframes` | Polars DataFrames | `DF_*` |
| `ml` | Linfa ML | `ML_*` |
| `rustpython` | RustPython | `RUSTPY_*` |

Build with flags:
```bash
cargo build --features "python,dataframes,ml"
```
