//! XDL Lexer implementation

use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, none_of},
    combinator::{map, map_res, opt, recognize, value},
    multi::many0,
    sequence::{delimited, pair, preceded},
    IResult,
};

/// Skip whitespace but NOT newlines (unlike multispace0)
/// This is critical for procedure call detection which relies on newline tokens
fn ws0(input: &str) -> IResult<&str, &str> {
    take_while(|c: char| c == ' ' || c == '\t' || c == '\r')(input)
}
use xdl_core::XdlResult;

#[derive(Debug, Clone, PartialEq)]
pub struct TokenSpan {
    pub token: Token,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Literals
    Integer(i64),
    Float(f64),
    String(String),

    // Keywords
    If,
    Then,
    Else,
    Endif,
    For,
    Endfor,
    Foreach,
    While,
    Endwhile,
    Repeat,
    Until,
    Break,
    Continue,
    Function,
    Endfunction,
    Procedure,
    Pro,
    Endpro,
    Return,
    Goto,
    Common,
    CompileOpt,
    Begin,
    End,
    Case,
    Of,
    Endcase,
    Switch,
    Endswitch,

    // Operators
    Plus,           // +
    Minus,          // -
    Multiply,       // *
    Divide,         // /
    Modulo,         // MOD
    Power,          // ^
    MatrixMultiply, // #

    // Assignment
    Assign,         // =
    PlusAssign,     // +=
    MinusAssign,    // -=
    MultiplyAssign, // *=
    DivideAssign,   // /=

    // Comparison
    Equal,        // EQ
    NotEqual,     // NE
    Less,         // LT
    Greater,      // GT
    LessEqual,    // LE
    GreaterEqual, // GE

    // Logical
    And, // AND
    Or,  // OR
    Not, // NOT
    Xor, // XOR

    // Bitwise (rarely used but part of IDL)
    BitwiseAnd, // AND (bitwise)
    BitwiseOr,  // OR (bitwise)
    BitwiseXor, // XOR (bitwise)
    BitwiseNot, // NOT (bitwise)

    // Delimiters
    LeftParen,    // (
    RightParen,   // )
    LeftBracket,  // [
    RightBracket, // ]
    LeftBrace,    // {
    RightBrace,   // }
    Comma,        // ,
    Semicolon,    // ;
    Colon,        // :
    DoubleColon,  // ::
    Dot,          // .
    Arrow,        // ->
    QuestionMark, // ?

    // Special
    Identifier(String),
    SystemVariable(String), // !PI, !X, etc.
    Label(String),          // label:
    Comment(String),        // ; comment
    Newline,
    EOF,
}

type ParseResult<'a, T> = IResult<&'a str, T>;

// Helper function to check if character is valid in identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

// Parse integers
fn parse_integer(input: &str) -> ParseResult<'_, Token> {
    map_res(digit1, |s: &str| s.parse::<i64>().map(Token::Integer))(input)
}

// Parse floating point numbers
fn parse_float(input: &str) -> ParseResult<'_, Token> {
    map_res(
        recognize(pair(digit1, pair(char('.'), opt(digit1)))),
        |s: &str| s.parse::<f64>().map(Token::Float),
    )(input)
}

// Parse numbers (float or integer)
fn parse_number(input: &str) -> ParseResult<'_, Token> {
    alt((parse_float, parse_integer))(input)
}

// Parse string literals
fn parse_string(input: &str) -> ParseResult<'_, Token> {
    alt((
        // Double quoted strings
        delimited(
            char('"'),
            map(many0(none_of("\"")), |chars| {
                Token::String(chars.into_iter().collect())
            }),
            char('"'),
        ),
        // Single quoted strings
        delimited(
            char('\''),
            map(many0(none_of("'")), |chars| {
                Token::String(chars.into_iter().collect())
            }),
            char('\''),
        ),
    ))(input)
}

