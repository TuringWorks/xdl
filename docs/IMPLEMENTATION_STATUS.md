# XDL Standard Library Implementation Status

**Last Updated:** 2025-12-31

## Overview
This document tracks the implementation progress of the XDL (eXtensible Data Language) standard library, a Rust-based implementation of IDL/GDL functionality.

## Completion Summary

### Fully Completed Phases ✅
- **Phase 5: Array Manipulation** (100%)
- **Phase 6: Mathematics** (100%) ✅ NEW
- **Phase 7: Statistics** (100%) - GPU-accelerated ✅ NEW
- **Phase 8: String Operations** (100%)
- **Phase 9: File I/O** (100%) ✅ NEW
- **Phase 10: Image I/O** (100%) ✅ NEW - PNG, JPEG, TIFF, BMP, GIF
- **Phase 11: Signal Processing** (100%) ✅ NEW
- **Phase 12: Linear Algebra** (100%) ✅ NEW
- **Phase 13: Image Processing** (100%) ✅ NEW
- **Phase 14: Time & Date** (100%) ✅ NEW
- **Phase 15: Type Conversion** (100%) ✅ NEW - Pointer/Object management
- **Phase 16: Data Structures** (100%) ✅ NEW - LIST, ORDEREDHASH, CREATE_STRUCT
- **Phase 17: Complex Numbers** (100%) ✅ NEW
- **Phase 18: System & Control** (100%) ✅ NEW

### Object-Oriented Syntax ✅ NEW
- **Array Methods**: 17+ methods (`arr->Sum()`, `arr->Mean()`, `arr->Sort()`, etc.)
- **String Methods**: 16+ methods (`str->ToUpper()`, `str->Contains()`, etc.)
- **DataFrame Methods**: 15+ methods (`df->Head()`, `df->Column()`, etc.)
- **Struct Field Access**: `point.x`, `point.y`

### MATLAB Compatibility ✅
- **MATLAB Transpiler**: 28/28 unit tests passing
- **Basic MATLAB Execution**: Working (.m files execute directly)
- **Function Mapping**: ~80 MATLAB functions mapped to XDL equivalents
- **Syntax Conversion**: 1-based → 0-based indexing, element-wise operators

### GPU Acceleration ✅ NEW
- **xdl-amp Backend**: Multi-backend GPU support
- **Accelerated Functions**: MIN, MAX, MEAN, TOTAL, MEDIAN, VARIANCE, STDDEV
- **Performance**: 10-50x speedup for large arrays (>10K elements)
- **Smart Dispatch**: Automatic CPU/GPU selection based on array size

### xdl-amp GPU Backends

| Platform | Backends | Priority |
|----------|----------|----------|
| **macOS** | MLX ✅, Metal, MPS, CoreML | MLX > MPS > Metal > CoreML |
| **Windows** | DirectX 12, DirectML, CUDA, cuDNN, Vulkan | cuDNN > CUDA > DirectML > DirectX12 |
| **Linux** | CUDA, cuDNN, ROCm, OpenCL, Vulkan | cuDNN > CUDA > ROCm > OpenCL |

**Apple MLX Backend** ✅ NEW (v0.1.5)
- Unified memory architecture (no CPU/GPU transfers)
- Lazy evaluation with JIT compilation
- Optimized for Apple Silicon (M1/M2/M3/M4)
- Complete FFT and linear algebra support
- Requires full Xcode installation

### MLX Performance Benchmarks (Apple Silicon)

**Matrix Multiplication Performance** (vs CPU baseline):

| Matrix Size | CPU | Metal | MLX | MLX vs Metal |
|-------------|-----|-------|-----|--------------|
| 100×100 | 0.94ms | 0.44ms (2.2x) | 0.55ms (1.7x) | Metal faster |
| 316×316 | 28.8ms | 1.5ms (19x) | 0.65ms (**44x**) | **MLX 2.4x faster** |
| 1000×1000 | 995ms | 7.4ms (135x) | 2.6ms (**388x**) | **MLX 2.9x faster** |
| 3162×3162 | 58.4s | 171ms (341x) | 38.5ms (**1517x**) | **MLX 4.4x faster** |

**Key Findings:**
- **MLX excels at matrix multiplication** - Up to 4.4x faster than Metal for large matrices
- **GPU overhead for small arrays** - CPU faster for arrays <100K elements
- **MLX unified memory advantage** - No explicit CPU↔GPU transfers required

