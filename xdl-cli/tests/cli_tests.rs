use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

#[test]
fn test_cli_test_command_runs_nested_xdl_files_and_reports_summary() {
    let binary = get_xdl_binary();
    let directory = test_directory("passing");
    fs::create_dir_all(directory.join("nested")).expect("create test directory");
    fs::write(directory.join("nested/pass.xdl"), "PRINT, 'ok'\n").expect("write passing test");

    let output = Command::new(&binary)
        .args(["test", directory.to_str().expect("utf-8 test path")])
        .output()
        .expect("run xdl test command");

    let _ = fs::remove_dir_all(&directory);
    assert!(
        output.status.success(),
        "expected passing test command, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("1 passed, 0 failed"), "stdout: {stdout}");
}

#[test]
fn test_cli_test_command_writes_truthful_manifest() {
    let binary = get_xdl_binary();
    let directory = test_directory("manifest");
    fs::create_dir_all(&directory).expect("create test directory");
    fs::write(directory.join("pass.xdl"), "PRINT, 'ok'\n").expect("write passing test");
    let manifest = directory.join("run.json");

    let output = Command::new(&binary)
        .args([
            "test",
            directory.to_str().expect("utf-8 test path"),
            "--manifest",
            manifest.to_str().expect("utf-8 manifest path"),
        ])
        .output()
        .expect("run xdl test command");

    assert!(
        output.status.success(),
        "expected passing test command, stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let manifest_json = fs::read_to_string(&manifest).expect("read run manifest");
    let _ = fs::remove_dir_all(&directory);
    assert!(manifest_json.contains("\"status\": \"passed\""));
    assert!(manifest_json.contains("\"discovered\": 1"));
    assert!(manifest_json.contains("file hashes"));
}

#[test]
fn test_cli_test_command_fails_for_invalid_xdl() {
    let binary = get_xdl_binary();
    let directory = test_directory("failing");
    fs::create_dir_all(&directory).expect("create test directory");
    fs::write(directory.join("fail.xdl"), "not valid xdl syntax").expect("write failing test");

    let output = Command::new(&binary)
        .args(["test", directory.to_str().expect("utf-8 test path")])
        .output()
        .expect("run xdl test command");

    let _ = fs::remove_dir_all(&directory);
    assert!(!output.status.success(), "invalid test must fail");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("0 passed, 1 failed"), "stdout: {stdout}");
}

fn test_directory(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("xdl-cli-{label}-{}-{nanos}", std::process::id()))
}
