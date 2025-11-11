# GDL/IDL to XDL Gap Analysis

This document identifies GDL/IDL functionality that has **not yet been ported** to XDL.

---

## Summary

**Currently Implemented in XDL:** ~60 functions/procedures  
**Missing from GDL/IDL:** ~400+ functions/procedures  
**Completion:** ~13% of full GDL/IDL functionality

---

## ✅ Already Implemented in XDL

### Math Functions (15 functions)
- ✅ SIN, COS, TAN, ASIN, ACOS, ATAN
- ✅ EXP, ALOG (LN), ALOG10
- ✅ SQRT, ABS
- ✅ FLOOR, CEIL, ROUND
- ✅ FIX, LONG, FLOAT, DOUBLE (type conversion)
- ✅ FINDGEN, INDGEN, RANDOMU

### Array Functions (16 functions)
- ✅ BYTARR, FLTARR (placeholders, need full implementation)
- ✅ N_ELEMENTS
- ✅ WHERE (placeholder)
- ✅ MIN, MAX, MEAN, TOTAL
- ✅ REVERSE, SORT
- ✅ SMOOTH, MOVING_AVERAGE, WMA, EMA, CUMULATIVE_AVERAGE

### Statistics Functions (10 functions)
- ✅ VARIANCE, STDDEV, MEDIAN
- ✅ MOMENT, MEANABSDEV, SKEWNESS, KURTOSIS
- ✅ GAUSS_PDF, T_PDF, CHISQR_PDF (basic placeholders)

### Graphics Procedures (11 procedures)
- ✅ PLOT, OPLOT, CONTOUR, SURFACE
- ✅ WINDOW, WSET, ERASE
- ✅ DEVICE, LOADCT, TVSCL, AXIS

### I/O Functions (6 functions)
- ✅ PRINT
- ✅ GET_LUN, FREE_LUN
- ✅ OPEN, CLOSE
- ✅ FILEPATH, READ_JPEG

### String Functions (5 functions)
- ✅ STRLEN, STRPOS, STRMID
- ✅ STRUPCASE, STRLOWCASE

### System Functions (7 procedures)
- ✅ HELP, CD, SPAWN
- ✅ CALL_PROCEDURE, DEFSYSV
- ✅ @, .COMPILE, .CONTINUE, CATCH

### Python Integration (3 functions)
- ✅ PYTHON_IMPORT, PYTHON_CALL, PYTHON_CALL_KW

### Data Structures (1 function)
- ✅ HASH (placeholder)

**Total Implemented:** ~60 functions/procedures

---

## ❌ Major Missing Categories

### 1. Array Creation Functions (PARTIALLY COMPLETE)

**✅ Implemented:**
- ✅ **BYTARR** - Create byte array (fully functional)
- ✅ **INTARR** - Create integer array (fully functional)
- ✅ **LONARR** - Create long integer array (fully functional)
- ✅ **FLTARR** - Create float array (fully functional)
- ✅ **DBLARR** - Create double precision array (fully functional)
- ✅ **STRARR** - Create string array (basic - returns numeric array placeholder)

**❌ Still Missing:**
- ❌ **COMPLEXARR** - Create complex array
- ❌ **DCOMPLEXARR** - Create double complex array
- ❌ **LON64ARR** - Create 64-bit integer array
- ❌ **UINTARR** - Create unsigned integer array
- ❌ **ULONARR** - Create unsigned long array
- ❌ **ULON64ARR** - Create unsigned 64-bit array
- ❌ **PTRARR** - Create pointer array
- ❌ **OBJARR** - Create object array

**Impact:** MEDIUM (core types now implemented, specialized types remaining)

**Status:** Core array creation functions now create actual arrays, not placeholders!
**Test File:** `examples/test_array_creation.xdl` (428 lines, all passing)

---

### 2. Array Generation Functions

**Missing:**
- ❌ **DINDGEN** - Double precision index generation
- ❌ **CINDGEN** - Complex index generation
- ❌ **BINDGEN** - Byte index generation
- ❌ **LINDGEN** - Long index generation
- ❌ **L64INDGEN** - 64-bit index generation
- ❌ **UINDGEN** - Unsigned index generation
- ❌ **ULINDGEN** - Unsigned long index generation
- ❌ **UL64INDGEN** - Unsigned 64-bit index generation
- ❌ **SINDGEN** - String index generation
- ❌ **RANDOMN** - Gaussian random numbers
- ❌ **RANDOM** - Random number generator

