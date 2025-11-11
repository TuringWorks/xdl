# Unsupported Test Cases

This directory contains test scripts that currently fail due to unsupported features or known issues.

## Overview

- **Total Scripts**: 22
- **Status**: Temporarily unsupported
- **Target**: To be fixed in future releases

## Categorized Issues

### Parse Errors (9 scripts)

These scripts fail during parsing due to unsupported syntax:

1. **advanced_control_flow_tests.xdl**
   - Error: Unexpected token: End
   - Issue: Complex control flow structure

2. **comprehensive_language_tests.xdl**
   - Error: Unexpected token: Colon
   - Issue: Label/goto syntax or ternary operators

3. **comprehensive_working_features_test.xdl**
   - Error: Unexpected token: Colon
   - Issue: Same as above

4. **idl_gdl_compatibility_tests.xdl**
   - Error: Unexpected token: Else
   - Issue: ELSE without proper IF/BEGIN structure

5. **integration_regression_tests.xdl**
   - Error: Unexpected token: Else
   - Issue: Same as above

6. **matlab_transpilation_tests.xdl**
   - Error: Unexpected token: RightParen
   - Issue: MATLAB-specific syntax

7. **test_for_loop.xdl**
   - Error: Expected 'endfor'
   - Issue: Complex FOR loop nesting

8. **test_graphics_1.xdl**
   - Error: Unexpected token: Dot
   - Issue: Dot notation or range operator

9. **test_parser_v2.xdl**
   - Error: Unexpected token: Comma
   - Issue: Special comma handling in arrays

10. **test_visualization.xdl**
    - Error: Expected ')' after function arguments
    - Issue: Keyword argument parsing

11. **test_fluid_partial.xdl**
    - Error: Unexpected token: Divide
    - Issue: Comment or division operator parsing

### Not Implemented Features (3 scripts)

These require features not yet implemented:

1. **test_string_basic.xdl**
   - Error: Not implemented: User-defined procedures
   - Needs: PRO/ENDPRO procedure definitions

2. **test_string_conversion.xdl**
   - Error: Not implemented: User-defined procedures
   - Needs: PRO/ENDPRO procedure definitions

3. **ml_cross_validation_test.xdl**
   - Error: Unexpected token: End
   - Needs: Investigation

### Runtime Errors (5 scripts)

These parse successfully but fail during execution:

1. **test_complex.xdl**
   - Error: Type mismatch: expected numeric, got DComplex
   - Issue: Complex number arithmetic not fully supported

2. **test_reform.xdl**
   - Error: Type mismatch: expected array, got Float
   - Issue: REFORM function type handling

3. **numpy_test.xdl**
   - Error: Cannot convert XdlValue to Python: Array
   - Issue: Python type conversion for arrays

4. **test_2d_indexing.xdl**
   - Error: Cannot index non-array value
   - Issue: Advanced array indexing edge case

5. **test_graphics.xdl**
   - Error: Runtime error (various)
   - Needs: Investigation

### Performance/Special Cases (3 scripts)

These are special cases or intentionally slow:

1. **test_cancel.xdl**
   - Timeout: >15 seconds
   - Note: Intentional stress test for GUI cancel functionality

2. **performance_stress_tests.xdl**
   - Parse error + performance issues
   - Note: Large-scale performance tests

3. **test_simple.xdl**
   - Error: Function not found: TEST
   - Note: Uses placeholder TEST function (not a real function)

## Priority for Fixes

### High Priority (Quick Wins - 6 scripts)
- User-defined procedures (PRO/ENDPRO) → Would fix 2 scripts
- ELSE parsing in control structures → Would fix 2 scripts
- Complex number arithmetic → Would fix 1 script
- test_reform type handling → Would fix 1 script

**Impact**: Fixing these would bring pass rate from 64% to 74%

### Medium Priority (7 scripts)
- Label/goto syntax (colon operator)
- Advanced FOR loop parsing
- Dot notation support
- Array comma handling
- Python array type conversion

**Impact**: Would bring pass rate to 86%

### Low Priority (9 scripts)
- MATLAB transpilation
- Performance test optimizations
- Special graphics edge cases
- Placeholder functions

## How to Re-enable Tests

When features are implemented:

1. Fix the underlying issue
2. Test the script: `./target/debug/xdl <script>.xdl`
3. If passing, move back to `tests_failing/` or preferably to `tests_working/`
4. Update this README

## Related Documentation

- See `/tmp/FINAL_TEST_REPORT.md` for comprehensive analysis
- See git history for what was tried to fix each script
- See `xdl-stdlib/src/lib.rs` for available functions
- See `xdl-parser/` for parser implementation

## Statistics

- Parse Errors: 11 (50%)
- Not Implemented: 3 (14%)
- Runtime Errors: 5 (23%)
- Special Cases: 3 (14%)

Last Updated: 2025-11-11
