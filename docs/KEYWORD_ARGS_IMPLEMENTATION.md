# Keyword Arguments Implementation for XDL Plot Function

## Summary

Implemented full keyword argument support for the `plot` procedure in XDL, allowing users to specify plot titles and axis labels using keyword syntax.

## Changes Made

### 1. Core Infrastructure (xdl-interpreter)

- **evaluator.rs**: Added keyword argument evaluation for both function calls and procedure calls
  - Evaluates `Keyword` structures from AST
  - Converts evaluated keywords to `HashMap<String, XdlValue>`
  - Passes keywords to stdlib functions

- **lib.rs**: Updated procedure call handling to process keyword arguments
  - Extracts keyword values from AST
  - Calls `call_procedure_with_keywords` instead of `call_procedure`

### 2. Standard Library (xdl-stdlib)

- **lib.rs**: Extended StandardLibrary to accept keyword arguments
  - Added `call_function_with_keywords` method
  - Added `call_procedure_with_keywords` method
  - Updated PLOT procedure routing to use `plot_with_keywords`

- **graphics_procs.rs**: Implemented `plot_with_keywords` function
  - Extracts `title`, `xtitle`, `ytitle` keywords (case-insensitive)
  - Creates `Plot2DConfig` with appropriate settings
  - Passes configuration to `plot_2d` function

### 3. Testing

Created test scripts demonstrating the new functionality:

- `test_plot_keywords.xdl`: Basic sine wave with titles
- `test_plot_keywords2.xdl`: Damped oscillation with different titles

## Usage

```xdl
; Generate data
x = findgen(100) / 10.0
y = sin(x)

; Plot with keyword arguments
plot, y, x, title='My Plot Title', xtitle='X Axis Label', ytitle='Y Axis Label'
```

## Supported Keywords

- `title` / `TITLE`: Main plot title
- `xtitle` / `XTITLE`: X-axis label
- `ytitle` / `YTITLE`: Y-axis label

Keywords are case-insensitive (both lowercase and uppercase work).

## Benefits

1. **User-friendly**: Natural syntax matching IDL/GDL conventions
2. **Extensible**: Infrastructure supports adding more keywords easily
3. **Backward compatible**: Old code without keywords still works
4. **Type-safe**: Keyword values are properly validated

## Future Enhancements

The keyword argument infrastructure can now be used to add more plotting options:

- Color selection (`color=`)
- Line styles (`linestyle=`)
- Symbol types (`psym=`)
- Axis ranges (`xrange=`, `yrange=`)
- Background color (`background=`)
- And more...

## Implementation Notes

- Keywords in XDL parser were already supported but not implemented in the evaluator
- The `Plot2DConfig` structure already had title fields, they just weren't populated from user input
- All keyword handling is centralized in the evaluator and stdlib layers