**Recommended Usage:**
- Use **MLX** for matrix-heavy computations (linear algebra, ML inference)
- Use **Metal** for thread-safe operations requiring parallel access
- Use **CPU** for small arrays (<10K elements) to avoid GPU dispatch overhead

Run benchmark: `cargo run --example mlx_benchmark --features mlx -p xdl-amp --release`

### Function Keyword Arguments ✅ NEW
- **Parser Support**: `NAME=value` and `/FLAG` syntax in function calls
- **Array Generation**: All 12 functions support `START` and `INCREMENT` keywords
- **MAKE_ARRAY**: Flexible array creation with `DIMENSION`, `VALUE`, `/INDEX` keywords
- **Example**: `arr = FINDGEN(5, START=10, INCREMENT=2)` → `[10, 12, 14, 16, 18]`

### Partial/Deferred Phases ⏸️
- **Phase 10: Image I/O** (Requires external image crates)
- **Phase 19: Graphics** (Extensive - partially complete via existing modules)

---

## Detailed Phase Breakdown

### Phase 5: Array Manipulation ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `ARRAY_INDICES` - Get indices for array reshaping
- `ARRAY_EQUAL` - Compare arrays for equality
- `UNIQ` - Find unique elements
- `PERMUTE` - Permute array dimensions
- `CONGRID` - Resample/resize arrays
- `INTERPOL` - Linear interpolation
- `WHERE` - Find elements matching condition
- `N_ELEMENTS` - Get total number of elements
- `REFORM` - Reshape arrays
- `TRANSPOSE` - Transpose arrays
- `ROTATE` - Rotate arrays
- `SHIFT` - Shift array elements
- `REBIN` - Resize arrays by integer factors
- `REPLICATE` - Replicate values
- `HISTOGRAM` - Compute histogram
- `MESHGRID` - Generate coordinate matrices
- `REVERSE` - Reverse array order
- `SORT` - Sort array elements

---

### Phase 6: Mathematics ✅ 100%
**Status:** Complete

**Implemented Functions:**

**Trigonometric:**
- `SIN`, `COS`, `TAN`
- `ASIN`, `ACOS`, `ATAN`, `ATAN2`
- `SINH`, `COSH`, `TANH`
- `ASINH`, `ACOSH`, `ATANH`

**Exponential/Logarithmic:**
- `EXP`, `ALOG` (LN), `ALOG10`, `ALOG2`
- `SQRT`, `ABS`, `POW`

**Rounding:**
- `FLOOR`, `CEIL`, `ROUND`

**Special Functions:**
- `GAMMA`, `LNGAMMA`
- `ERF`, `ERFC`
- `BESSEL_J` (Bessel function)
- `BETA` - Beta function
- `FACTORIAL`
- `GCD`, `LCM` - Greatest common divisor, Least common multiple
- `POLY` - Polynomial evaluation
- `BINOMIAL` - Binomial coefficient
- `PRIME` - Primality test
- `PRIMES` - Generate prime numbers
- `PRODUCT` - Array product

**Array Generation:**
- `FINDGEN`, `INDGEN`, `DINDGEN`, `BINDGEN`
- `LINDGEN`, `UINDGEN`, `ULINDGEN`, `L64INDGEN`
- `RANDOMU`, `RANDOMN`

**Calculus:**
- `DERIV` - Numerical derivative
- `INT_TABULATED` - Numerical integration (trapezoidal)

**Validation:**
- `FINITE` - Test for finite values
- `CHECK_MATH` - Check math errors
- `MACHAR` - Machine arithmetic parameters

---

### Phase 7: Statistics ✅ 100%
**Status:** Complete with GPU Acceleration

