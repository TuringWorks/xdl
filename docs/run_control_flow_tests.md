# XDL Control Flow Test Suite

This directory contains comprehensive tests for XDL's control flow constructs (loops and conditions).

## Test Files

### 1. `unit_control_flow_tests.xdl`

**Purpose**: Basic unit tests for individual control flow constructs
**Coverage**:

- Simple IF/THEN/ELSE statements
- Basic FOR and WHILE loops
- BREAK and CONTINUE statements
- Comparison operators (EQ, NE, LT, GT, LE, GE)
- Logical operators (AND, OR, NOT)
- Variable scoping in loops
- Arithmetic expressions in conditions

**Usage**:

```bash
xdl unit_control_flow_tests.xdl
```

### 2. `control_flow_tests.xdl`

**Purpose**: Comprehensive integration tests for control flow features
**Coverage**:

- Nested IF statements
- Nested loops (FOR and WHILE)
- Loop control (BREAK/CONTINUE)
- Combined control flow patterns
- Array processing with loops
- Edge cases and boundary conditions
- Complex conditional expressions

**Usage**:

```bash
xdl control_flow_tests.xdl
```

### 3. `advanced_control_flow_tests.xdl`

**Purpose**: Real-world algorithms demonstrating advanced control flow patterns
**Coverage**:

- Search algorithms (binary search)
- Sort algorithms (bubble sort)
- Mathematical algorithms (GCD, factorial, prime factorization)
- Pattern generation (Pascal's triangle, multiplication tables)
- Data processing (moving averages, local maxima)
- Error handling patterns
- Loop optimization techniques

**Usage**:

```bash
xdl advanced_control_flow_tests.xdl
```

## Test Categories

### Basic Control Structures

- [x] IF/THEN/ELSE conditionals
- [x] FOR loops with start, end, step
- [x] WHILE loops
- [x] BREAK and CONTINUE statements

### Comparison Operators

- [x] EQ (equal)
- [x] NE (not equal)
- [x] LT (less than)
- [x] GT (greater than)
- [x] LE (less than or equal)
- [x] GE (greater than or equal)

### Logical Operators

- [x] AND logical conjunction
- [x] OR logical disjunction
- [x] NOT logical negation

### Advanced Patterns

- [x] Nested control structures
- [x] Loop with array processing
- [x] Early termination patterns
- [x] Error handling with conditions
- [x] Complex algorithmic patterns

## Expected Behavior

### Parser Tests

These files test the XDL parser's ability to:

1. Recognize control flow keywords (IF, FOR, WHILE, etc.)
2. Parse nested structures correctly
3. Handle complex expressions in conditions
4. Generate proper AST nodes for control flow

### Interpreter Tests

These files test the XDL interpreter's ability to:

1. Execute conditional branches correctly
2. Iterate loops with proper variable updates
3. Handle BREAK and CONTINUE statements
4. Maintain proper variable scoping
5. Evaluate complex boolean expressions

## Running the Tests

### Individual Test Files

```bash
# Run basic unit tests
xdl unit_control_flow_tests.xdl

# Run comprehensive tests
xdl control_flow_tests.xdl

# Run advanced algorithm tests
xdl advanced_control_flow_tests.xdl
```

### All Tests (when test runner is implemented)

```bash
# Future test runner command
xdl test control_flow
```

## Integration with CI/CD

These tests should be integrated into the XDL build pipeline:

```yaml
# .github/workflows/ci.yml
- name: Run Control Flow Tests
  run: |
    cargo build --release
    ./target/release/xdl unit_control_flow_tests.xdl
    ./target/release/xdl control_flow_tests.xdl
    ./target/release/xdl advanced_control_flow_tests.xdl
```

## Development Guidelines

### Adding New Control Flow Tests

1. **Unit Tests**: Add simple, focused tests to `unit_control_flow_tests.xdl`
2. **Integration Tests**: Add complex scenarios to `control_flow_tests.xdl`
3. **Algorithm Tests**: Add real-world examples to `advanced_control_flow_tests.xdl`

### Test Naming Convention

- Prefix test descriptions with "Test N:" for unit tests
- Use descriptive comments for integration tests
- Include algorithm names for advanced tests

### Expected Output Format

Each test should produce clear, verifiable output:

```text
Test 1 PASS: Basic if-then
Test 2 PASS: Basic if-then-else
Test 3 PASS: If with EQ comparison
```

## Future Enhancements

### Planned Control Flow Features

- [ ] FOREACH loops for array iteration
- [ ] REPEAT/UNTIL loops
- [ ] CASE/OF statements (switch-like)
- [ ] Exception handling (TRY/CATCH)

### Additional Test Categories

- [ ] Performance benchmarks for loop constructs
- [ ] Memory usage tests for nested loops
- [ ] Stress tests with deep nesting
- [ ] Compatibility tests with IDL syntax

## Validation Checklist

When implementing control flow features, ensure:

- [ ] Parser recognizes all keywords correctly
- [ ] AST nodes are generated properly
- [ ] Interpreter executes control flow correctly
- [ ] Variable scoping works as expected
- [ ] Error handling is appropriate
- [ ] All test files run without errors
- [ ] Output matches expected results

## Contributing

When adding new control flow tests:

1. Follow existing naming conventions
2. Include clear comments explaining the test purpose
3. Add both positive and negative test cases
4. Update this documentation
5. Verify tests work with current parser implementation