**Impact:** MEDIUM - Used for initializing arrays with patterns

---

### 3. Array Manipulation Functions

**Missing:**
- ❌ **REFORM** - Change array dimensions without copying
- ❌ **TRANSPOSE** - Transpose array
- ❌ **ROTATE** - Rotate array
- ❌ **SHIFT** - Shift array elements
- ❌ **REBIN** - Resize array by replication/averaging
- ❌ **CONGRID** - Resize array with interpolation
- ❌ **REPLICATE** - Create array by replicating value
- ❌ **MAKE_ARRAY** - General array creation
- ❌ **ARRAY_INDICES** - Convert 1D to nD indices
- ❌ **ARRAY_EQUAL** - Test array equality
- ❌ **UNIQ** - Find unique elements
- ❌ **HISTOGRAM** - Compute histogram
- ❌ **WHERE** - Find array indices (needs full implementation)

**Impact:** HIGH - Essential for data manipulation

---

### 4. Matrix/Linear Algebra Functions

**Missing:**
- ❌ **INVERT** - Matrix inversion
- ❌ **DETERM** - Matrix determinant
- ❌ **IDENTITY** - Create identity matrix
- ❌ **TRACE** - Matrix trace
- ❌ **EIGENQL** - Eigenvalues/eigenvectors (QL method)
- ❌ **EIGENVEC** - Eigenvalues/eigenvectors
- ❌ **ELMHES** - Reduce to Hessenberg form
- ❌ **HQR** - Eigenvalues of Hessenberg matrix
- ❌ **TRIQL** - Tridiagonal QL eigenvalues
- ❌ **TRISOL** - Solve tridiagonal system
- ❌ **CHOLDC** - Cholesky decomposition
- ❌ **CHOLSOL** - Solve using Cholesky
- ❌ **LA_CHOLDC** - Linear algebra Cholesky
- ❌ **SVDC** - Singular value decomposition
- ❌ **SVSOL** - Solve using SVD
- ❌ **LA_SVD** - Linear algebra SVD
- ❌ **LUDC** - LU decomposition
- ❌ **LUSOL** - Solve using LU
- ❌ **LA_LUDC** - Linear algebra LU
- ❌ **CRAMER** - Cramer's rule solution
- ❌ **GS_ITER** - Gauss-Seidel iteration

**Impact:** HIGH - Critical for scientific computing

---

### 5. Mathematical/Special Functions

**Missing:**
- ❌ **SINH**, **COSH**, **TANH** - Hyperbolic trig
- ❌ **ASINH**, **ACOSH**, **ATANH** - Inverse hyperbolic
- ❌ **ATAN** (2-argument) - Two-argument arctangent
- ❌ **BESELJ**, **BESELI**, **BESELK**, **BESELY** - Bessel functions
- ❌ **BETA** - Beta function
- ❌ **LNBETA** - Log beta
- ❌ **GAMMA** - Gamma function
- ❌ **LNGAMMA** - Log gamma
- ❌ **IGAMMA** - Incomplete gamma
- ❌ **IBETA** - Incomplete beta
- ❌ **ERF** - Error function
- ❌ **ERFC** - Complementary error function
- ❌ **ERFCX** - Scaled complementary error
- ❌ **EXPINT** - Exponential integral
- ❌ **FACTORIAL** - Factorial function
- ❌ **POLY** - Polynomial evaluation
- ❌ **POLY_FIT** - Polynomial fitting
- ❌ **POLYFILLV** - Fill polygon vertices

**Impact:** MEDIUM-HIGH - Important for advanced mathematics

---

### 6. String Functions

**Missing:**
- ❌ **STRCOMPRESS** - Compress whitespace
- ❌ **STRTRIM** - Trim whitespace
- ❌ **STRJOIN** - Join strings
- ❌ **STRSPLIT** - Split strings
- ❌ **STRMATCH** - Pattern matching
- ❌ **STRCMP** - Compare strings
- ❌ **STREGEX** - Regular expressions
- ❌ **STRREPLACE** - Replace substrings
- ❌ **STRING** - Convert to string
- ❌ **STRMESSAGE** - System error messages
- ❌ **STRLEN** - String length (implemented)
- ❌ **STRPUT** - Insert substring
- ❌ **BYTE** - Convert to byte array

