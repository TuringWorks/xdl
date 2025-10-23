# XDL Examples - Quick Start Guide

## Getting Started

All examples are ready to run! Simply use the `xdl` command followed by the file path.

## XDL Examples (Native)

Located in `examples/xdl/`

| File | Description | Command |
|------|-------------|---------|
| `01_hello_world.xdl` | Basic variables and arithmetic | `xdl examples/xdl/01_hello_world.xdl` |
| `02_arrays_and_loops.xdl` | Arrays and FOR loops | `xdl examples/xdl/02_arrays_and_loops.xdl` |
| `03_plotting_basics.xdl` | Basic plotting with labels | `xdl examples/xdl/03_plotting_basics.xdl` |
| `04_trigonometry.xdl` | Trig functions and plotting | `xdl examples/xdl/04_trigonometry.xdl` |
| `05_conditionals.xdl` | IF/THEN statements | `xdl examples/xdl/05_conditionals.xdl` |

## MATLAB Examples (Auto-Transpiled)

Located in `examples/matlab/`

| File | Description | Command |
|------|-------------|---------|
| `01_simple_math.m` | Basic MATLAB arithmetic | `xdl examples/matlab/01_simple_math.m` |
| `02_trigonometry.m` | MATLAB trig functions | `xdl examples/matlab/02_trigonometry.m` |
| `03_simple_operations.m` | sqrt, exp, log operations | `xdl examples/matlab/03_simple_operations.m` |

## Run All Tests

```bash
./examples/test_all.sh
```

This will run all examples in sequence and verify they work correctly.

## Example Output

### XDL Example
```bash
$ xdl examples/xdl/01_hello_world.xdl
Hello, XDL World!
The sum of 10 and 20 is 30
Product: 200
```

### MATLAB Example
```bash
$ xdl examples/matlab/01_simple_math.m
Basic arithmetic:
a =
5
b =
10
```

## Using the GUI

1. Launch GUI:
   ```bash
   xdl-gui
   ```

2. Open any example:
   - File > Open...
   - Navigate to `examples/xdl/` or `examples/matlab/`
   - Select a file
   - Click Execute

## Key Features Demonstrated

- ✅ Variable assignment and arithmetic
- ✅ Arrays with FINDGEN and FLTARR
- ✅ FOR loops with BEGIN...END
- ✅ Mathematical functions (SIN, COS, TAN, SQRT, EXP, LOG)
- ✅ Plotting with titles and axis labels
- ✅ Conditional logic (IF/THEN)
- ✅ MATLAB to XDL transpilation

## Need Help?

- See `examples/README.md` for detailed documentation
- See `docs/MATLAB_SUPPORT.md` for MATLAB compatibility info
- Check `IMPLEMENTATION_SUMMARY.md` for technical details

## Pro Tips

1. **Plotting**: Examples with PLOT commands will generate `xdl_plot.png`
2. **MATLAB Files**: `.m` files are automatically detected and transpiled
3. **Errors**: Check the output for clear error messages
4. **Learning**: Start with `01_hello_world` examples and progress sequentially
