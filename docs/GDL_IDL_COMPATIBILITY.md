# GDL/IDL Compatibility Guide

This document describes XDL's compatibility with GDL/IDL syntax and functions for porting existing code.

## Status: ~90% Core Syntax Compatible

XDL now supports virtually all GDL/IDL syntax patterns commonly used in scientific computing scripts, including single-line control statements.

---

## ✅ Fully Compatible Syntax Features

### 1. Optional END Keywords (NEW!)

In GDL/IDL, when using `BEGIN...END` blocks, the explicit loop/conditional terminators are optional.

**XDL now supports both styles:**

#### FOR Loops

```idl
; Style 1: Single statement (no ENDFOR, no BEGIN...END)
for i=0,9 do a[i] = i * 2

; Style 2: BEGIN...END without ENDFOR
for i=0,9 do begin
  a[i] = i * 2
end

; Style 3: BEGIN...END with ENDFOR (also supported)
for i=0,9 do begin
  a[i] = i * 2
end
endfor
```

#### WHILE Loop Syntax

```idl
; Single statement (no ENDWHILE)
while count lt 10 do count = count + 1

; BEGIN...END without ENDWHILE
while count lt 10 do begin
  count = count + 1
end

; BEGIN...END with ENDWHILE (also supported)
while count lt 10 do begin
  count = count + 1
end
endwhile
```

#### Conditional Statements (IF-THEN-ELSE)

```idl
; Single statement (no ENDIF)
if x gt 0 then print, 'positive'
if x gt 0 then print, 'positive' else print, 'negative'

; BEGIN...END without ENDIF
if x gt 0 then begin
  print, 'positive'
end else begin
  print, 'negative'
end

; BEGIN...END with ENDIF (also supported)
if x gt 0 then begin
  print, 'positive'
end else begin
  print, 'negative'
end
endif
```

**Important:** Single-statement forms (without `BEGIN...END`) do **not** require explicit terminators. This matches GDL/IDL exactly!

---

### 2. Double Precision Literals (NEW!)

XDL now supports IDL/GDL double precision notation:

```idl
x = 1.d0          ; 1.0 as double
y = 99d-1         ; 9.9 as double
z = 1d0           ; 1.0 as double
ratio = 1.d0/3.d0 ; Double precision division
```

This is critical for numerical accuracy in ported code.

---

### 3. Array Generation Functions (EXPANDED!)

All standard *INDGEN family functions are now available:

| Function | Type | Example |
|----------|------|---------|
| `FINDGEN(n)` | Float | `x = findgen(100)` |
| `INDGEN(n)` | Integer | `i = indgen(10)` |
| `DINDGEN(n)` | Double | `d = dindgen(50)` ✅ NEW |
| `BINDGEN(n)` | Byte | `b = bindgen(256)` ✅ NEW |
| `LINDGEN(n)` | Long | `l = lindgen(1000)` ✅ NEW |
| `UINDGEN(n)` | Unsigned Int | `u = uindgen(100)` ✅ NEW |
| `ULINDGEN(n)` | Unsigned Long | `ul = ulindgen(100)` ✅ NEW |
| `L64INDGEN(n)` | 64-bit Long | `l64 = l64indgen(100)` ✅ NEW |

**Example Usage:**

```idl
; Generate array indices
nx = 100
x = dindgen(nx)/(nx-1.d0)  ; Range from 0 to 1

; Scale to range
xmin = -2*!pi
xmax = 2*!pi
x = xmin + (xmax-xmin)*dindgen(nx)/(nx-1.d0)
```

---

### 4. Array Creation Functions

All standard array creation functions work identically to GDL/IDL:

```idl
a = BYTARR(10)           ; Byte array
b = INTARR(10, 20)       ; 2D integer array
c = LONARR(100)          ; Long integer array
d = FLTARR(50, 50)       ; Float array
e = DBLARR(10, 10, 10)   ; Double precision array
s = STRARR(5)            ; String array
```

---

### 5. Control Flow

**Fully compatible with GDL/IDL syntax:**

#### FOR Loop Syntax

```idl
for i=0, n-1 do statement
for i=start, end, step do statement
for i=0, 9 do begin ... end
```

