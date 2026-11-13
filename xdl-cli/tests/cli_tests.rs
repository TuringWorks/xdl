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
    assert!(manifest_json.contains("\"status\": \"passed\""));
    assert!(manifest_json.contains("\"discovered\": 1"));
    assert!(manifest_json.contains("\"test_directory\": \".\""));
    assert!(manifest_json.contains("\"path\": \"pass.xdl\""));
    assert!(manifest_json.contains("\"completed_at\": \"unix:"));
    assert!(manifest_json.contains("\"duration_ms\":"));
    assert!(manifest_json.contains("\"kind\": \"xdl-test-source\""));
    assert!(manifest_json.contains(
        "\"sha256\": \"sha256:94ba589944019a00e78f1f53fe6e035e0d6850347c538b370145d4b4c6b80582\""
    ));
    assert!(manifest_json.contains("\"kind\": \"xdl-test-stdout\""));
    assert!(manifest_json.contains("\"path\": \"artifacts/pass.stdout\""));
    assert!(manifest_json.contains(
        "\"sha256\": \"sha256:dc51b8c96c2d745df3bd5590d990230a482fd247123599548e0632fdbf97fc22\""
    ));
    let stdout_artifact = directory.join("artifacts/pass.stdout");
    assert_eq!(
        fs::read_to_string(stdout_artifact).expect("read stdout artifact"),
        "ok\n"
    );
    let replay = Command::new(&binary)
        .args(["replay", manifest.to_str().expect("utf-8 manifest path")])
        .output()
        .expect("replay xdl test manifest");
    assert!(
        replay.status.success(),
        "expected replay to pass, stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(String::from_utf8_lossy(&replay.stdout).contains("1 passed, 0 failed"));
    fs::write(directory.join("pass.xdl"), "PRINT, 'changed'\n").expect("change test source");
    let changed_replay = Command::new(&binary)
        .args(["replay", manifest.to_str().expect("utf-8 manifest path")])
        .output()
        .expect("replay changed xdl test manifest");
    assert!(!changed_replay.status.success());
    let changed_diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&changed_replay.stdout),
        String::from_utf8_lossy(&changed_replay.stderr)
    );
    assert!(changed_diagnostic.contains("source hash mismatch"));
    let _ = fs::remove_dir_all(&directory);
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

#[test]
fn test_cli_replay_matches_recorded_failed_status() {
    let binary = get_xdl_binary();
    let directory = test_directory("replay-failed");
    fs::create_dir_all(&directory).expect("create test directory");
    fs::write(directory.join("fail.xdl"), "not valid xdl syntax").expect("write failing test");
    let manifest = directory.join("run.json");

    let run = Command::new(&binary)
        .args([
            "test",
            directory.to_str().expect("utf-8 test path"),
            "--manifest",
            manifest.to_str().expect("utf-8 manifest path"),
        ])
        .output()
        .expect("run failing xdl test command");
    assert!(!run.status.success());

    let replay = Command::new(&binary)
        .args(["replay", manifest.to_str().expect("utf-8 manifest path")])
        .output()
        .expect("replay failed xdl test manifest");
    assert!(
        replay.status.success(),
        "matching failed replay should succeed, stderr: {}",
        String::from_utf8_lossy(&replay.stderr)
    );
    assert!(String::from_utf8_lossy(&replay.stdout).contains("matched the recorded failed status"));
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn test_cli_test_command_writes_failure_artifact() {
    let binary = get_xdl_binary();
    let directory = test_directory("failing-manifest");
    fs::create_dir_all(&directory).expect("create test directory");
    fs::write(directory.join("fail.xdl"), "not valid xdl syntax").expect("write failing test");
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

    assert!(!output.status.success(), "invalid test must fail");
    let manifest_json = fs::read_to_string(&manifest).expect("read run manifest");
    assert!(manifest_json.contains("\"kind\": \"xdl-test-failure\""));
    assert!(manifest_json.contains("\"path\": \"artifacts/fail.failure.txt\""));
    let diagnostic = fs::read_to_string(directory.join("artifacts/fail.failure.txt"))
        .expect("read failure artifact");
    assert!(diagnostic.contains("Failed to execute program"));
    let _ = fs::remove_dir_all(&directory);
}

#[test]
fn test_cli_compatibility_report_json_is_explicit_about_scope() {
    let binary = get_xdl_binary();
    let output = Command::new(&binary)
        .args(["compatibility-report", "--format", "json"])
        .output()
        .expect("run compatibility report");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("xdl-cli-baseline-2026-11-12"));
    assert!(stdout.contains("array literals and basic matrix literals"));
    assert!(stdout.contains("unsupported"));
}

