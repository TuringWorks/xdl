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

        loop {
            // Skip leading/trailing newlines
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }
            // Check if we've reached the end
            if self.is_at_end() {
                break;
            }
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

            // GDL/IDL allows both 'end' and specific terminators (endif, endfor, endwhile)
            // to close a begin block. E.g., "for i=1,3 do begin ... endfor" is valid.
            while !self.is_at_end() {
                // Check if we hit 'end' or any of the terminators
                if self.check(&Token::End) {
                    self.advance(); // consume 'end'
                    break;
                }

                // Check if we hit a terminator (like endif, endfor, endwhile)
                let is_terminator = terminators.iter().any(|term| {
                    std::mem::discriminant(self.peek()) == std::mem::discriminant(term)
                });
                if is_terminator {
                    // Don't consume the terminator - let the caller handle it
                    break;
                }

                statements.push(self.parse_statement()?);
            }
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
        // Skip leading newlines
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

        // If we've reached EOF after skipping newlines, return an error
        if self.is_at_end() {
            return Err(XdlError::ParseError {
                message: "Unexpected end of file".to_string(),
                line: 0,
                column: self.current,
            });
        }

        match self.peek() {
            Token::If => self.parse_if_statement(),
            Token::For => self.parse_for_statement(),
            Token::Foreach => self.parse_foreach_statement(),
            Token::While => self.parse_while_statement(),
            Token::Repeat => self.parse_repeat_statement(),
            Token::Return => self.parse_return_statement(),
            Token::Goto => self.parse_goto_statement(),
            Token::Label(name) => {
                let label_name = name.clone();
                self.advance();
                Ok(Statement::Label {
                    name: label_name,
                    location: Location::unknown(),
                })
            }
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
            Token::Case => self.parse_case_statement(),
            Token::Switch => self.parse_switch_statement(),
            _ => {
                // Try to parse as procedure call, expression statement, or assignment
                if let Token::Identifier(name) = self.peek() {
                    let name = name.clone();
                    let start_pos = self.current;
                    self.advance(); // consume identifier

                    // Check if this is a procedure call (identifier followed by comma, newline, or end of statement)
                    if self.check(&Token::Comma)
                        || self.is_at_end()
                        || matches!(self.peek(), Token::EOF | Token::Newline)
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
    /// Supports both single-line (IF x THEN y) and multi-line (IF x THEN BEGIN...ENDIF) forms
    fn parse_if_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::If, "Expected 'if'")?;
        let condition = self.parse_expression()?;
        self.consume(Token::Then, "Expected 'then' after if condition")?;

        // Check if this is a block (BEGIN) or single-line form
        let is_block = self.check(&Token::Begin);

        let then_block = if is_block {
            // Multi-line form: IF x THEN BEGIN ... END/ENDIF
            self.parse_block_or_statement(&[Token::Else, Token::Endif])?
        } else {
            // Non-block form: IF x THEN statement(s) ENDIF
            // Parse statements until we hit ELSE or ENDIF
            let mut stmts = Vec::new();
            loop {
                // Skip newlines before checking for ELSE/ENDIF
                while matches!(self.peek(), Token::Newline) {
                    self.advance();
                }
                if matches!(self.peek(), Token::Else | Token::Endif | Token::EOF) {
                    break;
                }
                stmts.push(self.parse_statement()?);
            }
            stmts
        };

        // Skip newlines before ELSE check
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

        let else_block = if self.check(&Token::Else) {
            self.advance(); // consume 'else'
            if is_block || self.check(&Token::Begin) {
                // Multi-line else block
                Some(self.parse_block_or_statement(&[Token::Endif])?)
            } else {
                // Non-block else: parse until ENDIF
                let mut stmts = Vec::new();
                loop {
                    while matches!(self.peek(), Token::Newline) {
                        self.advance();
                    }
                    if matches!(self.peek(), Token::Endif | Token::EOF) {
                        break;
                    }
                    stmts.push(self.parse_statement()?);
                }
                Some(stmts)
            }
        } else {
            None
        };

        // Skip newlines before ENDIF check
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

        // Consume ENDIF if present
        if self.check(&Token::Endif) {
            self.advance(); // consume 'endif'
        }

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
        let body = if self.check(&Token::Begin) {
            self.parse_block_or_statement(&[Token::Endfor])?
        } else {
            // Parse statements until ENDFOR
            let mut stmts = Vec::new();
            loop {
                while matches!(self.peek(), Token::Newline) {
                    self.advance();
                }
                if matches!(self.peek(), Token::Endfor | Token::EOF) {
                    break;
                }
                stmts.push(self.parse_statement()?);
            }
            stmts
        };

        // Skip newlines before ENDFOR
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

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

            // Check if this is a trailing comma (end of line or statement)
            if matches!(self.peek(), Token::Newline | Token::EOF) {
                break;
            }

            // Check for /KEYWORD syntax (shorthand for KEYWORD=1)
            if matches!(self.peek(), Token::Divide) {
                let next_pos = self.current + 1;
                if next_pos < self.tokens.len() {
                    if let Token::Identifier(kw_name) = &self.tokens[next_pos] {
                        let kw_name = kw_name.clone();
                        self.advance(); // consume '/'
                        self.advance(); // consume identifier
                        keywords.push(Keyword {
                            name: kw_name,
                            value: Some(Expression::Literal {
                                value: XdlValue::Long(1),
                                location: Location::unknown(),
                            }),
                            location: Location::unknown(),
                        });
                        continue;
                    }
                }
            }

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

        // Check if this is OBJ_DESTROY
        if name.to_uppercase() == "OBJ_DESTROY" {
            return Ok(Statement::ObjectDestroy {
                objects: args,
                location: Location::unknown(),
            });
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

        // IDL syntax: RETURN or RETURN, value (comma is optional)
        let value = if matches!(self.peek(), Token::Newline | Token::EOF) {
            None
        } else {
            // Skip optional comma after RETURN
            if self.check(&Token::Comma) {
                self.advance();
            }
            // Check again after comma - might be end of statement
            if matches!(self.peek(), Token::Newline | Token::EOF) {
                None
            } else {
                Some(self.parse_expression()?)
            }
        };

        Ok(Statement::Return {
            value,
            location: Location::unknown(),
        })
    }

    /// Parse GOTO statement
    fn parse_goto_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Goto, "Expected 'goto'")?;

        // Get the label name
        let label = if let Token::Identifier(name) = self.peek() {
            name.clone()
        } else {
            return Err(XdlError::ParseError {
                message: "Expected label name after GOTO".to_string(),
                line: 1,
                column: self.current,
            });
        };

        self.advance(); // consume label identifier

        Ok(Statement::Goto {
            label,
            location: Location::unknown(),
        })
    }

    /// Parse CASE statement
    /// CASE expr OF
    ///     value1: statement
    ///     value2: BEGIN ... END
    ///     ELSE: statement
    /// ENDCASE
    fn parse_case_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Case, "Expected 'case'")?;
        let expr = self.parse_expression()?;
        self.consume(Token::Of, "Expected 'of' after case expression")?;

        // Skip newlines after OF
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

        let mut branches = Vec::new();
        let mut else_block = None;

        // Parse case branches until we hit ENDCASE
        while !matches!(self.peek(), Token::Endcase | Token::EOF) {
            // Skip newlines between branches
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }

            if matches!(self.peek(), Token::Endcase) {
                break;
            }

            // Check for ELSE branch
            if matches!(self.peek(), Token::Else) {
                self.advance(); // consume ELSE
                self.consume(Token::Colon, "Expected ':' after ELSE")?;

                // Parse ELSE body
                let body = if matches!(self.peek(), Token::Begin) {
                    self.parse_block_or_statement(&[Token::Endcase])?
                } else {
                    vec![self.parse_statement()?]
                };
                else_block = Some(body);

                // Skip to ENDCASE
                while matches!(self.peek(), Token::Newline) {
                    self.advance();
                }
                break;
            }

            // Parse case value(s) - can be comma-separated
            let mut values = Vec::new();
            loop {
                values.push(self.parse_expression()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance(); // consume comma
                } else {
                    break;
                }
            }

            self.consume(Token::Colon, "Expected ':' after case value")?;

            // Parse the body for this branch
            let body = if matches!(self.peek(), Token::Begin) {
                self.parse_block_or_statement(&[Token::Endcase])?
            } else {
                vec![self.parse_statement()?]
            };

            branches.push(CaseBranch { values, body, location: Location::unknown() });

            // Skip newlines after the statement
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }
        }

        self.consume(Token::Endcase, "Expected 'endcase' to close case statement")?;

        Ok(Statement::Case {
            expr,
            branches,
            else_block,
            location: Location::unknown(),
        })
    }

    /// Parse SWITCH statement (alias for CASE)
    fn parse_switch_statement(&mut self) -> XdlResult<Statement> {
        self.consume(Token::Switch, "Expected 'switch'")?;
        let expr = self.parse_expression()?;
        self.consume(Token::Of, "Expected 'of' after switch expression")?;

        // Skip newlines after OF
        while matches!(self.peek(), Token::Newline) {
            self.advance();
        }

        let mut branches = Vec::new();
        let mut else_block = None;

        // Parse switch branches until we hit ENDSWITCH
        while !matches!(self.peek(), Token::Endswitch | Token::EOF) {
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }

            if matches!(self.peek(), Token::Endswitch) {
                break;
            }

            if matches!(self.peek(), Token::Else) {
                self.advance();
                self.consume(Token::Colon, "Expected ':' after ELSE")?;
                let body = if matches!(self.peek(), Token::Begin) {
                    self.parse_block_or_statement(&[Token::Endswitch])?
                } else {
                    vec![self.parse_statement()?]
                };
                else_block = Some(body);
                while matches!(self.peek(), Token::Newline) {
                    self.advance();
                }
                break;
            }

            let mut values = Vec::new();
            loop {
                values.push(self.parse_expression()?);
                if matches!(self.peek(), Token::Comma) {
                    self.advance();
                } else {
                    break;
                }
            }

            self.consume(Token::Colon, "Expected ':' after case value")?;

            let body = if matches!(self.peek(), Token::Begin) {
                self.parse_block_or_statement(&[Token::Endswitch])?
            } else {
                vec![self.parse_statement()?]
            };

            branches.push(CaseBranch { values, body, location: Location::unknown() });

            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }
        }

        self.consume(Token::Endswitch, "Expected 'endswitch' to close switch statement")?;

        Ok(Statement::Switch {
            expr,
            branches,
            else_block,
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

        // Check if this is a class definition (ends with __define)
        if name.ends_with("__define") {
            return self.parse_class_definition_body(name);
        }

        // Check if this is a method definition (contains ::)
        if name.contains("::") {
            return self.parse_method_definition_body(name, false); // false = procedure
        }

        // Parse parameters and keywords
        let mut params = Vec::new();
        let mut keywords = Vec::new();

        // Check if there's a comma after the procedure name
        if self.check(&Token::Comma) {
            self.advance(); // consume first comma

            // Parse comma-separated parameters and keywords
            loop {
                // Check if we've reached the end of the parameter list
                if matches!(self.peek(), Token::Newline | Token::EOF) {
                    break;
                }

                // Get the parameter/keyword name
                let param_name = if let Token::Identifier(name) = self.peek() {
                    name.clone()
                } else {
                    break; // No more parameters
                };

                self.advance(); // consume identifier

                // Check if this is a keyword (has '=' after it)
                if self.check(&Token::Assign) {
                    self.advance(); // consume '='
                                    // For keyword declarations in procedure definitions,
                                    // we don't parse the default value at definition time
                                    // (IDL doesn't support default values in PRO declarations)
                    keywords.push(KeywordDecl {
                        name: param_name,
                        by_reference: false,
                        location: Location::unknown(),
                    });
                } else {
                    // Regular parameter
                    params.push(Parameter {
                        name: param_name,
                        by_reference: false,
                        optional: false,
                        location: Location::unknown(),
                    });
                }

                // Check for next comma
                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance(); // consume comma
            }
        }

        // Consume any remaining tokens until we hit a newline or start of body
        while matches!(self.peek(), Token::Comma | Token::Newline) {
            self.advance();
        }

        // Parse body
        let mut body = Vec::new();
        // Accept both ENDPRO and END as procedure terminators (IDL compatibility)
        loop {
            // Skip newlines before checking for terminator
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }
            if matches!(self.peek(), Token::Endpro | Token::End | Token::EOF) {
                break;
            }
            body.push(self.parse_statement()?);
        }

        // Consume either ENDPRO or END
        if !matches!(self.peek(), Token::Endpro | Token::End) {
            return Err(XdlError::ParseError {
                message: "Expected 'END' or 'ENDPRO' to close procedure".to_string(),
                line: 1,
                column: self.current,
            });
        }
        self.advance(); // consume ENDPRO or END

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

        // Check if this is a method definition (contains ::)
        if name.contains("::") {
            return self.parse_method_definition_body(name, true); // true = function
        }

        // Parse parameters and keywords
        let mut params = Vec::new();
        let mut keywords = Vec::new();

        // Check if there's an opening paren or comma after the function name
        let has_paren = self.check(&Token::LeftParen);
        if has_paren {
            self.advance(); // consume '('
        }

        // Check if there's a comma (for comma-separated params without parens)
        if has_paren || self.check(&Token::Comma) {
            if self.check(&Token::Comma) {
                self.advance(); // consume first comma
            }

            // Parse comma-separated parameters and keywords
            loop {
                // Check if we've reached the end of the parameter list
                if has_paren && self.check(&Token::RightParen) {
                    self.advance(); // consume ')'
                    break;
                }
                if !has_paren && matches!(self.peek(), Token::Newline | Token::EOF) {
                    break;
                }

                // Get the parameter/keyword name
                let param_name = if let Token::Identifier(name) = self.peek() {
                    name.clone()
                } else {
                    break; // No more parameters
                };

                self.advance(); // consume identifier

                // Check if this is a keyword (has '=' after it)
                if self.check(&Token::Assign) {
                    self.advance(); // consume '='
                    keywords.push(KeywordDecl {
                        name: param_name,
                        by_reference: false,
                        location: Location::unknown(),
                    });
                } else {
                    // Regular parameter
                    params.push(Parameter {
                        name: param_name,
                        by_reference: false,
                        optional: false,
                        location: Location::unknown(),
                    });
                }

                // Check for next comma
                if !self.check(&Token::Comma) {
                    if has_paren && self.check(&Token::RightParen) {
                        self.advance(); // consume ')'
                    }
                    break;
                }
                self.advance(); // consume comma
            }
        }

        // Consume any remaining tokens until we hit a newline or start of body
        while matches!(self.peek(), Token::Comma | Token::Newline) {
            self.advance();
        }

        // Parse body
        let mut body = Vec::new();
        // Accept both ENDFUNCTION and END as function terminators (IDL compatibility)
        loop {
            // Skip newlines before checking for terminator
            while matches!(self.peek(), Token::Newline) {
                self.advance();
            }
            if matches!(self.peek(), Token::Endfunction | Token::End | Token::EOF) {
                break;
            }
            body.push(self.parse_statement()?);
        }

        // Consume either ENDFUNCTION or END
        if !matches!(self.peek(), Token::Endfunction | Token::End) {
            return Err(XdlError::ParseError {
                message: "Expected 'END' or 'ENDFUNCTION' to close function".to_string(),
                line: 1,
                column: self.current,
            });
        }
        self.advance(); // consume ENDFUNCTION or END

        Ok(Statement::FunctionDef {
            name,
            params,
            keywords,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse class definition body (PRO ClassName__define)
    fn parse_class_definition_body(&mut self, full_name: String) -> XdlResult<Statement> {
        // Extract class name by removing __define suffix
        let class_name = full_name.trim_end_matches("__define").to_string();

        // Skip parameters/keywords if any (usually class definitions don't have params)
        while matches!(self.peek(), Token::Comma) {
            self.advance();
            if matches!(self.peek(), Token::Identifier(_)) {
                self.advance(); // skip parameter
            }
        }

        // Skip until newline
        while matches!(self.peek(), Token::Comma | Token::Newline) {
            self.advance();
        }

        // Parse body until ENDPRO
        let mut body = Vec::new();
        while !matches!(self.peek(), Token::Endpro | Token::EOF) {
            body.push(self.parse_statement()?);
        }

        self.consume(Token::Endpro, "Expected 'endpro' to close class definition")?;

        Ok(Statement::ClassDefinition {
            name: class_name,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse method definition body (PRO/FUNCTION ClassName::MethodName)
    fn parse_method_definition_body(
        &mut self,
        full_name: String,
        is_function: bool,
    ) -> XdlResult<Statement> {
        // Split on :: to get class name and method name
        let parts: Vec<&str> = full_name.split("::").collect();
        if parts.len() != 2 {
            return Err(XdlError::ParseError {
                message: format!(
                    "Invalid method name format '{}'. Expected ClassName::MethodName",
                    full_name
                ),
                line: 1,
                column: self.current,
            });
        }

        let class_name = parts[0].to_string();
        let method_name = parts[1].to_string();

        // Parse parameters and keywords (same as regular procedure/function)
        let mut params = Vec::new();
        let mut keywords = Vec::new();

        if self.check(&Token::Comma) {
            self.advance(); // consume first comma

            loop {
                if matches!(self.peek(), Token::Newline | Token::EOF) {
                    break;
                }

                let param_name = if let Token::Identifier(name) = self.peek() {
                    name.clone()
                } else {
                    break;
                };

                self.advance();

                if self.check(&Token::Assign) {
                    self.advance(); // consume '='
                    keywords.push(KeywordDecl {
                        name: param_name,
                        by_reference: false,
                        location: Location::unknown(),
                    });
                } else {
                    params.push(Parameter {
                        name: param_name,
                        by_reference: false,
                        optional: false,
                        location: Location::unknown(),
                    });
                }

                if !self.check(&Token::Comma) {
                    break;
                }
                self.advance();
            }
        }

        // Skip remaining tokens until body
        while matches!(self.peek(), Token::Comma | Token::Newline) {
            self.advance();
        }

        // Parse body
        let mut body = Vec::new();

        while !self.is_at_end() {
            // Check for end token
            if is_function && self.check(&Token::Endfunction) {
                break;
            }
            if !is_function && self.check(&Token::Endpro) {
                break;
            }

            body.push(self.parse_statement()?);
        }

        // Consume the appropriate end token
        if is_function {
            self.consume(
                Token::Endfunction,
                "Expected 'endfunction' to close method definition",
            )?;
        } else {
            self.consume(
                Token::Endpro,
                "Expected 'endpro' to close method definition",
            )?;
        }

        Ok(Statement::MethodDefinition {
            class_name,
            method_name,
            is_function,
            params,
            keywords,
            body,
            location: Location::unknown(),
        })
    }

    /// Parse expression with precedence
    fn parse_expression(&mut self) -> XdlResult<Expression> {
        self.parse_ternary()
    }

    /// Parse ternary operator (condition ? if_true : if_false)
    fn parse_ternary(&mut self) -> XdlResult<Expression> {
        let condition = self.parse_logical_or()?;

        // Check for ternary operator
        if self.check(&Token::QuestionMark) {
            self.advance(); // consume '?'
            let if_true = self.parse_expression()?;
            self.consume(Token::Colon, "Expected ':' in ternary expression")?;
            let if_false = self.parse_expression()?;

            Ok(Expression::Ternary {
                condition: Box::new(condition),
                if_true: Box::new(if_true),
                if_false: Box::new(if_false),
                location: Location::unknown(),
            })
        } else {
            Ok(condition)
        }
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

        // Handle postfix operations like array indexing, method calls, and field access
        loop {
            if self.check(&Token::LeftBracket) {
                // Array indexing: expr[index]
                self.advance(); // consume '['
                let indices = self.parse_array_indices()?;
                self.consume(Token::RightBracket, "Expected ']' after array indices")?;

                expr = Expression::ArrayRef {
                    array: Box::new(expr),
                    indices,
                    location: Location::unknown(),
                };
            } else if self.check(&Token::Arrow) {
                // Method call: expr->method(args)
                self.advance(); // consume '->'

                // Get method name
                let method = match self.advance() {
                    Token::Identifier(name) => name.clone(),
                    _ => {
                        return Err(XdlError::ParseError {
                            message: "Expected method name after '->'".to_string(),
                            line: 1, // TODO: track line numbers
                            column: self.current,
                        });
                    }
                };

                // Check if method has arguments
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

                    self.consume(Token::RightParen, "Expected ')' after method arguments")?;

                    expr = Expression::MethodCall {
                        object: Box::new(expr),
                        method,
                        args,
                        keywords: Vec::new(), // TODO: implement keyword arguments
                        location: Location::unknown(),
                    };
                } else {
                    // Method call without parentheses (treat as property access that returns a value)
                    expr = Expression::MethodCall {
                        object: Box::new(expr),
                        method,
                        args: vec![],
                        keywords: vec![],
                        location: Location::unknown(),
                    };
                }
            } else if self.check(&Token::Dot) {
                // Struct field access: expr.field
                self.advance(); // consume '.'

                // Get field name
                let field = match self.advance() {
                    Token::Identifier(name) => name.clone(),
                    _ => {
                        return Err(XdlError::ParseError {
                            message: "Expected field name after '.'".to_string(),
                            line: 1, // TODO: track line numbers
                            column: self.current,
                        });
                    }
                };

                expr = Expression::StructRef {
                    object: Box::new(expr),
                    field,
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
            // Check for wildcard * (means all elements)
            if self.check(&Token::Multiply) {
                self.advance(); // consume '*'
                indices.push(ArrayIndex::All);
            } else if self.check(&Token::Colon) {
                // Range with leading colon (e.g., [:5])
                self.advance(); // consume ':'

                // Check for * wildcard as end
                let end = if self.check(&Token::Multiply) {
                    self.advance(); // consume '*'
                    None // * means "to end"
                } else if self.check(&Token::RightBracket)
                    || self.check(&Token::Comma)
                    || self.check(&Token::Colon)
                {
                    None
                } else {
                    Some(self.parse_expression()?)
                };

                // Check for step (e.g., [:*:2] or [:10:2])
                let step = if self.check(&Token::Colon) {
                    self.advance();
                    if self.check(&Token::RightBracket) || self.check(&Token::Comma) {
                        None
                    } else {
                        Some(self.parse_expression()?)
                    }
                } else {
                    None
                };

                indices.push(ArrayIndex::Range {
                    start: None,
                    end: end.map(Box::new),
                    step: step.map(Box::new),
                });
            } else {
                // Parse first expression
                let first_expr = self.parse_expression()?;

                // Check if this is a range
                if self.check(&Token::Colon) {
                    self.advance(); // consume ':'

                    // Check for * wildcard as end
                    let end = if self.check(&Token::Multiply) {
                        self.advance(); // consume '*'
                        None // * means "to end"
                    } else if self.check(&Token::RightBracket)
                        || self.check(&Token::Comma)
                        || self.check(&Token::Colon)
                    {
                        None
                    } else {
                        Some(self.parse_expression()?)
                    };

                    // Check for step (e.g., 0:*:2 or 0:10:2)
                    let step = if self.check(&Token::Colon) {
                        self.advance();
                        if self.check(&Token::RightBracket) || self.check(&Token::Comma) {
                            None
                        } else {
                            Some(self.parse_expression()?)
                        }
                    } else {
                        None
                    };

                    indices.push(ArrayIndex::Range {
                        start: Some(Box::new(first_expr)),
                        end: end.map(Box::new),
                        step: step.map(Box::new),
                    });
                } else {
                    // Single index
                    indices.push(ArrayIndex::Single(Box::new(first_expr)));
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
                    let mut keywords = Vec::new();

                    if !self.check(&Token::RightParen) {
                        loop {
                            // Check for /FLAG keyword (e.g., /INDEX)
                            if self.check(&Token::Divide) {
                                self.advance(); // consume '/'
                                if let Token::Identifier(kw_name) = self.peek() {
                                    let kw_name = kw_name.clone();
                                    self.advance(); // consume keyword name
                                    keywords.push(Keyword {
                                        name: kw_name,
                                        value: Some(Expression::Literal {
                                            value: XdlValue::Long(1),
                                            location: Location::unknown(),
                                        }),
                                        location: Location::unknown(),
                                    });
                                    if self.check(&Token::Comma) {
                                        self.advance();
                                        continue;
                                    } else {
                                        break;
                                    }
                                }
                            }

                            // Check for keyword argument (identifier = expression)
                            if let Token::Identifier(kw_name) = self.peek() {
                                let kw_name_clone = kw_name.clone();
                                let next_pos = self.current + 1;

                                if next_pos < self.tokens.len()
                                    && matches!(self.tokens[next_pos], Token::Assign)
                                {
                                    // This is a keyword argument
                                    self.advance(); // consume identifier
                                    self.advance(); // consume '='
                                    let value = self.parse_expression()?;
                                    keywords.push(Keyword {
                                        name: kw_name_clone,
                                        value: Some(value),
                                        location: Location::unknown(),
                                    });
                                    if self.check(&Token::Comma) {
                                        self.advance();
                                        continue;
                                    } else {
                                        break;
                                    }
                                }
                            }

                            // Regular positional argument
                            args.push(self.parse_expression()?);
                            if self.check(&Token::Comma) {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }

                    self.consume(Token::RightParen, "Expected ')' after function arguments")?;

                    // Check if this is OBJ_NEW
                    if name.to_uppercase() == "OBJ_NEW" {
                        // First argument should be the class name (string literal)
                        let class_name = if !args.is_empty() {
                            match &args[0] {
                                Expression::Literal {
                                    value: XdlValue::String(s),
                                    ..
                                } => s.clone(),
                                _ => {
                                    // If not a string literal, we'll handle this at runtime
                                    return Err(XdlError::ParseError {
                                        message: "OBJ_NEW requires a string literal class name as first argument".to_string(),
                                        line: 1,
                                        column: self.current,
                                    });
                                }
                            }
                        } else {
                            // Empty OBJ_NEW() returns NULL object
                            String::new()
                        };

                        // Remaining arguments are constructor arguments
                        let constructor_args = if args.len() > 1 {
                            args[1..].to_vec()
                        } else {
                            Vec::new()
                        };

                        Ok(Expression::ObjectNew {
                            class_name,
                            args: constructor_args,
                            keywords: keywords.clone(), // Pass through keywords
                            location: Location::unknown(),
                        })
                    } else {
                        Ok(Expression::FunctionCall {
                            name,
                            args,
                            keywords, // Use parsed keywords
                            location: Location::unknown(),
                        })
                    }
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
