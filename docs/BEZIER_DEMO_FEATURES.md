# Bezier Demo Missing Features - Implementation Status

## Overview
This document tracks the implementation status of features needed for the original complex Bezier surface MATLAB demo to work natively in XDL.

## Completed Features ✓

### 1. NCHOOSEK Function (Binomial Coefficient)
**Status:** ✅ COMPLETED

**Implementation:**
- Added `nchoosek(n, k)` function in `xdl-stdlib/src/math.rs`
- Computes binomial coefficient: C(n,k) = n! / (k! * (n-k)!)
- Uses iterative computation to avoid overflow
- Registered in stdlib function registry

**Test:**
```xdl
print, nchoosek(5, 2)    ; Output: 10
print, nchoosek(10, 3)   ; Output: 120
```

**Files Modified:**
- `xdl-stdlib/src/math.rs` - Added function implementation
- `xdl-stdlib/src/lib.rs` - Registered function

---

## Pending Features 🔨

### 2. 2D Array Indexing
**Status:** ⏳ TODO

**Current Limitation:**
```xdl
arr = fltarr(10, 10)
val = arr[i, j]  ; ❌ ERROR: Multi-dimensional indexing requires nested array
```

**Workaround:**
```xdl
arr = fltarr(100)  ; Flatten to 1D
idx = i * 10 + j
val = arr[idx]     ; ✅ Works
```

**Implementation Requirements:**
- Extend lexer/parser to handle comma-separated indices
- Modify interpreter's array access logic to support multi-dimensional indexing
- Update `XdlValue` to track array dimensions/shape
- Implement row-major or column-major indexing (GDL uses column-major)

**Estimated Complexity:** HIGH (requires significant parser and runtime changes)

---

### 3. MESHGRID Function
**Status:** ⏳ TODO

**Purpose:**
Create 2D coordinate matrices from 1D coordinate vectors for plotting and surface generation.

**Expected Behavior:**
```matlab
x = [1, 2, 3]
y = [4, 5]
[X, Y] = meshgrid(x, y)
% X = [[1, 2, 3],
%      [1, 2, 3]]
% Y = [[4, 4, 4],
%      [5, 5, 5]]
```

**Current Workaround:**
```xdl
; Manual coordinate grid generation
for i = 0, nx-1 do begin
  for j = 0, ny-1 do begin
    U[idx] = x_vals[i]
    V[idx] = y_vals[j]
    idx = idx + 1
  end
endfor
end
endfor
```

**Implementation Requirements:**
- Add `meshgrid(x, y)` function to `array.rs`
- Handle returning multiple values (X and Y matrices)
- Implement proper 2D array representation

**Estimated Complexity:** MEDIUM

---

### 4. Line Continuation ($)
**Status:** ⏳ TODO

**Current Limitation:**
```xdl
cp_z = [[0.0, 0.5, 0.5, 0.0], $    ; ❌ Parser error
        [0.5, 1.5, 1.5, 0.5], $
        [0.5, 1.5, 1.5, 0.5]]
```

**Workaround:**
```xdl
; Split into separate lines/variables
cp_z_row1 = [0.0, 0.5, 0.5, 0.0]   ; ✅ Works
cp_z_row2 = [0.5, 1.5, 1.5, 0.5]
cp_z_row3 = [0.5, 1.5, 1.5, 0.5]
```

**Implementation Requirements:**
- Modify lexer to recognize `$` as line continuation
- Join continued lines before tokenization
- Handle nested array literals across multiple lines

**Estimated Complexity:** LOW-MEDIUM

---

### 5. Nested Function Definitions
**Status:** ⏳ TODO

**Current Limitation:**
```matlab
function result = bernstein(i, n, t)
    result = nchoosek(n, i) * (t.^i) .* (1-t).^(n-i);
end
```

**Workaround:**
```xdl
; Expand function inline or create separate PRO file
u_w0 = (1-u)^3
u_w1 = 3.0 * (1-u)^2 * u
u_w2 = 3.0 * (1-u) * u^2
u_w3 = u^3
```

**Implementation Requirements:**
- Add function definition support to parser (MATLAB `function` or GDL `PRO/FUNCTION`)
- Implement function scope and local variables
- Handle function return values
- Support nested function definitions within scripts

**Estimated Complexity:** VERY HIGH (requires significant language feature additions)

---

### 6. Complex Number Support
**Status:** ⏳ TODO

**Current Limitation:**
```matlab
z = -1-1i;        ; ❌ Not supported
w = 2 + 3i;       ; ❌ Not supported
```

**Workaround:**
```xdl
; Use real numbers only or separate real/imaginary parts
real_part = -1.0
imag_part = -1.0
```

**Implementation Requirements:**
- Add `Complex` type to `XdlValue` enum
- Implement imaginary unit (`i` or `j`)
- Override arithmetic operators for complex numbers
- Add functions: `real()`, `imag()`, `abs()`, `angle()`, `conj()`
- Support complex array operations

**Estimated Complexity:** HIGH

---

## Priority Recommendations

### Short Term (Quick Wins)
1. **Line Continuation ($)** - Improves code readability
2. **MESHGRID** - Commonly used, moderate effort

### Medium Term
3. **2D Array Indexing** - Major quality of life improvement
4. **Complex Numbers** - Expands application domains

### Long Term
5. **Nested Functions** - Full MATLAB/GDL compatibility
6. **Advanced Features** - Additional plotting options (subplot, figure, etc.)

---

## Testing Strategy

Each implemented feature should include:
1. Unit tests in the relevant module
2. Integration test in examples/
3. Documentation update in relevant README
4. Entry in CHANGELOG.md

---

## References

- GDL Documentation: https://gnudatalanguage.github.io/
- MATLAB Documentation: https://www.mathworks.com/help/matlab/
- Current XDL Implementation: `xdl-stdlib/`, `xdl-parser/`, `xdl-interpreter/`

---

Last Updated: 2025-10-23