// Parse labels (identifier followed by colon, but not :: and not keywords)
fn parse_label(input: &str) -> ParseResult<'_, Token> {
    let (remaining, name) = recognize(pair(
        take_while1(is_identifier_start),
        take_while(is_identifier_char),
    ))(input)?;

    // Check for colon after identifier (but not ::)
    if remaining.starts_with(':') && !remaining.starts_with("::") {
        // Check if this is a keyword - keywords should not be treated as labels
        let is_keyword = matches!(
            name.to_uppercase().as_str(),
            "IF" | "THEN" | "ELSE" | "ENDIF" | "FOR" | "ENDFOR" | "FOREACH" | "WHILE"
                | "ENDWHILE" | "REPEAT" | "UNTIL" | "BREAK" | "CONTINUE" | "FUNCTION"
                | "ENDFUNCTION" | "PROCEDURE" | "PRO" | "ENDPRO" | "RETURN" | "GOTO"
                | "COMMON" | "COMPILE_OPT" | "BEGIN" | "END" | "CASE" | "OF" | "ENDCASE"
                | "SWITCH" | "ENDSWITCH" | "MOD" | "EQ" | "NE" | "LT" | "GT" | "LE"
                | "GE" | "AND" | "OR" | "NOT" | "XOR"
        );

        if is_keyword {
            // Not a label, it's a keyword followed by colon
            Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))
        } else {
            // Skip the colon
            let remaining = &remaining[1..];
            Ok((remaining, Token::Label(name.to_string())))
        }
    } else {
        // Not a label
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))
    }
}

// Parse identifiers and keywords
fn parse_identifier_or_keyword(input: &str) -> ParseResult<'_, Token> {
    let (input, name) = recognize(pair(
        take_while1(is_identifier_start),
        take_while(is_identifier_char),
    ))(input)?;

    let token = match name.to_uppercase().as_str() {
        // Control flow keywords
        "IF" => Token::If,
        "THEN" => Token::Then,
        "ELSE" => Token::Else,
        "ENDIF" => Token::Endif,
        "FOR" => Token::For,
        "ENDFOR" => Token::Endfor,
        "FOREACH" => Token::Foreach,
        "WHILE" => Token::While,
        "ENDWHILE" => Token::Endwhile,
        "REPEAT" => Token::Repeat,
        "UNTIL" => Token::Until,
        "BREAK" => Token::Break,
        "CONTINUE" => Token::Continue,

        // Function/procedure keywords
        "FUNCTION" => Token::Function,
        "ENDFUNCTION" => Token::Endfunction,
        "PROCEDURE" | "PRO" => Token::Pro,
        "ENDPRO" => Token::Endpro,
        "RETURN" => Token::Return,
        "GOTO" => Token::Goto,

        // Other keywords
        "COMMON" => Token::Common,
        "COMPILE_OPT" => Token::CompileOpt,
        "BEGIN" => Token::Begin,
        "END" => Token::End,
        "CASE" => Token::Case,
        "OF" => Token::Of,
        "ENDCASE" => Token::Endcase,
        "SWITCH" => Token::Switch,
        "ENDSWITCH" => Token::Endswitch,

        // Operators (word forms)
        "MOD" => Token::Modulo,
        "EQ" => Token::Equal,
        "NE" => Token::NotEqual,
        "LT" => Token::Less,
        "GT" => Token::Greater,
        "LE" => Token::LessEqual,
        "GE" => Token::GreaterEqual,
        "AND" => Token::And,
        "OR" => Token::Or,
        "NOT" => Token::Not,
        "XOR" => Token::Xor,

        // Regular identifier
        _ => Token::Identifier(name.to_string()),
    };

    Ok((input, token))
}

// Parse system variables (!PI, !X, etc.)
fn parse_system_variable(input: &str) -> ParseResult<'_, Token> {
    preceded(
        char('!'),
        map(take_while1(is_identifier_char), |s: &str| {
            Token::SystemVariable(s.to_uppercase())
        }),
    )(input)
}

