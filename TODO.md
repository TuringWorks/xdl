# XDL Implementation TODO

**Last Updated:** 2025-12-30

This document tracks remaining IDL/GDL features that need to be implemented in XDL for full compatibility.

## Current Status

| Category | Status | Coverage |
|----------|--------|----------|
| Core Functions | 280+ implemented | ~87% |
| ML Functions | 60+ implemented | ~86% |
| Graphics | 50+ implemented | ~71% |
| Widget/GUI | 12 implemented | Placeholder |
| Scientific I/O | 11 implemented | Placeholder |
| **Total** | **400+ functions** | **~86%** |

## Priority Legend
- 🔴 **CRITICAL** - Fundamental language features required for basic programs
- 🟠 **HIGH** - Important features used in most IDL programs
- 🟡 **MEDIUM** - Commonly used but not essential
- 🟢 **LOW** - Nice to have, less frequently used

---

## ✅ Recently Implemented (Critical Features)

### 1. User-Defined Procedures (PRO/ENDPRO) ✅
**Status**: IMPLEMENTED (2025-12-30)
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
PRO my_procedure, arg1, arg2, KEYWORD=keyword
  ; procedure body
  PRINT, arg1, arg2
END
```

**Completed**:
- [x] Parser for PRO/ENDPRO blocks (accepts both END and ENDPRO)
- [x] Procedure storage in Context
- [x] Procedure call evaluation
- [x] Positional and keyword arguments
- [x] Tests in examples/tests/pro_endpro_test.xdl

### 2. User-Defined Functions (FUNCTION/ENDFUNCTION) ✅
**Status**: IMPLEMENTED (2025-12-30)
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
FUNCTION add_values, a, b
  RETURN, a + b
END
```

**Completed**:
- [x] Parser for FUNCTION/ENDFUNCTION blocks
- [x] Function storage in Context
- [x] Function call evaluation with return values
- [x] Nested function calls in expressions
- [x] Tests in examples/tests/function_test.xdl

---

## 🔴 CRITICAL Priority

### 3. GOTO Statements ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: Medium
**Files**: `xdl-parser/src/lexer.rs`, `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
label1:
  ; code
  GOTO, label2
label2:
  ; more code
```

**Completed**:
- [x] Add label parsing in lexer
- [x] Add GOTO statement parsing
- [x] Implement label resolution in evaluator with label map
- [x] Handle forward references
- [x] Add tests in examples/tests/goto_test.xdl

### 4. CASE/SWITCH Statements ✅
**Status**: FULLY IMPLEMENTED (2025-12-30)
**Effort**: Medium
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

**Completed**:
- [x] Complete CASE parser (no fallthrough)
- [x] Complete SWITCH parser
- [x] Implement ELSE clause handling
- [x] Support BEGIN/END blocks in cases
- [x] Support multiple values per case (comma-separated)
- [x] Add tests in examples/tests/case_switch_test.xdl

---

## 🟠 HIGH Priority

### 4. Scientific Data Formats ✅ (Placeholder)
**Status**: PLACEHOLDER IMPLEMENTATION (2025-12-30)
**Effort**: High
**Files**: `xdl-stdlib/src/scientific_io.rs`

**Note**: These functions provide API compatibility and file format validation.
Full parsing requires native libraries (cfitsio, hdf5, netcdf).

**Completed**:
- [x] FITS support (placeholder)
  - [x] `READFITS(filename)` - Validates FITS signature
  - [x] `WRITEFITS, filename, data, [header]` - Placeholder
  - [x] `HEADFITS(filename)` - Reads FITS header block (2880 bytes)
  - [x] `SXPAR(header, keyword)` - Extracts header keyword values
- [x] HDF5 support (placeholder)
  - [x] `H5F_OPEN` - Validates HDF5 signature
  - [x] `H5F_CLOSE` - Close file handle
  - [x] `H5D_READ` - Placeholder
- [x] NetCDF support (placeholder)
  - [x] `NCDF_OPEN` - Validates NetCDF-3/NetCDF-4 signature
  - [x] `NCDF_CLOSE` - Close file handle
  - [x] `NCDF_VARGET` - Placeholder
  - [x] `NCDF_INQUIRE` - Returns placeholder structure
