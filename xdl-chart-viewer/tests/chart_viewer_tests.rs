use std::process::Command;

#[test]
fn test_chart_viewer_help() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "xdl-chart-viewer", "--", "--help"])
        .current_dir("/Users/ravindraboddipalli/sources/xdl")
        .output()
        .expect("Failed to run xdl-chart-viewer --help");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("XDL Chart Viewer"));
}