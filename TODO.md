# XDL Implementation TODO

This document tracks missing IDL/GDL features that need to be implemented in XDL for full compatibility.

## Priority Legend
- 🔴 **CRITICAL** - Fundamental language features required for basic programs
- 🟠 **HIGH** - Important features used in most IDL programs
- 🟡 **MEDIUM** - Commonly used but not essential
- 🟢 **LOW** - Nice to have, less frequently used

---

## 🔴 CRITICAL Priority

### 1. CASE/SWITCH Control Structures
**Status**: Tokens exist, no parser/evaluator implementation
**Effort**: Medium
**Files**: `xdl-parser/src/parser.rs`, `xdl-interpreter/src/lib.rs`

IDL syntax:
```idl
CASE variable OF
  1: statement1
  2: statement2
  ELSE: default_statement
ENDCASE

SWITCH variable OF
  1: BEGIN
    statement1
    ; Falls through to next case
  END
  2: statement2
  ELSE: default_statement
ENDSWITCH
```

**Tasks**:
- [ ] Add AST nodes: `Statement::Case` and `Statement::Switch`
- [ ] Implement parser for CASE statement
- [ ] Implement parser for SWITCH statement
- [ ] Implement evaluator for CASE (no fallthrough)
- [ ] Implement evaluator for SWITCH (with fallthrough)
- [ ] Add tests

### 2. Pointer Operations
**Status**: Pointer type exists, no operations
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`, `xdl-core/src/types.rs`

**Tasks**:
- [ ] Implement `PTR_NEW([value])` - Create pointer
- [ ] Implement `PTR_VALID(ptr)` - Check if pointer valid
- [ ] Implement `PTR_FREE, ptr` - Free pointer memory
- [ ] Add pointer storage to Context (similar to objects)
- [ ] Implement dereference operator `*ptr`
- [ ] Implement pointer assignment `*ptr = value`
- [ ] Add garbage collection/reference counting
- [ ] Add tests

### 3. Critical Array Functions
**Status**: Some exist, many missing
**Effort**: Medium-High
**Files**: `xdl-stdlib/src/array.rs`

**Tasks**:
- [ ] Implement `REFORM(array, dims)` - Reshape without copying
- [ ] Implement `TRANSPOSE(array, [permutation])` - Matrix transpose
- [ ] Implement `REPLICATE(value, dims)` - Create filled array
- [ ] Implement `REBIN(array, dims, [SAMPLE=])` - Resize with interpolation
- [ ] Implement `REVERSE(array, [dim])` - Reverse array order
- [ ] Implement `SHIFT(array, shift)` - Circular shift
- [ ] Implement `ROTATE(array, direction)` - Rotate 90°
- [ ] Add tests for each function

---

## 🟠 HIGH Priority

### 4. File I/O Improvements
**Status**: Basic I/O exists, missing formats
**Effort**: High
**Files**: `xdl-stdlib/src/io.rs`, create `xdl-fits/`, `xdl-hdf5/`

**Tasks**:
- [ ] Implement FITS file support
  - [ ] `READFITS(filename)` - Read FITS file
  - [ ] `WRITEFITS, filename, data, [header]` - Write FITS file
  - [ ] `HEADFITS(filename)` - Read FITS header
  - [ ] `SXPAR(header, keyword)` - Extract header keyword
- [ ] Implement binary file I/O with structures
  - [ ] `READU, unit, structure` - Read binary struct
  - [ ] `WRITEU, unit, structure` - Write binary struct
- [ ] Add HDF5 support (optional module)
  - [ ] `H5F_OPEN/H5F_CREATE` - Open/create HDF5 file
  - [ ] `H5D_READ/H5D_WRITE` - Read/write datasets
- [ ] Add NetCDF support (optional module)
- [ ] Add tests

### 5. Structure Functions
**Status**: Basic struct support exists
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`

**Tasks**:
- [ ] Implement `CREATE_STRUCT(name, tag1, value1, ...)` - Dynamic struct creation
- [ ] Implement `TAG_NAMES(structure, [STRUCTURE_NAME=])` - Get field names
- [ ] Implement `N_TAGS(structure)` - Count fields
- [ ] Implement structure array operations
- [ ] Add anonymous structure support: `{field1: value1, field2: value2}`
- [ ] Add tests