- [x] Tests in examples/tests/scientific_io_test.xdl

**For full support**: Compile with `--features fits,hdf5,netcdf`

### 5. Widget/GUI System Enhancement ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: Very High
**Files**: `xdl-stdlib/src/widget.rs`

**Note**: CLI-based placeholder implementation providing full API compatibility.
Full GUI support requires native windowing backend (xdl-gui crate).

**Completed**:
- [x] `WIDGET_BASE` - Container widget with row/column/grid layouts
- [x] `WIDGET_BUTTON` - Button with text, bitmap, menu options
- [x] `WIDGET_SLIDER` - Slider with min/max/value
- [x] `WIDGET_TEXT` - Text input with editable, multiline options
- [x] `WIDGET_LABEL` - Text labels with alignment
- [x] `WIDGET_LIST` - List selection (single/multiple)
- [x] `WIDGET_DROPLIST` - Dropdown selection
- [x] `WIDGET_DRAW` - Drawing canvas for graphics
- [x] `WIDGET_CONTROL` - Runtime property modification
- [x] `WIDGET_INFO` - Query widget properties
- [x] `WIDGET_EVENT` - Event polling (placeholder)
- [x] `XMANAGER` - Event loop management
- [x] Tests in examples/tests/widget_test.xdl

---

## 🟡 MEDIUM Priority

### 6. Map Projections ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: High
**Files**: `xdl-stdlib/src/map.rs`

**Completed**:
- [x] `MAP_SET` - Set up map projection with keywords
- [x] `MAP_CONTINENTS` - Draw continent outlines (placeholder data)
- [x] `MAP_GRID` - Draw coordinate grid
- [x] `CONVERT_COORD` - Convert geographic to map coordinates
- [x] `MAP_STRUCT` - Get current projection info
- [x] Support common projections:
  - Cylindrical (Plate Carrée)
  - Mercator
  - Lambert Conic
  - Orthographic
  - Stereographic
  - Gnomonic
  - Sinusoidal
  - Mollweide
  - Hammer-Aitoff
  - Satellite
- [x] Tests in examples/tests/map_projection_test.xdl

**Note**: Full continent data requires external datasets (Natural Earth, GSHHG)

### 7. Advanced 3D Visualization ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: High
**Files**: `xdl-stdlib/src/viz3d_advanced.rs`

**Completed**:
- [x] `ISOSURFACE` - Isosurface extraction using marching cubes
- [x] `SHADE_VOLUME` - Maximum intensity projection volume rendering
- [x] `PARTICLE_TRACE` - Particle tracing with RK4 integration
- [x] `STREAMLINE` - Streamline visualization with adaptive step size
- [x] `VOXEL_PROJ` - Volume projection (maximum/average)
- [x] `POLYSHADE` - Mesh shading with vertex normals
- [x] Tests in examples/tests/viz3d_advanced_test.xdl

### 8. Additional String Functions
**Status**: Most implemented, some edge cases remain
**Effort**: Low
**Files**: `xdl-stdlib/src/string.rs`

**Tasks**:
- [ ] Enhance STREGEX with all IDL options
- [ ] Add READS (read from string)
- [ ] Improve FORMAT string handling
- [ ] Add tests

---

## 🟢 LOW Priority

### 9. Dialog Functions ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: Medium
**Files**: `xdl-stdlib/src/dialog.rs`

**Completed**:
- [x] `DIALOG_MESSAGE` - Display message dialog (CLI fallback)
- [x] `DIALOG_PICKFILE` - File picker dialog (CLI prompt)
- [x] `DIALOG_PRINTERSETUP` - Printer setup (placeholder)
- [x] `DIALOG_READ_TEXT` - Text input dialog (CLI prompt)
- Note: CLI-based prompts; GUI integration pending

