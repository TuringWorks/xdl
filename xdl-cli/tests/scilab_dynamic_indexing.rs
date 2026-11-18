use std::fs;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static TEMP_DIRECTORY_ID: AtomicU64 = AtomicU64::new(0);

fn temp_directory() -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let unique_id = TEMP_DIRECTORY_ID.fetch_add(1, Ordering::Relaxed);
    let directory = std::env::temp_dir().join(format!(
        "xdl-scilab-dynamic-index-{}-{nonce}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    directory
}

fn run_script(directory: &std::path::Path, name: &str, source: &str) -> (bool, String, String) {
    let script = directory.join(name);
    fs::write(&script, source).expect("write Scilab script");
    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(script)
        .current_dir(directory)
        .output()
        .expect("run XDL CLI");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

#[test]
fn scilab_dynamic_scalar_index_reads_preserve_one_based_semantics() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "dynamic-index.sce",
        "v = [10,20,30,40];\nfor i = 1:3\n  disp(v(i));\nend\ndisp(v(i + 1));\n",
    );

    assert!(
        success,
        "dynamic Scilab scalar index reads failed; stdout: {stdout}; stderr: {stderr}"
    );
    let printed: Vec<_> = stdout
        .lines()
        .filter_map(|line| line.trim().parse::<f64>().ok())
        .collect();
    assert_eq!(
        printed,
        [10.0, 20.0, 30.0, 40.0],
        "loop and computed indices must remain one-based; stdout: {stdout}"
    );

    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_dynamic_scalar_index_reads_reject_invalid_indices() {
    let directory = temp_directory();

    for (name, source) in [
        ("zero.sce", "v = [10,20,30];\nk = 0;\ndisp(v(k));\n"),
        ("negative.sce", "v = [10,20,30];\nk = -1;\ndisp(v(k));\n"),
        ("fractional.sce", "v = [10,20,30];\nk = 1.5;\ndisp(v(k));\n"),
        (
            "out-of-bounds.sce",
            "v = [10,20,30];\nk = 4;\ndisp(v(k));\n",
        ),
    ] {
        let (success, stdout, stderr) = run_script(&directory, name, source);
        assert!(
            !success,
            "invalid dynamic index in {name} should fail instead of wrapping, truncating, or silently indexing; stdout: {stdout}; stderr: {stderr}"
        );
    }

    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_dynamic_vector_ranges_preserve_inclusive_bounds_and_steps() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "dynamic-range.sce",
        "v = [10,20,30,40,50];\nstart = 2;\nfinish = 4;\nstep = 2;\ndisp(v(start:finish));\ndisp(v(start:step:finish));\ndisp(v(4:2));\n",
    );
    assert!(
        success,
        "dynamic Scilab range failed; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        stdout.contains("[20.000000, 30.000000, 40.000000]"),
        "{stdout}"
    );
    assert!(stdout.contains("[20.000000, 40.000000]"), "{stdout}");
    assert!(
        stdout.contains("[]"),
        "direction-mismatch range should be empty; {stdout}"
    );

    for (name, source) in [
        (
            "zero-step.sce",
            "v = [1,2,3];\nstep = 0;\ndisp(v(1:step:3));\n",
        ),
        (
            "out-of-range.sce",
            "v = [1,2,3];\nfinish = 4;\ndisp(v(1:finish));\n",
        ),
    ] {
        let (valid, run_stdout, run_stderr) = run_script(&directory, name, source);
        assert!(
            !valid,
            "invalid range {name} should fail; stdout: {run_stdout}; stderr: {run_stderr}"
        );
    }
    let _ = fs::remove_dir_all(directory);
}
