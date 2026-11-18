use std::fs;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn matlab_elseif_executes_exactly_the_selected_branch() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock after epoch")
        .as_nanos();
    let directory = std::env::temp_dir().join(format!("xdl-elseif-{}-{nonce}", std::process::id()));
    fs::create_dir_all(&directory).expect("create temporary directory");

    for (input, expected) in [(1, 11), (2, 22), (3, 33), (4, 44)] {
        let script_path = directory.join(format!("elseif-{input}.m"));
        fs::write(
            &script_path,
            format!(
                "mode = {input};\nif mode == 1\n  disp(11);\nelseif mode == 2\n  disp(22);\nelseif mode == 3\n  disp(33);\nelse\n  disp(44);\nend\n"
            ),
        )
        .expect("write MATLAB fixture");
        let output = Command::new(env!("CARGO_BIN_EXE_xdl"))
            .args(["-q"])
            .arg(&script_path)
            .current_dir(&directory)
            .output()
            .expect("run MATLAB elseif fixture");
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            output.status.success(),
            "mode {input}; stdout: {stdout}\nstderr: {stderr}"
        );
        let printed_values: Vec<_> = stdout
            .lines()
            .filter_map(|line| line.trim().parse::<i32>().ok())
            .collect();
        assert_eq!(printed_values, [expected], "mode {input}; stdout: {stdout}");
    }

    let _ = fs::remove_dir_all(&directory);
}
