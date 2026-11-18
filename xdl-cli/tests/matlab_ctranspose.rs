use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn matlab_conjugating_transpose_executes_for_real_rectangular_matrix() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("xdl-ctranspose-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script_path = directory.join("ctranspose.m");
    fs::write(
        &script_path,
        "A = [1, 2, 3; 4, 5, 6];\nB = A';\ndisp(size(B, 1));\ndisp(size(B, 2));\ndisp(B(1, 1));\ndisp(B(3, 2));\n",
    )
    .expect("write MATLAB fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB transpose fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&directory);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let values: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect();
    assert_eq!(values, [3.0, 2.0, 1.0, 6.0], "{stdout}");
}
