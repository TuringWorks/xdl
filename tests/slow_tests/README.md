# Slow Tests

This directory contains tests that take longer than 10 seconds to run. These tests are excluded from the standard `cargo test` runs to keep the test suite fast.

## Running Slow Tests

To run the slow tests, use one of the following commands:

### Run all slow tests
```bash
# From the xdl root directory
cargo test --test '*' -- --ignored
```

### Run specific slow test files
```bash
# Run MATLAB integration tests
cd xdl-matlab
cargo test --test control_flow_integration_test -- --ignored
cargo test --test array_literals_integration_test -- --ignored
```

### Run slow tests as regular Rust files
```bash
# Compile and run manually
rustc slow_tests/xdl-matlab/control_flow_integration_test.rs --test -L target/debug/deps --extern xdl_matlab=target/debug/libxdl_matlab.rlib
./control_flow_integration_test
```

## Test Organization

- `xdl-matlab/` - MATLAB transpiler integration tests that require file I/O
  - `control_flow_integration_test.rs` - Tests for control flow structures (switch/case, try/catch, loops)
  - `array_literals_integration_test.rs` - Tests for array literal transpilation

## Adding New Slow Tests

When adding a new test that takes longer than 10 seconds:

1. Add the `#[ignore]` attribute with a descriptive reason:
   ```rust
   #[test]
   #[ignore = "Slow test: <reason>"]
   fn my_slow_test() {
       // test code
   }
   ```

2. Move the test file to the appropriate subdirectory in `slow_tests/`

3. Update this README with information about the new test

## Why Separate Slow Tests?

- **Fast feedback loop**: Developers can run `cargo test` quickly during development
- **CI optimization**: Regular CI runs stay fast, slow tests can run in nightly or release builds
- **Clear separation**: Makes it obvious which tests are performance-sensitive
- **Selective execution**: Run slow tests only when needed (e.g., before releases)
