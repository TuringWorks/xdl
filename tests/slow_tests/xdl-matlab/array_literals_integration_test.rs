use std::fs;
use xdl_matlab::transpile_matlab_to_xdl;

#[test]
#[ignore = "Slow test: Integration test with file I/O"]
fn test_array_literals_file() {
    let matlab_code = fs::read_to_string("tests/array_literals_test.m")
        .expect("Failed to read array_literals_test.m");

    let result = transpile_matlab_to_xdl(&matlab_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let xdl_code = result.unwrap();

    println!("=== Transpiled XDL Code ===");
    println!("{}", xdl_code);
    println!("=== End ===");

    // Test for specific features

    // Simple row vector
    assert!(
        xdl_code.contains("[1, 2, 3, 4, 5]"),
        "Simple row vector not found"
    );

    // Array generation functions
    assert!(
        xdl_code.contains("FLTARR(5)"),
        "zeros() not transpiled correctly"
    );
    assert!(
        xdl_code.contains("FLTARR(3, 4) + 1"),
        "ones() not transpiled correctly"
    );
    assert!(
        xdl_code.contains("IDENTITY(4)"),
        "eye() not transpiled correctly"
    );

    // Colon ranges
    assert!(
        xdl_code.contains("FINDGEN"),
        "Colon ranges not using FINDGEN"
    );

    // linspace
    assert!(
        xdl_code.contains("linspace") || xdl_code.contains("FINDGEN"),
        "linspace not transpiled"
    );

    // Array operations
    assert!(
        xdl_code.contains("a + b") || xdl_code.contains("a  +  b"),
        "Array addition not found"
    );
    assert!(
        xdl_code.contains("a * b") || xdl_code.contains("a  *  b"),
        "Element-wise multiply not found"
    );
}
