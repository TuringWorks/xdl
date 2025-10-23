//! # XDL-MATLAB Compatibility Layer
//!
//! This crate provides MATLAB compatibility for XDL, allowing you to:
//! - Load and execute .m files
//! - Transpile MATLAB syntax to XDL
//! - Map MATLAB functions to XDL equivalents

pub mod function_map;
pub mod lexer;
pub mod transpiler;

pub use function_map::MATLAB_FUNCTION_MAP;
pub use transpiler::transpile_matlab_to_xdl;

/// Load and transpile a MATLAB .m file to XDL code
pub fn load_matlab_file(path: &std::path::Path) -> Result<String, String> {
    let matlab_code =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;

    transpile_matlab_to_xdl(&matlab_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_transpilation() {
        let matlab = "x = zeros(10, 1);";
        let result = transpile_matlab_to_xdl(matlab);
        assert!(result.is_ok());
    }
}
