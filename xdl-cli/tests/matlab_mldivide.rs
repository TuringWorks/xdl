use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn matlab_left_division_solves_vector_and_matrix_right_hand_sides() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("xdl-mldivide-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script_path = directory.join("mldivide.m");
    fs::write(
        &script_path,
        "A = [3, 1; 1, 2];\nb = [9; 8];\nx = A \\ b;\ndisp(x);\nB = [9, 7; 8, 5];\nX = A \\ B;\ndisp(X);\nTall = [1, 0; 1, 1; 1, 2];\nobserved = [1; 2; 2];\nfit = Tall \\ observed;\ndisp(fit);\nTall_B = [1, 2; 2, 4; 2, 4];\nfit_matrix = Tall \\ Tall_B;\ndisp(fit_matrix);\nWide = [1, 1];\nscalar_rhs = [2];\nminimum_norm = Wide \\ scalar_rhs;\ndisp(minimum_norm);\n",
    )
    .expect("write MATLAB fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB left-division fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&directory);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(stdout.contains("[2.000, 3.000]"), "{stdout}");
    assert!(stdout.contains("[2.000, 1.800, 3.000, 1.600]"), "{stdout}");
    assert!(stdout.contains("[1.167, 0.500]"), "{stdout}");
    assert!(
        stdout.contains("Array[2x2]: [1.167, 2.333, 0.500, 1.000]"),
        "{stdout}"
    );
    assert!(stdout.contains("[1.000000, 1.000000]"), "{stdout}");
}