#### WHILE Loops

```idl
while condition do statement
while condition do begin ... end
```

#### REPEAT-UNTIL Loops

```idl
repeat begin
  ; statements
end until condition
```

#### IF-THEN-ELSE

```idl
if condition then statement
if condition then begin ... end
if condition then begin ... end else begin ... end
```

#### BREAK and CONTINUE

```idl
for i=0,100 do begin
  if condition then break
  if other_condition then continue
end
```

---

## 🟡 Partially Compatible Features

### 1. CASE Statements

**Status:** Not yet implemented

```idl
; Not yet supported
case value of
  1: statement1
  2: statement2
  else: default_statement
endcase
```

### 2. FOREACH Loops

**Status:** Parser support exists, needs testing

```idl
foreach element, array, index do begin
  ; Process element
end
```

---

## ❌ Not Yet Implemented

### 1. .pro File Batch Execution

```idl
; Not yet supported
@myprogram    ; Execute myprogram.pro
```

**Workaround:** Use `xdl myprogram.pro` from command line

### 2. SIZE Function

```idl
; Critical for array introspection - not yet implemented
dims = SIZE(array)
```

### 3. GOTO and Labels

```idl
; Not recommended anyway, but not supported
label1:
  ; code
goto, label1
```

---

## 📝 Porting Checklist

When porting GDL/IDL code to XDL:

1. ✅ **Control Flow:** Most syntax works as-is
2. ✅ **Array Generation:** All *INDGEN functions available
3. ✅ **Double Precision:** Use .d0 notation freely
4. ✅ **Optional END Keywords:** Both styles work
5. ⚠️ **SIZE Function:** Replace with manual dimension tracking
6. ⚠️ **CASE Statements:** Replace with nested IF-THEN-ELSE
7. ⚠️ **@batch:** Use command line execution instead
8. ⚠️ **Complex Arrays:** Not yet supported (COMPLEXARR, DCOMPLEXARR)

---

## 🔬 Example: Porting Scientific Code

### Original GDL/IDL Code

```idl
; Generate data
nx = 100
xmin = -2*!pi
xmax = 2*!pi
x = xmin + (xmax-xmin)*dindgen(nx)/(nx-1.d0)

; Compute function
y = sin(x)

; Process with loop
for i=0,nx-1 do begin
  if y[i] lt 0 then y[i] = 0.d0
end

; Plot
plot, x, y
```

### XDL (100% Compatible!)

The above code runs **without modification** in XDL!

---

## 📊 Compatibility Statistics

| Category | Functions Available | Total in GDL/IDL | % Complete |
|----------|-------------------|-----------------|-----------|
| Math Functions | 15 | ~50 | 30% |
| Array Generation | 8 | 12 | 67% ✅ |
| Array Creation | 6 | 14 | 43% |
| Array Manipulation | 10 | ~40 | 25% |
| Control Flow | All core | All core | 95% ✅ |
| I/O Functions | 9 | ~30 | 30% |
| Graphics | 11 | ~60 | 18% |
| **Overall** | **~75** | **~450** | **~17%** |

---

## 🎯 High Priority Next Steps

1. **SIZE function** - Critical for array operations
2. **CASE statements** - Common control flow pattern
3. **More math functions** (SINH, COSH, GAMMA, BESSEL, etc.)
4. **Matrix operations** (INVERT, TRANSPOSE, etc.)
5. **String functions** (STRSPLIT, STRJOIN, etc.)

---

## 💡 Tips for Successful Porting

1. **Start with syntax:** Most control flow works as-is
2. **Test incrementally:** Port small sections and test
3. **Check function availability:** Refer to `docs/GDL_XDL_GAP_ANALYSIS.md`
4. **Use workarounds:** Many GDL/IDL patterns have XDL equivalents
5. **File issues:** Report compatibility problems to help improve XDL

---

## 📚 Additional Resources

- `GDL_XDL_GAP_ANALYSIS.md` - Complete function availability list
- `GDL_XDL_PORTING_STATUS.md` - Detailed porting status
- `examples/` - Working XDL code examples
- `tests/` - Test files showing expected syntax

---

**Last Updated:** 2025-10-22
**XDL Version:** 0.1.0
