# XDL Complete Function Reference

**Version**: 1.1
**Date**: December 2025
**Total Functions**: 220+

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

### Hyperbolic

| Function | Description | Example |
|----------|-------------|---------|
| `SINH(x)` | Hyperbolic sine | `y = SINH(1)` |
| `COSH(x)` | Hyperbolic cosine | `y = COSH(0)` |
| `TANH(x)` | Hyperbolic tangent | `y = TANH(1)` |
| `ASINH(x)` | Inverse hyperbolic sine | `y = ASINH(1)` |
| `ACOSH(x)` | Inverse hyperbolic cosine | `y = ACOSH(1)` |
| `ATANH(x)` | Inverse hyperbolic tangent | `y = ATANH(0.5)` |

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

### Type Conversion

| Function | Description | Example |
|----------|-------------|---------|
| `BYTE(x)` | Convert to byte (0-255) | `b = BYTE(100)` |
| `INT(x)` | Convert to 16-bit integer | `i = INT(3.7)` |
| `UINT(x)` | Convert to unsigned 16-bit | `u = UINT(100)` |
| `LONG(x)` | Convert to 32-bit integer | `l = LONG(100)` |
| `ULONG(x)` | Convert to unsigned 32-bit | `u = ULONG(100)` |
| `LONG64(x)` | Convert to 64-bit integer | `l = LONG64(100)` |
| `ULONG64(x)` | Convert to unsigned 64-bit | `u = ULONG64(100)` |
| `FLOAT(x)` | Convert to 32-bit float | `f = FLOAT(100)` |
| `DOUBLE(x)` | Convert to 64-bit double | `d = DOUBLE(100)` |
| `STRING(x)` | Convert to string | `s = STRING(42)` |
| `COMPLEX(re, im)` | Create complex number | `c = COMPLEX(1, 2)` |
| `DCOMPLEX(re, im)` | Create double complex | `c = DCOMPLEX(1, 2)` |

### Special Functions

| Function | Description | Example |
|----------|-------------|---------|
| `FFT(x)` | Fast Fourier Transform | `f = FFT(signal)` |
| `RANDOMU(seed, n)` | Uniform random | `r = RANDOMU(seed, 100)` |
| `RANDOMN(seed, n)` | Normal random | `r = RANDOMN(seed, 100)` |
| `MESHGRID(x, y)` | Create mesh grid | `[xx, yy] = MESHGRID(x, y)` |
| `ERF(x)` | Error function | `y = ERF(1.0)` |
| `ERFC(x)` | Complementary error function | `y = ERFC(1.0)` |
| `GAMMA(x)` | Gamma function | `y = GAMMA(5)` |
| `LNGAMMA(x)` | Log gamma function | `y = LNGAMMA(100)` |
| `FACTORIAL(n)` | Factorial (n!) | `y = FACTORIAL(5)` |
| `BESELJ(x, n)` | Bessel J function | `y = BESELJ(1.0, 0)` |
| `BESELY(x, n)` | Bessel Y function | `y = BESELY(1.0, 0)` |
| `BESELI(x, n)` | Modified Bessel I | `y = BESELI(1.0, 0)` |
| `BESELK(x, n)` | Modified Bessel K | `y = BESELK(1.0, 0)` |

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
| `SHIFT(arr, s)` | Circular shift elements | `b = SHIFT(arr, 2)` |
| `ROTATE(arr, dir)` | Rotate 2D array | `b = ROTATE(img, 1)` |
| `REPLICATE(val, n)` | Replicate value | `a = REPLICATE(0, 100)` |
| `REBIN(arr, dims)` | Resize by averaging | `b = REBIN(a, 100, 100)` |
| `CONGRID(arr, dims)` | Resize with interpolation | `b = CONGRID(a, 256, 256)` |
| `MAKE_ARRAY(dims)` | Create array | `a = MAKE_ARRAY(10, 10)` |
| `ARRAY_EQUAL(a, b)` | Compare arrays | `eq = ARRAY_EQUAL(a, b)` |
| `UNIQ(arr)` | Find unique elements | `u = arr[UNIQ(arr)]` |
| `HISTOGRAM(arr)` | Compute histogram | `h = HISTOGRAM(data)` |

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

### Basic Statistics

| Function | Description | Example |
|----------|-------------|---------|
| `VARIANCE(arr)` | Variance | `v = VARIANCE(data)` |
| `STDDEV(arr)` | Standard deviation | `s = STDDEV(data)` |
| `MEDIAN(arr)` | Median | `m = MEDIAN(data)` |
| `MOMENT(arr)` | Statistical moments | `mom = MOMENT(data)` |
| `MEANABSDEV(arr)` | Mean absolute deviation | `mad = MEANABSDEV(data)` |
| `SKEWNESS(arr)` | Skewness | `sk = SKEWNESS(data)` |
| `KURTOSIS(arr)` | Kurtosis | `k = KURTOSIS(data)` |
| `CORRELATE(x, y)` | Correlation coefficient | `r = CORRELATE(x, y)` |

### Probability Distributions

| Function | Description | Example |
|----------|-------------|---------|
| `GAUSS_PDF(x, mu, sigma)` | Gaussian PDF | `p = GAUSS_PDF(x, 0, 1)` |
| `T_PDF(x, df)` | Student's t PDF | `p = T_PDF(x, 10)` |
| `CHISQR_PDF(x, df)` | Chi-squared PDF | `p = CHISQR_PDF(x, 5)` |

### Curve Fitting