**Implemented Functions:**
- `VARIANCE`, `STDDEV` - Variance and standard deviation (**GPU-accelerated**)
- `MEDIAN` - Median value (**GPU-accelerated**)
- `MOMENT` - Statistical moments
- `MEANABSDEV` - Mean absolute deviation
- `SKEWNESS`, `KURTOSIS` - Distribution shape
- `CORRELATE` - Correlation coefficient
- `R_CORRELATE` - Spearman rank correlation ✅ NEW
- `A_CORRELATE` - Auto-correlation (in signal.rs)
- `C_CORRELATE` - Cross-correlation (in signal.rs)
- `REGRESS` - Linear regression
- `LINFIT` - Linear least squares fit
- `POLY_FIT` - Polynomial fitting
- `CURVEFIT` - Levenberg-Marquardt curve fitting ✅ NEW
- `LADFIT` - L1 regression (least absolute deviations) ✅ NEW
- `SVDFIT` - SVD-based fitting ✅ NEW
- `PERCENTILES` - Compute percentiles
- `ROBUST_MEAN` - Robust mean estimator
- `TRIMMED_MEAN` - Mean with outliers removed
- `RESISTANT_MEAN` - Resistant mean
- `RANDOM_POISSON` - Poisson random numbers

**GPU-Accelerated Functions (via xdl-amp):**
- `MIN`, `MAX`, `MEAN`, `TOTAL` - 10-50x speedup for large arrays
- `MEDIAN`, `VARIANCE`, `STDDEV` - SIMD-optimized with parallel execution

**Probability Distributions:**
- `GAUSS_PDF` - Gaussian probability density
- `T_PDF` - Student's t distribution
- `CHISQR_PDF` - Chi-square distribution

**Note:** All statistical functions support MultiDimArray types for N-dimensional array operations.

---

### Phase 8: String Operations ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `STRLEN` - String length
- `STRPOS` - Find substring position
- `STRMID` - Extract substring
- `STRUPCASE`, `STRLOWCASE` - Case conversion
- `STRTRIM` - Trim whitespace
- `STRJOIN` - Join strings
- `STRSPLIT` - Split strings
- `STRCMP` - String comparison
- `STRCOMPRESS` - Compress whitespace
- `STRMATCH` - Wildcard pattern matching
- `STRING` - Convert to string
- `STRREPLACE` - Replace substrings
- `STRPUT` - Insert/overlay string
- `STRMESSAGE` - Error message text
- `FORMAT_AXIS_VALUES` - Format numeric labels
- `STREGEX` - Regular expression matching (uses Rust regex crate)

**OOP String Methods (via arrow syntax):**
- `str->ToUpper()`, `str->ToLower()`, `str->Length()`
- `str->Contains()`, `str->IndexOf()`, `str->Split()`
- `str->StartsWith()`, `str->EndsWith()`, `str->Trim()`
- `str->Replace()`, `str->Substring()`, `str->Match()`

**All string operations complete.**

---

### Phase 9: File I/O ✅ 100%
**Status:** Complete

**Implemented Functions:**

**File Operations:**
- `FILE_BASENAME` - Extract filename
- `FILE_DIRNAME` - Extract directory
- `FILE_MKDIR` - Create directory
- `FILE_DELETE` - Delete files
- `FILE_COPY` - Copy files
- `FILE_MOVE` - Move files
- `FILE_TEST` - Test file existence
- `FILE_LINES` - Count file lines
- `FILE_INFO` - Get file metadata
- `FILE_EXPAND_PATH` - Expand path with home directory ✅ NEW
- `FILE_SAME` - Test if paths refer to same file ✅ NEW
- `FILE_CHMOD` - Change file permissions (Unix) ✅ NEW
- `FINDFILE` - Search for files in PATH ✅ NEW

**I/O Operations:**
- `GET_LUN` - Get logical unit number
- `FREE_LUN` - Free logical unit
- `OPEN`, `OPENR`, `OPENW`, `OPENU` - Open files
- `CLOSE` - Close files
- `READF`, `READU` - Read formatted/unformatted
- `WRITEF`, `PRINTF`, `WRITEU` - Write operations
- `FLUSH` - Flush output buffer
- `POINT_LUN` - Position file pointer
- `EOF` - End of file test
- `ASSOC` - Associate file with array
- `FILEPATH` - Locate files
- `READ_JPEG` - Read JPEG images

---

### Phase 12: Linear Algebra ✅ 100%
**Status:** Complete with nalgebra

