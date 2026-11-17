use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn matlab_two_output_find_returns_column_major_row_and_column_subscripts() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory =
        std::env::temp_dir().join(format!("xdl-find-outputs-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script_path = directory.join("find_outputs.m");
    fs::write(
        &script_path,
        "matrix = [0, 2, 0; 3, 0, 4];\n[row, col] = find(matrix);\ndisp(row);\ndisp(col);\n[first_rows, first_cols] = find(matrix, 2, 'first');\ndisp(first_rows);\ndisp(first_cols);\n[last_rows, last_cols, linear] = find(matrix, 2, 'last');\ndisp(last_rows);\ndisp(last_cols);\ndisp(linear);\nzeroes = zeros(2, 3);\n[empty_row, empty_col] = find(zeroes);\ndisp(size(empty_row, 1));\ndisp(size(empty_col, 2));\n",
    )
    .expect("write MATLAB fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB two-output find fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    let _ = fs::remove_dir_all(&directory);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("Array[3x1]: [2.000, 1.000, 2.000]"),
        "{stdout}"
    );
    assert!(
        stdout.contains("Array[3x1]: [1.000, 2.000, 3.000]"),
        "{stdout}"
    );
    assert!(stdout.contains("Array[2x1]: [2.000, 1.000]"), "{stdout}");
    assert!(stdout.contains("Array[2x1]: [1.000, 2.000]"), "{stdout}");
    assert!(stdout.contains("Array[2x1]: [1.000, 2.000]"), "{stdout}");
    assert!(stdout.contains("Array[2x1]: [2.000, 3.000]"), "{stdout}");
    assert!(stdout.contains("Array[2x1]: [3.000, 6.000]"), "{stdout}");
    let dimensions: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect();
    assert_eq!(dimensions, [0.0, 1.0], "{stdout}");
}
