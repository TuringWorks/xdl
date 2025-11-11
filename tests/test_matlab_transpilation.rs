//! Comprehensive MATLAB Transpilation Tests
//! Tests that MATLAB .m files are correctly transpiled to XDL

use std::fs;
use std::path::Path;
use xdl_matlab::transpile_matlab_to_xdl;

#[cfg(test)]
mod tests {
    use super::*;

    /// Test basic MATLAB syntax transpilation
    #[test]
    fn test_basic_syntax_transpilation() {
        let matlab_code = r#"
x = 42;
y = 3.14159;
z = 'Hello World';
flag = true;
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("x = 42"));
        assert!(xdl_code.contains("y = 3.14159"));
        assert!(xdl_code.contains("z = \"Hello World\""));
        assert!(xdl_code.contains("flag = 1"));
    }

    /// Test array creation and manipulation
    #[test]
    fn test_array_operations() {
        let matlab_code = r#"
arr = [1, 2, 3, 4, 5];
matrix = [1, 2, 3; 4, 5, 6];
range = 1:10;
zeros_arr = zeros(3, 4);
ones_arr = ones(2, 3);
eye_mat = eye(4);
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("arr = [1, 2, 3, 4, 5]"));
        assert!(xdl_code.contains("matrix = [[1, 2, 3], [4, 5, 6]]"));
        assert!(xdl_code.contains("FINDGEN"));
        assert!(xdl_code.contains("FLTARR(3, 4)"));
        assert!(xdl_code.contains("FLTARR(2, 3) + 1"));
        assert!(xdl_code.contains("IDENTITY(4)"));
    }

    /// Test mathematical functions
    #[test]
    fn test_mathematical_functions() {
        let matlab_code = r#"
result = sin(pi/2);
cos_val = cos(0);
sqrt_val = sqrt(16);
exp_val = exp(1);
log_val = log(10);
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("SIN"));
        assert!(xdl_code.contains("COS"));
        assert!(xdl_code.contains("SQRT"));
        assert!(xdl_code.contains("EXP"));
        assert!(xdl_code.contains("ALOG"));
    }

    /// Test control flow structures
    #[test]
    fn test_control_flow() {
        let matlab_code = r#"
if x > 10
    result = 'large';
elseif x > 5
    result = 'medium';
else
    result = 'small';
end

for i = 1:5
    sum = sum + i;
end

while count > 0
    count = count - 1;
end

switch value
    case 1
        result = 'one';
    case 2
        result = 'two';
    otherwise
        result = 'other';
end
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("if"));
        assert!(xdl_code.contains("then"));
        assert!(xdl_code.contains("endif"));
        assert!(xdl_code.contains("for"));
        assert!(xdl_code.contains("endfor"));
        assert!(xdl_code.contains("while"));
        assert!(xdl_code.contains("endwhile"));
        assert!(xdl_code.contains("CASE"));
        assert!(xdl_code.contains("ENDCASE"));
    }

    /// Test function definitions
    #[test]
    fn test_function_definitions() {
        let matlab_code = r#"
function result = add_numbers(a, b)
    result = a + b;
end

function [sum_val, prod_val] = compute_both(x, y)
    sum_val = x + y;
    prod_val = x * y;
end
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("FUNCTION add_numbers"));
        assert!(xdl_code.contains("FUNCTION compute_both"));
        assert!(xdl_code.contains("END"));
    }

    /// Test array indexing conversion (1-based to 0-based)
    #[test]
    fn test_array_indexing() {
        let matlab_code = r#"
first = arr(1);
second = arr(2);
last = arr(end);
slice = arr(2:5);
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        // MATLAB 1-based indexing should be converted to 0-based
        assert!(xdl_code.contains("arr[0]"));
        assert!(xdl_code.contains("arr[1]"));
        assert!(xdl_code.contains("arr[-1]"));
        assert!(xdl_code.contains("arr[1:4]"));
    }

    /// Test plotting commands
    #[test]
    fn test_plotting_commands() {
        let matlab_code = r#"
plot(x, y);
xlabel('X axis');
ylabel('Y axis');
title('My Plot');
figure;
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("PLOT"));
        assert!(xdl_code.contains("XTITLE"));
        assert!(xdl_code.contains("YTITLE"));
        assert!(xdl_code.contains("TITLE"));
        assert!(xdl_code.contains("WINDOW"));
    }

    /// Test statistical functions
    #[test]
    fn test_statistical_functions() {
        let matlab_code = r#"
avg = mean(data);
med = median(data);
stdev = std(data);
variance = var(data);
minimum = min(data);
maximum = max(data);
total = sum(data);
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("MEAN"));
        assert!(xdl_code.contains("MEDIAN"));
        assert!(xdl_code.contains("STDDEV"));
        assert!(xdl_code.contains("VARIANCE"));
        assert!(xdl_code.contains("MIN"));
        assert!(xdl_code.contains("MAX"));
        assert!(xdl_code.contains("TOTAL"));
    }

    /// Test complex expressions and operations
    #[test]
    fn test_complex_expressions() {
        let matlab_code = r#"
result = a + b .* c - d ./ e .^ f;
matrix_result = A * B + C .* D;
logical_result = (x > 0) && (y < 10) || (z == 5);
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        // Element-wise operations should be preserved
        assert!(xdl_code.contains(".*"));
        assert!(xdl_code.contains("./"));
        assert!(xdl_code.contains(".^"));
        // Matrix multiplication should be preserved
        assert!(xdl_code.contains("*"));
    }

    /// Test file I/O operations
    #[test]
    fn test_file_operations() {
        let matlab_code = r#"
fid = fopen('data.txt', 'r');
data = fscanf(fid, '%f');
fclose(fid);
save('results.mat', 'x', 'y');
load('results.mat');
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        // File operations should be commented out or mapped appropriately
        assert!(xdl_code.contains("fopen"));
        assert!(xdl_code.contains("fclose"));
        assert!(xdl_code.contains("save"));
        assert!(xdl_code.contains("load"));
    }

    /// Test error handling constructs
    #[test]
    fn test_error_handling() {
        let matlab_code = r#"
try
    result = risky_operation();
catch err
    disp('Error occurred');
    result = NaN;
end

break;
continue;
return;
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains("TRY"));
        assert!(xdl_code.contains("BREAK"));
        assert!(xdl_code.contains("CONTINUE"));
        assert!(xdl_code.contains("RETURN"));
    }

    /// Test comprehensive MATLAB file transpilation
    #[test]
    fn test_comprehensive_matlab_file() {
        // Test the comprehensive MATLAB test file
        let test_file = Path::new("tests/matlab_language_features_test.m");

        if test_file.exists() {
            let result = xdl_matlab::load_matlab_file(test_file);
            assert!(result.is_ok(), "Failed to transpile comprehensive MATLAB file");

            let xdl_code = result.unwrap();
            assert!(!xdl_code.is_empty(), "Transpiled code should not be empty");

            // Check for key transpilation features
            assert!(xdl_code.contains("disp"), "Should contain print statements");
            assert!(xdl_code.contains("if"), "Should contain conditional statements");
            assert!(xdl_code.contains("for"), "Should contain loops");
            assert!(xdl_code.contains("function"), "Should contain function definitions");
            assert!(xdl_code.contains("SIN"), "Should contain mathematical functions");
            assert!(xdl_code.contains("MEAN"), "Should contain statistical functions");
        } else {
            println!("Comprehensive MATLAB test file not found, skipping test");
        }
    }

    /// Test edge cases and special syntax
    #[test]
    fn test_edge_cases() {
        let matlab_code = r#"
% Comments should be preserved
x = 1; y = 2; z = 3;  % Multiple statements

% Special constants
pi_val = pi;
e_val = exp(1);
inf_val = inf;
nan_val = nan;

% Complex expressions
result = (a + b) * (c - d) / (e + f);
nested = func1(func2(x, y), func3(z));

% Empty arrays and edge cases
empty = [];
single = [42];
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();
        assert!(xdl_code.contains(";"), "Comments should be preserved");
        assert!(xdl_code.contains("!PI"), "pi should be mapped to !PI");
        assert!(xdl_code.contains("!E"), "exp(1) should be mapped to !E");
        assert!(xdl_code.contains("[]"), "Empty arrays should be preserved");
    }

    /// Test that transpilation preserves MATLAB semantics
    #[test]
    fn test_semantic_preservation() {
        // Test that key MATLAB semantics are preserved in transpilation

        let matlab_code = r#"
% 1-based indexing conversion
first_element = arr(1);      % Should become arr[0]
second_element = arr(2);     % Should become arr[1]
last_element = arr(end);     % Should become arr[-1]

% Range conversion
simple_range = 1:5;          % Should become FINDGEN expression
step_range = 1:2:10;         % Should become FINDGEN with step
descending = 10:-1:5;        % Should handle negative steps

% Function call conversion
sin_val = sin(pi/2);          % Should become SIN(!PI/2)
cos_val = cos(0);             % Should become COS(0)
mean_val = mean(data);        % Should become MEAN(data)
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();

        // Check 1-based to 0-based indexing conversion
        assert!(xdl_code.contains("arr[0]"), "arr(1) should become arr[0]");
        assert!(xdl_code.contains("arr[1]"), "arr(2) should become arr[1]");
        assert!(xdl_code.contains("arr[-1]"), "arr(end) should become arr[-1]");

        // Check range conversion
        assert!(xdl_code.contains("FINDGEN"), "Ranges should be converted to FINDGEN");

        // Check function mapping
        assert!(xdl_code.contains("SIN"), "sin should be mapped to SIN");
        assert!(xdl_code.contains("COS"), "cos should be mapped to COS");
        assert!(xdl_code.contains("MEAN"), "mean should be mapped to MEAN");
        assert!(xdl_code.contains("!PI"), "pi should be mapped to !PI");
    }

    /// Test that all MATLAB keywords are handled
    #[test]
    fn test_matlab_keywords() {
        let matlab_code = r#"
% Test all major MATLAB keywords and constructs
if true
    disp('if works');
end

for i = 1:3
    disp(i);
end

while false
    break;
end

switch x
    case 1
        result = 1;
    otherwise
        result = 0;
end

try
    risky = 1/0;
catch
    risky = NaN;
end

function test_func()
    return;
end
"#;

        let result = transpile_matlab_to_xdl(matlab_code);
        assert!(result.is_ok());

        let xdl_code = result.unwrap();

        // All major keywords should be handled
        assert!(xdl_code.contains("if"), "if keyword should be handled");
        assert!(xdl_code.contains("for"), "for keyword should be handled");
        assert!(xdl_code.contains("while"), "while keyword should be handled");
        assert!(xdl_code.contains("CASE"), "switch should become CASE");
        assert!(xdl_code.contains("TRY"), "try should be handled");
        assert!(xdl_code.contains("BREAK"), "break should be handled");
        assert!(xdl_code.contains("RETURN"), "return should be handled");
        assert!(xdl_code.contains("FUNCTION"), "function should be handled");
    }
}
