use std::process::Command;

#[test]
fn test_cli_version() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "--version"])
        .current_dir("/Users/ravindraboddipalli/sources/xdl")
        .output()
        .expect("Failed to run xdl --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xdl"));
}

#[test]
fn test_cli_help() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "--help"])
        .current_dir("/Users/ravindraboddipalli/sources/xdl")
        .output()
        .expect("Failed to run xdl --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Extended Data Language"));
}

#[test]
fn test_cli_execute_expression() {
    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "-e", "2 + 3"])
        .current_dir("/Users/ravindraboddipalli/sources/xdl")
        .output()
        .expect("Failed to run xdl -e '2 + 3'");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output the result of 2 + 3 = 5
    assert!(stdout.contains("5"));
}
