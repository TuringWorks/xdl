// Example: MATLAB to XDL transpilation
use xdl_matlab::transpile_matlab_to_xdl;

fn main() {
    let matlab_code = r#"
% Simple MATLAB script
x = zeros(10, 1);

for i = 1:10
    x(i) = sin(i * 0.1);
end

mean_x = mean(x);
disp('Mean:');
disp(mean_x);
"#;

    println!("=== MATLAB Code ===");
    println!("{}", matlab_code);
    
    println!("\n=== Transpiled XDL Code ===");
    match transpile_matlab_to_xdl(matlab_code) {
        Ok(xdl_code) => println!("{}", xdl_code),
        Err(e) => eprintln!("Error: {}", e),
    }
}
