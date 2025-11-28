//! Semantic tokens provider for enhanced syntax highlighting

use tower_lsp::lsp_types::{
    SemanticToken, SemanticTokenModifier, SemanticTokenType, SemanticTokens, SemanticTokensLegend,
    SemanticTokensResult,
};

use crate::document::DocumentState;

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,   // 0
            SemanticTokenType::FUNCTION,  // 1
            SemanticTokenType::METHOD,    // 2 (procedures)
            SemanticTokenType::VARIABLE,  // 3
            SemanticTokenType::PARAMETER, // 4
            SemanticTokenType::STRING,    // 5
            SemanticTokenType::NUMBER,    // 6
            SemanticTokenType::OPERATOR,  // 7
            SemanticTokenType::COMMENT,   // 8
            SemanticTokenType::NAMESPACE, // 9 (system variables)
            SemanticTokenType::CLASS,     // 10
            SemanticTokenType::PROPERTY,  // 11 (struct fields)
            SemanticTokenType::TYPE,      // 12
        ],
        token_modifiers: vec![
            SemanticTokenModifier::DEFINITION,  // 0
            SemanticTokenModifier::READONLY,    // 1
            SemanticTokenModifier::DECLARATION, // 2
        ],
    }
}

const TOKEN_KEYWORD: u32 = 0;
const TOKEN_FUNCTION: u32 = 1;
const TOKEN_METHOD: u32 = 2;
const TOKEN_VARIABLE: u32 = 3;
const TOKEN_STRING: u32 = 5;
const TOKEN_NUMBER: u32 = 6;
const TOKEN_OPERATOR: u32 = 7;
const TOKEN_COMMENT: u32 = 8;
const TOKEN_NAMESPACE: u32 = 9;

pub fn compute_semantic_tokens(doc: &DocumentState) -> Option<SemanticTokensResult> {
    let mut tokens: Vec<SemanticToken> = Vec::new();

    // Walk through the document line by line
    let mut prev_line = 0u32;
    let mut prev_char = 0u32;

    for (line_idx, line) in doc.content.lines().enumerate() {
        let line_str = line.to_string();
        let line_num = line_idx as u32;

        // Reset prev_char for each new line
        if line_num > prev_line {
            prev_char = 0;
        }

        // Tokenize the line
        let line_tokens = tokenize_line(&line_str);

        for (start, end, token_type) in line_tokens {
            let delta_line = line_num - prev_line;
            let delta_start = if delta_line == 0 {
                start - prev_char
            } else {
                start
            };

            tokens.push(SemanticToken {
                delta_line,
                delta_start,
                length: end - start,
                token_type,
                token_modifiers_bitset: 0,
            });

            prev_line = line_num;
            prev_char = start;
        }
    }

    Some(SemanticTokensResult::Tokens(SemanticTokens {
        result_id: None,
        data: tokens,
    }))
}

fn tokenize_line(line: &str) -> Vec<(u32, u32, u32)> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        // Skip whitespace
        if c.is_whitespace() {
            i += 1;
            continue;
        }

        // Comment
        if c == ';' {
            tokens.push((i as u32, chars.len() as u32, TOKEN_COMMENT));
            break;
        }

        // String
        if c == '\'' || c == '"' {
            let quote = c;
            let start = i;
            i += 1;
            while i < chars.len() && chars[i] != quote {
                if chars[i] == '\\' && i + 1 < chars.len() {
                    i += 1;
                }
                i += 1;
            }
            if i < chars.len() {
                i += 1; // closing quote
            }
            tokens.push((start as u32, i as u32, TOKEN_STRING));
            continue;
        }

        // Number
        if c.is_ascii_digit() || (c == '.' && i + 1 < chars.len() && chars[i + 1].is_ascii_digit())
        {
            let start = i;
            while i < chars.len()
                && (chars[i].is_ascii_digit()
                    || chars[i] == '.'
                    || chars[i] == 'e'
                    || chars[i] == 'E'
                    || chars[i] == 'd'
                    || chars[i] == 'D'
                    || chars[i] == '+'
                    || chars[i] == '-'
                    || chars[i] == 'b'
                    || chars[i] == 'B'
                    || chars[i] == 's'
                    || chars[i] == 'S'
                    || chars[i] == 'l'
                    || chars[i] == 'L'
                    || chars[i] == 'u'
                    || chars[i] == 'U')
            {
                i += 1;
            }
            tokens.push((start as u32, i as u32, TOKEN_NUMBER));
            continue;
        }

        // System variable
        if c == '!' {
            let start = i;
            i += 1;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            tokens.push((start as u32, i as u32, TOKEN_NAMESPACE));
            continue;
        }

        // Identifier or keyword
        if c.is_alphabetic() || c == '_' {
            let start = i;
            while i < chars.len() && (chars[i].is_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let token_type = classify_word(&word);
            tokens.push((start as u32, i as u32, token_type));
            continue;
        }

        // Operators
        if is_operator_char(c) {
            let start = i;
            // Handle multi-character operators
            let is_two_char_op = i + 1 < chars.len()
                && matches!(
                    (c, chars[i + 1]),
                    ('-', '>')
                        | (':', ':')
                        | ('+', '+')
                        | ('-', '-')
                        | ('+', '=')
                        | ('-', '=')
                        | ('*', '=')
                        | ('/', '=')
                );
            if is_two_char_op {
                i += 2;
            } else {
                i += 1;
            }
            tokens.push((start as u32, i as u32, TOKEN_OPERATOR));
            continue;
        }

        // Skip other characters
        i += 1;
    }

    tokens
}

