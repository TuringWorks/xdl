use std::fs;
use xdl_matlab::transpile_matlab_to_xdl;

#[test]
#[ignore = "Slow test: Integration test with file I/O"]
fn test_control_flow_file() {
    let matlab_code = fs::read_to_string("tests/control_flow_test.m")
        .expect("Failed to read control_flow_test.m");

    let result = transpile_matlab_to_xdl(&matlab_code);
    assert!(result.is_ok(), "Transpilation failed: {:?}", result.err());

    let xdl_code = result.unwrap();

    println!("=== Transpiled XDL Code ===");
    println!("{}", xdl_code);
    println!("=== End ===");

    // Test for switch/case
    assert!(
        xdl_code.contains("CASE x OF") || xdl_code.contains("CASE"),
        "Switch statement not found"
    );
    assert!(xdl_code.contains(": BEGIN"), "Case blocks not found");
    assert!(
        xdl_code.contains("ELSE: BEGIN"),
        "Otherwise clause not found"
    );
    assert!(xdl_code.contains("ENDCASE"), "ENDCASE not found");

    // Test for try/catch
    assert!(
        xdl_code.contains("TRY block"),
        "TRY block comment not found"
    );
    assert!(
        xdl_code.contains("CATCH block"),
        "CATCH block comment not found"
    );

    // Test for break/continue/return
    assert!(xdl_code.contains("BREAK"), "BREAK statement not found");
    assert!(
        xdl_code.contains("CONTINUE"),
        "CONTINUE statement not found"
    );
    assert!(xdl_code.contains("RETURN"), "RETURN statement not found");

    // Test for loops with steps
    assert!(xdl_code.contains("for"), "For loops not found");
    assert!(xdl_code.contains("endfor"), "endfor not found");

    // Test for while loops
    assert!(xdl_code.contains("while"), "While loops not found");
    assert!(xdl_code.contains("endwhile"), "endwhile not found");

    // Test for nested control flow
    assert!(xdl_code.contains("if"), "If statements not found");
    assert!(xdl_code.contains("endif"), "endif not found");
}
