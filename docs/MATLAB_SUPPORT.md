# MATLAB File Support in XDL

## Overview

XDL CLI and GUI now support loading and executing MATLAB `.m` files. The files are automatically detected by their `.m` extension and transpiled to XDL code before execution.

## Usage

### CLI

```bash
# Run a MATLAB .m file directly
xdl script.m

# The CLI will automatically:
# 1. Detect the .m extension
# 2. Transpile the MATLAB code to XDL
# 3. Execute the transpiled XDL code
```

### GUI

1. Open XDL GUI
2. Go to **File > Open...**
3. The file chooser now accepts both `.xdl` and `.m` files
4. Select a `.m` file
5. The MATLAB code will be automatically transpiled and loaded into the editor
6. Click **Execute** to run the code

## Features

- **Automatic Detection**: Files with `.m` extension are automatically recognized as MATLAB files
- **Seamless Transpilation**: MATLAB syntax is converted to XDL syntax behind the scenes
- **Function Mapping**: Common MATLAB functions are mapped to their XDL equivalents
- **Error Reporting**: Transpilation errors are clearly reported to the user

## Current Limitations

The MATLAB transpiler is a work in progress and has some limitations:

1. **Array Literals**: Array literal syntax `[1, 2, 3]` is not fully supported yet
   - Workaround: Use alternative syntax or XDL's array functions
   
2. **Complex Expressions**: Some complex MATLAB expressions may not transpile correctly
   
3. **Advanced Features**: Advanced MATLAB features like classes, packages, and some built-in functions may not be supported

## Example

### MATLAB Code (test_simple.m)
```matlab
% Simple calculation
x = 5;
y = x * 2;
disp(y);
```

### Execution
```bash
$ xdl test_simple.m
10
```

## Implementation Details

### CLI Integration
- File: `xdl-cli/src/main.rs`
- Function: `execute_file()`
- The function checks the file extension and calls `xdl_matlab::transpile_matlab_to_xdl()` for `.m` files

### GUI Integration
- File: `xdl-gui/src/gui.rs`
- Location: File Open dialog callback
- The GUI detects `.m` files and transpiles them before loading into the editor

## Future Improvements

1. Complete array literal support
2. Better error messages with line number mapping
3. Support for more MATLAB functions
4. MATLAB-specific debugging features
5. Option to view transpiled XDL code before execution