**Implemented Functions:**
- `IDENTITY` - Identity matrix
- `INVERT` - Matrix inversion
- `DETERM` - Determinant
- `CROSSP` - Cross product (3D vectors)
- `DOTP` - Dot product
- `NORM` - Vector/matrix norm
- `DIAGONAL` - Extract diagonal
- `TRACE` - Matrix trace
- `SVDC` - Singular value decomposition
- `LA_EIGENVAL` - Eigenvalues
- `LA_EIGENVEC` - Eigenvectors ✅ NEW
- `LUDC` - LU decomposition
- `LUSOL` - LU solve linear system
- `LA_LINEAR_EQUATION` - Solve Ax=b ✅ NEW
- `LA_LEAST_SQUARES` - Least squares solution ✅ NEW
- `LA_CHOLESKY` - Cholesky decomposition ✅ NEW
- `LA_TRIDC` - Tridiagonal decomposition ✅ NEW
- `QR` - QR decomposition ✅ NEW
- `RANK` - Matrix rank ✅ NEW
- `CRAMER` - Cramer's rule solver ✅ NEW
- `MATRIX_MULTIPLY` - Matrix multiplication ✅ NEW
- `COND` - Condition number ✅ NEW
- `PINV` - Moore-Penrose pseudoinverse ✅ NEW

---

### Phase 11: Signal Processing ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `FFT` - Fast Fourier Transform (1D, forward and inverse)
- `FFT_2D` / `FFT2` - 2D Fast Fourier Transform ✅ NEW
- `A_CORRELATE` - Auto-correlation
- `C_CORRELATE` - Cross-correlation
- `SMOOTH` - Boxcar smoothing
- `CONVOL` - 1D/2D convolution
- `DIGITAL_FILTER` - Filter coefficient generation
- `HILBERT` - Hilbert transform (phase-shift approximation)
- `MEDIAN_FILTER` - Median filtering
- `BUTTERWORTH` - Butterworth filter design ✅ NEW
- `HANNING` - Hanning window function ✅ NEW
- `HAMMING` - Hamming window function ✅ NEW
- `BLACKMAN` - Blackman window function ✅ NEW
- `SAVGOL` - Savitzky-Golay smoothing filter ✅ NEW
- `LEEFILT` - Lee filter for speckle noise ✅ NEW
- `WV_HAAR` - Haar wavelet transform ✅ NEW
- `WV_IHAAR` - Inverse Haar wavelet ✅ NEW
- `WV_DWT` - Discrete wavelet transform (Daubechies) ✅ NEW
- `POWER_SPECTRUM` - Power spectral density ✅ NEW

**Uses:** `rustfft` crate for FFT operations

---

### Phase 13: Image Processing ✅ 100%
**Status:** Complete

**Implemented Functions:**

**Edge Detection:**
- `SOBEL` - Sobel edge detection
- `ROBERTS` - Roberts cross edge detection
- `PREWITT` - Prewitt edge detection
- `CANNY` - Canny edge detection ✅ NEW
- `LAPLACIAN` - Laplacian edge detection ✅ NEW
- `EDGE_DOG` - Difference of Gaussians ✅ NEW

**Morphological Operations:**
- `DILATE` - Morphological dilation
- `ERODE` - Morphological erosion
- `MORPH_OPEN` - Morphological opening ✅ NEW
- `MORPH_CLOSE` - Morphological closing ✅ NEW

**Filtering:**
- `CONVOL` - 2D convolution
- `GAUSSIAN_FILTER` - Gaussian blur
- `THRESHOLD` - Binary thresholding
- `MEDIAN_2D` - 2D median filter ✅ NEW
- `HIST_EQUAL` - Histogram equalization ✅ NEW

**Transforms:**
- `HOUGH` - Hough transform for lines ✅ NEW
- `RADON` - Radon transform ✅ NEW

**Segmentation:**
- `WATERSHED` - Watershed segmentation ✅ NEW
- `LABEL_REGION` - Connected component labeling ✅ NEW

---

### Phase 14: Time & Date ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `SYSTIME` - System time
- `JULDAY` - Julian day number
- `CALDAT` - Calendar date from Julian
- `BIN_DATE` - Binary date/time array
- `TIMESTAMP` - Generate ISO 8601 timestamp ✅ NEW
- `TIMEGEN` - Generate time array ✅ NEW
- `DAYOFYEAR` - Day of year from date ✅ NEW
- `JS2JD` - Julian seconds to Julian date ✅ NEW
- `WEEKDAY` - Day of week from Julian date ✅ NEW
- `TIC` - Start timer
- `TOC` - Stop timer and report

---

### Phase 15: Type Conversion ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `BYTE`, `INT` (FIX), `LONG`
- `FLOAT` (FLT), `DOUBLE` (DBL)
- `UINT` - Unsigned 16-bit integer
- `ULONG` - Unsigned 32-bit integer
- `LONG64` - Signed 64-bit integer
- `ULONG64` - Unsigned 64-bit integer

