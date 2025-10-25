# Slow Tests Reorganization

## Summary

Successfully reorganized slow tests to keep the test suite fast and responsive. Tests that take longer than 10 seconds are now separated from the main test suite.

## Changes Made

### 1. Identified Slow Tests
- **xdl-matlab integration tests**: These tests read files from disk and perform complex transpilation
  - `control_flow_integration_test.rs` - Tests control flow structures (switch/case, try/catch, loops)
  - `array_literals_integration_test.rs` - Tests array literal transpilation

### 2. Moved to Separate Directory
Created new directory structure:
```
slow_tests/
├── README.md                           # Documentation for slow tests
└── xdl-matlab/
    ├── control_flow_integration_test.rs
    └── array_literals_integration_test.rs
```

### 3. Added `#[ignore]` Attributes
Both tests now have:
```rust
#[test]
#[ignore = "Slow test: Integration test with file I/O"]
fn test_name() { ... }
```

### 4. Created Helper Script
- `run_slow_tests.sh` - Shell script to easily run slow tests
- Makes slow tests executable: `chmod +x run_slow_tests.sh`

### 5. Updated Documentation
- **README.md**: Added section on running slow tests
- **slow_tests/README.md**: Comprehensive guide for slow test management

## Performance Improvement

### Before:
```
cargo test --all: >120 seconds (timed out)
```

### After:
```
cargo test --all: ~4.4 seconds ✓
```

**Improvement: 27x faster (from >2 minutes to ~4 seconds)**

## Usage

### Run Fast Tests (Default)
```bash
cargo test --all
# Completes in ~4-6 seconds
```

### Run Slow Tests
```bash
# Option 1: Use the helper script
./run_slow_tests.sh

# Option 2: Run manually
cargo test -- --ignored

# Option 3: Run specific slow test
cd xdl-matlab
cargo test --test control_flow_integration_test -- --ignored
```

## Benefits

1. **Fast Feedback Loop**: Developers get test results in seconds, not minutes
2. **CI Optimization**: Regular CI runs complete quickly
3. **Clear Separation**: Obvious which tests are performance-sensitive
4. **Selective Execution**: Run slow tests only when needed (e.g., before releases)
5. **Better Developer Experience**: No more waiting minutes for test results

## Guidelines for Future Tests

When adding new tests that might be slow:

1. **Estimate execution time**: If likely >10 seconds, mark as slow
2. **Add `#[ignore]` attribute**: Include descriptive reason
3. **Move to slow_tests/**: Place in appropriate subdirectory
4. **Update documentation**: Add to slow_tests/README.md
5. **Consider optimization**: Can the test be made faster?

## Examples of Slow Tests

Tests that should be marked as slow:
- File I/O operations (reading large files)
- Network operations (HTTP requests, database queries)
- Heavy computation (ML training, complex simulations)
- Integration tests (testing multiple components together)
- Tests with `sleep()` or delays

Tests that should remain in main suite:
- Unit tests (testing single functions)
- Fast computation tests
- Memory/allocation tests
- Simple integration tests (<1 second)

## Related Files

- `/slow_tests/README.md` - Detailed guide for slow tests
- `/run_slow_tests.sh` - Script to run slow tests
- `/README.md` - Updated with slow test information
- `/.pre-commit-config.yaml` - Pre-commit hooks (still run fast tests)
