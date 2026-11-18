use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn matlab_numeric_vector_indices_preserve_order_and_duplicates() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let unique_id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "xdl-matlab-vector-index-{}-{nanos}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script = directory.join("vector_index.m");
    fs::write(
        &script,
        "values = [10, 20, 30, 40]; indices = [4, 1, 4]; picked = values(indices); literal = values([2, 4]); empty = values(1:-1:3); all_values = values(:); matrix = [10, 20; 30, 40]; matrix_picks = matrix([2, 4]); matrix_all = matrix(:); disp(picked); disp(literal); disp(empty); disp(all_values); disp(matrix_picks); disp(matrix_all);\n",
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
    assert!(
        stdout.contains("[40.000000, 10.000000, 40.000000]"),
        "variable indices must preserve order and duplicates; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[20.000000, 40.000000]"),
        "literal vector indices must select expected elements; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[]"),
        "empty MATLAB index vectors should produce empty selections; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[10.000000, 20.000000, 30.000000, 40.000000]"),
        "A(:) should select all vector elements; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[30.000000, 40.000000]"),
        "matrix linear indices should follow MATLAB column-major order; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[10.000000, 30.000000, 20.000000, 40.000000]"),
        "matrix full indexing should flatten in MATLAB column-major order; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}