**Pointer Management:** ✅ NEW
- `PTR_NEW` - Create heap pointer
- `PTR_VALID` - Check pointer validity
- `PTR_FREE` - Free pointer
- `PTR_DEREF` - Dereference pointer

**Object Management:** ✅ NEW
- `OBJ_NEW` - Create object instance
- `OBJ_VALID` - Check object validity
- `OBJ_DESTROY` - Destroy object
- `OBJ_CLASS` - Get object class name
- `OBJ_ISA` - Check object inheritance

---

### Phase 16: Data Structures ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `SIZE` - Variable size and type info
- `N_PARAMS` - Number of parameters
- `TAG_NAMES` - Structure field names
- `N_TAGS` - Number of structure tags
- `HASH` - Hash table creation

**Collections:** ✅ NEW
- `LIST` - Create list
- `LIST_ADD` - Add item to list
- `LIST_COUNT` - Get list length
- `ORDEREDHASH` - Ordered hash table
- `DICTIONARY` - Dictionary (alias for hash)

**Structures:** ✅ NEW
- `CREATE_STRUCT` - Create structure with fields
- `STRUCT_ASSIGN` - Assign values to structure

**Heap Management:** ✅ NEW
- `HEAP_GC` - Garbage collection
- `HEAP_FREE` - Free all heap memory

---

### Phase 17: Complex Numbers ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `COMPLEX` - Create complex number
- `DCOMPLEX` - Create double-precision complex ✅ NEW
- `REAL` - Real part
- `IMAGINARY` (IMAG) - Imaginary part
- `CONJ` - Complex conjugate
- `ARG` / `PHASE` - Phase/argument ✅ NEW
- `COMPLEXARR` - Complex array creation ✅ NEW
- `DCOMPLEXARR` - Double complex array ✅ NEW
- `POLAR` - Create from polar coordinates ✅ NEW
- `COMPLEX_EXP` - Complex exponential ✅ NEW
- `COMPLEX_LOG` - Complex logarithm ✅ NEW
- `COMPLEX_SQRT` - Complex square root ✅ NEW
- `COMPLEX_SIN` - Complex sine ✅ NEW
- `COMPLEX_COS` - Complex cosine ✅ NEW

---

### Phase 18: System & Control ✅ 100%
**Status:** Complete

**Implemented Functions:**
- `MESSAGE` - Print message/error ✅ NEW
- `ON_ERROR` - Error handling mode ✅ NEW
- `MEMORY` - Memory usage info ✅ NEW
- `EXIT` - Exit session ✅ NEW
- `STOP` - Halt execution
- `RETALL` - Return to top level ✅ NEW
- `ROUTINE_INFO` - Query routine information ✅ NEW
- `HELP` - Display help
- `CD` - Change directory
- `SPAWN` - Execute system commands
- `WAIT` - Pause execution
- `EXECUTE` - Execute string as command ✅ NEW
- `N_PARAMS` - Number of procedure parameters ✅ NEW

**Control Flow Statements (fully implemented):**
- `CONTINUE` - Continue to next loop iteration
- `BREAK` - Break out of loop
- Works in FOR, WHILE, REPEAT loops and CASE statements

**Note:** Advanced heap and scope management (HEAP_GC, SCOPE_*) deferred.

---

## Deferred/Skipped Phases

### Phase 10: Image I/O ✅ 100%
**Status:** Complete (requires `image-io` feature)

**Implemented Functions:**
- `READ_PNG`, `WRITE_PNG` - PNG format ✅ NEW
- `READ_JPEG`, `WRITE_JPEG` - JPEG format ✅ NEW
- `READ_TIFF`, `WRITE_TIFF` - TIFF format ✅ NEW
- `READ_BMP`, `WRITE_BMP` - BMP format ✅ NEW
- `READ_GIF`, `WRITE_GIF` - GIF format ✅ NEW
- `READ_IMAGE`, `WRITE_IMAGE` - Auto-detect format ✅ NEW
- `QUERY_IMAGE` - Get image dimensions ✅ NEW
- `TV`, `TVSCL` - Display image (placeholder) ✅ NEW

### Phase 19: Graphics 🔄
**Status:** Partially complete via existing graphics modules
**Note:** Core plotting already implemented in `graphics_procs.rs`

