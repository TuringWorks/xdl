# GDL to XDL Porting Status

This document tracks which functions and procedures from GDL have been ported to XDL.

## Summary Statistics

**GDL Total Functions/Procedures**: ~450+
**XDL Implemented**: ~70
**Completion**: ~15%

---

## ✅ Already Implemented in XDL

### Math Functions (15)
- [x] SIN, COS, TAN
- [x] ASIN, ACOS, ATAN
- [x] EXP, ALOG/LN, ALOG10
- [x] SQRT, ABS, FLOOR, CEIL, ROUND
- [x] FFT

### Array Generation (3)
- [x] FINDGEN, INDGEN, RANDOMU

### Array Creation (6)
- [x] BYTARR, INTARR, LONARR, FLTARR, DBLARR, STRARR

### Array Manipulation (10)
- [x] N_ELEMENTS, WHERE, REFORM, TRANSPOSE
- [x] MIN, MAX, MEAN, TOTAL, REVERSE, SORT

### Array Processing (5)
- [x] SMOOTH, MOVING_AVERAGE, WMA, EMA, CUMULATIVE_AVERAGE

### Statistics (9)
- [x] VARIANCE, STDDEV, MEDIAN, MOMENT
- [x] MEANABSDEV, SKEWNESS, KURTOSIS
- [x] GAUSS_PDF, T_PDF, CHISQR_PDF

### String Functions (6)
- [x] STRLEN, STRPOS, STRMID, STRUPCASE, STRLOWCASE, STRING

### I/O Functions (9)
- [x] PRINT, GET_LUN, FILEPATH, READ_JPEG, READF
- [x] FREE_LUN, OPENR, OPENW, OPENU, CLOSE

### I/O Procedures (2)
- [x] PRINTF, WRITEF

### Graphics Procedures (11)
- [x] PLOT, OPLOT, CONTOUR, SURFACE
- [x] WINDOW, WSET, ERASE, DEVICE
- [x] LOADCT, TVSCL, AXIS

### System Procedures (8)
- [x] HELP, CD, SPAWN, CALL_PROCEDURE
- [x] DEFSYSV, @, .COMPILE, .CONTINUE, CATCH

### Python Integration (3)
- [x] PYTHON_IMPORT, PYTHON_CALL, PYTHON_CALL_KW

### Machine Learning (7)
- [x] XDLML_PARTITION, XDLML_SHUFFLE
- [x] XDLML_LINEAR_NORMALIZER, XDLML_RANGE_NORMALIZER
- [x] XDLML_VARIANCE_NORMALIZER, XDLML_TANH_NORMALIZER
- [x] XDLML_UNIT_NORMALIZER, XDLML_KMEANS

### Data Structures (1)
- [x] HASH

---

## 🔴 High Priority - Core Functions to Port

### Essential Math Functions
- [ ] SINH, COSH, TANH, ASINH, ACOSH, ATANH
- [ ] ALOG2 (log base 2)
- [ ] GAMMA, LNGAMMA, IGAMMA, BETA, IBETA
- [ ] BESELJ, BESELY, BESELI, BESELK (Bessel functions)
- [ ] ERF, ERFC, ERROREF (error functions)
- [ ] LEGENDRE, LAGUERRE, SPHER_HARM (special functions)
- [ ] VOIGT

### Type Conversion (Critical)
- [ ] BYTE, FIX, LONG, LONG64, FLOAT, DOUBLE
- [ ] UINT, ULONG, ULONG64, COMPLEX, DCOMPLEX
- [ ] STRING (full version)
- [ ] TYPENAME

### Array Generation
- [ ] BINDGEN, LINDGEN, SINDGEN, DINDGEN
- [ ] UINDGEN, ULINDGEN, L64INDGEN, UL64INDGEN
- [ ] CINDGEN, DCINDGEN
- [ ] COMPLEXARR, DCOMPLEXARR, UINTARR
- [ ] LON64ARR, ULON64ARR, ULONDARR, OBJARR, PTRARR
- [ ] RANDOMN (Gaussian random)
- [ ] MAKE_ARRAY

### Critical Array Manipulation
- [ ] SIZE (very important!)
- [ ] ROTATE, SHIFT, REBIN
- [ ] REPLICATE, REPLICATE_INPLACE
- [ ] ARRAY_EQUAL
- [ ] FINITE (check for NaN/Inf)
- [ ] HISTOGRAM

