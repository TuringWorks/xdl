use std::path::PathBuf;
use std::process::Command;

/// Get the path to the xdl binary (built by cargo test)
fn get_xdl_binary() -> PathBuf {
    // When running tests, cargo builds the binary and places it in target/debug
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    // Try debug build first, then release
    let debug_path = workspace_root.join("target/debug/xdl");
    let release_path = workspace_root.join("target/release/xdl");

    if debug_path.exists() {
        debug_path
    } else if release_path.exists() {
        release_path
    } else {
        // Fall back to using cargo run (slower but works during first test run)
        // Build the binary first
        let build_output = Command::new("cargo")
            .args(["build", "--bin", "xdl"])
            .current_dir(workspace_root)
            .output()
            .expect("Failed to build xdl");

        if !build_output.status.success() {
            panic!(
                "Failed to build xdl binary: {}",
                String::from_utf8_lossy(&build_output.stderr)
            );
        }

        debug_path
    }
}

#[test]
fn test_cli_version() {
    let binary = get_xdl_binary();

    let output = Command::new(&binary)
        .args(["--version"])
        .output()
        .expect("Failed to run xdl --version");

    assert!(
        output.status.success(),
        "xdl --version failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("xdl"),
        "Expected 'xdl' in version output, got: {}",
        stdout
    );
}

#[test]
fn test_cli_help() {
    let binary = get_xdl_binary();

    let output = Command::new(&binary)
        .args(["--help"])
        .output()
        .expect("Failed to run xdl --help");

    assert!(
        output.status.success(),
        "xdl --help failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Usage"),
        "Expected 'Usage' in help output, got: {}",
        stdout
    );
}

#[test]
fn test_cli_execute_expression() {
    let binary = get_xdl_binary();

    let output = Command::new(&binary)
        .args(["-e", "2 + 3"])
        .output()
        .expect("Failed to run xdl -e '2 + 3'");

    assert!(
        output.status.success(),
        "xdl -e '2 + 3' failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should output the result of 2 + 3 = 5
    assert!(
        stdout.contains("5"),
        "Expected '5' in output, got: {}",
        stdout
    );
}
