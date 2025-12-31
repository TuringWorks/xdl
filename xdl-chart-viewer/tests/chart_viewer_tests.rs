use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_chart_viewer_help() {
    // Get the workspace root (parent of xdl-chart-viewer)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    // Use --help-only flag which prints help without launching GUI
    // This allows the test to run in headless environments
    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl-chart-viewer", "--", "--help-only"])
        .current_dir(workspace_root)
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
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl-chart-viewer", "--", "--version"])
        .current_dir(workspace_root)
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