fn classify_word(word: &str) -> u32 {
    let upper = word.to_uppercase();

    // Keywords
    let keywords = [
        "IF",
        "THEN",
        "ELSE",
        "ENDIF",
        "FOR",
        "ENDFOR",
        "FOREACH",
        "WHILE",
        "ENDWHILE",
        "REPEAT",
        "UNTIL",
        "DO",
        "BEGIN",
        "END",
        "BREAK",
        "CONTINUE",
        "RETURN",
        "GOTO",
        "CASE",
        "ENDCASE",
        "SWITCH",
        "ENDSWITCH",
        "OF",
        "COMMON",
        "COMPILE_OPT",
        "FORWARD_FUNCTION",
        "ON_ERROR",
        "ON_IOERROR",
        "CATCH",
    ];
    if keywords.contains(&upper.as_str()) {
        return TOKEN_KEYWORD;
    }

    // Function/Procedure definition keywords
    if upper == "FUNCTION" || upper == "ENDFUNCTION" {
        return TOKEN_KEYWORD;
    }
    if upper == "PRO" || upper == "PROCEDURE" || upper == "ENDPRO" {
        return TOKEN_KEYWORD;
    }

    // Logical operators (keywords)
    let logical_ops = [
        "AND", "OR", "NOT", "XOR", "EQ", "NE", "LT", "GT", "LE", "GE", "MOD",
    ];
    if logical_ops.contains(&upper.as_str()) {
        return TOKEN_OPERATOR;
    }

    // Built-in functions (common ones)
    let builtin_funcs = [
        "SIN",
        "COS",
        "TAN",
        "ASIN",
        "ACOS",
        "ATAN",
        "SINH",
        "COSH",
        "TANH",
        "SQRT",
        "EXP",
        "ALOG",
        "ALOG10",
        "ABS",
        "CEIL",
        "FLOOR",
        "ROUND",
        "FIX",
        "FLOAT",
        "DOUBLE",
        "COMPLEX",
        "FINDGEN",
        "INDGEN",
        "DINDGEN",
        "FLTARR",
        "DBLARR",
        "INTARR",
        "BYTARR",
        "STRARR",
        "MAKE_ARRAY",
        "REPLICATE",
        "WHERE",
        "N_ELEMENTS",
        "SIZE",
        "REFORM",
        "TRANSPOSE",
        "REVERSE",
        "SHIFT",
        "ROTATE",
        "SORT",
        "UNIQ",
        "TOTAL",
        "MEAN",
        "MEDIAN",
        "VARIANCE",
        "STDDEV",
        "MIN",
        "MAX",
        "MOMENT",
        "CORRELATE",
        "HISTOGRAM",
        "STRLEN",
        "STRMID",
        "STRPOS",
        "STRTRIM",
        "STRUPCASE",
        "STRLOWCASE",
        "STRING",
        "STRSPLIT",
        "STRJOIN",
        "STRCMP",
        "READ_ASCII",
        "READ_CSV",
        "READ_BINARY",
        "FILE_TEST",
        "FILE_INFO",
        "FILE_SEARCH",
        "FILE_LINES",
        "PTR_NEW",
        "OBJ_NEW",
        "BYTSCL",
        "CONGRID",
        "REBIN",
        "INTERPOLATE",
        "FFT",
    ];
    if builtin_funcs.contains(&upper.as_str()) {
        return TOKEN_FUNCTION;
    }

    // Built-in procedures (common ones)
    let builtin_procs = [
        "PRINT",
        "PRINTF",
        "WRITEF",
        "WRITEU",
        "READF",
        "READU",
        "OPENR",
        "OPENW",
        "OPENU",
        "CLOSE",
        "FREE_LUN",
        "PLOT",
        "OPLOT",
        "CONTOUR",
        "SURFACE",
        "SHADE_SURF",
        "TV",
        "TVSCL",
        "WINDOW",
        "WSET",
        "WDELETE",
        "DEVICE",
        "ERASE",
        "HELP",
        "STOP",
        "MESSAGE",
        "PTR_FREE",
        "OBJ_DESTROY",
    ];
    if builtin_procs.contains(&upper.as_str()) {
        return TOKEN_METHOD;
    }

    // Default to variable
    TOKEN_VARIABLE
}

fn is_operator_char(c: char) -> bool {
    matches!(
        c,
        '+' | '-'
            | '*'
            | '/'
            | '^'
            | '#'
            | '='
            | '<'
            | '>'
            | '.'
            | ':'
            | ','
            | '['
            | ']'
            | '('
            | ')'
            | '{'
            | '}'
    )
}