**Impact:** MEDIUM - Needed for text processing

---

### 7. I/O Functions and Procedures

**Missing:**
- ❌ **READ** - Read from terminal
- ❌ **READF** - Read from file
- ❌ **READS** - Read from string
- ❌ **READU** - Read unformatted
- ❌ **WRITEU** - Write unformatted
- ❌ **PRINTF** - Formatted print to file
- ❌ **OPENR**, **OPENW**, **OPENU** - Open file variants
- ❌ **ASSOC** - Associate array with file
- ❌ **POINT_LUN** - Position file pointer
- ❌ **EOF** - Test end of file
- ❌ **FLUSH** - Flush file buffer
- ❌ **FSTAT** - File status
- ❌ **FILE_INFO** - File information
- ❌ **FILE_TEST** - Test file existence
- ❌ **FILE_SEARCH** - Search for files
- ❌ **FILE_LINES** - Count file lines
- ❌ **FILE_DELETE** - Delete files
- ❌ **FILE_COPY** - Copy files
- ❌ **FILE_MOVE** - Move files
- ❌ **FILE_MKDIR** - Create directory

**Impact:** HIGH - Essential for file operations

---

### 8. Image I/O Functions

**Missing:**
- ❌ **READ_PNG** - Read PNG image
- ❌ **WRITE_PNG** - Write PNG image
- ❌ **READ_TIFF** - Read TIFF image
- ❌ **WRITE_TIFF** - Write TIFF image
- ❌ **READ_BMP** - Read BMP image
- ❌ **WRITE_BMP** - Write BMP image
- ❌ **WRITE_JPEG** - Write JPEG image
- ❌ **READ_GIF** - Read GIF image
- ❌ **WRITE_GIF** - Write GIF image
- ❌ **QUERY_IMAGE** - Query image info

**Impact:** MEDIUM - Important for image processing

---

### 9. Graphics Functions (Additional)

**Missing:**
- ❌ **PLOTS** - Draw lines/points
- ❌ **XYOUTS** - Draw text
- ❌ **POLYFILL** - Fill polygon
- ❌ **USERSYM** - Define user symbol
- ❌ **ARROW** - Draw arrow
- ❌ **SHADE_SURF** - Shaded surface
- ❌ **SHADE_VOLUME** - Shaded volume
- ❌ **ISOSURFACE** - Isosurface rendering
- ❌ **PARTICLE_TRACE** - Particle tracing
- ❌ **STREAMLINE** - Streamline plots
- ❌ **VECTOR_FIELD** - Vector field plots
- ❌ **IMAGE** - Display image
- ❌ **TV** - Display array as image
- ❌ **TVRD** - Read from display
- ❌ **COLORBAR** - Draw colorbar
- ❌ **LEGEND** - Draw legend
- ❌ **MAP_SET** - Set up map projection
- ❌ **MAP_CONTINENTS** - Draw continents
- ❌ **MAP_GRID** - Draw map grid

**Impact:** MEDIUM - For visualization

---

### 10. Statistical Functions (Advanced)

**Missing:**
- ❌ **CORRELATE** - Correlation coefficient
- ❌ **R_CORRELATE** - Rank correlation
- ❌ **M_CORRELATE** - Multiple correlation
- ❌ **REGRESS** - Linear regression
- ❌ **LINFIT** - Linear fit
- ❌ **CURVEFIT** - Non-linear curve fitting
- ❌ **SVDFIT** - SVD fit
- ❌ **LADFIT** - Least absolute deviation fit
- ❌ **POLY_FIT** - Polynomial fit
- ❌ **SFIT** - Surface fit
- ❌ **KURTOSIS** - Kurtosis (implemented)
- ❌ **HISTOGRAM** - Histogram computation
- ❌ **HIST_EQUAL** - Histogram equalization
- ❌ **KS_TEST** - Kolmogorov-Smirnov test
- ❌ **F_TEST** - F-test
- ❌ **T_TEST** - t-test
- ❌ **CHI2_TEST** - Chi-square test
- ❌ **BINOMIAL** - Binomial distribution
- ❌ **POISSON** - Poisson distribution

