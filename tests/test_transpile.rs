use xdl_matlab::transpile_matlab_to_xdl;

fn main() {
    let matlab_code = std::fs::read_to_string("test_matlab.m").unwrap();
    match transpile_matlab_to_xdl(&matlab_code) {
        Ok(xdl) => {
            println!("=== Transpiled XDL Code ===");
            println!("{}", xdl);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
        }
    }
}
