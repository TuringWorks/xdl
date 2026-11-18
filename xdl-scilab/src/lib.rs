//! A deliberately small, independent Scilab-to-XDL source transpiler.
//!
//! Supported input includes arithmetic, comma-separated vector literals, rectangular numeric
//! semicolon-separated 2D matrix literals, nested `if` / `elseif` / `else` / `end` blocks,
//! bounded literal `for` ranges, a narrow `while` subset, one single-output function with
//! zero to four inputs and a straight-line assignment body, and `disp(expr)`. Scilab `%pi`, `%e`,
//! `%t`, and `%f` are constants; `//` and `/* ... */` are comments. This is not a general
//! Scilab implementation.

use std::path::Path;

mod lexer;
mod transpiler;

pub use lexer::{lex, Line, LineKind};
pub use transpiler::transpile_scilab_to_xdl;

/// Read and transpile a Scilab source file.
pub fn load_scilab_file(path: &Path) -> Result<String, String> {
    let source = std::fs::read_to_string(path)
        .map_err(|error| format!("Failed to read Scilab file: {error}"))?;
    transpile_scilab_to_xdl(&source)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn supported_constants_are_translated() {
        let result =
            transpile_scilab_to_xdl("p = %pi\ne = %e\nt = %t\nf = %f\nT = %T\nF = %F\n").unwrap();
        assert_eq!(result, "p = !PI\ne = !E\nt = 1\nf = 0\nT = 1\nF = 0\n");
    }

    #[test]
    fn comments_and_strings_preserve_percent_constants() {
        let result =
            transpile_scilab_to_xdl("x = %pi + 1 // comment\ns = \"rate %pi // text\"\n").unwrap();
        assert_eq!(result, "x = !PI + 1\ns = \"rate %pi // text\"\n");
    }

    #[test]
    fn supports_multiline_block_comments_and_rejects_unterminated_ones() {
        let result = transpile_scilab_to_xdl("x = 1\n/* hidden\n hidden */\ny = 2\n").unwrap();
        assert_eq!(result, "x = 1\ny = 2\n");
        assert!(transpile_scilab_to_xdl("x = 1 /* never closed\n")
            .unwrap_err()
            .contains("unterminated Scilab block comment"));
    }

    #[test]
    fn lowers_else_if_and_nested_blocks() {
        let source = "if x > 0 then\n  if x == 1 then\n    y = 1\n  else\n    y = 2\n  end\nelseif x < 0 then\n  y = -1\nelse\n  y = 0\nend\n";
        let result = transpile_scilab_to_xdl(source).unwrap();
        assert!(result.contains("if x GT 0 then\n"));
        assert!(result.contains("if x EQ 1 then\n"));
        assert!(result.contains("else\nif x LT 0 then\n"));
        assert!(result.ends_with("endif\nendif\n"));
    }

    #[test]
    fn lowers_inclusive_literal_ranges_and_nested_end_blocks() {
        let source = "for i = 2:4\nif i > 2 then\nx = i\nend\nfor j = 5:-2:1\ny = j\nend\nend\n";
        let result = transpile_scilab_to_xdl(source).unwrap();
        assert!(result.starts_with("FOR i = 2, 4, 1 DO BEGIN\n"));
        assert!(result.contains("FOR j = 5, 1, -2 DO BEGIN\n"));
        assert!(result.ends_with("ENDFOR\nENDFOR\n"));
        assert!(result.contains("endif\n"));
    }

    #[test]
    fn lowers_while_forms_and_nested_end_blocks() {
        let source = "i = 0\nwhile i < 3 do\nif i >= 0 then\ni = i + 1\nend\nend\nwhile i > 0 then\ni = i - 1\nend\nwhile i == 0\ni = 1\nend\n";
        let result = transpile_scilab_to_xdl(source).unwrap();
        assert!(result.contains("while i LT 3 do begin\n"));
        assert!(result.contains("if i GE 0 then\n"));
        assert!(result.contains("endwhile\n"));
        assert!(result.contains("while i GT 0 do begin\ni = i - 1\nendwhile\n"));
        assert!(result.ends_with("while i EQ 0 do begin\ni = 1\nendwhile\n"));
    }

    #[test]
    fn rejects_unsupported_while_headers_and_missing_end() {
        for (source, expected) in [
            (
                "while i < 3, i = i + 1\nend\n",
                "unsupported Scilab while header",
            ),
            (
                "while i < 3 else\ni = i + 1\nend\n",
                "unsupported Scilab while header",
            ),
            (
                "while i < 3\ni = i + 1\n",
                "missing 'end' for Scilab while loop",
            ),
            ("while i < 3 do\nx = 1\nendwhile\n", "unexpected 'endwhile'"),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} for {source:?}"
            );
        }
    }

    #[test]
    fn rejects_unsupported_and_non_integer_for_ranges() {
        for (source, expected) in [
            (
                "for i = 1:2:3:4\nx = i\nend\n",
                "unsupported Scilab for header",
            ),
            ("for i = 1:0:3\nx = i\nend\n", "step must be nonzero"),
            ("for i = 1.5:3\nx = i\nend\n", "integer literal bounds"),
            ("for i = 1:1.0:3\nx = i\nend\n", "integer literal bounds"),
            (
                "for i = 1:2147483648\nx = i\nend\n",
                "integer literal bounds",
            ),
            ("for i = start:stop\nx = i\nend\n", "integer literal bounds"),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} for {source:?}"
            );
        }
    }

    #[test]
    fn rejects_ranges_over_the_iteration_limit_and_allows_direction_mismatch() {
        assert!(transpile_scilab_to_xdl("for i = 1:1000001\nx = i\nend\n")
            .unwrap_err()
            .contains("1,000,000 iteration limit"));
        assert_eq!(
            transpile_scilab_to_xdl("for i = 5:1\nx = i\nend\n").unwrap(),
            "FOR i = 5, 1, 1 DO BEGIN\nx = i\nENDFOR\n"
        );
    }

    #[test]
    fn rejects_unsupported_percent_forms_and_statements() {
        assert!(transpile_scilab_to_xdl("x = %nan\n")
            .unwrap_err()
            .contains("unsupported Scilab percent form"));
        assert!(transpile_scilab_to_xdl("plot(x)\n")
            .unwrap_err()
            .contains("unsupported Scilab syntax"));
        assert!(transpile_scilab_to_xdl("if x then\ny = 1\n")
            .unwrap_err()
            .contains("missing 'end'"));
    }

    #[test]
    fn vector_scalar_indices_are_validated_at_runtime_and_use_one_based_semantics() {
        let result = transpile_scilab_to_xdl(
            "values = [10, 20, 30]\ni = 2\na = values(i)\nb = values(i + 1)\nc = values(1:2:3)\n",
        )
        .unwrap();
        assert!(result.contains("a = values[SCILAB_INDEX(i)]"), "{result}");
        assert!(
            result.contains("b = values[SCILAB_INDEX(i + 1)]"),
            "{result}"
        );
        assert!(result.contains("c = values[0:3:2]"), "{result}");
    }

    #[test]
    fn lowers_single_output_functions_with_one_argument_and_zero_arguments() {
        let output = transpile_scilab_to_xdl(
            "function y = twice(x)\ny = x * 2\nendfunction\nvalue = twice(21)\n",
        )
        .unwrap();
        assert_eq!(
            output,
            "FUNCTION twice, x\ny = x * 2\nRETURN, y\nENDFUNCTION\nvalue = twice(21)\n"
        );
        let call_before_definition = transpile_scilab_to_xdl(
            "value = twice(21)\nfunction y = twice(x)\ny = x * 2\nendfunction\n",
        )
        .unwrap();
        assert_eq!(call_before_definition, output);

        let zero_arg = transpile_scilab_to_xdl(
            "function answer = constant_value()\nanswer = 42\nendfunction\nvalue = constant_value()\n",
        )
        .unwrap();
        assert!(zero_arg.contains("FUNCTION constant_value\n"));
        assert!(zero_arg.ends_with("value = constant_value()\n"));
    }

    #[test]
    fn lowers_multiple_same_file_functions_and_rejects_duplicate_names() {
        let source = "first_value = twice(21)\nfunction answer = constant_value()\nanswer = 42\nendfunction\nsecond_value = constant_value()\nfunction y = twice(x)\ny = x * 2\nendfunction\n";
        let output = transpile_scilab_to_xdl(source).unwrap();
        assert_eq!(
            output,
            "FUNCTION constant_value\nanswer = 42\nRETURN, answer\nENDFUNCTION\nFUNCTION twice, x\ny = x * 2\nRETURN, y\nENDFUNCTION\nfirst_value = twice(21)\nsecond_value = constant_value()\n"
        );

        let duplicate = transpile_scilab_to_xdl(
            "function y = twice(x)\ny = x * 2\nendfunction\nfunction z = TWICE(x)\nz = x * 2\nendfunction\n",
        )
        .unwrap_err();
        assert!(duplicate.contains("duplicate Scilab function name 'TWICE'"));
    }

    #[test]
    fn lowers_single_output_functions_with_multiple_inputs() {
        let two_inputs = transpile_scilab_to_xdl(
            "function result = affine(scale, value)\nresult = scale * value\nendfunction\nout = affine(3, 7)\n",
        )
        .unwrap();
        assert_eq!(
            two_inputs,
            "FUNCTION affine, scale, value\nresult = scale * value\nRETURN, result\nENDFUNCTION\nout = affine(3, 7)\n"
        );

        let four_inputs = transpile_scilab_to_xdl(
            "function result = combine(a, b, c, d)\nresult = a + b + c + d\nendfunction\nout = combine(1, 2, 3, 4)\n",
        )
        .unwrap();
        assert!(four_inputs.starts_with("FUNCTION combine, a, b, c, d\n"));
        assert!(four_inputs.ends_with("out = combine(1, 2, 3, 4)\n"));

        let call_before_definition = transpile_scilab_to_xdl(
            "out = combine(1, 2, 3, 4)\nfunction result = combine(a, b, c, d)\nresult = a + b + c + d\nendfunction\n",
        )
        .unwrap();
        assert_eq!(call_before_definition, four_inputs);
    }

    #[test]
    fn rejects_malformed_or_unsupported_scilab_functions() {
        for (source, expected) in [
            (
                "function [a,b] = pair(x)\na = x\nendfunction\n",
                "unsupported Scilab function header",
            ),
            (
                "function y = f(a,b,c,d,e)\ny = a\nendfunction\n",
                "unsupported Scilab function header",
            ),
            (
                "function y = f(a,,c)\ny = a\nendfunction\n",
                "malformed Scilab function parameter list",
            ),
            (
                "function y = f(a,b,a)\ny = a\nendfunction\n",
                "duplicate Scilab function parameter 'a'",
            ),
            (
                "function result = f(a,result)\nresult = a\nendfunction\n",
                "cannot also be an input parameter",
            ),
            ("function y = f(x)\ny = x\n", "missing 'endfunction'"),
            (
                "function y = f(x)\nx = x + 1\nendfunction\n",
                "must assign its output",
            ),
            (
                "function y = f(x)\nif x then\ny = 1\nend\nendfunction\n",
                "only straight-line assignments",
            ),
            ("endfunction\n", "unexpected 'endfunction'"),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} for {source:?}"
            );
        }
    }

    #[test]
    fn lowers_disp_and_preserves_comparison_text_inside_strings() {
        let output = transpile_scilab_to_xdl("disp(x >= 2)\ns = \"a >= b\"\n").unwrap();
        assert_eq!(output, "PRINT, x GE 2\ns = \"a >= b\"\n");
    }

    #[test]
    fn accepts_comma_separated_vector_literals_and_calls() {
        let result = transpile_scilab_to_xdl("v = [1, 2, 3]\nx = sum(v)\n").unwrap();
        assert_eq!(result, "v = [1, 2, 3]\nx = sum(v)\n");
    }

    #[test]
    fn lowers_rectangular_numeric_matrix_literals_to_ordered_nested_arrays() {
        let output = transpile_scilab_to_xdl("A = [1,2;3,4]\n").unwrap();
        assert_eq!(output, "A = [[1, 2], [3, 4]]\n");
    }

    #[test]
    fn lowers_two_subscript_reads_only_for_known_matrix_variables() {
        let output = transpile_scilab_to_xdl(
            "A = [1,2;3,4]\nfirst = A(1,1)\nlast = A(2,2)\nresult = max(A, 4)\nvalue = sum([1, 2])\n",
        )
        .unwrap();
        assert_eq!(
            output,
            "A = [[1, 2], [3, 4]]\nfirst = A[0, 0]\nlast = A[1, 1]\nresult = max(A, 4)\nvalue = sum([1, 2])\n"
        );
    }

    #[test]
    fn lowers_scalar_reads_from_tracked_numeric_vectors_and_preserves_calls() {
        let output = transpile_scilab_to_xdl(
            "v = [10,20,30]\nfirst = v(1)\nlast = v(3)\nresult = sum(v)\nother = sum([1, 2])\n",
        )
        .unwrap();
        assert_eq!(
            output,
            "v = [10,20,30]\nfirst = v[0]\nlast = v[2]\nresult = sum(v)\nother = sum([1, 2])\n"
        );
    }

    #[test]
    fn lowers_inclusive_vector_ranges_to_validated_runtime_helper() {
        let output = transpile_scilab_to_xdl(
            "v = [10,20,30,40,50]\na = v(2:4)\nb = v(1:2:5)\nc = v(2:3:5)\n",
        )
        .unwrap();
        assert_eq!(
            output,
            "v = [10,20,30,40,50]\na = SCILAB_INDEX_RANGE(v, 2, 1, 4)\nb = SCILAB_INDEX_RANGE(v, 1, 2, 5)\nc = SCILAB_INDEX_RANGE(v, 2, 3, 5)\n"
        );
        let dynamic = transpile_scilab_to_xdl(
            "v = [10,20,30,40,50]\nstart = 2\nstride = 2\nfinish = 5\na = v(start:finish)\nb = v(start:stride:finish)\n",
        )
        .unwrap();
        assert!(
            dynamic.contains("a = SCILAB_INDEX_RANGE(v, start, 1, finish)"),
            "{dynamic}"
        );
        assert!(
            dynamic.contains("b = SCILAB_INDEX_RANGE(v, start, stride, finish)"),
            "{dynamic}"
        );
    }

    #[test]
    fn vector_ranges_reject_negative_zero_and_malformed_parts() {
        for (source, expected) in [
            ("v = [1,2,3]\nx = v(-1:3)\n", "positive integer literal"),
            ("v = [1,2,3]\nx = v(1:0:3)\n", "positive integer literal"),
            ("v = [1,2,3]\nx = v(1:-1:3)\n", "positive integer literal"),
            ("v = [1,2,3]\nx = v(1:2:3:4)\n", "positive integer literal"),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} error for {source:?}"
            );
        }
    }

    #[test]
    fn vector_reads_reject_invalid_indices_dimensions_and_targets() {
        for (source, expected) in [
            ("v = [1,2,3]\nx = v(0)\n", "positive integer"),
            ("v = [1,2,3]\nx = v(-1)\n", "positive integer"),
            ("v = [1,2,3]\nx = v(1.5)\n", "positive integer"),
            ("v = [1,2,3]\nx = v(1,2)\n", "1D literal reads require 1"),
            (
                "v = [1,2,3]\nv(1) = 4\n",
                "indexed assignment targets are unsupported",
            ),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} error for {source:?}"
            );
        }
        let dynamic = transpile_scilab_to_xdl("v = [1,2,3]\nx = v(k)\n").unwrap();
        assert!(dynamic.contains("x = v[SCILAB_INDEX(k)]"), "{dynamic}");
    }

    #[test]
    fn matrix_reads_reject_nonpositive_or_nonliteral_indices() {
        for (source, expected) in [
            ("A = [1,2;3,4]\nx = A(0,1)\n", "one-based"),
            ("A = [1,2;3,4]\nx = A(i,1)\n", "integer literal"),
            ("A = [1,2;3,4]\nx = A(1)\n", "integer literal"),
            ("A = [1,2;3,4]\nx = A(1.5,2)\n", "integer literal"),
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains(expected),
                "expected {expected:?} error for {source:?}"
            );
        }
    }

    #[test]
    fn rejects_non_rectangular_or_non_numeric_matrix_literals() {
        for source in [
            "A = [1,2;3]\n",
            "A = [1,x;3,4]\n",
            "A = [1;;3]\n",
            "A = [NaN,2;3,4]\n",
        ] {
            assert!(
                transpile_scilab_to_xdl(source)
                    .unwrap_err()
                    .contains("matrix literals must have rectangular rows"),
                "should reject {source:?}"
            );
        }
    }

    #[test]
    fn lowers_identifier_and_numeric_row_vector_transpose_without_rewriting_strings() {
        let result = transpile_scilab_to_xdl(
            "row = [1, 2, 3]\ncolumn = row'\ncolumn2 = [4, 5, 6]'\ntext = 'keep this string'\n",
        )
        .unwrap();
        assert_eq!(
            result,
            "row = [1, 2, 3]\ncolumn = TRANSPOSE(row)\ncolumn2 = TRANSPOSE([4, 5, 6])\ntext = 'keep this string'\n"
        );
    }

    #[test]
    fn reports_missing_outer_end_even_when_nested_end_is_last() {
        let source = "if outer then\nif inner then\nx = 1\nend\n";
        assert!(transpile_scilab_to_xdl(source)
            .unwrap_err()
            .contains("missing 'end' for if block"));
    }
}