---

## Statistics

### Total Functions Implemented
- **Core Functions:** ~300+ (including 110+ newly implemented)
- **ML Functions:** 75+ (separate ML module with Linfa integration)
- **Graphics Procedures:** 50+ (separate graphics module)
- **GPU Backends:** 12 production-ready backends
- **Total:** 500+ functions/procedures

### Code Metrics
- **Lines of Code:** ~15,000+ (stdlib only)
- **Test Coverage:** Growing (unit tests in each module)
- **Build Status:** ✅ Clean build with all features

### Testing Infrastructure ✅
- **Comprehensive Test Suite**: 5 major test files covering all language features
- **MATLAB Compatibility Tests**: 28 unit tests for transpilation pipeline
- **MATLAB Execution Tests**: Direct .m file execution verification
- **Automated Test Runner**: `tests/test_all.sh` for comprehensive validation
- **Parser Fixes**: Resolved complex control flow syntax issues

---

## Next Steps

### High Priority
1. ✅ Complete remaining string functions (regex support)
2. ✅ Complete Phase 11 signal processing basics
3. ✅ Add more image processing filters
4. ✅ Extend linear algebra functions

### Medium Priority
1. Add full structure support for Phase 16
2. Implement pointer/object management (PTR_NEW, OBJ_NEW)
3. ✅ Complete time/date utilities
4. Add Phase 10 image I/O (with image crate integration)

### Low Priority
1. Extended graphics functions
2. ✅ Advanced statistical functions
3. Performance optimization passes

---

## Dependencies

### Core Dependencies
- `xdl-core` - Core types and error handling
- `nalgebra` - Linear algebra operations
- `num-complex` - Complex number support
- `rand` - Random number generation

### GPU Acceleration (xdl-amp)
- `mlx-rs` - Apple MLX bindings (macOS, optional, requires Xcode)
- `metal` - Apple Metal GPU (macOS)
- `cudarc` - NVIDIA CUDA (optional)
- `ash` - Vulkan (optional)

### Development Dependencies
- `cargo-test` - Unit testing
- `criterion` - Benchmarking (future)

---

## Testing Strategy

Each module includes:
- Unit tests for individual functions
- Integration tests for complex workflows
- Example scripts in `/examples`
- Test data in `/tests`

Run tests: `cargo test --all`

---

## Contributing

When adding new functions:
1. Add to appropriate module (math.rs, array.rs, etc.)
2. Register in `lib.rs` dispatch tables
3. Add unit tests
4. Update this status document
5. Run `cargo fmt --all` before committing

---

## Version History

- **v0.1.0** (2025-01) - Initial implementation with 220+ functions
- Phases 5-18 mostly complete
- Core mathematical, array, string, and I/O operations functional
- Linear algebra with nalgebra integration
- Time/date handling
- System control and error handling

- **v0.1.1** (2025-11) - GPU acceleration and MultiDimArray support
- Added GPU acceleration for MIN, MAX, MEAN, TOTAL functions (10-50x speedup)
- Extended statistical functions to support MultiDimArray types
- Improved CT visualization with proper windowing and normalization
- Added 3D volume visualization to medical imaging demo
- Added comprehensive user guides for medical and geophysical demos

- **v0.1.2** (2025-12) - OOP syntax, GPU acceleration, and keyword arguments
- Full Object-Oriented syntax support (`arr->Sum()`, `str->ToUpper()`, etc.)
- Added 17+ Array methods, 16+ String methods, 15+ DataFrame methods
- GPU acceleration for MEDIAN, VARIANCE, STDDEV (SIMD-optimized)
- STREGEX regular expression support complete
- CONTINUE/BREAK control flow statements complete
- Function keyword arguments (`NAME=value`, `/FLAG` syntax)
- MAKE_ARRAY function with DIMENSION, VALUE, /INDEX keywords
- All array generation functions support START/INCREMENT keywords
- Documentation updates for 70+ new functions

