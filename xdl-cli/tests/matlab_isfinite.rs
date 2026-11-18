use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn matlab_non_finite_constants_round_trip_through_classification_functions() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("xdl-isfinite-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script_path = directory.join("isfinite.m");
    fs::write(
        &script_path,
        "disp(isnan(NaN));\ndisp(isnan(Inf));\ndisp(isfinite(Inf));\ndisp(isfinite(-Inf));\ndisp(isinf(Inf));\ndisp(isinf(-Inf));\nmask = isnan([NaN, Inf, -Inf, 2]);\ndisp(mask);\nfinite_mask = isfinite([NaN, Inf, -Inf, 2]);\ndisp(finite_mask);\ninf_mask = isinf([NaN, Inf, -Inf, 2]);\ndisp(inf_mask);\n",
    )
    .expect("write MATLAB fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB isfinite fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    let values: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<i32>().ok())
        .collect();
    assert_eq!(values, [1, 0, 0, 0, 1, 1], "{stdout}");
    assert!(
        stdout.contains("[1.000000, 0.000000, 0.000000, 0.000000]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("[0.000000, 0.000000, 0.000000, 1.000000]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("[0.000000, 1.000000, 1.000000, 0.000000]"),
        "{stdout}"
    );
    let _ = fs::remove_dir_all(&directory);
}