**Impact:** MEDIUM - For statistical analysis

---

### 11. Signal Processing Functions

**Missing:**
- ❌ **FFT** - Fast Fourier Transform
- ❌ **FFT_POWERSPEC** - FFT power spectrum
- ❌ **CONVOL** - Convolution
- ❌ **CONVOLVE** - Convolution (different algorithm)
- ❌ **DEFROI** - Define region of interest
- ❌ **DIGITAL_FILTER** - Digital filter design
- ❌ **FIR_FILTER** - FIR filter
- ❌ **IIR_FILTER** - IIR filter
- ❌ **LEEFILT** - Lee filter
- ❌ **MEDIAN** - Median filter (implemented as stat function)
- ❌ **MORPH_CLOSE** - Morphological closing
- ❌ **MORPH_OPEN** - Morphological opening
- ❌ **MORPH_DILATE** - Morphological dilation
- ❌ **MORPH_ERODE** - Morphological erosion
- ❌ **SOBEL** - Sobel edge detection
- ❌ **ROBERTS** - Roberts edge detection
- ❌ **PREWITT** - Prewitt edge detection
- ❌ **HOUGH** - Hough transform
- ❌ **RADON** - Radon transform
- ❌ **HILBERT** - Hilbert transform
- ❌ **WAVELET** - Wavelet transform

**Impact:** MEDIUM-HIGH - For signal/image processing

---

### 12. Interpolation Functions

**Missing:**
- ❌ **INTERPOL** - Linear interpolation
- ❌ **INTERPOLATE** - Multi-dimensional interpolation
- ❌ **BILINEAR** - Bilinear interpolation
- ❌ **TRIGRID** - Triangular grid interpolation
- ❌ **TRIANGULATE** - Delaunay triangulation
- ❌ **SPL_INIT** - Spline initialization
- ❌ **SPL_INTERP** - Spline interpolation
- ❌ **SPLINE** - Spline fitting
- ❌ **SPLINE_P** - Parametric spline

**Impact:** MEDIUM - For data interpolation

---

### 13. Time and Date Functions

**Missing:**
- ❌ **SYSTIME** - System time
- ❌ **JULDAY** - Julian day number
- ❌ **CALDAT** - Calendar date
- ❌ **BIN_DATE** - Binary date/time
- ❌ **TIMESTAMP** - Current timestamp
- ❌ **TIC**, **TOC** - Timing functions

**Impact:** LOW-MEDIUM - For timing and dating

---

### 14. Type Conversion Functions

**Partially Implemented:**
- ✅ **FIX**, **LONG**, **FLOAT**, **DOUBLE** (basic)
- ❌ **BYTE** - Convert to byte
- ❌ **COMPLEX** - Create complex number
- ❌ **DCOMPLEX** - Create double complex
- ❌ **UINT** - Convert to unsigned int
- ❌ **ULONG** - Convert to unsigned long
- ❌ **LONG64** - Convert to 64-bit long
- ❌ **ULONG64** - Convert to unsigned 64-bit
- ❌ **PTR_NEW** - Create pointer
- ❌ **PTR_VALID** - Test pointer validity
- ❌ **PTR_FREE** - Free pointer

**Impact:** MEDIUM - For type manipulation

---

### 15. Structure and Object Functions

**Missing:**
- ❌ **CREATE_STRUCT** - Create structure
- ❌ **STRUCT_ASSIGN** - Assign structure
- ❌ **TAG_NAMES** - Structure tag names
- ❌ **N_TAGS** - Number of structure tags
- ❌ **OBJ_NEW** - Create object
- ❌ **OBJ_DESTROY** - Destroy object
- ❌ **OBJ_VALID** - Test object validity
- ❌ **OBJ_CLASS** - Object class
- ❌ **OBJ_ISA** - Test object inheritance

**Impact:** HIGH - For structured data (not yet designed in XDL)

---

### 16. Control Flow (Some Implemented)