#[test]
fn test_cli_compatibility_report_rejects_unknown_format() {
    let binary = get_xdl_binary();
    let output = Command::new(&binary)
        .args(["compatibility-report", "--format", "yaml"])
        .output()
        .expect("run compatibility report");

    assert!(!output.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostic.contains("unsupported compatibility report format"));
}

#[test]
fn test_cli_compatibility_report_scans_matlab_corpus() {
    let binary = get_xdl_binary();
    let directory = test_directory("matlab-corpus");
    fs::create_dir_all(directory.join("nested")).expect("create corpus directory");
    fs::write(directory.join("nested/good.m"), "disp(1 + 2);\n")
        .expect("write supported MATLAB file");
    fs::write(directory.join("nested/good.m.expected"), "3\n")
        .expect("write MATLAB output reference");
    fs::write(directory.join("matrix.m"), "y = [1 2; 3 4];\n").expect("write matrix MATLAB file");
    fs::write(
        directory.join("matrix.m.expected.json"),
        r#"{"abs_tolerance": 0.0, "provenance": "test fixture", "values": {"y": [1.0, 2.0, 3.0, 4.0]}}"#,
    )
    .expect("write MATLAB numeric reference");

    let output = Command::new(&binary)
        .args([
            "compatibility-report",
            "--corpus",
            directory.to_str().expect("utf-8 corpus path"),
            "--format",
            "json",
            "--execute",
        ])
        .output()
        .expect("run MATLAB corpus report");

    let _ = fs::remove_dir_all(&directory);
    assert!(
        output.status.success(),
        "corpus report failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("nested/good.m"));
    assert!(stdout.contains("matrix.m"));
    assert!(stdout.contains("stdout-reference"));
    assert!(stdout.contains("numeric-reference"));
    assert!(stdout.contains("numeric-reference-provenance"));
    assert!(stdout.contains("verified"));
}

#[test]
fn test_cli_compatibility_report_can_require_references() {
    let binary = get_xdl_binary();
    let directory = test_directory("reference-gate");
    fs::create_dir_all(&directory).expect("create corpus directory");
    fs::write(directory.join("unreferenced.m"), "x = 1 + 2;\n")
        .expect("write unreferenced MATLAB file");

    let output = Command::new(&binary)
        .args([
            "compatibility-report",
            "--corpus",
            directory.to_str().expect("utf-8 corpus path"),
            "--execute",
            "--require-reference",
        ])
        .output()
        .expect("run reference gate");

    let _ = fs::remove_dir_all(&directory);
    assert!(!output.status.success());
    let diagnostic = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostic.contains("lack passing declared references"));
}

#[test]
fn test_cli_compatibility_report_rejects_empty_numeric_provenance() {
    let binary = get_xdl_binary();
    let directory = test_directory("empty-provenance");
    fs::create_dir_all(&directory).expect("create corpus directory");
    fs::write(directory.join("case.m"), "x = 3;\n").expect("write MATLAB case");
    fs::write(
        directory.join("case.m.expected.json"),
        r#"{"provenance":" ","values":{"x":[3.0]}}"#,
    )
    .expect("write invalid provenance reference");

    let output = Command::new(&binary)
        .args([
            "compatibility-report",
            "--corpus",
            directory.to_str().expect("utf-8 corpus path"),
            "--execute",
            "--format",
            "json",
        ])
        .output()
        .expect("run invalid provenance report");

    let _ = fs::remove_dir_all(&directory);
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("failed"));
    assert!(stdout.contains("provenance must not be empty"));
}

fn test_directory(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("xdl-cli-{label}-{}-{nanos}", std::process::id()))
}
