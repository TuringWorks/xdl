# Changelog

All notable changes to the XDL project are documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.5] - 2025-12-30

### Added
- **Apple MLX Backend** for xdl-amp GPU acceleration
  - Unified memory architecture (no CPU/GPU transfers)
  - Lazy evaluation with JIT compilation
  - Optimized for Apple Silicon (M1/M2/M3/M4)
  - FFT operations: fft_1d, ifft_1d, fft_2d, rfft_1d
  - Linear algebra: qr, svd, cholesky, inv, solve, eigh, norm
  - Activations: sigmoid, softmax, tanh, erf
- **MLX Performance Benchmark** (`examples/mlx_benchmark.rs`)
  - MLX achieves up to 1517x speedup over CPU for large matrix multiplication
  - MLX is 4.4x faster than Metal for matmul operations
- Updated documentation with MLX benchmark results

### Changed
- Updated backend priority on macOS: MLX > MPS > Metal > CoreML
- MLXOps is standalone (not implementing GpuDevice trait due to thread safety)

### Fixed
- Fixed mlx-rs API compatibility for v0.25.3
- Fixed FFT function axis parameters

## [0.1.4] - 2025-12-29

### Added
- **Phase 10: Image I/O** - Complete implementation
  - READ_PNG, WRITE_PNG - PNG format support
  - READ_JPEG, WRITE_JPEG - JPEG format support
  - READ_TIFF, WRITE_TIFF - TIFF format support
  - READ_BMP, WRITE_BMP - BMP format support
  - READ_GIF, WRITE_GIF - GIF format support
  - READ_IMAGE, WRITE_IMAGE - Auto-detect format
  - QUERY_IMAGE - Get image dimensions
  - TV, TVSCL - Display image (placeholder)
- **Phase 15: Pointer/Object Management**
  - PTR_NEW, PTR_VALID, PTR_FREE, PTR_DEREF
  - OBJ_NEW, OBJ_VALID, OBJ_DESTROY, OBJ_CLASS, OBJ_ISA
- **Phase 16: Data Structures**
  - LIST, LIST_ADD, LIST_COUNT
  - ORDEREDHASH, DICTIONARY
  - CREATE_STRUCT, STRUCT_ASSIGN
  - HEAP_GC, HEAP_FREE
- Comprehensive unit test suite (29 tests)

## [0.1.3] - 2025-12-28

### Added
- **80+ new stdlib functions** across multiple phases:
  - Phase 6 Mathematics: PRIME, PRIMES, BINOMIAL, GCD, LCM, BETA, DERIV, INT_TABULATED, POLY, PRODUCT, MACHAR, CHECK_MATH, FINITE
  - Phase 7 Statistics: R_CORRELATE, LADFIT, SVDFIT, CURVEFIT, PERCENTILES, ROBUST_MEAN, TRIMMED_MEAN, RESISTANT_MEAN, RANDOM_POISSON
  - Phase 9 File I/O: FILE_EXPAND_PATH, FILE_SAME, FILE_CHMOD, FINDFILE, FILE_BASENAME, FILE_DIRNAME, FILE_LINES, POINT_LUN, READU, WRITEU, ASSOC
  - Phase 11 Signal: FFT_2D, HANNING, HAMMING, BLACKMAN, BUTTERWORTH, SAVGOL, LEEFILT, WV_HAAR, WV_IHAAR, WV_DWT, POWER_SPECTRUM
  - Phase 12 Linear Algebra: LA_EIGENVEC, LA_LINEAR_EQUATION, LA_LEAST_SQUARES, LA_CHOLESKY, LA_TRIDC, QR, RANK, CRAMER, MATRIX_MULTIPLY, COND, PINV
  - Phase 13 Image: CANNY, HOUGH, RADON, WATERSHED, LABEL_REGION, MORPH_OPEN, MORPH_CLOSE, HIST_EQUAL, EDGE_DOG, LAPLACIAN, MEDIAN_2D
  - Phase 14 Time: WEEKDAY, BIN_DATE, TIMESTAMP, TIMEGEN, DAYOFYEAR, JS2JD
  - Phase 17 Complex: DCOMPLEX, COMPLEXARR, DCOMPLEXARR, ARG, POLAR, COMPLEX_EXP/LOG/SQRT/SIN/COS
  - Phase 18 System: MEMORY, EXIT, RETALL, ROUTINE_INFO, MESSAGE, ON_ERROR, EXECUTE, N_PARAMS