| Function | Description | Example |
|----------|-------------|---------|
| `LINFIT(x, y)` | Linear fit (y = a + bx) | `coef = LINFIT(x, y)` |
| `POLY_FIT(x, y, n)` | Polynomial fit | `coef = POLY_FIT(x, y, 2)` |
| `REGRESS(x, y)` | Multiple regression | `result = REGRESS(x, y)` |

### Interpolation

| Function | Description | Example |
|----------|-------------|---------|
| `INTERPOL(y, x, xnew)` | Linear interpolation | `ynew = INTERPOL(y, x, xnew)` |
| `SPLINE(x, y, xnew)` | Cubic spline | `ynew = SPLINE(x, y, xnew)` |
| `BILINEAR(z, x, y)` | Bilinear interpolation | `znew = BILINEAR(z, xnew, ynew)` |

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
| `STRJOIN(arr, delim)` | Join strings | `s = STRJOIN(['a','b'], ',')` |
| `STRSPLIT(s, delim)` | Split string | `arr = STRSPLIT('a,b,c', ',')` |
| `STRCOMPRESS(s)` | Compress whitespace | `c = STRCOMPRESS(s)` |
| `STRCMP(s1, s2)` | Compare strings | `eq = STRCMP('abc', 'ABC')` |
| `STRREPLACE(s, old, new)` | Replace in string | `r = STRREPLACE(s, 'old', 'new')` |
| `STREGEX(s, pattern)` | Regex match | `m = STREGEX(s, '[0-9]+')` |
| `STRMATCH(s, pattern)` | Wildcard match | `m = STRMATCH(s, '*.txt')` |

---

## I/O Functions

### File Operations

| Function/Procedure | Description | Example |
|--------------------|-------------|---------|
| `PRINT, args` | Print to console | `PRINT, 'Hello'` |
| `OPENR, unit, file` | Open for reading | `OPENR, 1, 'data.txt'` |
| `OPENW, unit, file` | Open for writing | `OPENW, 2, 'out.txt'` |
| `OPENU, unit, file` | Open for read/write | `OPENU, 1, 'data.bin'` |
| `READF, unit, var` | Read formatted | `READF, 1, data` |
| `WRITEF, unit, data` | Write formatted | `WRITEF, 2, data` |
| `READ, var` | Read from stdin | `READ, input` |
| `READS, str, var` | Read from string | `READS, line, val` |
| `READU, unit, var` | Read binary data | `READU, 1, arr` |
| `WRITEU, unit, data` | Write binary data | `WRITEU, 1, arr` |
| `CLOSE, unit` | Close file | `CLOSE, 1` |
| `GET_LUN` | Get free unit | `lun = GET_LUN()` |
| `FREE_LUN, unit` | Free unit | `FREE_LUN, lun` |
| `POINT_LUN, unit, pos` | Seek position | `POINT_LUN, 1, 0` |
| `EOF(unit)` | Check end of file | `IF EOF(1) THEN ...` |
| `FLUSH, unit` | Flush buffer | `FLUSH, 1` |

### File System

| Function | Description | Example |
|----------|-------------|---------|
| `FILE_TEST(path)` | Test file exists | `IF FILE_TEST('f.txt') THEN ...` |
| `FILE_INFO(path)` | Get file metadata | `info = FILE_INFO('f.txt')` |
| `FILE_SEARCH(pattern)` | Search for files | `files = FILE_SEARCH('*.dat')` |
| `FILE_MKDIR(path)` | Create directory | `FILE_MKDIR, 'subdir'` |
| `FILE_DELETE(path)` | Delete file | `FILE_DELETE, 'temp.txt'` |
| `FILE_COPY(src, dst)` | Copy file | `FILE_COPY, 's.txt', 'd.txt'` |
| `FILE_MOVE(src, dst)` | Move/rename file | `FILE_MOVE, 'old', 'new'` |
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

### General

| Function/Procedure | Description | Example |
|--------------------|-------------|---------|
| `HELP` | Display help | `HELP` |
| `CD, path` | Change directory | `CD, '/home/user'` |
| `SPAWN, cmd` | Execute system command | `SPAWN, 'ls -la'` |
| `WAIT, seconds` | Pause execution | `WAIT, 2.5` |
| `STOP` | Stop execution | `STOP` |
| `CATCH, var` | Error handling | `CATCH, err` |

### Timing

| Function | Description | Example |
|----------|-------------|---------|
| `SYSTIME()` | Current time string | `t = SYSTIME()` |
| `SYSTIME(1)` | Seconds since epoch | `s = SYSTIME(1)` |
| `TIC()` | Start timer | `TIC` |
| `TOC()` | Elapsed time (seconds) | `elapsed = TOC()` |
| `JULDAY(m, d, y)` | Julian day number | `jd = JULDAY(12, 25, 2024)` |
| `CALDAT(jd)` | Calendar date from Julian | `date = CALDAT(jd)` |

### Type Inspection

| Function | Description | Example |
|----------|-------------|---------|
| `SIZE(x)` | Array dimensions info | `dims = SIZE(arr)` |
| `N_ELEMENTS(x)` | Number of elements | `n = N_ELEMENTS(arr)` |
| `TYPENAME(x)` | Type name as string | `t = TYPENAME(x)` |
| `ISA(x, type)` | Check type | `IF ISA(x, 'FLOAT') THEN ...` |

### Structure Functions

| Function | Description | Example |
|----------|-------------|---------|
| `CREATE_STRUCT(tags, vals)` | Create structure | `s = CREATE_STRUCT('a', 1, 'b', 2)` |
| `N_TAGS(struct)` | Number of fields | `n = N_TAGS(mystruct)` |
| `TAG_NAMES(struct)` | Field names | `names = TAG_NAMES(mystruct)` |

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