**Missing:**
- ❌ **SWITCH/CASE** - Switch statement
- ❌ **ON_ERROR** - Error handling
- ❌ **ON_IOERROR** - I/O error handling
- ❌ **MESSAGE** - Display message/error
- ❌ **RETURN** - Return from procedure
- ❌ **STOP** - Stop execution
- ❌ **CONTINUE** - Continue loop
- ❌ **BREAK** - Break loop
- ❌ **GOTO** - Goto statement

**Impact:** MEDIUM - Some exist in parser, need runtime support

---

### 17. Widgets and GUI (Minimal Implementation)

**Missing:**
- ❌ **WIDGET_BASE** - Create widget base
- ❌ **WIDGET_BUTTON** - Create button
- ❌ **WIDGET_SLIDER** - Create slider
- ❌ **WIDGET_TEXT** - Create text widget
- ❌ **WIDGET_LABEL** - Create label
- ❌ **WIDGET_LIST** - Create list
- ❌ **WIDGET_TABLE** - Create table
- ❌ **WIDGET_DRAW** - Create drawing area
- ❌ **WIDGET_CONTROL** - Control widgets
- ❌ **WIDGET_EVENT** - Handle widget events
- ❌ **WIDGET_INFO** - Widget information
- ❌ **XMANAGER** - Event manager
- ❌ **XREGISTERED** - Check registration
- ❌ **XLOADCT** - Load color table (GUI)
- ❌ **XPALETTE** - Palette editor

**Impact:** LOW-MEDIUM - For interactive applications

---

### 18. HDF/NetCDF/Scientific Data Formats

**Missing:**
- ❌ **HDF_SD_START** - Open HDF file
- ❌ **HDF_SD_SELECT** - Select HDF dataset
- ❌ **HDF_SD_GETDATA** - Read HDF data
- ❌ **NCDF_CREATE** - Create NetCDF file
- ❌ **NCDF_OPEN** - Open NetCDF file
- ❌ **NCDF_VARDEF** - Define NetCDF variable
- ❌ **NCDF_VARPUT** - Write NetCDF variable
- ❌ **NCDF_VARGET** - Read NetCDF variable
- ❌ **FITS_READ** - Read FITS file
- ❌ **FITS_WRITE** - Write FITS file
- ❌ **FITS_OPEN** - Open FITS file

**Impact:** HIGH - Critical for scientific data

**Note:** XDL has modules in `xdl-ffi/` (gsl.rs, hdf5.rs, netcdf.rs) but they're not connected

---

### 19. Database Functions

**Missing:**
- ❌ All database functionality
- ❌ **DB_OPEN** - Open database
- ❌ **DB_QUERY** - Query database
- ❌ etc.

**Impact:** LOW - Niche functionality

---

### 20. Miscellaneous Functions

**Missing:**
- ❌ **DIALOG_MESSAGE** - Display dialog
- ❌ **DIALOG_PICKFILE** - File picker dialog
- ❌ **DIALOG_PRINTERSETUP** - Printer setup
- ❌ **EXECUTE** - Execute command string
- ❌ **RESOLVE_ROUTINE** - Resolve procedure/function
- ❌ **ROUTINE_INFO** - Get routine information
- ❌ **SCOPE_VARNAME** - Variable names in scope
- ❌ **SCOPE_LEVEL** - Current scope level
- ❌ **MEMORY** - Memory usage
- ❌ **HEAP_GC** - Garbage collection
- ❌ **BINDGEN** through **UL64INDGEN** - Array generation

**Impact:** VARIES

---

## Priority Ranking

### 🔴 CRITICAL (Must Have for Basic Functionality)

1. **Array creation functions** (INTARR, DBLARR, etc.) - Currently return placeholders
2. **WHERE function** (full implementation) - Currently placeholder
3. **Basic file I/O** (READF, WRITEF, OPENR, OPENW, OPENU)
4. **REFORM, TRANSPOSE** - Essential array reshaping
5. **STRING type conversion** - Convert values to strings
6. **FFT** - Fourier transforms (very common in scientific code)

### 🟡 HIGH (Important for Scientific Computing)

