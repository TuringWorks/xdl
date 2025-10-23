//! MATLAB to XDL function name mapping

use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Mapping of MATLAB function names to XDL equivalents
pub static MATLAB_FUNCTION_MAP: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut map = HashMap::new();

    // Array creation
    map.insert("zeros", "FLTARR"); // zeros(n) -> FLTARR(n)
    map.insert("ones", "FLTARR"); // ones(n) -> FLTARR(n) + 1 (handled specially)
    map.insert("eye", "IDENTITY"); // eye(n) -> IDENTITY(n)
    map.insert("rand", "RANDOMU"); // rand(n) -> RANDOMU(seed, n)
    map.insert("randn", "RANDOMN"); // randn(n) -> RANDOMN(seed, n)

    // Array info
    map.insert("size", "SIZE");
    map.insert("length", "N_ELEMENTS");
    map.insert("numel", "N_ELEMENTS");

    // Math functions
    map.insert("sin", "SIN");
    map.insert("cos", "COS");
    map.insert("tan", "TAN");
    map.insert("asin", "ASIN");
    map.insert("acos", "ACOS");
    map.insert("atan", "ATAN");
    map.insert("atan2", "ATAN");
    map.insert("exp", "EXP");
    map.insert("log", "ALOG");
    map.insert("log10", "ALOG10");
    map.insert("sqrt", "SQRT");
    map.insert("abs", "ABS");
    map.insert("floor", "FLOOR");
    map.insert("ceil", "CEIL");
    map.insert("round", "ROUND");
    map.insert("mod", "MOD");

    // Statistics
    map.insert("mean", "MEAN");
    map.insert("median", "MEDIAN");
    map.insert("std", "STDDEV");
    map.insert("var", "VARIANCE");
    map.insert("min", "MIN");
    map.insert("max", "MAX");
    map.insert("sum", "TOTAL");

    // Linear algebra
    map.insert("transpose", "TRANSPOSE");
    map.insert("inv", "INVERT");
    map.insert("det", "DETERM");

    // Plotting
    map.insert("plot", "PLOT");
    map.insert("xlabel", "XTITLE");
    map.insert("ylabel", "YTITLE");
    map.insert("title", "TITLE");
    map.insert("figure", "WINDOW");
    map.insert("hold", "OPLOT"); // hold on -> use OPLOT
    map.insert("clf", "ERASE");
    map.insert("close", "WDELETE");

    // Array manipulation
    map.insert("reshape", "REFORM");
    map.insert("sort", "SORT");
    map.insert("find", "WHERE");
    map.insert("repmat", "REBIN");

    // I/O
    map.insert("disp", "PRINT");
    map.insert("fprintf", "PRINTF");
    map.insert("sprintf", "STRING");

    // Logical
    map.insert("all", "MIN"); // all -> MIN (non-zero check)
    map.insert("any", "MAX"); // any -> MAX (non-zero check)

    // Type conversion
    map.insert("double", "DOUBLE");
    map.insert("single", "FLOAT");
    map.insert("int32", "LONG");
    map.insert("uint32", "ULONG");

    map
});

/// Get XDL equivalent for a MATLAB function name
pub fn get_xdl_function(matlab_func: &str) -> Option<&'static str> {
    MATLAB_FUNCTION_MAP.get(matlab_func).copied()
}

/// Check if a MATLAB function needs special handling during transpilation
pub fn needs_special_handling(matlab_func: &str) -> bool {
    matches!(
        matlab_func,
        "ones" | "rand" | "randn" | "eye" | "linspace" | "logspace"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_function_mapping() {
        assert_eq!(get_xdl_function("zeros"), Some("FLTARR"));
        assert_eq!(get_xdl_function("sin"), Some("SIN"));
        assert_eq!(get_xdl_function("plot"), Some("PLOT"));
        assert_eq!(get_xdl_function("nonexistent"), None);
    }
}
