use std::path::PathBuf;
use std::process::Command;

/// Get the path to the xdl-chart-viewer binary (built by cargo test)
fn get_chart_viewer_binary() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    // Try debug build first, then release
    let debug_path = workspace_root.join("target/debug/xdl-chart-viewer");
    let release_path = workspace_root.join("target/release/xdl-chart-viewer");

    if debug_path.exists() {
        debug_path
    } else if release_path.exists() {
        release_path
    } else {
        // Build the binary first
        let build_output = Command::new("cargo")
            .args(["build", "--bin", "xdl-chart-viewer"])
            .current_dir(workspace_root)
            .output()
            .expect("Failed to build xdl-chart-viewer");

        if !build_output.status.success() {
            panic!(
                "Failed to build xdl-chart-viewer binary: {}",
                String::from_utf8_lossy(&build_output.stderr)
            );
        }

        debug_path
    }
}

#[test]
fn test_chart_viewer_help() {
    let binary = get_chart_viewer_binary();

    // Use --help-only flag which prints help without launching GUI
    // This allows the test to run in headless environments
    let output = Command::new(&binary)
        .args(["--help-only"])
        .output()
        .expect("Failed to run xdl-chart-viewer --help-only");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("XDL Chart Viewer"),
        "Expected 'XDL Chart Viewer' in output, got: {}",
        stdout
    );
}

#[test]
fn test_chart_viewer_version() {
    let binary = get_chart_viewer_binary();

    let output = Command::new(&binary)
        .args(["--version"])
        .output()
        .expect("Failed to run xdl-chart-viewer --version");

    assert!(
        output.status.success(),
        "Command failed with stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("xdl-chart-viewer"),
        "Expected version info in output, got: {}",
        stdout
    );
}