7. **Matrix operations** (INVERT, ##, TRANSPOSE)
8. **Linear algebra** (SVDC, LUDC, eigenvalues)
9. **INTERPOL/INTERPOLATE** - Interpolation
10. **CONVOL** - Convolution
11. **HISTOGRAM** - Data analysis
12. **CORRELATE** - Correlation
13. **REPLICATE** - Array replication
14. **NetCDF/HDF5 I/O** - Scientific data formats
15. **More array generation** (RANDOMN, etc.)

### 🟢 MEDIUM (Nice to Have)

16. **Special functions** (BESSEL, GAMMA, ERF)
17. **String manipulation** (STRSPLIT, STRJOIN, etc.)
18. **Image I/O** (PNG, TIFF)
19. **Advanced graphics** (PLOTS, XYOUTS, POLYFILL)
20. **Curve fitting** (CURVEFIT, POLY_FIT)
21. **Signal processing** (filters, wavelets)
22. **Time/date functions**

### 🔵 LOW (Can Wait)

23. **Widgets/GUI** - Complex, low priority
24. **Database functions** - Niche
25. **Object-oriented features** - Major design work needed
26. **Structures** - Requires type system work

---

## Recommended Implementation Order

### Phase 1: Core Functionality (Next 3-6 months)
1. ✅ Implement proper array creation (INTARR, DBLARR, STRARR, etc.)
2. ✅ Full WHERE implementation with conditional support
3. ✅ STRING() type conversion function
4. ✅ Basic file I/O (READF, WRITEF, OPENR, OPENW)
5. ✅ REFORM and TRANSPOSE

### Phase 2: Scientific Computing (6-12 months)
6. ✅ Matrix operations (##, INVERT, DETERM)
7. ✅ FFT and inverse FFT
8. ✅ Linear algebra (SVD, LU, eigenvalues via GSL)
9. ✅ INTERPOL/INTERPOLATE
10. ✅ CONVOL
11. ✅ HISTOGRAM

### Phase 3: Data I/O (12-18 months)
12. ✅ NetCDF support (integrate xdl-ffi/netcdf.rs)
13. ✅ HDF5 support (integrate xdl-ffi/hdf5.rs)
14. ✅ FITS I/O
15. ✅ Image I/O (PNG, TIFF, complete JPEG)

### Phase 4: Advanced Features (18-24 months)
16. ✅ Special functions (via GSL)
17. ✅ Curve fitting
18. ✅ Advanced signal processing
19. ✅ More string functions
20. ✅ Time/date functions

### Phase 5: Nice-to-Have (Future)
21. ⏳ Widgets (if needed)
22. ⏳ Object system
23. ⏳ Structure system
24. ⏳ Database integration

---

## Notes on Existing Placeholders

Several functions are **registered** but return **placeholders** instead of working implementations:

1. **BYTARR, FLTARR** - Return string like `"FLTARR(10)"` instead of array
2. **WHERE** - Returns string `"WHERE result placeholder"`
3. **HASH** - Returns string representation, not real hash table
4. **INDGEN** - Returns single Long(0) instead of array
5. **GAUSS_PDF, T_PDF, CHISQR_PDF** - Return placeholder values

These need **full implementation** before they're truly usable.

---

## Estimated Workload

- **Critical functions (Phase 1):** ~2-3 months full-time work
- **High priority (Phase 2-3):** ~6-12 months
- **Medium/Low priority (Phase 4-5):** ~12-24 months
- **Full GDL/IDL parity:** ~3-5 years

**Total estimated functions needed:** ~350-450 functions/procedures

---

## Conclusion

XDL has a **solid foundation** with ~60 functions implemented, covering:
- ✅ Basic math and trig
- ✅ Basic statistics
- ✅ Array operations (min, max, mean, sort, etc.)
- ✅ Moving averages (comprehensive suite)
- ✅ Graphics framework (basic)
- ✅ Python integration

**Critical gaps:**
- ❌ Array creation functions don't create real arrays
- ❌ No matrix operations or linear algebra
- ❌ No file I/O beyond basic operations
- ❌ No FFT or convolution
- ❌ No scientific data format support (though modules exist)
- ❌ Limited string manipulation
- ❌ No interpolation

**Recommendation:** Focus on Phase 1 (core functionality) to make XDL practically usable for scientific computing. The array creation functions and file I/O are **blocking** for most real-world use cases.
