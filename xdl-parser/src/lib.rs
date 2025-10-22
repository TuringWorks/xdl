//! # XDL Parser
//!
//! Parser for the Extended Data Language (XDL/IDL) using nom combinator library.

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;

pub use ast::*;
pub use error::*;
pub use lexer::*;
pub use parser::*;

/// Parse XDL source code into an AST
pub fn parse_xdl(input: &str) -> crate::XdlResult<Program> {
    let tokens = lexer::tokenize(input)?;
    parser::parse_program(&tokens)
}

/// Parse a single XDL expression
pub fn parse_expression(input: &str) -> crate::XdlResult<Expression> {
    let tokens = lexer::tokenize(input)?;
    parser::parse_expression(&tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_parse() {
        let input = "x = 42";
        let result = parse_xdl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_expression_parse() {
        let input = "2 + 3 * 4";
        let result = parse_expression(input);
        assert!(result.is_ok());
    }
}
