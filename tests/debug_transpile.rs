// Quick debug to see transpiled MATLAB code
fn main() {
    let matlab = r#"
t = (0:L-1)*T;
"#;

    println!("=== MATLAB Input ===");
    println!("{}", matlab);
    println!("\n=== XDL Output ===");

    match xdl_matlab::transpile_matlab_to_xdl(matlab) {
        Ok(xdl) => {
            println!("{}", xdl);
            println!("\n=== Line by line ===");
            for (i, line) in xdl.lines().enumerate() {
                println!("{}: '{}'", i+1, line);
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }
}
