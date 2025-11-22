use std::path::PathBuf;
use std::process::Command;

#[test]
fn test_chart_viewer_help() {
    // Get the workspace root (parent of xdl-chart-viewer)
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let workspace_root = manifest_dir.parent().expect("Failed to get workspace root");

    let output = Command::new("cargo")
        .args(["run", "--bin", "xdl-chart-viewer", "--", "--help"])
        .current_dir(workspace_root)
        .output()
        .expect("Failed to run xdl-chart-viewer --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("XDL Chart Viewer"));
}