## [0.1.2] - 2025-12-27

### Added
- **Object-Oriented Syntax Support**
  - Array methods: `arr->Sum()`, `arr->Mean()`, `arr->Sort()`, etc. (17+ methods)
  - String methods: `str->ToUpper()`, `str->Contains()`, etc. (16+ methods)
  - DataFrame methods: `df->Head()`, `df->Column()`, etc. (15+ methods)
  - Struct field access: `point.x`, `point.y`
- **GPU Acceleration** for statistical functions
  - MEDIAN, VARIANCE, STDDEV - SIMD-optimized with parallel execution
- **Function Keyword Arguments**
  - `NAME=value` and `/FLAG` syntax in function calls
  - All array generation functions support START/INCREMENT keywords
  - MAKE_ARRAY with DIMENSION, VALUE, /INDEX keywords
- STREGEX regular expression support
- CONTINUE/BREAK control flow statements

## [0.1.1] - 2025-11

### Added
- **GPU Acceleration** via xdl-amp
  - MIN, MAX, MEAN, TOTAL - 10-50x speedup for large arrays
  - Smart dispatch: automatic CPU/GPU selection based on array size
- Extended statistical functions for MultiDimArray types
- 3D volume visualization for medical imaging demo
- Comprehensive user guides for medical and geophysical demos

### Improved
- CT visualization with proper windowing and normalization

## [0.1.0] - 2025-01

### Added
- Initial XDL implementation with 220+ functions
- **Core Language Features**
  - Variables, arrays, control flow (IF/THEN/ELSE, FOR, WHILE, REPEAT)
  - Functions and procedures
  - Expression evaluation
- **Phase 5: Array Manipulation** - 100% complete
  - ARRAY_INDICES, ARRAY_EQUAL, UNIQ, PERMUTE, CONGRID
  - INTERPOL, WHERE, N_ELEMENTS, REFORM, TRANSPOSE
  - ROTATE, SHIFT, REBIN, REPLICATE, HISTOGRAM, MESHGRID
  - REVERSE, SORT
- **Phase 6: Mathematics** - 100% complete
  - Trigonometric, exponential, logarithmic functions
  - Special functions (GAMMA, ERF, BESSEL)
  - Array generation (FINDGEN, INDGEN, RANDOMU, RANDOMN)
- **Phase 7: Statistics** - 100% complete
  - VARIANCE, STDDEV, MEDIAN, MOMENT
  - CORRELATE, REGRESS, LINFIT, POLY_FIT
  - Probability distributions
- **Phase 8: String Operations** - 100% complete
  - All string manipulation functions
- **Phase 9: File I/O** - 100% complete
  - File operations, LUN management
- **Phase 11: Signal Processing** - 100% complete
  - FFT, convolution, filtering
- **Phase 12: Linear Algebra** - 100% complete
  - Matrix operations, decompositions, eigenvalues
- **Phase 13: Image Processing** - 100% complete
  - Edge detection, morphological operations
- **Phase 14: Time & Date** - 100% complete
- **MATLAB Compatibility**
  - MATLAB transpiler (28/28 tests passing)
  - ~80 MATLAB functions mapped
- **Python Integration**
  - Python 3.13 support via PyO3
- **Graphics System**
  - 2D plotting (PLOT, CONTOUR)
  - 3D visualization (SURFACE, SHADE_SURF)
  - ECharts integration for web rendering

### Dependencies
- nalgebra for linear algebra
- rustfft for FFT operations
- image crate for image I/O
- polars for DataFrames

---

## Pending Features

See [TODO.md](TODO.md) for detailed pending items:

### Critical
- User-defined procedures (PRO/ENDPRO)
- GOTO statements
- Complete CASE/SWITCH statements

### High Priority
- Scientific data formats (FITS, HDF5, NetCDF)
- Widget/GUI system enhancements

### Medium Priority
- Map projections
- Advanced 3D visualization (isosurfaces, volume rendering)

---

## Links

- [Implementation Status](docs/IMPLEMENTATION_STATUS.md)
- [Gap Analysis](docs/GDL_XDL_GAP_ANALYSIS.md)
- [Function Reference](docs/FUNCTION_REFERENCE.md)
