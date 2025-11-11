//! XDL Parser implementation

use crate::ast::*;
use crate::lexer::Token;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Parser state tracking current position in token stream
struct Parser<'a> {
    tokens: &'a [Token],
    current: usize,
}

impl<'a> Parser<'a> {
    fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, current: 0 }
    }

    /// Get current token without consuming it
    fn peek(&self) -> &Token {
        self.tokens.get(self.current).unwrap_or(&Token::EOF)
    }

    /// Get current token and advance
    fn advance(&mut self) -> &Token {
        if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            self.current += 1;
            token
        } else {
            &Token::EOF
        }
    }

    /// Check if current token matches expected token
    fn check(&self, expected: &Token) -> bool {
        std::mem::discriminant(self.peek()) == std::mem::discriminant(expected)
    }

    /// Consume token if it matches expected, otherwise error
    fn consume(&mut self, expected: Token, message: &str) -> XdlResult<()> {
        if self.check(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(XdlError::ParseError {
                message: format!("{}, got {:?}", message, self.peek()),
                line: 1, // TODO: track line numbers
                column: self.current,
            })
        }
    }

    /// Check if we're at end of tokens
    fn is_at_end(&self) -> bool {
        matches!(self.peek(), Token::EOF)
    }

    /// Parse the entire program
    fn parse_program(&mut self) -> XdlResult<Program> {
        let mut statements = Vec::new();

        while !self.is_at_end() {
            statements.push(self.parse_statement()?);
        }

        Ok(Program {
            statements,
            location: Location::unknown(),
        })
    }

    /// Parse a block (begin...end) or statements until terminator
    fn parse_block_or_statement(&mut self, terminators: &[Token]) -> XdlResult<Vec<Statement>> {
        // Check if this is a begin...end block
        if self.check(&Token::Begin) {
            self.advance(); // consume 'begin'
            let mut statements = Vec::new();
            while !self.check(&Token::End) && !self.is_at_end() {
                statements.push(self.parse_statement()?);
            }
            self.consume(Token::End, "Expected 'end' to close begin block")?;
            Ok(statements)
        } else {
            // Parse statements until we hit a terminator
            // The key insight: parse_statement() handles nested constructs recursively
            let mut statements = Vec::new();
            while !self.is_at_end() {
                let is_terminator = terminators.iter().any(|term| {
                    std::mem::discriminant(self.peek()) == std::mem::discriminant(term)
                });
                if is_terminator {
                    break;
                }
                statements.push(self.parse_statement()?);
            }
            Ok(statements)
        }
    }

    /// Parse a statement
    fn parse_statement(&mut self) -> XdlResult<Statement> {
        match self.peek() {
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            Token::Foreach => self.parse_foreach_statement(),
            Token::While => self.parse_while_statement(),
            Token::Repeat => self.parse_repeat_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Break => {
                self.advance();
                Ok(Statement::Break {
                    location: Location::unknown(),
                })
            }
            Token::Continue => {
                self.advance();
                Ok(Statement::Continue {
                    location: Location::unknown(),
                })
            }
            Token::Pro | Token::Procedure => self.parse_procedure_definition(),
            Token::Function => self.parse_function_definition(),
            _ => {
                // Try to parse as procedure call, expression statement, or assignment
                if let Token::Identifier(name) = self.peek() {
                    let name = name.clone();
                    let start_pos = self.current;
                    self.advance(); // consume identifier

                    // Check if this is a procedure call (identifier followed by comma or end of statement)
                    if self.check(&Token::Comma)
                        || self.is_at_end()
                        || matches!(self.peek(), Token::EOF)
                    {
                        return self.parse_procedure_call(name);
                    }

                    // Not a procedure call, backtrack and parse as expression
                    self.current = start_pos;
                }

                let expr = self.parse_expression()?;

                // Check if this is an assignment
                if self.check(&Token::Assign) {
                    self.advance(); // consume '='
                    let value = self.parse_expression()?;
                    Ok(Statement::Assignment {
                        target: expr,
                        value,
                        location: Location::unknown(),
                    })
                } else {
                    Ok(Statement::Expression {
                        expr,
                        location: Location::unknown(),
                    })
                }
            }
        }
    }

    /// Parse if statement
    fn parse_if_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::If, "Expected 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::Then, "Expected 'then' after if condition")?;

        // Parse then block - supports begin...end blocks or statements until else/endif
        let then_block = self.parse_block_or_statement(&[Token::Else, Token::Endif])?;

        let else_block = if self.check(&Token::Else) {
            self.advance(); // consume 'else'
                            // Parse else block - supports begin...end blocks or statements until endif
            Some(self.parse_block_or_statement(&[Token::Endif])?)
        } else {
            None
        };

        // Consume endif - required for if statements
        self.consume(Token::Endif, "Expected 'endif' to close if statement")?;

        Ok(Statement::If {
            condition,
            then_block,
            else_block,
            location: Location::unknown(),
        })
    }

    /// Parse for statement
    fn parse_for_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::For, "Expected 'for'")?;

        // Parse variable = start, end [, step]
        let variable = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err(XdlError::ParseError {
                message: "Expected variable name in for loop".to_string(),
                line: 1,
                column: self.current,
            });
        };

        self.consume(Token::Assign, "Expected '=' after for variable")?;
        let start = self.parse_expression()?;
        self.consume(Token::Comma, "Expected ',' after for start value")?;
        let end = self.parse_expression()?;

        let step = if self.check(&Token::Comma) {
            self.advance(); // consume ','
            Some(self.parse_expression()?)
        } else {
            None
        };

        // Check for 'do' keyword (optional)
        if matches!(self.peek(), Token::Identifier(s) if s.to_uppercase() == "DO") {
            self.advance(); // consume 'do'
        }

        // Parse body - support both 'begin...end' and multiple statements until endfor
        let body = self.parse_block_or_statement(&[Token::Endfor])?;

        self.consume(Token::Endfor, "Expected 'endfor' to close for loop")?;

        Ok(Statement::For {
            variable,
            start,
            end,
            step,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse procedure call
    fn parse_procedure_call(&mut self, name: String) -> XdlResult<Statement> {
        let mut args = Vec::new();
        let mut keywords = Vec::new();

        // Parse comma-separated arguments
        while self.check(&Token::Comma) {
            self.advance(); // consume comma

            // Check for keyword argument (identifier = expression)
            if let Token::Identifier(kw_name) = self.peek() {
                let kw_name = kw_name.clone();
                let next_pos = self.current + 1;

                if next_pos < self.tokens.len() && matches!(self.tokens[next_pos], Token::Assign) {
                    // This is a keyword argument
                    self.advance(); // consume identifier
                    self.advance(); // consume '='
                    let value = self.parse_expression()?;
                    keywords.push(Keyword {
                        name: kw_name,
                        value: Some(value),
                        location: Location::unknown(),
                    });
                    continue;
                }
            }

            // Regular positional argument
            args.push(self.parse_expression()?);
        }

        Ok(Statement::ProcedureCall {
            name,
            args,
            keywords,
            location: Location::unknown(),
        })
    }

    /// Parse foreach statement
    fn parse_foreach_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Foreach, "Expected 'foreach'")?;

        // Parse variable name
        let variable = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err(XdlError::ParseError {
                message: "Expected variable name in foreach loop".to_string(),
                line: 1,
                column: self.current,
            });
        };

        self.consume(Token::Comma, "Expected ',' after foreach variable")?;
        let iterable = self.parse_expression()?;

        // Optional index variable
        let index_var = if self.check(&Token::Comma) {
            self.advance(); // consume ','
            if let Token::Identifier(name) = self.advance() {
                Some(name.clone())
            } else {
                return Err(XdlError::ParseError {
                    message: "Expected index variable name".to_string(),
                    line: 1,
                    column: self.current,
                });
            }
        } else {
            None
        };

        // Check for 'do' keyword (optional)
        if matches!(self.peek(), Token::Identifier(s) if s.to_uppercase() == "DO") {
            self.advance(); // consume 'do'
        }

        // Parse body - support both 'begin...end' and multiple statements
        let body = self.parse_block_or_statement(&[Token::Endfor])?;

        self.consume(
            Token::Endfor,
            "Expected 'endfor' or 'endforeach' to close foreach loop",
        )?;

        Ok(Statement::Foreach {
            variable,
            iterable,
            index_var,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse while statement
    fn parse_while_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::While, "Expected 'while'")?;
        let condition = self.parse_expression()?;

        // Check for 'do' keyword (optional)
        if matches!(self.peek(), Token::Identifier(s) if s.to_uppercase() == "DO") {
            self.advance(); // consume 'do'
        }

        // Parse body - support both 'begin...end' and multiple statements
        let body = self.parse_block_or_statement(&[Token::Endwhile])?;

        self.consume(Token::Endwhile, "Expected 'endwhile' to close while loop")?;

        Ok(Statement::While {
            condition,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse repeat statement
    fn parse_repeat_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Repeat, "Expected 'repeat'")?;

        // Check if we have a 'begin' block
        let body = self.parse_block_or_statement(&[Token::Until])?;

        self.consume(Token::Until, "Expected 'until' to close repeat loop")?;
        let condition = self.parse_expression()?;

        Ok(Statement::Repeat {
            body,
            condition,
            location: Location::unknown(),
        })
    }

    /// Parse return statement
    fn parse_return_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Return, "Expected 'return'")?;

        let value = if matches!(self.peek(), Token::Newline | Token::EOF) {
            None
        } else {
            Some(self.parse_expression()?)
        };

        Ok(Statement::Return {
            value,
            location: Location::unknown(),
        })
    }

    /// Parse procedure definition
    fn parse_procedure_definition(&mut self) -> XdlResult<Statement> {
        self.advance(); // consume 'pro' or 'procedure'

        let name = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err(XdlError::ParseError {
                message: "Expected procedure name".to_string(),
                line: 1,
                column: self.current,
            });
        };

        // Parse parameters (simplified)
        let params = Vec::new(); // TODO: implement parameter parsing
        let keywords = Vec::new(); // TODO: implement keyword parsing

        // Parse body
        let mut body = Vec::new();
        while !matches!(self.peek(), Token::Endpro | Token::EOF) {
            body.push(self.parse_statement()?);
        }

        self.consume(Token::Endpro, "Expected 'endpro' to close procedure")?;

        Ok(Statement::ProcedureDef {
            name,
            params,
            keywords,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse function definition
    fn parse_function_definition(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Function, "Expected 'function'")?;

        let name = if let Token::Identifier(name) = self.advance() {
            name.clone()
        } else {
            return Err(XdlError::ParseError {
                message: "Expected function name".to_string(),
                line: 1,
                column: self.current,
            });
        };

        // Parse parameters (simplified)
        let params = Vec::new(); // TODO: implement parameter parsing
        let keywords = Vec::new(); // TODO: implement keyword parsing

        // Parse body
        let mut body = Vec::new();
        while !matches!(self.peek(), Token::Endfunction | Token::EOF) {
            body.push(self.parse_statement()?);
        }

        self.consume(
            Token::Endfunction,
            "Expected 'endfunction' to close function",
        )?;

        Ok(Statement::FunctionDef {
            name,
            params,
            keywords,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse expression with precedence
    fn parse_expression(&mut self) -> XdlResult<Expression> {
        self.parse_logical_or()
    }

    /// Parse logical OR expressions
    fn parse_logical_or(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_logical_and()?;

        while self.check(&Token::Or) {
            self.advance();
            let right = self.parse_logical_and()?;
            expr = Expression::Binary {
                op: BinaryOp::Or,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse logical AND expressions
    fn parse_logical_and(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_equality()?;

        while self.check(&Token::And) {
            self.advance();
            let right = self.parse_equality()?;
            expr = Expression::Binary {
                op: BinaryOp::And,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse equality expressions (EQ, NE)
    fn parse_equality(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_comparison()?;

        loop {
            let op = match self.peek() {
                Token::Equal => BinaryOp::Equal,
                Token::NotEqual => BinaryOp::NotEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_comparison()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse comparison expressions (LT, GT, LE, GE)
    fn parse_comparison(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_addition()?;

        loop {
            let op = match self.peek() {
                Token::Less => BinaryOp::Less,
                Token::Greater => BinaryOp::Greater,
                Token::LessEqual => BinaryOp::LessEqual,
                Token::GreaterEqual => BinaryOp::GreaterEqual,
                _ => break,
            };

            self.advance();
            let right = self.parse_addition()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse addition and subtraction
    fn parse_addition(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_multiplication()?;

        loop {
            let op = match self.peek() {
                Token::Plus => BinaryOp::Add,
                Token::Minus => BinaryOp::Subtract,
                _ => break,
            };

            self.advance();
            let right = self.parse_multiplication()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse multiplication, division, and modulo
    fn parse_multiplication(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_power()?;

        loop {
            let op = match self.peek() {
                Token::Multiply => BinaryOp::Multiply,
                Token::Divide => BinaryOp::Divide,
                Token::Modulo => BinaryOp::Modulo,
                Token::MatrixMultiply => BinaryOp::MatrixMultiply,
                _ => break,
            };

            self.advance();
            let right = self.parse_power()?;
            expr = Expression::Binary {
                op,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse power expressions (right associative)
    fn parse_power(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_unary()?;

        if self.check(&Token::Power) {
            self.advance();
            let right = self.parse_power()?; // Right associative
            expr = Expression::Binary {
                op: BinaryOp::Power,
                left: Box::new(expr),
                right: Box::new(right),
                location: Location::unknown(),
            };
        }

        Ok(expr)
    }

    /// Parse unary expressions
    fn parse_unary(&mut self) -> XdlResult<Expression> {
        match self.peek() {
            Token::Not => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Not,
                    expr: Box::new(expr),
                    location: Location::unknown(),
                })
            }
            Token::Minus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Minus,
                    expr: Box::new(expr),
                    location: Location::unknown(),
                })
            }
            Token::Plus => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Unary {
                    op: UnaryOp::Plus,
                    expr: Box::new(expr),
                    location: Location::unknown(),
                })
            }
            _ => self.parse_postfix(),
        }
    }

    /// Parse postfix expressions (array indexing, function calls, etc.)
    fn parse_postfix(&mut self) -> XdlResult<Expression> {
        let mut expr = self.parse_primary()?;

        // Handle postfix operations like array indexing
        loop {
            if self.check(&Token::LeftBracket) {
                self.advance(); // consume '['
                let indices = self.parse_array_indices()?;
                self.consume(Token::RightBracket, "Expected ']' after array indices")?;

                expr = Expression::ArrayRef {
                    array: Box::new(expr),
                    indices,
                    location: Location::unknown(),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    /// Parse array indices (single index or range)
    fn parse_array_indices(&mut self) -> XdlResult<Vec<ArrayIndex>> {
        let mut indices = Vec::new();

        loop {
            // Check for range with leading colon (e.g., [:5])
            if self.check(&Token::Colon) {
                self.advance(); // consume ':'
                let end = if self.check(&Token::RightBracket) || self.check(&Token::Comma) {
                    None
                } else {
                    Some(self.parse_expression()?)
                };

                indices.push(ArrayIndex::Range {
                    start: None,
                    end,
                    step: None,
                });
            } else {
                // Parse first expression
                let first_expr = self.parse_expression()?;

                // Check if this is a range
                if self.check(&Token::Colon) {
                    self.advance(); // consume ':'

                    let end = if self.check(&Token::RightBracket) || self.check(&Token::Comma) {
                        None
                    } else {
                        Some(self.parse_expression()?)
                    };

                    // Check for step (e.g., 0:10:2)
                    let step = if self.check(&Token::Colon) {
                        self.advance();
                        Some(self.parse_expression()?)
                    } else {
                        None
                    };

                    indices.push(ArrayIndex::Range {
                        start: Some(first_expr),
                        end,
                        step,
                    });
                } else {
                    // Single index
                    indices.push(ArrayIndex::Single(first_expr));
                }
            }

            // Check for multiple dimensions (e.g., arr[i, j])
            if self.check(&Token::Comma) {
                self.advance();
            } else {
                break;
            }
        }

        Ok(indices)
    }

    /// Parse primary expressions (literals, identifiers, parenthesized expressions)
    fn parse_primary(&mut self) -> XdlResult<Expression> {
        match self.advance() {
            Token::Integer(value) => Ok(Expression::Literal {
                value: XdlValue::Long(*value as i32),
                location: Location::unknown(),
            }),
            Token::Float(value) => Ok(Expression::Literal {
                value: XdlValue::Double(*value),
                location: Location::unknown(),
            }),
            Token::String(value) => Ok(Expression::Literal {
                value: XdlValue::String(value.clone()),
                location: Location::unknown(),
            }),
            Token::Identifier(name) => {
                let name = name.clone();
                // Check if this is a function call
                if self.check(&Token::LeftParen) {
                    self.advance(); // consume '('
                    let mut args = Vec::new();

                    if !self.check(&Token::RightParen) {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.check(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    self.consume(Token::RightParen, "Expected ')' after function arguments")?;

                    Ok(Expression::FunctionCall {
                        name,
                        args,
                        keywords: Vec::new(), // TODO: implement keyword arguments
                        location: Location::unknown(),
                    })
                } else {
                    Ok(Expression::Variable {
                        name,
                        location: Location::unknown(),
                    })
                }
            }
            Token::SystemVariable(name) => {
                let name = name.clone();
                Ok(Expression::SystemVariable {
                    name,
                    location: Location::unknown(),
                })
            }
            Token::LeftParen => {
                let expr = self.parse_expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(expr)
            }
            Token::LeftBracket => {
                // Array definition [1, 2, 3]
                let mut elements = Vec::new();

                if !self.check(&Token::RightBracket) {
                    loop {
                        elements.push(self.parse_expression()?);
                        if self.check(&Token::Comma) {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }

                self.consume(Token::RightBracket, "Expected ']' after array elements")?;

                Ok(Expression::ArrayDef {
                    elements,
                    location: Location::unknown(),
                })
            }
            token => Err(XdlError::ParseError {
                message: format!("Unexpected token: {:?}", token),
                line: 1,
                column: self.current,
            }),
        }
    }
}

// Public interface functions
pub fn parse_program(tokens: &[Token]) -> XdlResult<Program> {
    let mut parser = Parser::new(tokens);
    parser.parse_program()
}

pub fn parse_expression(tokens: &[Token]) -> XdlResult<Expression> {
    let mut parser = Parser::new(tokens);
    parser.parse_expression()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple_assignment() {
        let input = "x = 42";
        let tokens = tokenize(input).unwrap();
        let program = parse_program(&tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::Assignment { target, value, .. } => {
                assert!(matches!(target, Expression::Variable { name, .. } if name == "x"));
                assert!(matches!(
                    value,
                    Expression::Literal {
                        value: XdlValue::Long(42),
                        ..
                    }
                ));
            }
            _ => panic!("Expected assignment statement"),
        }
    }

    #[test]
    fn test_parse_arithmetic_expression() {
        let input = "2 + 3 * 4";
        let tokens = tokenize(input).unwrap();
        let expr = parse_expression(&tokens).unwrap();

        match expr {
            Expression::Binary {
                op: BinaryOp::Add,
                left,
                right,
                ..
            } => {
                assert!(matches!(
                    left.as_ref(),
                    Expression::Literal {
                        value: XdlValue::Long(2),
                        ..
                    }
                ));
                match right.as_ref() {
                    Expression::Binary {
                        op: BinaryOp::Multiply,
                        ..
                    } => {}
                    _ => panic!("Expected multiplication on right side"),
                }
            }
            _ => panic!("Expected binary addition expression"),
        }
    }

    #[test]
    fn test_parse_function_call() {
        let input = "sin(x)";
        let tokens = tokenize(input).unwrap();
        let expr = parse_expression(&tokens).unwrap();

        match expr {
            Expression::FunctionCall { name, args, .. } => {
                assert_eq!(name, "sin");
                assert_eq!(args.len(), 1);
                assert!(matches!(args[0], Expression::Variable { name: ref n, .. } if n == "x"));
            }
            _ => panic!("Expected function call expression"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let input = "if x eq 42 then\n  y = 1\nendif";
        let tokens = tokenize(input).unwrap();
        let program = parse_program(&tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                assert!(matches!(
                    condition,
                    Expression::Binary {
                        op: BinaryOp::Equal,
                        ..
                    }
                ));
                assert_eq!(then_block.len(), 1);
                assert!(else_block.is_none());
            }
            _ => panic!("Expected if statement"),
        }
    }

    #[test]
    fn test_parse_for_loop() {
        let input = "for i = 0, 10\n  x = i\nendfor";
        let tokens = tokenize(input).unwrap();
        let program = parse_program(&tokens).unwrap();

        assert_eq!(program.statements.len(), 1);
        match &program.statements[0] {
            Statement::For {
                variable,
                start,
                end,
                step,
                body,
                ..
            } => {
                assert_eq!(variable, "i");
                assert!(matches!(
                    start,
                    Expression::Literal {
                        value: XdlValue::Long(0),
                        ..
                    }
                ));
                assert!(matches!(
                    end,
                    Expression::Literal {
                        value: XdlValue::Long(10),
                        ..
                    }
                ));
                assert!(step.is_none());
                assert_eq!(body.len(), 1);
            }
            _ => panic!("Expected for statement"),
        }
    }
}
