# MATLAB Transpiler Fix - Output Capture

## Problem

MATLAB `.m` files were not displaying output in the xdl-gui results pane. While the code executed successfully, calls to `disp()` produced no visible output.

## Root Cause

The MATLAB-to-XDL transpiler was converting MATLAB function calls incorrectly:

### Before Fix

```matlab
disp('Hello');
disp(x);
```

Was transpiled to:

```xdl
PRINT ( 'Hello' )
PRINT ( x )
```

This syntax is **invalid** in XDL. XDL procedures require comma-separated arguments:

```xdl
PRINT, 'Hello'
PRINT, x
```

## Solution

Modified `xdl-matlab/src/transpiler.rs` to detect standalone procedure calls (PRINT, PRINTF) and convert them from MATLAB function syntax `func(arg)` to XDL procedure syntax `PROC, arg`.

### Changes

In the `collect_expression_until_newline()` function:

1. **Detection**: Check if an identifier is followed by `(` and appears at statement start
2. **Special Handling**: For PRINT/PRINTF procedures:
   - Insert comma after function name
   - Skip the opening parenthesis
   - Collect arguments **with proper function name mapping**
   - Skip the closing parenthesis
3. **Function Mapping**: Inside PRINT arguments, map MATLAB functions (e.g., `log` → `ALOG`)
4. **Result**: Proper XDL procedure syntax with correct function names

### After Fix

```matlab
disp('Hello');
disp(x);
```

Now correctly transpiles to:

```xdl
PRINT, 'Hello'
PRINT, x
```

### Complex Case: Nested Functions

The fix also handles nested function calls:

```matlab
disp(log(10));  % MATLAB
```

Correctly transpiles to:

```xdl
PRINT, ALOG(10)  ; XDL with proper function mapping
```

Without function mapping, `log` would not be converted to `ALOG` inside the PRINT arguments, causing a "Function not found" error.

## Testing

Created `examples/matlab/test_gui_output.m` to verify the fix:

```matlab
disp('Testing GUI output capture');
x = 42;
disp('The answer is:');
disp(x);
```

### CLI Test

```bash
$ xdl examples/matlab/test_gui_output.m
Testing GUI output capture
The answer is:
42
```

### GUI Test

1. Open `xdl-gui`
2. File > Open... > `examples/matlab/test_gui_output.m`
3. Click Execute
4. Output appears in results pane ✓

## Impact

- ✅ All MATLAB `disp()` calls now work correctly
- ✅ Output is captured in GUI results pane
- ✅ CLI output works as expected
- ✅ All existing MATLAB examples work properly

## Files Modified

- `xdl-matlab/src/transpiler.rs` - Added special handling for procedure calls
- Rebuilt `xdl-cli` and `xdl-gui` with fixed transpiler

## Future Improvements

The same technique can be extended to handle other XDL procedures that may need comma syntax:

- PLOT
- Other output procedures
- User-defined procedures (if added)

Currently, mathematical functions like `sin(x)` are correctly handled as expressions and don't need this special treatment.