### Matrix Operations
- [ ] MATRIX_MULTIPLY (##)
- [ ] TRANSPOSE (already have)
- [ ] INVERT
- [ ] DETERM
- [ ] LUDC, LUSOL
- [ ] SVDC (SVD decomposition)
- [ ] CHOLDC, CHOLSOL
- [ ] ELMHES, HQR, TRIRED, TRIQL
- [ ] LA_CHOLDC, LA_CHOLSOL, LA_ELMHES, LA_TRIRED, LA_LEAST_SQUARES

### String Functions (Extended)
- [ ] STRCMP, STRCOMPRESS, STREGEX
- [ ] STRJOIN, STRSPLIT, STRTOK
- [ ] STRTRIM, STRPUT
- [ ] FULSTR (not common, low priority)

### I/O Functions (Critical)
- [ ] READ, READU, READS
- [ ] EOF, FSTAT
- [ ] POINT_LUN, SKIP_LUN, TRUNCATE_LUN
- [ ] COPY_LUN
- [ ] ASSOC (binary file access)

### File Operations
- [ ] FILE_SEARCH, FINDFILE
- [ ] FILE_TEST, FILE_INFO
- [ ] FILE_BASENAME, FILE_DIRNAME, FILE_EXPAND_PATH
- [ ] FILE_LINES
- [ ] FILE_COPY, FILE_DELETE, FILE_MOVE, FILE_MKDIR
- [ ] FILE_LINK, FILE_READLINK, FILE_SAME

### Date/Time
- [ ] SYSTIME
- [ ] JULDAY, CALDAT
- [ ] TIMESTAMP, TIMESTAMPTOVALUES

### System Functions
- [ ] GETENV, SETENV
- [ ] MEMORY
- [ ] COMMAND_LINE_ARGS
- [ ] ROUTINE_INFO, ROUTINE_NAMES, ROUTINE_DIR, ROUTINE_FILEPATH
- [ ] RESOLVE_ROUTINE
- [ ] SCOPE_LEVEL, SCOPE_VARFETCH, SCOPE_TRACEBACK, SCOPE_VARNAME
- [ ] ARG_PRESENT, N_PARAMS
- [ ] KEYWORD_SET

### Image Processing
- [ ] CONVOL (convolution)
- [ ] DILATE_INTERNALGDL, ERODE_INTERNALGDL
- [ ] ROBERTS, SOBEL, PREWITT
- [ ] RADON
- [ ] POLY_2D
- [ ] LABEL_REGION

### Interpolation & Fitting
- [ ] INTERPOL, INTERPOLATE
- [ ] SPL_INIT, SPL_INTERP
- [ ] VALUE_LOCATE
- [ ] GAUSSFIT
- [ ] TRIANGULATE, TRIGRID, QGRID3

### Optimization & Root Finding
- [ ] NEWTON, FX_ROOT, FZ_ROOTS
- [ ] AMOEBA, POWELL, DFPMIN
- [ ] BROYDEN
- [ ] SIMPLEX (linear programming)
- [ ] LINBCG (linear systems)

### Numerical Integration
- [ ] QSIMP, QROMB, QROMO
- [ ] GAUSSINT
- [ ] RK4 (Runge-Kutta)

### Structures & Objects
- [ ] CREATE_STRUCT, STRUCT_ASSIGN
- [ ] TAG_NAMES, N_TAGS
- [ ] OBJ_NEW, OBJ_VALID, OBJ_DESTROY
- [ ] OBJ_CLASS, OBJ_ISA, OBJ_HASMETHOD
- [ ] PTR_NEW, PTR_VALID, PTR_FREE
- [ ] HEAP_FREE, HEAP_GC, HEAP_REFCOUNT

### Data Collections
- [ ] LIST (already have HASH)
- [ ] ORDEREDHASH

---

## 🟡 Medium Priority - Nice to Have

### Graphics (2D)
- [ ] PLOTS, POLYFILL, XYOUTS
- [ ] PLOT_IO, PLOT_OI, PLOT_OO
- [ ] USERSYM
- [ ] TV, TVRD, TVCRS

### Graphics (3D)
- [ ] SHADE_SURF, SCALE3, T3D

### Color Management
- [ ] TVLCT
- [ ] CURSOR

### Widget System (if GUI is planned)
- [ ] WIDGET_BASE, WIDGET_BUTTON, WIDGET_TEXT, etc.
- [ ] WIDGET_EVENT, WIDGET_INFO, WIDGET_CONTROL
- [ ] WDELETE, WSHOW, WINDOW (graphical)

### File Format I/O
- [ ] HDF5 functions (H5*)
- [ ] HDF4 functions (HDF_*)
- [ ] NetCDF functions (NCDF_*)
- [ ] TIFF functions (TIFF_*)
- [ ] GRIB functions (GRIBAPI_*)

### External Libraries
- [ ] CALL_EXTERNAL
- [ ] LINKIMAGE, UNLINKIMAGE, UNLINKSYMBOL
- [ ] DLM_LOAD

### Advanced Features
- [ ] CATCH (exception handling)
- [ ] ON_ERROR
- [ ] MESSAGE
- [ ] JOURNAL, RECALL_COMMANDS

---

## 🟢 Low Priority - Specialized/Legacy

### Parallel/MPI
- [ ] MPIDL_* functions (if MPI support needed)

### Semaphores/Shared Memory
- [ ] SEM_CREATE, SEM_DELETE, SEM_LOCK, SEM_RELEASE
- [ ] SHMMAP, SHMUNMAP, SHMVAR, SHMDEBUG

### GMEM (Process Management)
- [ ] GMEM_* functions

### Obsolete/Platform-Specific
- [ ] CHECK_MATH
- [ ] MACHAR
- [ ] CDF_EPOCH
- [ ] Various *_EXISTS functions (feature detection)

### Rarely Used
- [ ] SPRSAB, SPRSAX, SPRSIN, SPRSTP (sparse matrices)
- [ ] WTN (wavelet transform)
- [ ] HANNING (window function)
- [ ] CROSSP (cross product)
- [ ] FULSTR
- [ ] PM (process management)

---

## 📊 Recommended Implementation Order

### Phase 1: Core Language Features (High Impact)
1. **Type system** - All type conversion functions (BYTE, FIX, LONG, etc.)
2. **SIZE function** - Critically important for array introspection
3. **Array generation** - Complete the *INDGEN family
4. **FINITE** - NaN/Inf handling
5. **Basic string operations** - STRCMP, STRJOIN, STRTRIM

### Phase 2: Essential I/O
1. **READ/READU/READS** - File input
2. **File operations** - FILE_TEST, FILE_INFO, FILE_SEARCH
3. **ASSOC** - Direct access files
4. **EOF, FSTAT** - File status

### Phase 3: Core Math/Science
1. **Hyperbolic functions** - SINH, COSH, TANH
2. **Special functions** - GAMMA, BETA, ERF, Bessel functions
3. **Linear algebra** - INVERT, LUDC, SVDC
4. **Interpolation** - INTERPOL, SPL_INIT/SPL_INTERP

### Phase 4: Advanced Arrays
1. **HISTOGRAM**
2. **ROTATE, SHIFT, REBIN**
3. **REPLICATE**
4. **CONVOL**

### Phase 5: System Integration
1. **Structures** - CREATE_STRUCT, TAG_NAMES
2. **Objects** - OBJ_NEW, PTR_NEW
3. **Introspection** - ROUTINE_INFO, SCOPE_*
4. **Environment** - GETENV, SETENV

### Phase 6: Specialized (As Needed)
1. File formats (HDF5, NetCDF, etc.)
2. Advanced graphics
3. Widgets (if GUI planned)
4. Parallel computing support

---

## 🎯 Quick Wins (Easy to Implement, High Value)

These are simple but commonly used functions:

1. **FINITE** - Check for NaN/Inf (trivial)
2. **ISHFT** - Bit shift (one-liner)
3. **SIGNUM** - Sign function (one-liner)
4. **LOGICAL_AND/OR/TRUE** - Boolean ops (simple)
5. **EMPTY** - Check if undefined (simple)
6. **ISA** - Type checking (moderate)
7. **KEYWORD_SET** - Check keyword (simple with keyword system)

---

## Notes

- Functions marked with `_INTERNALGDL` are GDL-specific extensions
- Many `*_EXISTS` functions check for optional library support
- Widget functions are lower priority unless GUI is a core goal
- File format I/O depends on external library availability
- MPI/parallel functions only needed if parallel computing is a goal

**Recommendation**: Focus on Phase 1-3 first. These provide 80% of typical scientific computing needs.
