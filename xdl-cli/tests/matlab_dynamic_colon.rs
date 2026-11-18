use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn matlab_colon_vectors_accept_runtime_scalar_bounds_and_steps() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let unique_id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "xdl-matlab-dynamic-colon-{}-{nanos}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script = directory.join("dynamic_colon.m");
    fs::write(
        &script,
        "stop = 5; stride = 2; ascending = 1:stop; stepped = 1:stride:stop; descending = stop:-stride:1; empty = 1:-stride:stop; descending_default = stop:1; disp(ascending); disp(stepped); disp(descending); disp(numel(empty)); disp(numel(descending_default));\n",
    )
    .expect("write MATLAB fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    for expected in [
        "[1.000000, 2.000000, 3.000000, 4.000000, 5.000000]",
        "[1.000000, 3.000000, 5.000000]",
        "[5.000000, 3.000000, 1.000000]",
        "0",
    ] {
        assert!(
            stdout.contains(expected),
            "missing {expected}; stdout: {stdout}"
        );
    }
    let empty_counts: Vec<f64> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect();
    assert_eq!(empty_counts, [0.0, 0.0], "{stdout}");
    let _ = fs::remove_dir_all(directory);
}
