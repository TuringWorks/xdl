//! MATLAB Lexer - Tokenizes MATLAB .m files

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Function,
    End,
    If,
    Else,
    Elseif,
    For,
    While,
    Switch,
    Case,
    Otherwise,
    Break,
    Continue,
    Return,
    Global,
    Persistent,
    Try,
    Catch,

    // Literals
    Number(f64),
    String(String),

    // Identifiers
    Identifier(String),

    // Operators
    Plus,              // +
    Minus,             // -
    Multiply,          // *
    Divide,            // /
    Power,             // ^
    ElementMultiply,   // .*
    ElementDivide,     // ./
    ElementPower,      // .^
    LeftDivide,        // \
    ElementLeftDivide, // .\

    // Comparison
    Equal,        // ==
    NotEqual,     // ~=
    Less,         // <
    Greater,      // >
    LessEqual,    // <=
    GreaterEqual, // >=

    // Logical
    And,      // &
    Or,       // |
    Not,      // ~
    ShortAnd, // &&
    ShortOr,  // ||

    // Assignment
    Assign, // =

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
    Dot,          // .

    // Special
    Transpose, // '
    Newline,
    Comment(String), // % comment
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
    line: usize,
    column: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            position: 0,
            line: 1,
            column: 1,
        }
    }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while !self.is_at_end() {
            self.skip_whitespace_except_newline();

            if self.is_at_end() {
                break;
            }

            let token = self.next_token()?;

            // Skip newlines after semicolons (statement separators)
            if token.kind != TokenKind::Newline || !tokens.is_empty() {
                tokens.push(token);
            }
        }

        tokens.push(Token {
            kind: TokenKind::EOF,
            lexeme: String::new(),
            line: self.line,
            column: self.column,
        });

        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, String> {
        let start_line = self.line;
        let start_column = self.column;

        let ch = self.current_char();

        // Comments
        if ch == '%' {
            return Ok(self.read_comment(start_line, start_column));
        }

        // Newline
        if ch == '\n' || ch == '\r' {
            self.advance();
            if ch == '\r' && self.current_char() == '\n' {
                self.advance();
            }
            self.line += 1;
            self.column = 1;
            return Ok(Token {
                kind: TokenKind::Newline,
                lexeme: "\n".to_string(),
                line: start_line,
                column: start_column,
            });
        }

        // Numbers
        if ch.is_ascii_digit() || (ch == '.' && self.peek_char().is_ascii_digit()) {
            return Ok(self.read_number(start_line, start_column)?);
        }

        // Strings
        if ch == '\'' && !self.is_after_identifier() {
            return Ok(self.read_string(start_line, start_column)?);
        }

        // Identifiers and keywords
        if ch.is_alphabetic() || ch == '_' {
            return Ok(self.read_identifier(start_line, start_column));
        }

        // Operators and delimiters
        self.advance();

        let token_kind = match ch {
            '+' => TokenKind::Plus,
            '-' => TokenKind::Minus,
            '*' => TokenKind::Multiply,
            '/' => TokenKind::Divide,
            '^' => TokenKind::Power,
            '\\' => TokenKind::LeftDivide,
            '(' => TokenKind::LeftParen,
            ')' => TokenKind::RightParen,
            '[' => TokenKind::LeftBracket,
            ']' => TokenKind::RightBracket,
            '{' => TokenKind::LeftBrace,
            '}' => TokenKind::RightBrace,
            ',' => TokenKind::Comma,
            ';' => TokenKind::Semicolon,
            ':' => TokenKind::Colon,
            '\'' => TokenKind::Transpose,
            '~' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenKind::NotEqual
                } else {
                    TokenKind::Not
                }
            }
            '=' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenKind::Equal
                } else {
                    TokenKind::Assign
                }
            }
            '<' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                }
            }
            '>' => {
                if self.current_char() == '=' {
                    self.advance();
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                }
            }
            '&' => {
                if self.current_char() == '&' {
                    self.advance();
                    TokenKind::ShortAnd
                } else {
                    TokenKind::And
                }
            }
            '|' => {
                if self.current_char() == '|' {
                    self.advance();
                    TokenKind::ShortOr
                } else {
                    TokenKind::Or
                }
            }
            '.' => match self.current_char() {
                '*' => {
                    self.advance();
                    TokenKind::ElementMultiply
                }
                '/' => {
                    self.advance();
                    TokenKind::ElementDivide
                }
                '^' => {
                    self.advance();
                    TokenKind::ElementPower
                }
                '\\' => {
                    self.advance();
                    TokenKind::ElementLeftDivide
                }
                _ => TokenKind::Dot,
            },
            _ => {
                return Err(format!(
                    "Unexpected character '{}' at line {}, column {}",
                    ch, start_line, start_column
                ))
            }
        };

        Ok(Token {
            kind: token_kind,
            lexeme: ch.to_string(),
            line: start_line,
            column: start_column,
        })
    }

    fn read_comment(&mut self, line: usize, column: usize) -> Token {
        let mut comment = String::new();
        self.advance(); // skip '%'

        while !self.is_at_end() && self.current_char() != '\n' && self.current_char() != '\r' {
            comment.push(self.current_char());
            self.advance();
        }

        Token {
            kind: TokenKind::Comment(comment.trim().to_string()),
            lexeme: format!("%{}", comment),
            line,
            column,
        }
    }

    fn read_number(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut num_str = String::new();

        // Integer part
        while !self.is_at_end() && self.current_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();
        }

        // Decimal part
        if !self.is_at_end() && self.current_char() == '.' && self.peek_char().is_ascii_digit() {
            num_str.push(self.current_char());
            self.advance();

            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                num_str.push(self.current_char());
                self.advance();
            }
        }

        // Scientific notation
        if !self.is_at_end() && (self.current_char() == 'e' || self.current_char() == 'E') {
            num_str.push(self.current_char());
            self.advance();

            if !self.is_at_end() && (self.current_char() == '+' || self.current_char() == '-') {
                num_str.push(self.current_char());
                self.advance();
            }

            while !self.is_at_end() && self.current_char().is_ascii_digit() {
                num_str.push(self.current_char());
                self.advance();
            }
        }

        let value = num_str
            .parse::<f64>()
            .map_err(|_| format!("Invalid number: {}", num_str))?;

        Ok(Token {
            kind: TokenKind::Number(value),
            lexeme: num_str,
            line,
            column,
        })
    }

    fn read_string(&mut self, line: usize, column: usize) -> Result<Token, String> {
        let mut string = String::new();
        self.advance(); // skip opening '

        while !self.is_at_end() && self.current_char() != '\'' {
            if self.current_char() == '\'' && self.peek_char() == '\'' {
                // Escaped quote
                string.push('\'');
                self.advance();
                self.advance();
            } else {
                string.push(self.current_char());
                self.advance();
            }
        }

        if self.is_at_end() {
            return Err("Unterminated string".to_string());
        }

        self.advance(); // skip closing '

        Ok(Token {
            kind: TokenKind::String(string.clone()),
            lexeme: format!("'{}'", string),
            line,
            column,
        })
    }

    fn read_identifier(&mut self, line: usize, column: usize) -> Token {
        let mut ident = String::new();

        while !self.is_at_end()
            && (self.current_char().is_alphanumeric() || self.current_char() == '_')
        {
            ident.push(self.current_char());
            self.advance();
        }

        let kind = match ident.as_str() {
            "function" => TokenKind::Function,
            "end" => TokenKind::End,
            "if" => TokenKind::If,
            "else" => TokenKind::Else,
            "elseif" => TokenKind::Elseif,
            "for" => TokenKind::For,
            "while" => TokenKind::While,
            "switch" => TokenKind::Switch,
            "case" => TokenKind::Case,
            "otherwise" => TokenKind::Otherwise,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "return" => TokenKind::Return,
            "global" => TokenKind::Global,
            "persistent" => TokenKind::Persistent,
            "try" => TokenKind::Try,
            "catch" => TokenKind::Catch,
            _ => TokenKind::Identifier(ident.clone()),
        };

        Token {
            kind,
            lexeme: ident,
            line,
            column,
        }
    }

    fn current_char(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.input[self.position]
        }
    }

    fn peek_char(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn advance(&mut self) {
        if !self.is_at_end() {
            self.position += 1;
            self.column += 1;
        }
    }

    fn is_at_end(&self) -> bool {
        self.position >= self.input.len()
    }

    fn skip_whitespace_except_newline(&mut self) {
        while !self.is_at_end() {
            let ch = self.current_char();
            if ch == ' ' || ch == '\t' {
                self.advance();
            } else if ch == '.'
                && self.peek_char() == '.'
                && self.position + 2 < self.input.len()
                && self.input[self.position + 2] == '.'
            {
                // Line continuation ...
                self.advance();
                self.advance();
                self.advance();
                // Skip rest of line
                while !self.is_at_end() && self.current_char() != '\n' {
                    self.advance();
                }
                if !self.is_at_end() {
                    self.advance(); // skip newline
                    self.line += 1;
                    self.column = 1;
                }
            } else {
                break;
            }
        }
    }

    fn is_after_identifier(&self) -> bool {
        // Check if previous non-whitespace character was alphanumeric
        // This helps distinguish transpose ' from string delimiter '
        if self.position == 0 {
            return false;
        }

        let mut pos = self.position - 1;
        while pos > 0 {
            let ch = self.input[pos];
            if ch == ' ' || ch == '\t' {
                pos -= 1;
                continue;
            }
            return ch.is_alphanumeric() || ch == ')' || ch == ']' || ch == '}';
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_tokens() {
        let mut lexer = Lexer::new("x = 5 + 3");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens.len(), 6); // x, =, 5, +, 3, EOF
    }

    #[test]
    fn test_element_wise_ops() {
        let mut lexer = Lexer::new("a .* b ./ c .^ d");
        let tokens = lexer.tokenize().unwrap();
        assert!(matches!(tokens[1].kind, TokenKind::ElementMultiply));
        assert!(matches!(tokens[3].kind, TokenKind::ElementDivide));
        assert!(matches!(tokens[5].kind, TokenKind::ElementPower));
    }

    #[test]
    fn test_comment() {
        let mut lexer = Lexer::new("x = 5 % this is a comment\ny = 10");
        let tokens = lexer.tokenize().unwrap();
        let comment_token = tokens
            .iter()
            .find(|t| matches!(t.kind, TokenKind::Comment(_)));
        assert!(comment_token.is_some());
    }
}
