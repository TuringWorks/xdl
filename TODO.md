# XDL Implementation TODO

**Last Updated:** 2025-12-30

This document tracks remaining IDL/GDL features that need to be implemented in XDL for full compatibility.

## Current Status

| Category | Status | Coverage |
|----------|--------|----------|
| Core Functions | 250+ implemented | ~83% |
| ML Functions | 60+ implemented | ~86% |
| Graphics | 50+ implemented | ~71% |
| **Total** | **360+ functions** | **~82%** |

## Priority Legend
- 🔴 **CRITICAL** - Fundamental language features required for basic programs
- 🟠 **HIGH** - Important features used in most IDL programs
- 🟡 **MEDIUM** - Commonly used but not essential
- 🟢 **LOW** - Nice to have, less frequently used

---

## 🔴 CRITICAL Priority

### 1. User-Defined Procedures (PRO/ENDPRO)
**Status**: Not implemented - Most critical missing feature
**Effort**: High
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
PRO my_procedure, arg1, arg2, KEYWORD=keyword
  ; procedure body
  PRINT, arg1, arg2
END
```

**Tasks**:
- [ ] Add AST node: `Statement::Procedure`
- [ ] Implement parser for PRO/ENDPRO blocks
- [ ] Implement procedure storage in Context
- [ ] Implement procedure call evaluation
- [ ] Support keyword arguments in procedures
- [ ] Add tests

**Impact**: Cannot define reusable procedures - major limitation for real IDL/GDL code

### 2. GOTO Statements
**Status**: Not implemented
**Effort**: Medium
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
label1:
  ; code
  GOTO, label2
label2:
  ; more code
```

**Tasks**:
- [ ] Add label parsing
- [ ] Add GOTO statement parsing
- [ ] Implement label resolution in evaluator
- [ ] Handle forward references
- [ ] Add tests

### 3. CASE/SWITCH Statements (Enhancement)
**Status**: Tokens exist, limited implementation
**Effort**: Medium
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

**Tasks**:
- [ ] Complete CASE parser (no fallthrough)
- [ ] Complete SWITCH parser (with fallthrough)
- [ ] Implement ELSE clause handling
- [ ] Support BEGIN/END blocks in cases
- [ ] Add tests

---

## 🟠 HIGH Priority

### 4. Scientific Data Formats
**Status**: Modules exist in xdl-ffi but not integrated
**Effort**: High
**Files**: `xdl-ffi/src/fits.rs`, `xdl-ffi/src/hdf5.rs`, `xdl-ffi/src/netcdf.rs`

**Tasks**:
- [ ] FITS support
  - [ ] `READFITS(filename)` - Read FITS file
  - [ ] `WRITEFITS, filename, data, [header]` - Write FITS file
  - [ ] `HEADFITS(filename)` - Read FITS header
  - [ ] `SXPAR(header, keyword)` - Extract header keyword
- [ ] HDF5 support
  - [ ] `H5F_OPEN/H5F_CREATE` - Open/create HDF5 file
  - [ ] `H5D_READ/H5D_WRITE` - Read/write datasets
- [ ] NetCDF support
  - [ ] `NCDF_OPEN/NCDF_CREATE` - Open/create files
  - [ ] `NCDF_VARGET/NCDF_VARPUT` - Read/write variables

### 5. Widget/GUI System Enhancement
**Status**: Basic implementation exists
**Effort**: Very High
**Files**: `xdl-gui/`

**Tasks**:
- [ ] Complete WIDGET_BASE with all options
- [ ] Add WIDGET_BUTTON, WIDGET_SLIDER, WIDGET_TEXT
- [ ] Implement WIDGET_CONTROL for runtime modification
- [ ] Add event handling via XMANAGER
- [ ] Add WIDGET_DRAW for graphics
- [ ] Add tests

---

## 🟡 MEDIUM Priority

### 6. Map Projections
**Status**: Not implemented
**Effort**: High
**Files**: Create `xdl-stdlib/src/map.rs`

**Tasks**:
- [ ] `MAP_SET` - Set up map projection
- [ ] `MAP_CONTINENTS` - Draw continent outlines
- [ ] `MAP_GRID` - Draw coordinate grid
- [ ] Support common projections (Mercator, Lambert, etc.)
- [ ] Add tests

### 7. Advanced 3D Visualization
**Status**: Basic 3D exists, advanced features missing
**Effort**: High
**Files**: `xdl-viz3d/`

**Tasks**:
- [ ] `ISOSURFACE` - Isosurface extraction
- [ ] `SHADE_VOLUME` - Volume rendering
- [ ] `PARTICLE_TRACE` - Particle tracing
- [ ] `STREAMLINE` - Streamline visualization
- [ ] Add tests

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

### 9. Dialog Functions
**Status**: Not implemented
**Effort**: Medium
**Files**: Create `xdl-stdlib/src/dialog.rs`

**Tasks**:
- [ ] `DIALOG_MESSAGE` - Display message dialog
- [ ] `DIALOG_PICKFILE` - File picker dialog
- [ ] `DIALOG_PRINTERSETUP` - Printer setup
- [ ] Add tests

### 10. Advanced Scope Functions
**Status**: Basic scope exists
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`

**Tasks**:
- [ ] `SCOPE_VARNAME` - Variable names in scope
- [ ] `SCOPE_LEVEL` - Current scope level
- [ ] `SCOPE_TRACEBACK` - Call stack trace
- [ ] Add tests

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
