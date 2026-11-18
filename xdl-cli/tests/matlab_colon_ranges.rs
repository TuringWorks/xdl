use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIRECTORY_COUNTER: AtomicU64 = AtomicU64::new(0);

#[test]
fn matlab_literal_colon_vectors_execute_with_inclusive_directional_semantics() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let counter = TEMP_DIRECTORY_COUNTER.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "xdl-matlab-colon-{}-{nanos}-{counter}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script_path = directory.join("colon.m");
    fs::write(
        &script_path,
        "ascending = 1:5; descending = 5:-2:1; empty = 1:-1:3; disp(ascending(1)); disp(ascending(5)); disp(descending(1)); disp(descending(3)); disp(size(empty, 2));\n",
    )
    .expect("write MATLAB fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB colon fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let values: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect();
    assert_eq!(values, [1.0, 5.0, 5.0, 1.0, 0.0], "{stdout}");
    let _ = fs::remove_dir_all(&directory);
}
