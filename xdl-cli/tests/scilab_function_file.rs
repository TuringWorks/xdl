use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_ID: AtomicU64 = AtomicU64::new(0);

#[test]
fn scilab_file_can_call_multiple_same_file_functions() {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let unique_id = TEMP_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "xdl-scilab-functions-{}-{nanos}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    let script = directory.join("helpers.sci");
    fs::write(
        &script,
        "answer = triple_then_add(5)\ndisp(answer)\nfunction out = triple_then_add(x)\nscaled = triple(x)\nout = scaled + 1\nendfunction\nfunction out = triple(x)\nout = x * 3\nendfunction\n",
    )
    .expect("write Scilab fixture");
    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script)
        .current_dir(&directory)
        .output()
        .expect("run Scilab fixture");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "16"),
        "expected nested same-file function result 16; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}
