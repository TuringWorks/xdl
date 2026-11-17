use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn temporary_directory() -> std::path::PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("xdl-readmatrix-{}-{nanos}", std::process::id()))
}

#[test]
fn matlab_readmatrix_imports_numeric_csv_for_indexing() {
    let directory = temporary_directory();
    fs::create_dir_all(&directory).expect("create temporary directory");
    let csv_path = directory.join("samples.csv");
    let script_path = directory.join("read_samples.m");
    fs::write(&csv_path, "1,2.5\n-3,4\n").expect("write numeric CSV");
    let csv_path = csv_path.to_string_lossy();
    fs::write(
        &script_path,
        format!(
            "samples = readmatrix('{csv_path}');\ndisp(samples(1, 1));\ndisp(samples(1, 2));\ndisp(samples(2, 1));\ndisp(samples(2, 2));\ndisp(size(samples, 1));\ndisp(size(samples, 2));\nx = samples(:, 1);\ny = samples(:, 2);\nplot(x, y);\n"
        ),
    )
    .expect("write MATLAB fixture");

    let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
        .args(["-q"])
        .arg(&script_path)
        .current_dir(&directory)
        .output()
        .expect("run MATLAB CSV fixture");
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
    assert_eq!(values, [1.0, 2.5, -3.0, 4.0, 2.0, 2.0], "{stdout}");
    let plot_path = directory.join("xdl_plot.png");
    assert!(
        plot_path.is_file() && fs::metadata(&plot_path).unwrap().len() > 0,
        "CSV data did not produce a non-empty native plot"
    );
    let _ = fs::remove_dir_all(&directory);
}