### 6. Type Inquiry Functions
**Status**: Partially implemented
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`

**Tasks**:
- [ ] Enhance `SIZE(variable, [/DIMENSIONS, /N_DIMENSIONS, /N_ELEMENTS, /TNAME, /TYPE])`
- [ ] Implement `N_ELEMENTS(variable)` - Count total elements
- [ ] Implement `N_DIMS(variable)` - Get number of dimensions
- [ ] Implement `TYPENAME(variable)` - Get type as string
- [ ] Add tests

### 7. Advanced Array Functions
**Status**: Missing
**Effort**: High
**Files**: `xdl-stdlib/src/array.rs`

**Tasks**:
- [ ] Implement `CONGRID(array, dims, [/INTERP, /CUBIC])` - Smart resize
- [ ] Implement `HISTOGRAM(array, [binsize=, min=, max=])` - Compute histogram
- [ ] Implement `ARRAY_INDICES(array, indices, [/DIMENSIONS])` - Convert 1D to ND indices
- [ ] Implement `ARRAY_EQUAL(a, b)` - Deep array comparison
- [ ] Implement `UNIQ(array, [sort_indices])` - Find unique elements
- [ ] Add tests

---

## 🟡 MEDIUM Priority

### 8. String Processing
**Status**: Basic functions exist
**Effort**: Medium
**Files**: `xdl-stdlib/src/string.rs`

**Tasks**:
- [ ] Implement `STRSPLIT(string, pattern, [/EXTRACT, /REGEX])` - Split string
- [ ] Implement `STRJOIN(array, [delimiter])` - Join strings
- [ ] Implement `STRMATCH(string, pattern, [/FOLD_CASE])` - Wildcard matching
- [ ] Implement `STREGEX(string, regex, [/EXTRACT, /SUBEXPR])` - Regex matching
- [ ] Implement `STRCOMPRESS(string, [/REMOVE_ALL])` - Remove whitespace
- [ ] Implement `STRTRIM(string, [flag])` - Trim whitespace
- [ ] Implement `STRCMP(str1, str2, [n], [/FOLD_CASE])` - Compare strings
- [ ] Add tests

### 9. Mathematical Functions
**Status**: Many implemented
**Effort**: Medium-High
**Files**: `xdl-stdlib/src/math.rs`, `xdl-stdlib/src/signal.rs`

**Tasks**:
- [ ] Implement `INTERPOLATE(array, x, [y], [/GRID, /CUBIC])` - Multi-dim interpolation
- [ ] Implement `CONVOL(array, kernel, [/EDGE_TRUNCATE, /EDGE_WRAP])` - Convolution
- [ ] Implement `SMOOTH(array, width, [/EDGE_TRUNCATE])` - Smoothing filter
- [ ] Implement `MEDIAN(array, width)` - Median filter
- [ ] Implement `DERIV(x, y)` - Numerical derivative
- [ ] Implement `INT_TABULATED(x, y, [/DOUBLE])` - Numerical integration
- [ ] Add tests

### 10. Logical Array Operations
**Status**: Missing
**Effort**: Low
**Files**: `xdl-stdlib/src/array.rs`

**Tasks**:
- [ ] Implement `ALL(array, [dim])` - Test if all true
- [ ] Implement `ANY(array, [dim])` - Test if any true
- [ ] Implement `FINITE(value)` - Test for finite values
- [ ] Add tests

### 11. Conversion Functions
**Status**: Basic conversions exist
**Effort**: Medium
**Files**: `xdl-stdlib/src/system.rs`

**Tasks**:
- [ ] Implement explicit type conversions:
  - [ ] `BYTE(value)`, `FIX(value)`, `LONG(value)`
  - [ ] `FLOAT(value)`, `DOUBLE(value)`
  - [ ] `COMPLEX(real, [imag])`, `DCOMPLEX(real, [imag])`
  - [ ] `UINT(value)`, `ULONG(value)`, `ULONG64(value)`, `LONG64(value)`
- [ ] Implement `STRING(value, [FORMAT=])` with format support
- [ ] Add tests

---

## 🟢 LOW Priority

### 12. Hash Tables and Lists
**Status**: Not implemented
**Effort**: Medium
**Files**: Create `xdl-core/src/collections.rs`

**Tasks**:
- [ ] Implement `HASH()` - Create hash table
- [ ] Implement `LIST()` - Create list
- [ ] Implement indexing and operations
- [ ] Add tests

### 13. Regular Expression Support
**Status**: Not implemented
**Effort**: Medium
**Files**: `xdl-stdlib/src/string.rs`

**Tasks**:
- [ ] Full `STREGEX` implementation with captures
- [ ] `STRSPLIT` with regex support
- [ ] Add regex compilation and caching
- [ ] Add tests

### 14. Widget System (GUI)
**Status**: Partial implementation exists
**Effort**: Very High
**Files**: Multiple in `xdl-gui/`

**Tasks**:
- [ ] Widget creation functions
- [ ] Event handling system
- [ ] Layout management
- [ ] This is a large subsystem - defer to later

### 15. Execute and Compilation Functions
**Status**: Partial
**Effort**: High
**Files**: `xdl-stdlib/src/system.rs`

**Tasks**:
- [ ] Enhance `EXECUTE(string)` - Execute string as code
- [ ] Implement `CALL_FUNCTION(name, args)`
- [ ] Implement `CALL_METHOD(obj, method, args)`
- [ ] Add tests

---

## Implementation Order Recommendation

Based on impact and dependencies:

1. **Week 1**: CASE/SWITCH statements (critical control flow)
2. **Week 2**: Critical array functions (REFORM, TRANSPOSE, REPLICATE, REBIN)
3. **Week 3**: Pointer operations (PTR_NEW, PTR_VALID, PTR_FREE)
4. **Week 4**: Type inquiry functions (SIZE, N_ELEMENTS, TYPENAME)
5. **Week 5**: Structure functions (CREATE_STRUCT, TAG_NAMES)
6. **Week 6**: String processing (STRSPLIT, STRJOIN, STREGEX)
7. **Week 7**: Advanced array functions (CONGRID, HISTOGRAM, etc.)
8. **Week 8**: Mathematical functions (INTERPOLATE, CONVOL, SMOOTH)
9. **Week 9**: File I/O improvements (FITS, binary structures)
10. **Week 10+**: Lower priority features

---

## Testing Strategy

For each feature:
1. Create test file in `examples/tests/feature_name_test.xdl`
2. Add unit tests in Rust test modules
3. Add integration tests comparing with GDL/IDL output
4. Document in user guide

---

## Notes

- Some features may require new XdlValue variants or core type changes
- Performance-critical functions should use vectorized operations
- Consider SIMD optimizations for array operations
- Maintain IDL/GDL compatibility as primary goal
