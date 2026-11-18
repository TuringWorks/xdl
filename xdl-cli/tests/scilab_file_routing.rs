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
        "xdl-scilab-routing-{}-{nonce}-{unique_id}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("create temporary directory");
    directory
}

fn run_script(directory: &std::path::Path, name: &str, source: &str) -> (bool, String, String) {
    let script = directory.join(name);
    fs::write(&script, source).expect("write script");
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
fn sci_and_sce_use_scilab_transpilation_and_m_remains_matlab() {
    let directory = temp_directory();

    for (extension, source, expected) in [
        (
            "sce",
            "pi = %pi;\nif %t then\n  disp(11);\nelseif %f then\n  disp(22);\nelse\n  disp(33);\nend\n",
            "11",
        ),
        (
            "sci",
            "if %f then\n  disp(44);\nelseif %t then\n  disp(55);\nelse\n  disp(66);\nend\n",
            "55",
        ),
        (
            "SCE",
            "if %t then\n  disp(56);\nelse\n  disp(57);\nend\n",
            "56",
        ),
        (
            "SCI",
            "if %f then\n  disp(58);\nelse\n  disp(59);\nend\n",
            "59",
        ),
        ("m", "disp(77);\n", "77"),
        ("xdl", "PRINT, 88\n", "88"),
    ] {
        let (success, stdout, stderr) =
            run_script(&directory, &format!("routing.{extension}"), source);
        assert!(
            success,
            ".{extension} failed; stdout: {stdout}\nstderr: {stderr}"
        );
        let printed: Vec<_> = stdout
            .lines()
            .filter_map(|line| line.trim().parse::<i32>().ok())
            .collect();
        assert_eq!(
            printed,
            [expected.parse::<i32>().unwrap()],
            ".{extension}; stdout: {stdout}"
        );
    }

    let (success, stdout, stderr) = run_script(
        &directory,
        "vector.sce",
        "values = [11, 22, 33];\ndisp(values);\n",
    );
    assert!(
        success,
        "vector literal failed; stdout: {stdout}\nstderr: {stderr}"
    );
    for value in ["11", "22", "33"] {
        assert!(
            stdout.contains(value),
            "missing {value} in output: {stdout}"
        );
    }

    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_row_vector_transpose_executes_as_a_three_row_column() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "transpose.sce",
        "row = [11, 22, 33];\ncolumn = row';\ndisp(column);\n",
    );
    assert!(
        success,
        "transpose failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[[11.000000], [22.000000], [33.000000]]"),
        "expected a 3x1 column vector; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_single_output_function_can_be_called_from_same_file() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "function.sci",
        "answer = twice(21)\ndisp(answer)\nfunction result = twice(value)\nresult = value * 2\nendfunction\n",
    );
    assert!(
        success,
        "Scilab function failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "42"),
        "stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_function_with_four_inputs_executes_in_same_file() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "four-input-function.sci",
        "answer = combine(1, 2, 3, 4)\ndisp(answer)\nfunction result = combine(a, b, c, d)\nresult = a + b + c + d\nendfunction\n",
    );
    assert!(
        success,
        "Scilab multi-input function failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "10"),
        "expected four-input result 10; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_rectangular_matrix_literal_preserves_shape_and_row_column_order() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "matrix.sce",
        "A = [1,2;3,4];\ndisp(A);\ndisp(A(1,1));\ndisp(A(1,2));\ndisp(A(2,1));\ndisp(A(2,2));\n",
    );
    assert!(
        success,
        "matrix literal failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.contains("[[1.000000, 2.000000], [3.000000, 4.000000]]"),
        "expected ordered 2x2 values from Scilab disp; stdout: {stdout}"
    );
    for value in ["1.000000", "2.000000", "3.000000", "4.000000"] {
        assert!(
            stdout.matches(value).count() >= 2,
            "expected matrix display plus scalar read {value}; stdout: {stdout}"
        );
    }
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_vector_literal_supports_one_based_scalar_reads() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "vector-read.sce",
        "v = [10,20,30];\ndisp(v(1));\ndisp(v(3));\ndisp(total(v));\n",
    );
    assert!(
        success,
        "vector scalar reads failed; stdout: {stdout}\nstderr: {stderr}"
    );
    for expected in ["10", "30", "60"] {
        assert!(
            stdout
                .lines()
                .any(|line| line.trim().starts_with(&format!("{expected}.000000"))),
            "expected {expected} from vector reads and preserved sum call; stdout: {stdout}"
        );
    }
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_vector_literal_supports_inclusive_one_based_range_reads() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "vector-range.sce",
        "v = [10,20,30,40,50];\na = v(2:4);\nb = v(1:2:5);\nc = v(2:3:5);\ndisp(a);\ndisp(n_elements(a));\ndisp(b);\ndisp(n_elements(b));\ndisp(c);\ndisp(n_elements(c));\n",
    );
    assert!(
        success,
        "vector range reads failed; stdout: {stdout}; stderr: {stderr}"
    );
    assert!(
        stdout.contains("[20.000000, 30.000000, 40.000000]"),
        "2:4 should select a three-element vector; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[10.000000, 30.000000, 50.000000]"),
        "1:2:5 should include the inclusive endpoint; stdout: {stdout}"
    );
    assert!(
        stdout.contains("[20.000000, 50.000000]"),
        "2:3:5 should advance by the step without inventing an endpoint; stdout: {stdout}"
    );
    assert_eq!(
        stdout.lines().filter(|line| line.trim() == "3").count(),
        2,
        "the first two range results should have three elements; stdout: {stdout}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "2"),
        "the non-divisible stepped result should have two elements; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_vector_range_read_rejects_invalid_components_clearly() {
    let directory = temp_directory();
    for (name, source, expected) in [
        (
            "negative.sce",
            "v = [1,2,3];\ndisp(v(-1:3));\n",
            "positive integer",
        ),
        (
            "zero-step.sce",
            "v = [1,2,3];\ndisp(v(1:0:3));\n",
            "positive integer",
        ),
    ] {
        let (success, stdout, stderr) = run_script(&directory, name, source);
        assert!(!success, "unsupported vector range {name} should fail");
        assert!(
            stdout.to_ascii_lowercase().contains(expected)
                || stderr.to_ascii_lowercase().contains(expected),
            "expected a clear invalid-range diagnostic; stdout: {stdout}; stderr: {stderr}"
        );
    }
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_vector_scalar_read_accepts_runtime_variable_index() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "vector-bad-index.sce",
        "v = [10,20,30];\nk = 2;\ndisp(v(k));\n",
    );
    assert!(success, "variable vector index failed: {stderr}");
    assert!(
        stdout.lines().any(|line| line.trim().starts_with("20")),
        "expected the second vector element; stdout: {stdout}; stderr: {stderr}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_inclusive_for_range_executes_and_sums_literal_bounds() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "range.sce",
        "total = 0;\nfor i = 1:5\n  total = total + i;\nend\ndisp(total);\nstepped_total = 0;\nfor j = 1:2:5\n  stepped_total = stepped_total + j;\nend\ndisp(stepped_total);\n",
    );
    assert!(
        success,
        "for range failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "15"),
        "inclusive 1:5 range should sum to 15; stdout: {stdout}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "9"),
        "inclusive 1:2:5 range should sum to 9; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}

#[test]
fn scilab_for_range_with_mismatched_direction_executes_zero_iterations() {
    let directory = temp_directory();
    let (success, stdout, stderr) = run_script(
        &directory,
        "empty-range.sce",
        "total = 0;\nfor i = 5:1\n  total = total + i;\nend\ndisp(total);\n",
    );
    assert!(
        success,
        "empty range failed; stdout: {stdout}\nstderr: {stderr}"
    );
    assert!(
        stdout.lines().any(|line| line.trim() == "0"),
        "positive step with descending bounds should execute zero iterations; stdout: {stdout}"
    );
    let _ = fs::remove_dir_all(directory);
}