- **v0.1.3** (2025-12) - Major stdlib expansion: 80+ new functions
- Phase 6 Mathematics: PRIME, PRIMES, BINOMIAL, GCD, LCM, BETA, DERIV, INT_TABULATED, POLY, PRODUCT, MACHAR, CHECK_MATH, FINITE
- Phase 7 Statistics: R_CORRELATE, LADFIT, SVDFIT, CURVEFIT, PERCENTILES, ROBUST_MEAN, TRIMMED_MEAN, RESISTANT_MEAN, RANDOM_POISSON
- Phase 9 File I/O: FILE_EXPAND_PATH, FILE_SAME, FILE_CHMOD, FINDFILE, FILE_BASENAME, FILE_DIRNAME, FILE_LINES, POINT_LUN, READU, WRITEU, ASSOC
- Phase 11 Signal: FFT_2D, HANNING, HAMMING, BLACKMAN, BUTTERWORTH, SAVGOL, LEEFILT, WV_HAAR, WV_IHAAR, WV_DWT, POWER_SPECTRUM
- Phase 12 Linear Algebra: LA_EIGENVEC, LA_LINEAR_EQUATION, LA_LEAST_SQUARES, LA_CHOLESKY, LA_TRIDC, QR, RANK, CRAMER, MATRIX_MULTIPLY, COND, PINV
- Phase 13 Image: CANNY, HOUGH, RADON, WATERSHED, LABEL_REGION, MORPH_OPEN, MORPH_CLOSE, HIST_EQUAL, EDGE_DOG, LAPLACIAN, MEDIAN_2D
- Phase 14 Time: WEEKDAY, BIN_DATE, TIMESTAMP, TIMEGEN, DAYOFYEAR, JS2JD
- Phase 17 Complex: DCOMPLEX, COMPLEXARR, DCOMPLEXARR, ARG, POLAR, COMPLEX_EXP/LOG/SQRT/SIN/COS
- Phase 18 System: MEMORY, EXIT, RETALL, ROUTINE_INFO, MESSAGE, ON_ERROR, EXECUTE, N_PARAMS

- **v0.1.4** (2025-12) - Complete stdlib implementation: 30+ additional functions
- Phase 10 Image I/O: READ_PNG, WRITE_PNG, READ_JPEG, WRITE_JPEG, READ_TIFF, WRITE_TIFF, READ_BMP, WRITE_BMP, READ_GIF, WRITE_GIF, READ_IMAGE, WRITE_IMAGE, QUERY_IMAGE, TV, TVSCL
- Phase 15 Pointer/Object: PTR_NEW, PTR_VALID, PTR_FREE, PTR_DEREF, OBJ_NEW, OBJ_VALID, OBJ_DESTROY, OBJ_CLASS, OBJ_ISA
- Phase 16 Data Structures: LIST, LIST_ADD, LIST_COUNT, ORDEREDHASH, DICTIONARY, CREATE_STRUCT, STRUCT_ASSIGN, HEAP_GC, HEAP_FREE
- Added comprehensive unit test suite (29 tests covering all new functions)

- **v0.1.5** (2025-12) - Apple MLX backend for xdl-amp
- Added Apple MLX backend with unified memory architecture
- MLX base operations: add, sub, mul, div, pow, matmul, sin, cos, exp, log, sqrt
- MLX reductions: sum, max, min, median, variance, stddev
- MLX extended operations (new capabilities):
  - FFT: fft_1d, ifft_1d, fft_2d, rfft_1d
  - Linear algebra: qr, svd, cholesky, inv, solve, eigh, norm
  - Activations: sigmoid, softmax, tanh, erf
- Updated backend priority on macOS: MLX > MPS > Metal > CoreML
- Requires full Xcode installation (not Command Line Tools)
- **Performance benchmark**: MLX up to 1517x faster than CPU, 4.4x faster than Metal for large matrix multiplication

- **v0.1.6** (2025-12) - OpenCL and DirectX 12 GPU backends
- OpenCL backend fully implemented with complete kernel support
- DirectX 12 backend via DirectML delegation
- All 12 GPU backends now production-ready
- Vulkan backend with compute shader support
- Updated GPU documentation for all platforms

- **v0.1.7** (2025-12) - Documentation consolidation and updates
- Updated all GPU documentation for implemented backends
- Fixed ML documentation formatting issues
- Consolidated documentation structure
- Total functions now exceeds 500+

---

## References

- IDL Documentation: https://www.l3harrisgeospatial.com/docs/routines.html
- GDL Project: https://github.com/gnudatalanguage/gdl
- nalgebra: https://nalgebra.org/
- Apple MLX: https://github.com/ml-explore/mlx
- mlx-rs (Rust bindings): https://github.com/oxideai/mlx-rs