### 10. Advanced Scope Functions ✅
**Status**: IMPLEMENTED (2025-12-30)
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`

**Completed**:
- [x] `SCOPE_VARNAME` - Variable names in scope (placeholder)
- [x] `SCOPE_LEVEL` - Current scope level
- [x] `SCOPE_TRACEBACK` - Call stack trace
- [x] Tests in examples/tests/scope_test.xdl

### 11. Object System Enhancement
**Status**: Basic OOP support exists
**Effort**: High
**Files**: `xdl-core/src/types.rs`, `xdl-interpreter/`

**Tasks**:
- [ ] Full class inheritance
- [ ] Method overriding
- [ ] Property getters/setters
- [ ] Class methods vs instance methods
- [ ] Add tests

---

## Recently Completed ✅

The following features have been implemented and are no longer pending:

### Array Functions (All Complete)
- ✅ REFORM, TRANSPOSE, REPLICATE, REBIN, REVERSE, SHIFT, ROTATE
- ✅ ARRAY_INDICES, ARRAY_EQUAL, UNIQ, HISTOGRAM, WHERE
- ✅ CONGRID, PERMUTE, INTERPOL, MESHGRID

### String Functions (All Complete)
- ✅ STRSPLIT, STRJOIN, STRMATCH, STREGEX
- ✅ STRCOMPRESS, STRTRIM, STRCMP, STRREPLACE

### Mathematical Functions (All Complete)
- ✅ CONVOL, SMOOTH, MEDIAN filter
- ✅ DERIV, INT_TABULATED
- ✅ All special functions (GAMMA, ERF, BESSEL, etc.)

### Pointer/Object Operations (All Complete)
- ✅ PTR_NEW, PTR_VALID, PTR_FREE, PTR_DEREF
- ✅ OBJ_NEW, OBJ_VALID, OBJ_DESTROY, OBJ_CLASS, OBJ_ISA

### Data Structures (All Complete)
- ✅ CREATE_STRUCT, TAG_NAMES, N_TAGS
- ✅ HASH, LIST, ORDEREDHASH, DICTIONARY
- ✅ HEAP_GC, HEAP_FREE

### Type Functions (All Complete)
- ✅ SIZE with all keywords
- ✅ N_ELEMENTS, N_DIMS, TYPENAME
- ✅ All type conversions (BYTE through ULONG64)

### File I/O (All Complete)
- ✅ All file operations (READ*, WRITE*, OPEN*, etc.)
- ✅ All FILE_* utility functions

### Signal Processing (All Complete)
- ✅ FFT, FFT_2D
- ✅ All window functions (HANNING, HAMMING, BLACKMAN)
- ✅ All filters (BUTTERWORTH, SAVGOL, LEEFILT)
- ✅ Wavelets (WV_HAAR, WV_DWT)

### Image Processing (All Complete)
- ✅ All edge detection (SOBEL, CANNY, etc.)
- ✅ All morphological operations
- ✅ HOUGH, RADON transforms
- ✅ WATERSHED, LABEL_REGION

### Linear Algebra (All Complete)
- ✅ All decompositions (SVD, LU, QR, Cholesky)
- ✅ All eigenvalue functions
- ✅ Matrix operations (INVERT, DETERM, TRACE, etc.)

### Statistics (All Complete)
- ✅ All fitting functions (CURVEFIT, POLY_FIT, etc.)
- ✅ All correlation functions
- ✅ All distribution functions

---

## Implementation Recommendations

### Phase 1: Critical Language Features
1. User-defined procedures (PRO/ENDPRO) - **Highest priority**
2. Complete CASE/SWITCH statements
3. GOTO statements (if needed for legacy code)

### Phase 2: Scientific Data Formats
4. FITS file support
5. HDF5 file support
6. NetCDF file support

### Phase 3: Advanced Visualization
7. Map projections
8. Isosurface rendering
9. Volume visualization

### Phase 4: GUI and Dialogs
10. Widget system enhancements
11. Dialog functions
12. Advanced scope functions

---

## Testing Strategy

For each feature:
1. Create test file in `examples/tests/feature_name_test.xdl`
2. Add unit tests in Rust test modules
3. Add integration tests comparing with GDL/IDL output
4. Document in user guide

---

## Notes

- PRO/ENDPRO is the most critical missing feature for real-world IDL code compatibility
- Scientific data formats (FITS, HDF5, NetCDF) are important for astronomy/science users
- Widget system is extensive - prioritize based on user needs
- Consider SIMD optimizations for any new array operations
- Maintain IDL/GDL compatibility as primary goal