// Parse comments
fn parse_comment(input: &str) -> ParseResult<'_, Token> {
    preceded(
        char(';'),
        map(take_while(|c| c != '\n'), |s: &str| {
            Token::Comment(s.to_string())
        }),
    )(input)
}

// Parse operators
fn parse_operator(input: &str) -> ParseResult<'_, Token> {
    alt((
        value(Token::PlusAssign, tag("+=")),
        value(Token::MinusAssign, tag("-=")),
        value(Token::MultiplyAssign, tag("*=")),
        value(Token::DivideAssign, tag("/=")),
        value(Token::Arrow, tag("->")),
        value(Token::MatrixMultiply, char('#')),
        value(Token::Power, char('^')),
        value(Token::Plus, char('+')),
        value(Token::Minus, char('-')),
        value(Token::Multiply, char('*')),
        value(Token::Divide, char('/')),
        value(Token::Assign, char('=')),
        value(Token::QuestionMark, char('?')),
    ))(input)
}

// Parse delimiters
fn parse_delimiter(input: &str) -> ParseResult<'_, Token> {
    alt((
        value(Token::LeftParen, char('(')),
        value(Token::RightParen, char(')')),
        value(Token::LeftBracket, char('[')),
        value(Token::RightBracket, char(']')),
        value(Token::LeftBrace, char('{')),
        value(Token::RightBrace, char('}')),
        value(Token::Comma, char(',')),
        value(Token::Semicolon, char(';')),
        value(Token::DoubleColon, tag("::")),
        value(Token::Colon, char(':')),
        value(Token::Dot, char('.')),
    ))(input)
}

// Parse a single token
fn parse_token(input: &str) -> ParseResult<'_, Token> {
    preceded(
        ws0, // Use ws0 instead of multispace0 to preserve newlines as tokens
        alt((
            parse_comment,
            parse_string,
            parse_number,
            parse_system_variable,
            parse_label, // Try label before identifier (label: vs identifier)
            parse_identifier_or_keyword,
            parse_operator,
            parse_delimiter,
            value(Token::Newline, char('\n')),
        )),
    )(input)
}

// Main tokenizer function
pub fn tokenize(input: &str) -> XdlResult<Vec<Token>> {
    let mut remaining = input;
    let mut tokens = Vec::new();

    while !remaining.is_empty() {
        // Handle line continuation: $ followed by optional whitespace and newline
        if remaining.starts_with('$') {
            let after_dollar = &remaining[1..];
            // Skip whitespace after $
            let trimmed = after_dollar.trim_start_matches([' ', '\t', '\r']);
            // If followed by newline or end of input, it's a line continuation
            if trimmed.is_empty() || trimmed.starts_with('\n') {
                if let Some(stripped) = trimmed.strip_prefix('\n') {
                    remaining = stripped;
                } else {
                    remaining = trimmed;
                }
                continue;
            }
            // Otherwise, skip the $ as unknown character
            remaining = after_dollar;
            continue;
        }

        match parse_token(remaining) {
            Ok((rest, token)) => {
                // Skip comments for now, but keep them for potential use
                match token {
                    Token::Comment(_) => {} // Skip comments
                    _ => tokens.push(token),
                }
                remaining = rest;
            }
            Err(_) => {
                // Skip unknown characters
                remaining = &remaining[1..];
            }
        }
    }

    tokens.push(Token::EOF);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple() {
        let input = "x = 42";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Integer(42),
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_tokenize_string() {
        let input = r#"print, "Hello, World!""#;
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("print".to_string()),
                Token::Comma,
                Token::String("Hello, World!".to_string()),
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_tokenize_keywords() {
        let input = "if x eq 42 then";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::If,
                Token::Identifier("x".to_string()),
                Token::Equal,
                Token::Integer(42),
                Token::Then,
                Token::EOF
            ]
        );
    }

    #[test]
    fn test_tokenize_system_variable() {
        let input = "!PI";
        let tokens = tokenize(input).unwrap();
        assert_eq!(
            tokens,
            vec![Token::SystemVariable("PI".to_string()), Token::EOF]
        );
    }
}
