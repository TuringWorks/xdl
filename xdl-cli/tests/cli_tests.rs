use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_cli_version() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "--version"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run xdl --version");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xdl"));
}

#[test]
fn test_cli_help() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "--help"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run xdl --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Usage"));
}

#[test]
fn test_cli_execute_expression() {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl", "--", "-e", "2 + 3"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run xdl -e '2 + 3'");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output the result of 2 + 3 = 5
    assert!(stdout.contains("5"));
}
