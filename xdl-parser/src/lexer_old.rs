//! XDL Lexer implementation

use xdl_core::{XdlResult, XdlError};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while, take_while1},
    character::complete::{char, digit1, multispace0, multispace1, none_of},
    combinator::{map, map_res, opt, recognize, value},
    multi::{many0, many_till},
    sequence::{delimited, pair, preceded, terminated},
    IResult,
};

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
    If, Then, Else, Endif,
    For, Endfor, Foreach,
    While, Endwhile,
    Repeat, Until,
    Break, Continue,
    Function, Endfunction,
    Procedure, Pro, Endpro,
    Return,
    Common,
    CompileOpt,
    Begin, End,
    Case, Of, Endcase,
    Switch, Endswitch,

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
    Equal,          // EQ
    NotEqual,       // NE
    Less,           // LT
    Greater,        // GT
    LessEqual,      // LE
    GreaterEqual,   // GE

    // Logical
    And,            // AND
    Or,             // OR
    Not,            // NOT
    Xor,            // XOR

    // Bitwise (rarely used but part of IDL)
    BitwiseAnd,     // AND (bitwise)
    BitwiseOr,      // OR (bitwise)
    BitwiseXor,     // XOR (bitwise)
    BitwiseNot,     // NOT (bitwise)

    // Delimiters
    LeftParen,      // (
    RightParen,     // )
    LeftBracket,    // [
    RightBracket,   // ]
    LeftBrace,      // {
    RightBrace,     // }
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    Dot,            // .
    Arrow,          // ->

    // Special
    Identifier(String),
    SystemVariable(String), // !PI, !X, etc.
    Label(String),          // label:
    Comment(String),        // ; comment
    Newline,
    EOF,
}

type ParseResult<T> = IResult<&str, T>;

// Helper function to check if character is valid in identifier
fn is_identifier_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

fn is_identifier_start(c: char) -> bool {
    c.is_alphabetic() || c == '_'
}

// Parse integers
fn parse_integer(input: &str) -> ParseResult<Token> {
    map_res(digit1, |s: &str| {
        s.parse::<i64>().map(Token::Integer)
    })(input)
}

// Parse floating point numbers
fn parse_float(input: &str) -> ParseResult<Token> {
    map_res(
        recognize(pair(
            digit1,
            pair(char('.'), opt(digit1))
        )),
        |s: &str| s.parse::<f64>().map(Token::Float)
    )(input)
}

// Parse numbers (float or integer)
fn parse_number(input: &str) -> ParseResult<Token> {
    alt((parse_float, parse_integer))(input)
}

// Parse string literals
fn parse_string(input: &str) -> ParseResult<Token> {
    alt((
        // Double quoted strings
        delimited(
            char('"'),
            map(many0(none_of("\"")), |chars| {
                Token::String(chars.into_iter().collect())
            }),
            char('"')
        ),
        // Single quoted strings
        delimited(
            char('\''),
            map(many0(none_of("'")), |chars| {
                Token::String(chars.into_iter().collect())
            }),
            char('\'')
        )
    ))(input)
}

// Parse identifiers and keywords
fn parse_identifier_or_keyword(input: &str) -> ParseResult<Token> {
    let (input, name) = recognize(pair(
        take_while1(is_identifier_start),
        take_while(is_identifier_char)
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
fn parse_system_variable(input: &str) -> ParseResult<Token> {
    preceded(
        char('!'),
        map(
            take_while1(is_identifier_char),
            |s: &str| Token::SystemVariable(s.to_uppercase())
        )
    )(input)
}

// Parse comments
fn parse_comment(input: &str) -> ParseResult<Token> {
    preceded(
        char(';'),
        map(
            take_while(|c| c != '\n'),
            |s: &str| Token::Comment(s.to_string())
        )
    )(input)
}

// Parse operators
fn parse_operator(input: &str) -> ParseResult<Token> {
    alt((
        value(Token::PlusAssign, tag("+=")),
        value(Token::MinusAssign, tag("-=")),
        value(Token::MultiplyAssign, tag("*=")),
        value(Token::DivideAssign, tag("/=")),
        value(Token::Arrow, tag("->"))),
        value(Token::MatrixMultiply, char('#')),
        value(Token::Power, char('^')),
        value(Token::Plus, char('+')),
        value(Token::Minus, char('-')),
        value(Token::Multiply, char('*')),
        value(Token::Divide, char('/')),
        value(Token::Assign, char('=')),
    ))(input)
}

// Parse delimiters
fn parse_delimiter(input: &str) -> ParseResult<Token> {
    alt((
        value(Token::LeftParen, char('(')),
        value(Token::RightParen, char(')')),
        value(Token::LeftBracket, char('[')),
        value(Token::RightBracket, char(']')),
        value(Token::LeftBrace, char('{')),
        value(Token::RightBrace, char('}')),
        value(Token::Comma, char(',')),
        value(Token::Semicolon, char(';')),
        value(Token::Colon, char(':')),
        value(Token::Dot, char('.')),
    ))(input)
}

// Parse a single token
fn parse_token(input: &str) -> ParseResult<Token> {
    preceded(
        multispace0,
        alt((
            parse_comment,
            parse_string,
            parse_number,
            parse_system_variable,
            parse_identifier_or_keyword,
            parse_operator,
            parse_delimiter,
            value(Token::Newline, char('\n')),
        ))
    )(input)
}

// Main tokenizer function
pub fn tokenize(input: &str) -> XdlResult<Vec<Token>> {
    let mut remaining = input;
    let mut tokens = Vec::new();

    while !remaining.is_empty() {
        match parse_token(remaining) {
            Ok((rest, token)) => {
                // Skip comments for now, but keep them for potential use
                match token {
                    Token::Comment(_) => {}, // Skip comments
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
