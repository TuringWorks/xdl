//! MATLAB to XDL Transpiler
//!
//! Converts MATLAB syntax to XDL-compatible code

use crate::function_map::get_xdl_function;
use crate::lexer::{Lexer, Token, TokenKind};
use std::collections::HashSet;

pub struct Transpiler {
    tokens: Vec<Token>,
    position: usize,
    output: String,
    indent_level: usize,
    // Subplot state for tiledlayout
    subplot_rows: usize,
    subplot_cols: usize,
    current_tile: usize,
    scope_start: usize,
    function_parameters: HashSet<String>,
    function_output: Option<String>,
}

impl Transpiler {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            position: 0,
            output: String::new(),
            indent_level: 0,
            subplot_rows: 1,
            subplot_cols: 1,
            current_tile: 0,
            scope_start: 0,
            function_parameters: HashSet::new(),
            function_output: None,
        }
    }

    fn current_token(&self) -> Token {
        self.tokens.get(self.position).cloned().unwrap_or(Token {
            kind: TokenKind::EOF,
            lexeme: String::new(),
            line: 0,
            column: 0,
        })
    }

    fn advance(&mut self) {
        if self.position < self.tokens.len() {
            self.position += 1;
        }
    }

    fn is_assigned_before(&self, name: &str, position: usize) -> bool {
        self.function_parameters.contains(name)
            || self.tokens[self.scope_start.min(position)..position.min(self.tokens.len())]
                .windows(2)
                .any(|pair| {
                    matches!(&pair[0].kind, TokenKind::Identifier(candidate) if candidate == name)
                        && matches!(pair[1].kind, TokenKind::Assign)
                })
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_line(&mut self, s: &str) {
        self.emit(&"  ".repeat(self.indent_level));
        self.emit(s);
        self.emit("\n");
    }

    pub fn transpile(&mut self) -> Result<String, String> {
        while self.position < self.tokens.len() {
            let token = self.current_token();

            match &token.kind {
                TokenKind::Comment(c) => {
                    self.emit_line(&format!("; {}", c));
                    self.advance();
                }
                TokenKind::Function => {
                    self.transpile_function()?;
                }
                TokenKind::For => {
                    self.transpile_for_loop()?;
                }
                TokenKind::While => {
                    self.transpile_while_loop()?;
                }
                TokenKind::If => {
                    self.transpile_if_statement()?;
                }
                TokenKind::Switch => {
                    self.transpile_switch_statement()?;
                }
                TokenKind::Try => {
                    self.transpile_try_catch()?;
                }
                TokenKind::Break => {
                    self.emit_line("BREAK");
                    self.advance();
                    // Skip optional semicolon/newline
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Semicolon | TokenKind::Newline
                    ) {
                        self.advance();
                    }
                }
                TokenKind::Continue => {
                    self.emit_line("CONTINUE");
                    self.advance();
                    // Skip optional semicolon/newline
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Semicolon | TokenKind::Newline
                    ) {
                        self.advance();
                    }
                }
                TokenKind::Return => {
                    self.emit_line("RETURN");
                    self.advance();
                    // Skip optional semicolon/newline
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Semicolon | TokenKind::Newline
                    ) {
                        self.advance();
                    }
                }
                TokenKind::Identifier(_) | TokenKind::LeftBracket => {
                    self.transpile_statement()?;
                }
                TokenKind::Newline | TokenKind::Semicolon => {
                    self.advance();
                }
                TokenKind::EOF => break,
                _ => {
                    self.advance();
                }
            }
        }

        Ok(self.output.clone())
    }

    fn transpile_function(&mut self) -> Result<(), String> {
        self.advance(); // skip 'function'

        // Parse output variables [out1, out2, ...] = funcname(...)
        // or just funcname(...)
        let mut outputs = Vec::new();

        // Check for output variables
        if matches!(self.current_token().kind, TokenKind::LeftBracket) {
            self.advance(); // skip '['
            while !matches!(self.current_token().kind, TokenKind::RightBracket) {
                if let TokenKind::Identifier(name) = &self.current_token().kind {
                    outputs.push(name.clone());
                    self.advance();
                }
                if matches!(self.current_token().kind, TokenKind::Comma) {
                    self.advance();
                }
            }
            self.advance(); // skip ']'

            // Expect '='
            if matches!(self.current_token().kind, TokenKind::Assign) {
                self.advance();
            }
        } else if let TokenKind::Identifier(name) = &self.current_token().kind {
            // Check if next is '=' (single output)
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len()
                && matches!(self.tokens[next_pos].kind, TokenKind::Assign)
            {
                outputs.push(name.clone());
                self.advance(); // skip output name
                self.advance(); // skip '='
            }
        }

        if outputs.len() > 1 {
            return Err(
                "MATLAB functions with multiple output variables are not supported yet".to_string(),
            );
        }

        // Get function name
        let func_name = if let TokenKind::Identifier(name) = &self.current_token().kind {
            name.clone()
        } else {
            return Err("Expected function name".to_string());
        };
        self.advance();

        // Get parameters
        let mut params = Vec::new();
        if matches!(self.current_token().kind, TokenKind::LeftParen) {
            self.advance(); // skip '('
            while !matches!(self.current_token().kind, TokenKind::RightParen) {
                if let TokenKind::Identifier(name) = &self.current_token().kind {
                    params.push(name.clone());
                    self.advance();
                }
                if matches!(self.current_token().kind, TokenKind::Comma) {
                    self.advance();
                }
            }
            self.advance(); // skip ')'
        }

        let param_names = params.clone();
        let previous_scope_start = self.scope_start;
        let previous_parameters =
            std::mem::replace(&mut self.function_parameters, params.into_iter().collect());
        let previous_output =
            std::mem::replace(&mut self.function_output, outputs.first().cloned());
        self.scope_start = self.position;

        // Emit XDL function
        let signature = if self.function_parameters.is_empty() {
            format!("FUNCTION {}", func_name)
        } else {
            format!("FUNCTION {}, {}", func_name, param_names.join(", "))
        };
        self.emit_line(&signature);
        self.indent_level += 1;

        // Skip to end
        let mut depth = 1;
        while depth > 0 && self.position < self.tokens.len() {
            match &self.current_token().kind {
                TokenKind::End => {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                TokenKind::Function
                | TokenKind::For
                | TokenKind::While
                | TokenKind::If
                | TokenKind::Switch
                | TokenKind::Try => {
                    depth += 1;
                }
                _ => {}
            }

            if depth > 0 {
                self.transpile_statement()?;
            }
        }

        if let Some(output) = &self.function_output {
            self.emit_line(&format!("RETURN, {output}"));
        }
        self.indent_level -= 1;
        self.emit_line("END");
        self.scope_start = previous_scope_start;
        self.function_parameters = previous_parameters;
        self.function_output = previous_output;

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_for_loop(&mut self) -> Result<(), String> {
        self.advance(); // skip 'for'

        // Get loop variable
        let var_name = if let TokenKind::Identifier(name) = &self.current_token().kind {
            name.clone()
        } else {
            return Err("Expected loop variable".to_string());
        };
        self.advance();

        // Expect '='
        if !matches!(self.current_token().kind, TokenKind::Assign) {
            return Err("Expected '=' in for loop".to_string());
        }
        self.advance();

        // Get range (start:end or start:step:end or array)
        let range_expr = self.collect_expression_until_newline()?;

        // Check if this is a simple range expression with colons
        let xdl_range = if range_expr.contains(':') && !range_expr.contains('(') {
            // Simple range: try to convert
            match self.convert_range(&range_expr) {
                Ok(r) => r,
                Err(_) => {
                    // Complex range expression, use convert_range_to_findgen
                    self.convert_range_to_findgen(&range_expr)
                }
            }
        } else {
            // Not a simple range, output as-is
            range_expr
        };

        self.emit_line(&format!("for {} = {}", var_name, xdl_range));
        self.indent_level += 1;

        // Process body until 'end'
        while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
            self.transpile_statement()?;
        }

        self.indent_level -= 1;
        self.emit_line("endfor");

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_while_loop(&mut self) -> Result<(), String> {
        self.advance(); // skip 'while'

        let condition = self.collect_expression_until_newline()?;

        self.emit_line(&format!("while {}", condition));
        self.indent_level += 1;

        while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
            self.transpile_statement()?;
        }

        self.indent_level -= 1;
        self.emit_line("endwhile");

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_if_statement(&mut self) -> Result<(), String> {
        self.advance(); // skip 'if'

        let condition = self.collect_expression_until_newline()?;

        self.emit_line(&format!("if {} then", condition));
        self.indent_level += 1;

        while !matches!(
            self.current_token().kind,
            TokenKind::End | TokenKind::Else | TokenKind::Elseif | TokenKind::EOF
        ) {
            self.transpile_statement()?;
        }

        if matches!(self.current_token().kind, TokenKind::Else) {
            self.indent_level -= 1;
            self.emit_line("else");
            self.indent_level += 1;
            self.advance();

            while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
                self.transpile_statement()?;
            }
        }

        self.indent_level -= 1;
        self.emit_line("endif");

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_switch_statement(&mut self) -> Result<(), String> {
        self.advance(); // skip 'switch'

        // Get switch expression
        let switch_expr = self.collect_expression_until_newline()?;

        self.emit_line(&format!("CASE {} OF", switch_expr));
        self.indent_level += 1;

        // Process case statements
        while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
            match &self.current_token().kind {
                TokenKind::Case => {
                    self.advance(); // skip 'case'

                    // Collect case value(s)
                    let mut case_values = Vec::new();

                    // Check if it's a cell array {val1, val2, ...}
                    if matches!(self.current_token().kind, TokenKind::LeftBrace) {
                        self.advance(); // skip '{'
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightBrace | TokenKind::EOF
                        ) {
                            let mut value = String::new();
                            while !matches!(
                                self.current_token().kind,
                                TokenKind::Comma
                                    | TokenKind::RightBrace
                                    | TokenKind::Newline
                                    | TokenKind::EOF
                            ) {
                                value.push_str(&self.current_token().lexeme);
                                self.advance();
                            }
                            if !value.trim().is_empty() {
                                case_values.push(value.trim().to_string());
                            }
                            if matches!(self.current_token().kind, TokenKind::Comma) {
                                self.advance();
                            }
                        }
                        if matches!(self.current_token().kind, TokenKind::RightBrace) {
                            self.advance();
                        }
                    } else {
                        // Single case value
                        let mut value = String::new();
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                        ) {
                            value.push_str(&self.current_token().lexeme);
                            self.advance();
                        }
                        case_values.push(value.trim().to_string());
                    }

                    // Skip newline after case
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }

                    // Emit case values (XDL supports multiple values with commas)
                    for val in case_values.iter() {
                        self.emit_line(&format!("{}: BEGIN", val));
                    }
                    self.indent_level += 1;

                    // Process statements until next case/otherwise/end
                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Case | TokenKind::Otherwise | TokenKind::End | TokenKind::EOF
                    ) {
                        self.transpile_statement()?;
                    }

                    self.indent_level -= 1;
                    self.emit_line("END");
                }
                TokenKind::Otherwise => {
                    self.advance(); // skip 'otherwise'

                    // Skip newline
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }

                    self.emit_line("ELSE: BEGIN");
                    self.indent_level += 1;

                    // Process statements until end
                    while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
                        self.transpile_statement()?;
                    }

                    self.indent_level -= 1;
                    self.emit_line("END");
                }
                _ => {
                    self.advance();
                }
            }
        }

        self.indent_level -= 1;
        self.emit_line("ENDCASE");

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_try_catch(&mut self) -> Result<(), String> {
        self.advance(); // skip 'try'

        // XDL doesn't have direct try/catch, so we'll emit comments and the code
        self.emit_line("; TRY block (error handling not directly supported in XDL)");
        self.emit_line("BEGIN");
        self.indent_level += 1;

        // Process try block
        while !matches!(
            self.current_token().kind,
            TokenKind::Catch | TokenKind::End | TokenKind::EOF
        ) {
            self.transpile_statement()?;
        }

        self.indent_level -= 1;
        self.emit_line("END");

        // Handle catch block if present
        if matches!(self.current_token().kind, TokenKind::Catch) {
            self.advance(); // skip 'catch'

            // Skip optional error variable
            if let TokenKind::Identifier(_err_var) = &self.current_token().kind {
                self.advance();
            }

            // Skip newline
            if matches!(
                self.current_token().kind,
                TokenKind::Newline | TokenKind::Semicolon
            ) {
                self.advance();
            }

            self.emit_line("; CATCH block (error handling not directly supported in XDL)");
            self.emit_line("BEGIN");
            self.indent_level += 1;

            // Process catch block
            while !matches!(self.current_token().kind, TokenKind::End | TokenKind::EOF) {
                self.transpile_statement()?;
            }

            self.indent_level -= 1;
            self.emit_line("END");
        }

        if matches!(self.current_token().kind, TokenKind::End) {
            self.advance();
        }

        Ok(())
    }

    fn transpile_statement(&mut self) -> Result<(), String> {
        // Handle control flow statements that might appear nested
        match &self.current_token().kind {
            TokenKind::For => return self.transpile_for_loop(),
            TokenKind::While => return self.transpile_while_loop(),
            TokenKind::If => return self.transpile_if_statement(),
            TokenKind::Switch => return self.transpile_switch_statement(),
            TokenKind::Try => return self.transpile_try_catch(),
            TokenKind::Break => {
                self.emit_line("BREAK");
                self.advance();
                if matches!(
                    self.current_token().kind,
                    TokenKind::Semicolon | TokenKind::Newline
                ) {
                    self.advance();
                }
                return Ok(());
            }
            TokenKind::Continue => {
                self.emit_line("CONTINUE");
                self.advance();
                if matches!(
                    self.current_token().kind,
                    TokenKind::Semicolon | TokenKind::Newline
                ) {
                    self.advance();
                }
                return Ok(());
            }
            TokenKind::Return => {
                if let Some(output) = &self.function_output {
                    self.emit_line(&format!("RETURN, {output}"));
                } else {
                    self.emit_line("RETURN");
                }
                self.advance();
                if matches!(
                    self.current_token().kind,
                    TokenKind::Semicolon | TokenKind::Newline
                ) {
                    self.advance();
                }
                return Ok(());
            }
            _ => {}
        }

        // Check if this is a graphics command that should be handled specially
        if let TokenKind::Identifier(name) = &self.current_token().kind {
            match name.as_str() {
                "figure" | "clf" | "close" => {
                    // Ignore figure management commands
                    self.advance();
                    // Skip any arguments
                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }
                    self.emit_line("; (figure management command ignored)");
                    return Ok(());
                }
                "hold" => {
                    // Ignore hold on/off commands
                    self.advance();
                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }
                    self.emit_line("; (hold command ignored - XDL doesn't support hold on/off)");
                    return Ok(());
                }
                "surf" | "mesh" | "surfc" | "meshc" => {
                    // Convert MATLAB 3D surface plots to XDL SURFACE/CONTOUR
                    let func_name = name.clone();
                    self.advance(); // skip function name

                    if matches!(self.current_token().kind, TokenKind::LeftParen) {
                        self.advance(); // skip '('

                        // Collect arguments: surf(X, Y, Z) or surf(Z)
                        let mut args = Vec::new();
                        let mut current_arg = String::new();
                        let mut paren_depth = 0;

                        while !matches!(self.current_token().kind, TokenKind::EOF) {
                            if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                paren_depth += 1;
                                current_arg.push_str(&self.current_token().lexeme);
                            } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                                if paren_depth > 0 {
                                    paren_depth -= 1;
                                    current_arg.push_str(&self.current_token().lexeme);
                                } else {
                                    // End of function call
                                    if !current_arg.trim().is_empty() {
                                        args.push(current_arg.trim().to_string());
                                    }
                                    break;
                                }
                            } else if matches!(self.current_token().kind, TokenKind::Comma)
                                && paren_depth == 0
                            {
                                if !current_arg.trim().is_empty() {
                                    args.push(current_arg.trim().to_string());
                                }
                                current_arg.clear();
                            } else {
                                current_arg.push_str(&self.current_token().lexeme);
                            }
                            self.advance();
                        }

                        // Convert to XDL SURFACE command
                        // surf(X, Y, Z) -> SURFACE, Z
                        // surf(Z) -> SURFACE, Z
                        let z_data = if args.len() >= 3 {
                            &args[2] // surf(X, Y, Z) - use Z
                        } else if args.len() == 1 {
                            &args[0] // surf(Z) - use Z
                        } else {
                            // Fallback
                            "data"
                        };

                        self.emit_line(&format!("; {} - 3D surface plotting", func_name));
                        self.emit_line(&format!("SURFACE, {}", z_data));

                        // Skip to end of statement
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                        ) {
                            self.advance();
                        }
                        if matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon
                        ) {
                            self.advance();
                        }

                        return Ok(());
                    }

                    // If we couldn't parse it, emit comment
                    self.emit_line(&format!("; {} - Could not convert to XDL", func_name));
                    return Ok(());
                }
                "tiledlayout" => {
                    // tiledlayout(rows, cols) - set up subplot grid
                    self.advance(); // skip 'tiledlayout'
                    if matches!(self.current_token().kind, TokenKind::LeftParen) {
                        self.advance(); // skip '('

                        // Parse rows and cols
                        if let TokenKind::Number(rows) = &self.current_token().kind {
                            self.subplot_rows = *rows as usize;
                            self.advance();
                        }
                        if matches!(self.current_token().kind, TokenKind::Comma) {
                            self.advance();
                        }
                        if let TokenKind::Number(cols) = &self.current_token().kind {
                            self.subplot_cols = *cols as usize;
                            self.advance();
                        }
                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance();
                        }
                    }

                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }

                    self.current_tile = 0; // Reset tile counter
                    self.emit_line(&format!(
                        "; tiledlayout({}, {}) - creating {} subplots",
                        self.subplot_rows,
                        self.subplot_cols,
                        self.subplot_rows * self.subplot_cols
                    ));
                    return Ok(());
                }
                _ => {}
            }
        }

        // Check for "ax = nexttile" pattern before normal statement processing
        if let TokenKind::Identifier(_var_name) = &self.current_token().kind {
            let next_pos = self.position + 1;
            if next_pos < self.tokens.len()
                && matches!(self.tokens[next_pos].kind, TokenKind::Assign)
            {
                let after_assign = self.position + 2;
                if after_assign < self.tokens.len() {
                    if let TokenKind::Identifier(func_name) = &self.tokens[after_assign].kind {
                        if func_name == "nexttile" {
                            // This is "ax = nexttile", handle specially
                            self.current_tile += 1;
                            // Skip past the assignment
                            self.advance(); // skip variable name
                            self.advance(); // skip '='
                            self.advance(); // skip 'nexttile'

                            // Skip any remaining tokens on this line
                            while !matches!(
                                self.current_token().kind,
                                TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                            ) {
                                self.advance();
                            }
                            if matches!(
                                self.current_token().kind,
                                TokenKind::Newline | TokenKind::Semicolon
                            ) {
                                self.advance();
                            }

                            self.emit_line(&format!(
                                "; ax = nexttile - now plotting to tile {}",
                                self.current_tile
                            ));
                            return Ok(());
                        }
                    }
                }
            }
        }

        // Check if statement starts with nexttile
        if let TokenKind::Identifier(name) = &self.current_token().kind {
            if name == "nexttile" {
                // nexttile or ax = nexttile - move to next tile
                self.current_tile += 1;
                self.advance();
                while !matches!(
                    self.current_token().kind,
                    TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                ) {
                    self.advance();
                }
                if matches!(
                    self.current_token().kind,
                    TokenKind::Newline | TokenKind::Semicolon
                ) {
                    self.advance();
                }
                self.emit_line(&format!(
                    "; nexttile - now plotting to tile {}",
                    self.current_tile
                ));
                return Ok(());
            }

            match name.as_str() {
                "comet3" | "comet" | "plot3" => {
                    // Map 3D plot commands to PLOT3D with tile-specific filename
                    self.advance(); // skip command name

                    // Skip axis handle if present: comet3(ax, ...)
                    if matches!(self.current_token().kind, TokenKind::LeftParen) {
                        self.advance(); // skip '('

                        // Check if first arg looks like an axis handle
                        if let TokenKind::Identifier(id) = &self.current_token().kind {
                            if id.starts_with("ax") {
                                self.advance(); // skip axis handle
                                if matches!(self.current_token().kind, TokenKind::Comma) {
                                    self.advance(); // skip comma
                                }
                            }
                        }

                        // Collect remaining arguments (x, y, z or x, y)
                        let mut args = Vec::new();
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            if let TokenKind::Identifier(arg) = &self.current_token().kind {
                                args.push(arg.clone());
                            }
                            self.advance();
                            if matches!(self.current_token().kind, TokenKind::Comma) {
                                self.advance();
                            }
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance();
                        }

                        // Generate PLOT3D command with tile-specific filename
                        let filename = if self.current_tile > 0 {
                            format!("'tile{}_plot.png'", self.current_tile)
                        } else {
                            "'xdl_plot.png'".to_string()
                        };

                        if args.len() >= 3 {
                            self.emit_line(&format!(
                                "PLOT3D, {}, {}, {}, filename={}",
                                args[0], args[1], args[2], filename
                            ));
                        } else if args.len() == 2 {
                            self.emit_line(&format!(
                                "PLOT, {}, {}, filename={}",
                                args[0], args[1], filename
                            ));
                        }
                    }

                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }
                    return Ok(());
                }
                "xlabel" | "ylabel" | "title" => {
                    let command = name.clone();
                    self.advance();
                    if !matches!(self.current_token().kind, TokenKind::LeftParen) {
                        return Err(format!(
                            "MATLAB {command} requires a quoted string argument"
                        ));
                    }
                    self.advance();
                    let label = match self.current_token().kind.clone() {
                        TokenKind::String(value) => value,
                        _ => {
                            return Err(format!(
                                "MATLAB {command} currently supports literal strings only"
                            ))
                        }
                    };
                    self.advance();
                    if !matches!(self.current_token().kind, TokenKind::RightParen) {
                        return Err(format!(
                            "MATLAB {command} currently supports one string argument"
                        ));
                    }
                    self.advance();
                    while matches!(self.current_token().kind, TokenKind::Semicolon) {
                        self.advance();
                    }

                    let keyword = match command.as_str() {
                        "title" => "TITLE",
                        "xlabel" => "XTITLE",
                        "ylabel" => "YTITLE",
                        _ => {
                            return Err(format!("unsupported MATLAB plot label command: {command}"))
                        }
                    };
                    let line_end = self.output.trim_end_matches('\n').len();
                    let line_start = self.output[..line_end].rfind('\n').map_or(0, |i| i + 1);
                    let last_line = self.output[line_start..line_end].trim_start();
                    if !last_line.to_ascii_uppercase().starts_with("PLOT,") {
                        return Err(format!(
                            "MATLAB {command} must immediately follow a supported plot(x, y) call"
                        ));
                    }
                    self.output.insert_str(
                        line_end,
                        &format!(", {keyword}='{}'", label.replace('\'', "''")),
                    );
                    return Ok(());
                }
                "legend" | "grid" | "zlabel" => {
                    return Err(format!("MATLAB {name} is not supported by the current single-plot transpilation subset"));
                }
                "axis" => {
                    // axis equal, axis([xmin xmax ymin ymax]), etc. - ignore for now
                    self.advance();
                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }
                    if matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon
                    ) {
                        self.advance();
                    }
                    self.emit_line("; (axis command - XDL uses automatic axis scaling)");
                    return Ok(());
                }
                _ => {}
            }
        }

        // Check for meshgrid pattern: [X, Y] = meshgrid(...)
        if matches!(self.current_token().kind, TokenKind::LeftBracket) {
            // Look ahead to see if this is [id, id] = meshgrid(...)
            let saved_pos = self.position;
            let mut is_meshgrid = false;
            let mut is_matlab_size = false;
            let mut is_matlab_find = false;
            let mut output_vars = Vec::new();

            self.advance(); // skip '['
                            // Collect output variable names
            while !matches!(
                self.current_token().kind,
                TokenKind::RightBracket | TokenKind::EOF
            ) {
                if let TokenKind::Identifier(name) = &self.current_token().kind {
                    output_vars.push(name.clone());
                }
                self.advance();
            }
            if matches!(self.current_token().kind, TokenKind::RightBracket) {
                self.advance(); // skip ']'
            }
            if matches!(self.current_token().kind, TokenKind::Assign) {
                self.advance(); // skip '='
                if let TokenKind::Identifier(fname) = &self.current_token().kind {
                    if fname == "meshgrid" {
                        is_meshgrid = true;
                    } else if fname.eq_ignore_ascii_case("size") {
                        is_matlab_size = true;
                    } else if fname.eq_ignore_ascii_case("find") {
                        is_matlab_find = true;
                    }
                }
            }

            if is_matlab_find && output_vars.len() > 3 {
                return Err("MATLAB find supports at most three output variables".to_string());
            }

            if is_matlab_find && (2..=3).contains(&output_vars.len()) {
                self.advance(); // skip `find`
                if matches!(self.current_token().kind, TokenKind::LeftParen) {
                    self.advance();
                    let mut arguments = Vec::new();
                    let mut argument = String::new();
                    let mut nested = 0usize;
                    while !matches!(self.current_token().kind, TokenKind::EOF) {
                        match self.current_token().kind {
                            TokenKind::LeftParen => {
                                nested += 1;
                                argument.push_str(&self.current_token().lexeme);
                            }
                            TokenKind::RightParen if nested > 0 => {
                                nested -= 1;
                                argument.push_str(&self.current_token().lexeme);
                            }
                            TokenKind::RightParen => {
                                arguments.push(argument.trim().to_string());
                                break;
                            }
                            TokenKind::Comma if nested == 0 => {
                                arguments.push(argument.trim().to_string());
                                argument.clear();
                            }
                            _ => argument.push_str(&self.current_token().lexeme),
                        }
                        self.advance();
                    }
                    if !(1..=3).contains(&arguments.len()) {
                        return Err(
                            "MATLAB multi-output find supports one to three arguments".to_string()
                        );
                    }
                    let array_argument = arguments[0].as_str();
                    let is_known_array = !array_argument.is_empty()
                        && array_argument
                            .chars()
                            .all(|character| character.is_ascii_alphanumeric() || character == '_')
                        && self.is_assigned_before(array_argument, self.position);
                    if is_known_array {
                        let forwarded_arguments = arguments.join(", ");
                        self.emit_line(&format!(
                            "{} = MATLAB_FIND_ROWS({forwarded_arguments})",
                            output_vars[0]
                        ));
                        self.emit_line(&format!(
                            "{} = MATLAB_FIND_COLUMNS({forwarded_arguments})",
                            output_vars[1]
                        ));
                        if output_vars.len() == 3 {
                            self.emit_line(&format!(
                                "{} = MATLAB_FIND_INDICES({forwarded_arguments})",
                                output_vars[2]
                            ));
                        }
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                        ) {
                            self.advance();
                        }
                        if matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon
                        ) {
                            self.advance();
                        }
                        return Ok(());
                    }
                    return Err(
                        "MATLAB multi-output find requires one previously assigned array variable"
                            .to_string(),
                    );
                }
            }

            if is_matlab_size && output_vars.len() >= 2 {
                self.advance(); // skip `size`
                if matches!(self.current_token().kind, TokenKind::LeftParen) {
                    self.advance(); // skip `(`
                    let mut arguments = Vec::new();
                    let mut current_argument = String::new();
                    let mut paren_depth = 0usize;
                    while !matches!(self.current_token().kind, TokenKind::EOF) {
                        match &self.current_token().kind {
                            TokenKind::LeftParen => {
                                paren_depth += 1;
                                current_argument.push_str(&self.current_token().lexeme);
                            }
                            TokenKind::RightParen if paren_depth > 0 => {
                                paren_depth -= 1;
                                current_argument.push_str(&self.current_token().lexeme);
                            }
                            TokenKind::RightParen => {
                                if !current_argument.trim().is_empty() {
                                    arguments.push(current_argument.trim().to_string());
                                }
                                break;
                            }
                            TokenKind::Comma if paren_depth == 0 => {
                                arguments.push(current_argument.trim().to_string());
                                current_argument.clear();
                            }
                            _ => current_argument.push_str(&self.current_token().lexeme),
                        }
                        self.advance();
                    }

                    let array_argument = (arguments.len() == 1)
                        .then(|| arguments[0].trim())
                        .filter(|argument| {
                            !argument.is_empty()
                                && argument.chars().all(|character| {
                                    character.is_ascii_alphanumeric() || character == '_'
                                })
                        });
                    if let Some(array_argument) = array_argument {
                        for (index, output) in output_vars.iter().enumerate() {
                            if index + 1 == output_vars.len() {
                                self.emit_line(&format!(
                                    "{output} = MATLAB_SIZE_TAIL({array_argument}, {})",
                                    index + 1
                                ));
                            } else {
                                self.emit_line(&format!(
                                    "{output} = MATLAB_SIZE({array_argument}, {})",
                                    index + 1
                                ));
                            }
                        }
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                        ) {
                            self.advance();
                        }
                        if matches!(
                            self.current_token().kind,
                            TokenKind::Newline | TokenKind::Semicolon
                        ) {
                            self.advance();
                        }
                        return Ok(());
                    }
                }
            }

            if is_meshgrid && output_vars.len() == 2 {
                // Generate loop-based meshgrid code
                self.advance(); // skip 'meshgrid'
                if matches!(self.current_token().kind, TokenKind::LeftParen) {
                    self.advance(); // skip '('

                    // Collect the argument
                    let mut arg = String::new();
                    let mut paren_depth = 0;
                    while !matches!(self.current_token().kind, TokenKind::EOF) {
                        if matches!(self.current_token().kind, TokenKind::LeftParen) {
                            paren_depth += 1;
                            arg.push('(');
                        } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                            if paren_depth > 0 {
                                paren_depth -= 1;
                                arg.push(')');
                            } else {
                                break;
                            }
                        } else {
                            arg.push_str(&self.current_token().lexeme);
                        }
                        self.advance();
                    }

                    // Skip to end of statement
                    while !matches!(
                        self.current_token().kind,
                        TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
                    ) {
                        self.advance();
                    }

                    // Generate XDL code
                    let x_var = &output_vars[0];
                    let y_var = &output_vars[1];

                    // Convert range expression to FINDGEN if it contains ':'
                    let converted_arg = if arg.trim().contains(':') {
                        self.convert_range_to_findgen(arg.trim())
                    } else {
                        arg.trim().to_string()
                    };

                    self.emit_line("; meshgrid converted to XDL loops");
                    self.emit_line(&format!("x_vec = {}", converted_arg));
                    self.emit_line(&format!("y_vec = {}", converted_arg));
                    self.emit_line("nx = N_ELEMENTS(x_vec)");
                    self.emit_line("ny = N_ELEMENTS(y_vec)");
                    self.emit_line(&format!("{} = FLTARR(nx, ny)", x_var));
                    self.emit_line(&format!("{} = FLTARR(nx, ny)", y_var));
                    self.emit_line("for i = 0, nx - 1 do begin");
                    self.indent_level += 1;
                    self.emit_line("for j = 0, ny - 1 do begin");
                    self.indent_level += 1;
                    self.emit_line(&format!("{}[i, j] = x_vec[i]", x_var));
                    self.emit_line(&format!("{}[i, j] = y_vec[j]", y_var));
                    self.indent_level -= 1;
                    self.emit_line("endfor");
                    self.indent_level -= 1;
                    self.emit_line("endfor");

                    return Ok(());
                }
            }

            // Not meshgrid, restore position and process normally
            self.position = saved_pos;
        }

        let expr = self.collect_expression_until_newline()?;
        if !expr.trim().is_empty() {
            self.emit_line(&expr);
        }
        Ok(())
    }

    fn collect_expression_until_newline(&mut self) -> Result<String, String> {
        let mut expr = String::new();

        loop {
            let token = self.current_token();
            if matches!(
                token.kind,
                TokenKind::Newline | TokenKind::Semicolon | TokenKind::EOF
            ) {
                break;
            }

            match &token.kind {
                TokenKind::LeftBracket => {
                    // Check if this is a multiple output assignment: [X, Y] = func(...)
                    // by looking ahead for identifiers followed by ] =
                    let is_multiple_output = if expr.trim().is_empty() {
                        let mut check_pos = self.position + 1;
                        let mut found_ids = false;
                        let mut found_bracket = false;
                        let mut found_assign = false;

                        // Look ahead to check pattern: [ id , id ... ] =
                        while check_pos < self.tokens.len() && check_pos < self.position + 20 {
                            match &self.tokens[check_pos].kind {
                                TokenKind::Identifier(_) => found_ids = true,
                                TokenKind::RightBracket if found_ids => found_bracket = true,
                                TokenKind::Assign if found_bracket => {
                                    found_assign = true;
                                    break;
                                }
                                TokenKind::Comma => {} // Continue checking
                                _ if !found_bracket => {}
                                _ => break, // Stop if unexpected token after ]
                            }
                            check_pos += 1;
                        }
                        found_assign
                    } else {
                        false
                    };

                    if is_multiple_output {
                        // Handle multiple output assignment: [X, Y] = func(...)
                        // Just output the bracket and let normal processing continue
                        expr.push('[');
                        self.advance();
                        continue;
                    }

                    // Check if this is an array literal or array indexing
                    // Array literal: appears at start of expression or after = or ,
                    // Array indexing: appears after an identifier
                    let is_array_literal = expr.is_empty()
                        || expr.trim().ends_with('=')
                        || expr.trim().ends_with(',')
                        || expr.trim().ends_with('(');

                    if is_array_literal {
                        // Parse as array literal
                        match self.parse_array_literal() {
                            Ok(array_str) => {
                                expr.push_str(&array_str);
                                continue;
                            }
                            Err(e) => {
                                expr.push_str(&format!("/* array parse error: {} */", e));
                                self.advance();
                                continue;
                            }
                        }
                    } else {
                        // Array indexing - adjust for 0-based
                        expr.push('[');
                        self.advance();

                        // Collect index expression
                        let mut index_expr = String::new();
                        let mut paren_depth = 0;
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightBracket | TokenKind::EOF
                        ) || paren_depth > 0
                        {
                            if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                paren_depth += 1;
                            } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                                paren_depth -= 1;
                            }
                            index_expr.push_str(&self.current_token().lexeme);
                            self.advance();
                        }

                        let array_name = expr
                            .trim()
                            .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                            .next_back()
                            .unwrap_or_default();
                        expr.push_str(
                            &translate_matlab_index(&index_expr, array_name)
                                .unwrap_or_else(|| format!("({}) - 1", index_expr)),
                        );

                        if matches!(self.current_token().kind, TokenKind::RightBracket) {
                            expr.push(']');
                            self.advance();
                        }
                        continue;
                    }
                }
                TokenKind::Colon => {
                    // Colon operator: could be part of a range expression
                    // If we're building a standalone range (not in array indexing context)
                    // we need to convert to FINDGEN
                    expr.push_str(" : ");
                    self.advance();
                    continue;
                }
                TokenKind::Identifier(name) => {
                    // Map MATLAB constants to XDL system variables
                    // BUT: Don't map single-letter identifiers on LHS of assignment
                    let is_lhs_of_assignment =
                        expr.trim().is_empty() || expr.trim().ends_with('\n');
                    let mapped_name = match name.as_str() {
                        "pi" => "!PI",
                        "e" if !is_lhs_of_assignment => "!E", // Only map 'e' if not on LHS
                        _ => name.as_str(),
                    };

                    // Map MATLAB function to XDL
                    let func_name = if let Some(xdl_func) = get_xdl_function(mapped_name) {
                        xdl_func
                    } else {
                        mapped_name
                    };

                    // Check if this is a standalone procedure call (PRINT, PLOT, etc.)
                    // by looking ahead for parenthesis
                    let next_pos = self.position + 1;
                    let is_procedure_call = next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                        && expr.trim().is_empty(); // Statement starts with function

                    // Special handling for randn(size(x)) or rand(size(x)) -> RANDOMN(seed, N_ELEMENTS(x))
                    if (mapped_name == "randn" || mapped_name == "rand")
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        // randn is normal (RANDOMN); rand is uniform (RANDOMU).
                        // Mapping both to RANDOMU gave randn uniform statistics.
                        let xdl_func = if mapped_name == "randn" {
                            "RANDOMN"
                        } else {
                            "RANDOMU"
                        };
                        self.advance(); // skip 'randn'/'rand'
                        self.advance(); // skip '('

                        // Check if argument is size(something)
                        if let TokenKind::Identifier(func) = &self.current_token().kind {
                            if func == "size" {
                                self.advance(); // skip 'size'
                                if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                    self.advance(); // skip '('

                                    // Get the variable name
                                    let mut var_name = String::new();
                                    while !matches!(
                                        self.current_token().kind,
                                        TokenKind::RightParen | TokenKind::EOF
                                    ) {
                                        var_name.push_str(&self.current_token().lexeme);
                                        self.advance();
                                    }

                                    if matches!(self.current_token().kind, TokenKind::RightParen) {
                                        self.advance(); // skip ')' for size
                                    }
                                    if matches!(self.current_token().kind, TokenKind::RightParen) {
                                        self.advance(); // skip ')' for randn/rand
                                    }

                                    // Generate: RANDOMN(seed, N_ELEMENTS(var))
                                    // Use a fixed seed for reproducibility (can be made configurable later)
                                    expr.push_str(&format!(
                                        "{}(1, N_ELEMENTS({}))",
                                        xdl_func,
                                        var_name.trim()
                                    ));
                                    continue;
                                }
                            }
                        }

                        // Fall back to collecting regular arguments
                        let mut args = Vec::new();
                        let mut current_arg = String::new();
                        let mut paren_depth = 0;

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) || paren_depth > 0
                        {
                            if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                paren_depth += 1;
                                current_arg.push('(');
                            } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                                paren_depth -= 1;
                                if paren_depth >= 0 {
                                    current_arg.push(')');
                                }
                            } else if matches!(self.current_token().kind, TokenKind::Comma)
                                && paren_depth == 0
                            {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                current_arg.push_str(&self.current_token().lexeme);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // Generate: RANDOMN(seed, n) or RANDOMU(seed, n)
                        if !args.is_empty() {
                            expr.push_str(&format!("{}(1, {})", xdl_func, args.join(", ")));
                        } else {
                            expr.push_str(&format!("{}(1, 1)", xdl_func)); // Default to single random value
                        }
                        continue;
                    }

                    // Special handling for complex(real, imag) - XDL doesn't support complex, use real part only
                    if mapped_name == "complex"
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        self.advance(); // skip 'complex'
                        self.advance(); // skip '('

                        // Collect arguments: real, imag
                        let mut args = Vec::new();
                        let mut current_arg = String::new();
                        let mut paren_depth = 0;

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) || paren_depth > 0
                        {
                            if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                paren_depth += 1;
                                current_arg.push('(');
                            } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                                paren_depth -= 1;
                                if paren_depth >= 0 {
                                    current_arg.push(')');
                                }
                            } else if matches!(self.current_token().kind, TokenKind::Comma)
                                && paren_depth == 0
                            {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                // Map constants
                                let token_str = match &self.current_token().kind {
                                    TokenKind::Identifier(name) => match name.as_str() {
                                        "pi" => "!PI",
                                        "e" => "!E",
                                        _ => &self.current_token().lexeme,
                                    },
                                    _ => &self.current_token().lexeme,
                                };
                                current_arg.push_str(token_str);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // XDL doesn't have a complex constructor, so for exp(complex(0, t)):
                        // we'll convert to just the imaginary part for now (exp(i*t) pattern)
                        // In practice, complex(0, t) means 0 + i*t, so exp(complex(0, t)) = exp(i*t)
                        // = cos(t) + i*sin(t). For real plots, we can just use the real part: cos(t)
                        if args.len() >= 2 {
                            let real_part = &args[0];
                            let imag_part = &args[1];
                            // If real part is 0, this is purely imaginary
                            if real_part.trim() == "0" {
                                // For exp(i*t) pattern, just use i*t directly as the argument
                                // But since XDL has no complex type, we use the imaginary part directly
                                expr.push_str(imag_part);
                            } else {
                                // Use real part for now
                                expr.push_str(real_part);
                            }
                        } else if args.len() == 1 {
                            // Just real part
                            expr.push_str(&args[0]);
                        } else {
                            expr.push_str("/* complex() needs 1-2 args */");
                        }
                        continue;
                    }

                    // Special handling for zeros(n) or zeros(m, n) -> FLTARR(m, n)
                    if mapped_name == "zeros"
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        self.advance(); // skip 'zeros'
                        self.advance(); // skip '('

                        let mut args = Vec::new();
                        let mut current_arg = String::new();

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            if matches!(self.current_token().kind, TokenKind::Comma) {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                current_arg.push_str(&self.current_token().lexeme);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // Generate FLTARR with appropriate dimensions
                        expr.push_str(&format!("FLTARR({})", args.join(", ")));
                        continue;
                    }

                    // Special handling for ones(n) or ones(m, n) -> FLTARR(m, n) + 1
                    if mapped_name == "ones"
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        self.advance(); // skip 'ones'
                        self.advance(); // skip '('

                        let mut args = Vec::new();
                        let mut current_arg = String::new();

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            if matches!(self.current_token().kind, TokenKind::Comma) {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                current_arg.push_str(&self.current_token().lexeme);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // Generate FLTARR with appropriate dimensions + 1
                        expr.push_str(&format!("FLTARR({}) + 1", args.join(", ")));
                        continue;
                    }

                    // Special handling for eye(n) -> IDENTITY(n)
                    if mapped_name == "eye"
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        self.advance(); // skip 'eye'
                        self.advance(); // skip '('

                        let mut args = Vec::new();
                        let mut current_arg = String::new();

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            if matches!(self.current_token().kind, TokenKind::Comma) {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                current_arg.push_str(&self.current_token().lexeme);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // Generate IDENTITY
                        // eye(n) -> IDENTITY(n), eye(m,n) -> use IDENTITY(n) for square matrix
                        if args.len() == 1 {
                            expr.push_str(&format!("IDENTITY({})", args[0]));
                        } else if args.len() >= 2 {
                            // For non-square, we need a custom approach but IDENTITY only works for square
                            expr.push_str(&format!("IDENTITY({})", args[0]));
                        }
                        continue;
                    }

                    // Special handling for linspace(start, end, n) -> FINDGEN(n) * (end-start) / (n-1) + start
                    if mapped_name == "linspace"
                        && next_pos < self.tokens.len()
                        && matches!(self.tokens[next_pos].kind, TokenKind::LeftParen)
                    {
                        self.advance(); // skip 'linspace'
                        self.advance(); // skip '('

                        // Collect arguments: start, end, n
                        let mut args = Vec::new();
                        let mut current_arg = String::new();
                        let mut paren_depth = 0;

                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) || paren_depth > 0
                        {
                            if matches!(self.current_token().kind, TokenKind::LeftParen) {
                                paren_depth += 1;
                                current_arg.push('(');
                            } else if matches!(self.current_token().kind, TokenKind::RightParen) {
                                paren_depth -= 1;
                                if paren_depth >= 0 {
                                    current_arg.push(')');
                                }
                            } else if matches!(self.current_token().kind, TokenKind::Comma)
                                && paren_depth == 0
                            {
                                args.push(current_arg.trim().to_string());
                                current_arg = String::new();
                            } else {
                                // Map constants like pi -> !PI
                                let token_str = match &self.current_token().kind {
                                    TokenKind::Identifier(name) => match name.as_str() {
                                        "pi" => "!PI",
                                        "e" => "!E",
                                        _ => &self.current_token().lexeme,
                                    },
                                    _ => &self.current_token().lexeme,
                                };
                                current_arg.push_str(token_str);
                            }
                            self.advance();
                        }

                        if !current_arg.trim().is_empty() {
                            args.push(current_arg.trim().to_string());
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }

                        // Generate XDL equivalent: FINDGEN(n) * (end-start) / (n-1) + start
                        if args.len() == 3 {
                            let start = &args[0];
                            let end = &args[1];
                            let n = &args[2];
                            expr.push_str(&format!(
                                "FINDGEN({}) * (({}) - ({})) / ({} - 1) + ({})",
                                n, end, start, n, start
                            ));
                        } else {
                            // If wrong number of args, just emit a comment
                            expr.push_str(&format!(
                                "/* linspace error: expected 3 args, got {} */",
                                args.len()
                            ));
                        }
                        continue;
                    }

                    // Special handling for meshgrid - DISABLED
                    // XDL doesn't support multiple output assignment [X, Y] = ...
                    // So we handle this at the statement level instead
                    // (see transpile_statement for meshgrid handling)

                    // Special handling for PLOT command with line styles
                    if is_procedure_call && func_name == "PLOT" {
                        // Add tile comment if we're in a subplot
                        if self.current_tile > 0 {
                            self.emit_line(&format!(
                                "  ; Tile {} of {}",
                                self.current_tile,
                                self.subplot_rows * self.subplot_cols
                            ));
                        }
                        expr.push_str("PLOT, ");
                        self.advance(); // skip function name
                        self.advance(); // skip '('

                        let mut arg_count = 0;
                        // Collect arguments until ')'
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            match &self.current_token().kind {
                                TokenKind::String(s) => {
                                    // Check if this is a line style string (contains -, :, ., or color letters)
                                    if s.contains('-')
                                        || s.contains(':')
                                        || s.contains('.')
                                        || s.contains('r')
                                        || s.contains('g')
                                        || s.contains('b')
                                        || s.contains('*')
                                        || s.contains('o')
                                        || s.contains('+')
                                    {
                                        // This is likely a line style, skip it
                                        self.advance();
                                        // Skip comma if present
                                        if matches!(self.current_token().kind, TokenKind::Comma) {
                                            self.advance();
                                        }
                                        continue;
                                    } else {
                                        if arg_count > 0 {
                                            expr.push_str(", ");
                                        }
                                        expr.push_str(&format!("'{}'", s));
                                        arg_count += 1;
                                    }
                                }
                                TokenKind::Identifier(n) => {
                                    if arg_count > 0 {
                                        expr.push_str(", ");
                                    }
                                    // Map MATLAB constants
                                    let mapped = match n.as_str() {
                                        "pi" => "!PI",
                                        "e" => "!E",
                                        _ => n.as_str(),
                                    };
                                    expr.push_str(mapped);
                                    arg_count += 1;
                                }
                                TokenKind::Number(n) => {
                                    if arg_count > 0 {
                                        expr.push_str(", ");
                                    }
                                    expr.push_str(&n.to_string());
                                    arg_count += 1;
                                }
                                TokenKind::Comma => {
                                    // Skip commas, we'll add them ourselves
                                }
                                TokenKind::LeftParen => expr.push('('),
                                TokenKind::RightParen => break,
                                _ => expr.push_str(&self.current_token().lexeme),
                            }
                            self.advance();
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }
                        continue;
                    }

                    if is_procedure_call && matches!(func_name, "PRINT" | "PRINTF") {
                        // Convert MATLAB func(arg) to XDL FUNC, arg
                        expr.push_str(func_name);
                        expr.push_str(", ");
                        self.advance(); // skip function name
                        self.advance(); // skip '('

                        // Collect arguments until ')'
                        while !matches!(
                            self.current_token().kind,
                            TokenKind::RightParen | TokenKind::EOF
                        ) {
                            match &self.current_token().kind {
                                TokenKind::Identifier(n) => {
                                    let following_paren = self.position + 1;
                                    if following_paren < self.tokens.len()
                                        && matches!(
                                            self.tokens[following_paren].kind,
                                            TokenKind::LeftParen
                                        )
                                        && self.is_assigned_before(n, self.position)
                                    {
                                        self.advance(); // array name
                                        self.advance(); // opening parenthesis
                                        let mut index_tokens = Vec::new();
                                        let mut nested_parens = 0usize;
                                        while !matches!(
                                            self.current_token().kind,
                                            TokenKind::EOF | TokenKind::RightParen
                                        ) || nested_parens > 0
                                        {
                                            match self.current_token().kind {
                                                TokenKind::LeftParen => nested_parens += 1,
                                                TokenKind::RightParen => nested_parens -= 1,
                                                _ => {}
                                            }
                                            index_tokens.push(self.current_token().clone());
                                            self.advance();
                                        }
                                        if matches!(
                                            self.current_token().kind,
                                            TokenKind::RightParen
                                        ) {
                                            self.advance();
                                        }

                                        let index_parts = split_subscript_tokens(&index_tokens);
                                        let scalar_indices = index_parts
                                            .iter()
                                            .map(|part| {
                                                !part.is_empty()
                                                    && !part.iter().any(|token| {
                                                        matches!(
                                                            token.kind,
                                                            TokenKind::Colon | TokenKind::Comma
                                                        )
                                                    })
                                            })
                                            .all(|scalar| scalar);
                                        if scalar_indices && index_parts.len() == 1 {
                                            expr.push_str("MATLAB_INDEX(");
                                            expr.push_str(n);
                                            expr.push_str(", ");
                                            expr.push_str(&translate_matlab_linear_index(
                                                &index_parts[0],
                                                n,
                                            ));
                                            expr.push(')');
                                            continue;
                                        }
                                        if scalar_indices && index_parts.len() > 1 {
                                            let translated = index_parts
                                                .iter()
                                                .map(|part| {
                                                    let index = part
                                                        .iter()
                                                        .map(|token| token.lexeme.as_str())
                                                        .collect::<String>();
                                                    translate_matlab_index(&index, n)
                                                })
                                                .collect::<Option<Vec<_>>>();
                                            if let Some(indices) = translated {
                                                expr.push_str(n);
                                                expr.push_str(" [");
                                                expr.push_str(&indices.join(", "));
                                                expr.push(']');
                                                continue;
                                            }
                                        }
                                        // Preserve unsupported ranges and malformed
                                        // subscripts as source so they fail visibly.
                                        expr.push_str(n);
                                        expr.push('(');
                                        expr.push_str(
                                            &index_tokens
                                                .iter()
                                                .map(|token| token.lexeme.as_str())
                                                .collect::<String>(),
                                        );
                                        expr.push(')');
                                        continue;
                                    }
                                    // Map MATLAB function names to XDL
                                    if let Some(xdl_func) = get_xdl_function(n) {
                                        expr.push_str(xdl_func);
                                    } else {
                                        expr.push_str(n);
                                    }
                                }
                                TokenKind::Number(n) => expr.push_str(&n.to_string()),
                                TokenKind::String(s) => expr.push_str(&format!("'{}'", s)),
                                TokenKind::Comma => expr.push_str(", "),
                                TokenKind::LeftParen => expr.push('('),
                                _ => expr.push_str(&self.current_token().lexeme),
                            }
                            self.advance();
                        }

                        if matches!(self.current_token().kind, TokenKind::RightParen) {
                            self.advance(); // skip ')'
                        }
                        continue;
                    } else {
                        expr.push_str(func_name);
                    }
                }
                TokenKind::Number(n) => expr.push_str(&n.to_string()),
                TokenKind::String(s) => expr.push_str(&format!("'{}'", s)),
                TokenKind::LeftParen => {
                    // Check if this is a range expression like (0:L-1) or (1:10)
                    // We need to lookahead to see if there's a colon inside
                    let start_pos = self.position;
                    self.advance(); // skip '('

                    // Collect tokens until ')' to check for colon
                    let mut range_tokens = Vec::new();
                    let mut paren_depth = 1;
                    let mut has_colon = false;

                    while paren_depth > 0 && !matches!(self.current_token().kind, TokenKind::EOF) {
                        match &self.current_token().kind {
                            TokenKind::LeftParen => paren_depth += 1,
                            TokenKind::RightParen => paren_depth -= 1,
                            TokenKind::Colon => has_colon = true,
                            _ => {}
                        }
                        if paren_depth > 0 {
                            range_tokens.push(self.current_token().clone());
                        }
                        self.advance();
                    }

                    let index_parts = split_subscript_tokens(&range_tokens);
                    let preceding_name = expr
                        .trim()
                        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_'))
                        .next_back()
                        .unwrap_or_default()
                        .to_string();
                    let is_known_array = !preceding_name.is_empty()
                        && self.is_assigned_before(&preceding_name, start_pos);
                    let is_assignment_target =
                        matches!(self.current_token().kind, TokenKind::Assign);

                    if paren_depth == 0 && index_parts.len() > 1 && is_known_array {
                        let translated = index_parts
                            .iter()
                            .map(|part| {
                                if part.iter().any(|token| {
                                    matches!(&token.kind, TokenKind::Identifier(name) if name.eq_ignore_ascii_case("end"))
                                        || matches!(&token.kind, TokenKind::End)
                                }) {
                                    None
                                } else {
                                    let index = part
                                        .iter()
                                        .map(|token| token.lexeme.as_str())
                                        .collect::<String>();
                                    if index_parts.len() == 2 {
                                        translate_matlab_matrix_index(
                                            &index,
                                            is_assignment_target,
                                        )
                                    } else {
                                        translate_matlab_index(&index, &preceding_name)
                                    }
                                }
                            })
                            .collect::<Option<Vec<_>>>();

                        if let Some(translated) = translated {
                            expr.push('[');
                            expr.push_str(&translated.join(", "));
                            expr.push(']');
                        } else {
                            // Leave unsupported multidimensional forms visible so
                            // XDL reports them instead of fabricating a subscript.
                            expr.push('(');
                            expr.push_str(
                                &range_tokens
                                    .iter()
                                    .map(|token| token.lexeme.as_str())
                                    .collect::<String>(),
                            );
                            expr.push(')');
                        }
                    } else if has_colon && paren_depth == 0 {
                        let range = range_tokens
                            .iter()
                            .map(|token| token.lexeme.as_str())
                            .collect::<String>();
                        let is_vector_subscript = is_known_array;

                        if is_vector_subscript {
                            if let Some(index) = translate_matlab_index(&range, &preceding_name) {
                                expr.push('[');
                                expr.push_str(&index);
                                expr.push(']');
                            } else {
                                expr.push('(');
                                expr.push_str(&self.parse_range_expression(&range_tokens));
                                expr.push(')');
                            }
                        } else {
                            // Preserve function-call syntax for names that are not
                            // known array variables; convert a range argument to a vector.
                            if !preceding_name.is_empty() {
                                expr.push('(');
                            }
                            expr.push_str(&self.parse_range_expression(&range_tokens));
                            if !preceding_name.is_empty() {
                                expr.push(')');
                            }
                        }
                    } else {
                        let index = range_tokens
                            .iter()
                            .map(|token| token.lexeme.as_str())
                            .collect::<String>();
                        if is_known_array {
                            if index_parts.len() == 1
                                && !has_colon
                                && !matches!(self.current_token().kind, TokenKind::Assign)
                            {
                                let expression_prefix = expr.trim_end();
                                if expression_prefix.ends_with(&preceding_name) {
                                    expr.truncate(expression_prefix.len() - preceding_name.len());
                                }
                                let index =
                                    translate_matlab_linear_index(&range_tokens, &preceding_name);
                                expr.push_str("MATLAB_INDEX(");
                                expr.push_str(&preceding_name);
                                expr.push_str(", ");
                                expr.push_str(&index);
                                expr.push(')');
                            } else if let Some(index) =
                                translate_matlab_index(&index, &preceding_name)
                            {
                                expr.push('[');
                                expr.push_str(&index);
                                expr.push(']');
                            } else {
                                expr.push('(');
                                expr.push_str(&index);
                                expr.push(')');
                            }
                        } else {
                            // Not a known array subscript or range; leave the call intact.
                            self.position = start_pos;
                            self.advance();
                            expr.push('(');
                        }
                    }
                    continue;
                }
                TokenKind::ElementMultiply => expr.push_str(" * "),
                TokenKind::ElementDivide => expr.push_str(" / "),
                TokenKind::ElementPower => expr.push_str(" ^ "),
                TokenKind::Dot
                    if self
                        .tokens
                        .get(self.position + 1)
                        .is_some_and(|next| matches!(next.kind, TokenKind::Transpose)) =>
                {
                    // This bounded slice supports MATLAB's nonconjugating
                    // transpose only for a directly named array: A.'.
                    let trimmed = expr.trim_end();
                    let operand_start = trimmed
                        .char_indices()
                        .rev()
                        .take_while(|(_, ch)| ch.is_ascii_alphanumeric() || *ch == '_')
                        .last()
                        .map(|(index, _)| index);
                    if let Some(start) = operand_start {
                        let operand = trimmed[start..].to_owned();
                        if operand
                            .chars()
                            .next()
                            .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_')
                        {
                            expr.truncate(start);
                            expr.push_str(&format!("TRANSPOSE({operand})"));
                            self.advance(); // dot
                            self.advance(); // transpose apostrophe
                            continue;
                        }
                    }
                    return Err(
                        "MATLAB nonconjugating transpose currently requires a named array (A.')"
                            .to_string(),
                    );
                }
                TokenKind::Transpose => {
                    return Err(
                        "MATLAB conjugating transpose (A') is not supported yet".to_string()
                    );
                }
                TokenKind::Comment(_) => {
                    // Skip comments in expressions
                    self.advance();
                    continue;
                }
                _ => expr.push_str(&token.lexeme),
            }

            expr.push(' ');
            self.advance();
        }

        // Skip newline/semicolon
        if matches!(
            self.current_token().kind,
            TokenKind::Newline | TokenKind::Semicolon
        ) {
            self.advance();
        }

        let expr_trimmed = expr.trim();

        // Post-process: detect standalone range expressions with colons
        // Pattern: "var = start : end" or "var = start : step : end"
        // Only convert if this looks like a simple assignment with a range
        if expr_trimmed.contains(" = ") && expr_trimmed.contains(" : ") {
            // Split by = to get left and right sides
            let parts: Vec<&str> = expr_trimmed.splitn(2, " = ").collect();
            if parts.len() == 2 {
                let lhs = parts[0].trim();
                let rhs = parts[1].trim();

                // Check if RHS contains colon - indicating a range expression
                // The convert_range_to_findgen function handles both simple and complex ranges
                let colon_count = rhs.matches(" : ").count();

                // Don't convert if it looks like array indexing or cell arrays
                let is_array_op = rhs.contains("[") || rhs.contains("{");

                if colon_count > 0 && !is_array_op {
                    // This looks like a range expression: convert it
                    let converted = self.convert_range_to_findgen(rhs);
                    return Ok(format!("{} = {}", lhs, converted));
                }
            }
        }

        Ok(expr_trimmed.to_string())
    }

    fn convert_range(&self, range: &str) -> Result<String, String> {
        // Keep MATLAB's bounds: `1:10` -> `1, 10`, `1:2:10` -> `1, 10, 2`.
        // The loop variable is then MATLAB's value, so `disp(i)` and the
        // body's index adjustment `x(i-1)` both see the MATLAB number.
        // Shifting the bounds to 0-based while leaving `i` unshifted printed
        // 0, 0.1, 0.2 for `for i = 1:3; disp(i*0.1); end`.

        let parts: Vec<&str> = range.split(':').collect();

        match parts.len() {
            2 => {
                // start:end -> start, end
                let start: i32 = parts[0].trim().parse().map_err(|_| "Invalid range start")?;
                let end: i32 = parts[1].trim().parse().map_err(|_| "Invalid range end")?;
                Ok(format!("{}, {}", start, end))
            }
            3 => {
                // start:step:end -> start, end, step
                let start: i32 = parts[0].trim().parse().map_err(|_| "Invalid range start")?;
                let step: i32 = parts[1].trim().parse().map_err(|_| "Invalid range step")?;
                let end: i32 = parts[2].trim().parse().map_err(|_| "Invalid range end")?;
                Ok(format!("{}, {}, {}", start, end, step))
            }
            _ => Ok(range.to_string()), // Return as-is if not a simple range
        }
    }

    fn parse_range_expression(&self, tokens: &[Token]) -> String {
        // Parse range expressions like 0:L-1 or 1:2:10
        // Split by colons
        let mut parts = Vec::new();
        let mut current_part = Vec::new();

        for token in tokens {
            if matches!(token.kind, TokenKind::Colon) {
                parts.push(current_part.clone());
                current_part.clear();
            } else {
                current_part.push(token.clone());
            }
        }
        if !current_part.is_empty() {
            parts.push(current_part);
        }

        // Convert tokens to string expressions
        let expr_parts: Vec<String> = parts
            .iter()
            .map(|part| {
                part.iter()
                    .map(|t| {
                        match &t.kind {
                            TokenKind::Identifier(name) => {
                                // Map constants
                                match name.as_str() {
                                    "pi" => "!PI",
                                    "e" => "!E",
                                    _ => &t.lexeme,
                                }
                                .to_string()
                            }
                            TokenKind::Number(n) => n.to_string(),
                            _ => t.lexeme.clone(),
                        }
                    })
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect();

        // Handle different range formats
        match expr_parts.len() {
            2 => {
                // start:end -> FINDGEN((end) - (start) + 1) + (start)
                let start = expr_parts[0].trim();
                let end = expr_parts[1].trim();

                // For 0:L-1 specifically, this is just FINDGEN(L)
                if start == "0" {
                    format!("FINDGEN(({})+1)", end)
                } else {
                    format!("FINDGEN(({})-({}) +1) + ({})", end, start, start)
                }
            }
            3 => {
                // start:step:end -> (FINDGEN(((end)-(start))/(step)+1) * (step)) + (start)
                let start = expr_parts[0].trim();
                let step = expr_parts[1].trim();
                let end = expr_parts[2].trim();
                format!(
                    "(FINDGEN((({})-({}))/({}) +1) * ({})) + ({})",
                    end, start, step, step, start
                )
            }
            _ => {
                // Shouldn't happen, but fall back to comment
                "/* unhandled range expression */".to_string()
            }
        }
    }

    /// Parse MATLAB array literal [1, 2, 3] or [1, 2; 3, 4] to XDL array syntax
    fn parse_array_literal(&mut self) -> Result<String, String> {
        // Skip opening bracket
        self.advance();

        let mut rows: Vec<Vec<String>> = Vec::new();
        let mut current_row: Vec<String> = Vec::new();
        let mut current_element = String::new();
        let mut paren_depth = 0;
        let mut bracket_depth = 0;

        while !matches!(self.current_token().kind, TokenKind::EOF) {
            match &self.current_token().kind {
                TokenKind::RightBracket if bracket_depth == 0 && paren_depth == 0 => {
                    // End of array literal
                    if !current_element.trim().is_empty() {
                        current_row.push(current_element.trim().to_string());
                    }
                    if !current_row.is_empty() {
                        rows.push(current_row.clone());
                    }
                    self.advance();
                    break;
                }
                TokenKind::Comma if paren_depth == 0 && bracket_depth == 0 => {
                    // Element separator within a row
                    if !current_element.trim().is_empty() {
                        current_row.push(current_element.trim().to_string());
                        current_element.clear();
                    }
                    self.advance();
                }
                TokenKind::Semicolon if paren_depth == 0 && bracket_depth == 0 => {
                    // Row separator
                    if !current_element.trim().is_empty() {
                        current_row.push(current_element.trim().to_string());
                        current_element.clear();
                    }
                    if !current_row.is_empty() {
                        rows.push(current_row.clone());
                        current_row.clear();
                    }
                    self.advance();
                }
                TokenKind::Colon => {
                    // Handle colon operator for ranges
                    current_element.push(':');
                    self.advance();
                }
                TokenKind::LeftParen => {
                    paren_depth += 1;
                    current_element.push('(');
                    self.advance();
                }
                TokenKind::RightParen => {
                    paren_depth -= 1;
                    current_element.push(')');
                    self.advance();
                }
                TokenKind::LeftBracket => {
                    bracket_depth += 1;
                    current_element.push('[');
                    self.advance();
                }
                TokenKind::RightBracket => {
                    bracket_depth -= 1;
                    current_element.push(']');
                    self.advance();
                }
                TokenKind::Identifier(name) => {
                    // Map constants and check for function calls
                    let mapped = match name.as_str() {
                        "pi" => "!PI".to_string(),
                        "e" => "!E".to_string(),
                        other => {
                            // Check if this is a function call
                            if get_xdl_function(other).is_some() {
                                get_xdl_function(other).unwrap().to_string()
                            } else {
                                other.to_string()
                            }
                        }
                    };
                    current_element.push_str(&mapped);
                    self.advance();
                }
                TokenKind::Number(n) => {
                    // If we have an existing element and it's also a number, treat space as separator
                    if current_element.trim().parse::<f64>().is_ok() {
                        // Push the previous number as an element
                        current_row.push(current_element.trim().to_string());
                        current_element.clear();
                    }
                    current_element.push_str(&n.to_string());
                    self.advance();
                }
                TokenKind::Newline => {
                    // Skip newlines inside array literals
                    self.advance();
                }
                _ => {
                    current_element.push_str(&self.current_token().lexeme);
                    self.advance();
                }
            }
        }

        // Convert to XDL array syntax
        if rows.is_empty() {
            return Ok("[]".to_string());
        }

        if rows.len() == 1 {
            // Simple vector (1D array)
            let elements = &rows[0];
            if elements.len() == 1 {
                // Check if single element contains colon (range expression)
                let elem = &elements[0];
                if elem.contains(':') {
                    // Parse as range expression
                    Ok(self.convert_range_to_findgen(elem))
                } else {
                    Ok(format!("[{}]", elem))
                }
            } else {
                // Multiple elements: [e1, e2, e3, ...]
                let converted: Vec<String> = elements
                    .iter()
                    .map(|e| {
                        if e.contains(':') {
                            self.convert_range_to_findgen(e)
                        } else {
                            e.clone()
                        }
                    })
                    .collect();
                Ok(format!("[{}]", converted.join(", ")))
            }
        } else {
            // 2D matrix: [[row1], [row2], ...] or use TRANSPOSE if needed
            let row_strs: Vec<String> = rows
                .iter()
                .map(|row| {
                    let elements: Vec<String> = row
                        .iter()
                        .map(|e| {
                            if e.contains(':') {
                                self.convert_range_to_findgen(e)
                            } else {
                                e.clone()
                            }
                        })
                        .collect();
                    format!("[{}]", elements.join(", "))
                })
                .collect();
            Ok(format!("[{}]", row_strs.join(", ")))
        }
    }

    /// Convert MATLAB range expression (e.g., "1:10" or "0:0.1:1") to XDL FINDGEN-based expression
    fn convert_range_to_findgen(&self, range_str: &str) -> String {
        let parts: Vec<&str> = range_str.split(':').collect();

        match parts.len() {
            2 => {
                // start:end
                let start = parts[0].trim();
                let end = parts[1].trim();

                // Try to parse as numbers for special cases
                if let (Ok(s), Ok(e)) = (start.parse::<f64>(), end.parse::<f64>()) {
                    let count = (e - s + 1.0).max(0.0) as i32;
                    if s == 0.0 {
                        format!("FINDGEN({})", count)
                    } else {
                        format!("FINDGEN({}) + {}", count, start)
                    }
                } else {
                    // Complex expression
                    format!("FINDGEN((({}) - ({})) + 1) + ({})", end, start, start)
                }
            }
            3 => {
                // start:step:end
                let start = parts[0].trim();
                let step = parts[1].trim();
                let end = parts[2].trim();

                if let (Ok(s), Ok(st), Ok(e)) = (
                    start.parse::<f64>(),
                    step.parse::<f64>(),
                    end.parse::<f64>(),
                ) {
                    let count = ((e - s) / st + 1.0).max(0.0) as i32;
                    if s == 0.0 {
                        format!("FINDGEN({}) * {}", count, step)
                    } else {
                        format!("FINDGEN({}) * {} + {}", count, step, start)
                    }
                } else {
                    // Complex expression
                    format!(
                        "FINDGEN(((({}) - ({})) / ({})) + 1) * ({}) + ({})",
                        end, start, step, step, start
                    )
                }
            }
            _ => format!("/* invalid range: {} */", range_str),
        }
    }
}

/// Translate MATLAB's 1-based vector subscript into XDL's zero-based range.
/// XDL range ends are exclusive, so MATLAB `start:end` maps to
/// `(start - 1):end`; MATLAB's `start:step:end` maps to
/// `(start - 1):end:step`.
fn split_subscript_tokens(tokens: &[Token]) -> Vec<Vec<Token>> {
    let mut parts = vec![Vec::new()];
    let (mut paren_depth, mut bracket_depth, mut brace_depth) = (0usize, 0usize, 0usize);
    for token in tokens {
        match &token.kind {
            TokenKind::LeftParen => paren_depth += 1,
            TokenKind::RightParen => paren_depth = paren_depth.saturating_sub(1),
            TokenKind::LeftBracket => bracket_depth += 1,
            TokenKind::RightBracket => bracket_depth = bracket_depth.saturating_sub(1),
            TokenKind::LeftBrace => brace_depth += 1,
            TokenKind::RightBrace => brace_depth = brace_depth.saturating_sub(1),
            TokenKind::Comma if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                parts.push(Vec::new());
                continue;
            }
            _ => {}
        }
        if let Some(part) = parts.last_mut() {
            part.push(token.clone());
        }
    }
    parts
}

fn translate_matlab_index(index: &str, array_name: &str) -> Option<String> {
    let parts: Vec<&str> = index.split(':').map(str::trim).collect();
    match parts.as_slice() {
        [""] => Some(":".to_string()),
        ["", ""] => Some(":".to_string()),
        [single] => {
            let expanded = expand_matlab_end(single, array_name);
            Some(expanded.parse::<i64>().map_or_else(
                |_| format!("({expanded}) - 1"),
                |value| {
                    value.checked_sub(1).map_or_else(
                        || format!("({expanded}) - 1"),
                        |zero_based| zero_based.to_string(),
                    )
                },
            ))
        }
        ["", end] if !end.is_empty() => Some(format!(":{}", expand_matlab_end(end, array_name))),
        [start, ""] if !start.is_empty() => {
            Some(format!("{}:", offset_matlab_index(start, array_name)))
        }
        [start, end] if !start.is_empty() && !end.is_empty() => {
            let zero_based_start = offset_matlab_index(start, array_name);
            Some(format!(
                "{zero_based_start}:{}",
                expand_matlab_end(end, array_name)
            ))
        }
        ["", end, step] if !end.is_empty() && step.parse::<u64>().is_ok_and(|value| value > 0) => {
            Some(format!(":{}:{step}", expand_matlab_end(end, array_name)))
        }
        [start, step, ""]
            if !start.is_empty() && step.parse::<u64>().is_ok_and(|value| value > 0) =>
        {
            Some(format!(
                "{}:*:{step}",
                offset_matlab_index(start, array_name)
            ))
        }
        [start, step, end]
            if !start.is_empty()
                && !step.is_empty()
                && !end.is_empty()
                && step.parse::<u64>().is_ok_and(|value| value > 0) =>
        {
            let zero_based_start = offset_matlab_index(start, array_name);
            Some(format!(
                "{zero_based_start}:{}:{step}",
                expand_matlab_end(end, array_name)
            ))
        }
        _ => None,
    }
}

/// Translate one MATLAB matrix-axis subscript for the evaluator's inclusive
/// XDL ranges. MATLAB is one-based and inclusive too, so both range endpoints
/// shift down by one. A bare `:` remains a full-axis selection.
fn translate_matlab_matrix_index(index: &str, assignment_target: bool) -> Option<String> {
    let parts: Vec<&str> = index.split(':').map(str::trim).collect();
    let zero_based = |value: &str| {
        value.parse::<i64>().map_or_else(
            |_| format!("({value}) - 1"),
            |value| {
                value
                    .checked_sub(1)
                    .map_or_else(|| format!("({value}) - 1"), |value| value.to_string())
            },
        )
    };
    match parts.as_slice() {
        [single] if !single.is_empty() => Some(zero_based(single)),
        ["", ""] => Some(":".to_string()),
        [start, end] if !start.is_empty() && !end.is_empty() => {
            let end = if assignment_target {
                end.to_string()
            } else {
                zero_based(end)
            };
            Some(format!("{}:{end}", zero_based(start)))
        }
        ["", end] if !end.is_empty() => Some(format!(
            ":{}",
            if assignment_target {
                end.to_string()
            } else {
                zero_based(end)
            }
        )),
        [start, ""] if !start.is_empty() => Some(format!("{}:", zero_based(start))),
        _ => None,
    }
}

/// Translate one scalar MATLAB linear subscript to its one-based XDL helper
/// argument. `end` is the total number of elements for linear indexing.
fn translate_matlab_linear_index(tokens: &[Token], array_name: &str) -> String {
    tokens
        .iter()
        .map(|token| match &token.kind {
            TokenKind::Identifier(name) if name.eq_ignore_ascii_case("end") => {
                format!("MATLAB_NUMEL({array_name})")
            }
            TokenKind::End => format!("MATLAB_NUMEL({array_name})"),
            _ => token.lexeme.clone(),
        })
        .collect()
}

fn offset_matlab_index(start: &str, array_name: &str) -> String {
    let expanded = expand_matlab_end(start, array_name);
    expanded.parse::<i64>().map_or_else(
        |_| format!("({expanded}) - 1"),
        |value| {
            value.checked_sub(1).map_or_else(
                || format!("({expanded}) - 1"),
                |zero_based| zero_based.to_string(),
            )
        },
    )
}

fn expand_matlab_end(expression: &str, array_name: &str) -> String {
    let mut result = String::with_capacity(expression.len());
    let mut chars = expression.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch.is_ascii_alphabetic() || ch == '_' {
            let mut identifier = String::from(ch);
            while chars
                .peek()
                .is_some_and(|next| next.is_ascii_alphanumeric() || *next == '_')
            {
                if let Some(next) = chars.next() {
                    identifier.push(next);
                }
            }
            if identifier == "end" {
                result.push_str(&format!("N_ELEMENTS({array_name})"));
            } else {
                result.push_str(&identifier);
            }
        } else {
            result.push(ch);
        }
    }
    result
}

/// Main transpilation function
pub fn transpile_matlab_to_xdl(matlab_code: &str) -> Result<String, String> {
    let mut lexer = Lexer::new(matlab_code);
    let tokens = lexer.tokenize()?;

    let mut transpiler = Transpiler::new(tokens);
    transpiler.transpile()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_assignment() {
        let matlab = "x = 5;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("x = 5"));
    }

    #[test]
    fn test_function_mapping() {
        let matlab = "y = sin(x);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("SIN"));
    }

    #[test]
    fn matlab_nonconjugating_transpose_maps_named_arrays_to_transpose() {
        let result = transpile_matlab_to_xdl("A = [1, 2; 3, 4]; B = A.';").unwrap();
        assert!(result.contains("B = TRANSPOSE(A)"), "{result}");
    }

    #[test]
    fn matlab_conjugating_transpose_remains_explicitly_unsupported() {
        let error = transpile_matlab_to_xdl("A = [1, 2; 3, 4]; B = A'").unwrap_err();
        assert!(error.contains("conjugating transpose"), "{error}");
    }

    #[test]
    fn test_simple_array_literal() {
        let matlab = "a = [1, 2, 3, 4, 5];";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("a = [1, 2, 3, 4, 5]"));
    }

    #[test]
    fn test_array_literal_with_spaces() {
        let matlab = "b = [1 2 3];";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("[1") && result.contains("2") && result.contains("3]"));
    }

    #[test]
    fn negative_array_elements_remain_signed_numbers() {
        let result = transpile_matlab_to_xdl("values = [0, 1, -1];").unwrap();
        assert!(result.contains("values = [0, 1, -1]"), "{result}");
    }

    #[test]
    fn test_column_vector() {
        let matlab = "c = [1; 2; 3];";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Should be represented as nested arrays
        assert!(result.contains("c = [[1], [2], [3]]"));
    }

    #[test]
    fn test_matrix_literal() {
        let matlab = "M = [1, 2, 3; 4, 5, 6];";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Should be represented as nested arrays: [[1, 2, 3], [4, 5, 6]]
        assert!(result.contains("M = [[1, 2, 3], [4, 5, 6]]"));
    }

    #[test]
    fn test_colon_range_simple() {
        let matlab = "x = 1:10;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Should convert to FINDGEN
        assert!(result.contains("FINDGEN"));
    }

    #[test]
    fn test_colon_range_with_step() {
        let matlab = "y = 0:0.1:1;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Should convert to FINDGEN with step
        assert!(result.contains("FINDGEN"));
    }

    #[test]
    fn matlab_vector_ranges_translate_to_zero_based_xdl_slices() {
        let result = transpile_matlab_to_xdl(
            "x = [10, 20, 30, 40, 50, 60, 70]; a = x(2:5); b = x(1:2:7); c = x(3);",
        )
        .unwrap();
        assert!(result.contains("a = x [1:5]"), "{result}");
        assert!(result.contains("b = x [0:7:2]"), "{result}");
        assert!(result.contains("c = MATLAB_INDEX(x, 3)"), "{result}");
    }

    #[test]
    fn matlab_vector_range_supports_variable_start() {
        let result = transpile_matlab_to_xdl("x = [1, 2, 3]; y = x(first:last);").unwrap();
        assert!(result.contains("y = x [(first) - 1:last]"), "{result}");
    }

    #[test]
    fn matlab_end_and_omitted_vector_bounds_map_to_xdl_ranges() {
        let result = transpile_matlab_to_xdl(
            "x = [10, 20, 30, 40, 50]; a = x(end); b = x(3:end); c = x(1:2:end); d = x(:); e = x(2:);",
        )
        .unwrap();
        assert!(
            result.contains("a = MATLAB_INDEX(x, MATLAB_NUMEL(x))"),
            "{result}"
        );
        assert!(result.contains("b = x [2:N_ELEMENTS(x)]"), "{result}");
        assert!(result.contains("c = x [0:N_ELEMENTS(x):2]"), "{result}");
        assert!(result.contains("d = x [:]"), "{result}");
        assert!(result.contains("e = x [1:]"), "{result}");
    }

    #[test]
    fn matlab_matrix_scalar_subscripts_translate_per_dimension() {
        let result = transpile_matlab_to_xdl(
            "matrix = [1, 2, 3; 4, 5, 6]; first = matrix(1, 2); last = matrix(2, 3); linear = matrix(2); matrix(2, 2:3) = [55, 66];",
        )
        .unwrap();
        assert!(result.contains("first = matrix [0, 1]"), "{result}");
        assert!(result.contains("last = matrix [1, 2]"), "{result}");
        assert!(
            result.contains("linear = MATLAB_INDEX(matrix, 2)"),
            "{result}"
        );
        assert!(result.contains("matrix [1, 1:3]= [55, 66]"), "{result}");
    }

    #[test]
    fn matlab_two_dimensional_range_reads_translate_with_inclusive_matlab_bounds() {
        let result = transpile_matlab_to_xdl(
            "matrix = [1, 2, 3; 4, 5, 6]; row = matrix(1, 2:3); column = matrix(1:2, 2); last_column = matrix(:, 3); last_row = matrix(2, :);",
        )
        .unwrap();
        assert!(result.contains("row = matrix [0, 1:2]"), "{result}");
        assert!(result.contains("column = matrix [0:1, 1]"), "{result}");
        assert!(result.contains("last_column = matrix [:, 2]"), "{result}");
        assert!(result.contains("last_row = matrix [1, :]"), "{result}");
    }

    #[test]
    fn matlab_two_axis_range_assignment_translates_both_axes_for_runtime_writes() {
        let result = transpile_matlab_to_xdl(
            "matrix = [1, 2, 3; 4, 5, 6]; matrix(1:2, 2:3) = [7, 8; 9, 10]; matrix(1:2, 2:3) = 5;",
        )
        .expect("transpile block assignment");
        assert!(
            result.contains("matrix [0:2, 1:3]= [[7, 8], [9, 10]]"),
            "{result}"
        );
        assert!(result.contains("matrix [0:2, 1:3]= 5"), "{result}");
    }

    #[test]
    fn matlab_scalar_indexed_assignment_translates_known_array_targets() {
        let result = transpile_matlab_to_xdl(
            "values = [10, 20, 30]; values(2) = 99; matrix = [1, 2, 3; 4, 5, 6]; matrix(2, 3) = 60; disp(values(2)); disp(matrix(2, 3));",
        )
        .unwrap();
        assert!(result.contains("values [1]= 99"), "{result}");
        assert!(result.contains("matrix [1, 2]= 60"), "{result}");
        assert!(
            result.contains("PRINT, MATLAB_INDEX(values, 2)"),
            "{result}"
        );
        assert!(result.contains("PRINT, matrix [1, 2]"), "{result}");
    }

    #[test]
    fn matlab_size_multiple_outputs_use_dimension_and_tail_helpers() {
        let result = transpile_matlab_to_xdl(
            "matrix = [1, 2, 3; 4, 5, 6]; [rows, columns] = size(matrix); [first, second, third] = size(matrix);",
        )
        .unwrap();
        assert!(result.contains("rows = MATLAB_SIZE(matrix, 1)"), "{result}");
        assert!(
            result.contains("columns = MATLAB_SIZE_TAIL(matrix, 2)"),
            "{result}"
        );
        assert!(
            result.contains("first = MATLAB_SIZE(matrix, 1)"),
            "{result}"
        );
        assert!(
            result.contains("second = MATLAB_SIZE(matrix, 2)"),
            "{result}"
        );
        assert!(
            result.contains("third = MATLAB_SIZE_TAIL(matrix, 3)"),
            "{result}"
        );
    }

    #[test]
    fn matlab_find_two_outputs_lower_for_known_matrix_variables() {
        let result =
            transpile_matlab_to_xdl("matrix = [0, 2; 3, 0]; [row, col] = find(matrix);").unwrap();
        assert!(
            result.contains("row = MATLAB_FIND_ROWS(matrix)"),
            "{result}"
        );
        assert!(
            result.contains("col = MATLAB_FIND_COLUMNS(matrix)"),
            "{result}"
        );
        let three_outputs =
            transpile_matlab_to_xdl("matrix = [0, 2; 3, 0]; [row, col, linear] = find(matrix);")
                .unwrap();
        assert!(
            three_outputs.contains("linear = MATLAB_FIND_INDICES(matrix)"),
            "{three_outputs}"
        );
        assert!(transpile_matlab_to_xdl("[a, b, c, d] = find(matrix);").is_err());
        let limited =
            transpile_matlab_to_xdl("matrix = [0, 2; 3, 0]; [row, col] = find(matrix, 2, 'last');")
                .unwrap();
        assert!(
            limited.contains("MATLAB_FIND_ROWS(matrix, 2, 'last')"),
            "{limited}"
        );
        assert!(
            limited.contains("MATLAB_FIND_COLUMNS(matrix, 2, 'last')"),
            "{limited}"
        );
    }

    #[test]
    fn range_arguments_to_unknown_functions_remain_function_calls() {
        let result =
            transpile_matlab_to_xdl("y = user_function(2:5); z = user_function(7);").unwrap();
        assert!(result.contains("user_function (FINDGEN"), "{result}");
        assert!(!result.contains("user_function ["), "{result}");
        assert!(result.contains("user_function (7 )"), "{result}");
    }

    #[test]
    fn matlab_function_array_parameters_support_scalar_and_range_subscripts() {
        let result =
            transpile_matlab_to_xdl("function y = select_values(x)\ny = x(3);\nz = x(2:4);\nend")
                .unwrap();
        assert!(result.contains("FUNCTION select_values, x"), "{result}");
        assert!(result.contains("y = MATLAB_INDEX(x, 3)"), "{result}");
        assert!(result.contains("z = x [1:4]"), "{result}");
        assert!(result.contains("RETURN, y"), "{result}");
    }

    #[test]
    fn matlab_multiple_function_outputs_fail_explicitly() {
        let result = transpile_matlab_to_xdl(
            "function [first, second] = split_values(x)\nfirst = x;\nsecond = x;\nend",
        );
        assert!(result.is_err());
        assert!(result
            .expect_err("multiple outputs are unsupported")
            .contains("multiple output variables are not supported"));
    }

    #[test]
    fn test_zeros_function() {
        let matlab = "z = zeros(5);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("FLTARR(5)"));
    }

    #[test]
    fn test_zeros_function_2d() {
        let matlab = "z = zeros(3, 4);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("FLTARR(3, 4)"));
    }

    #[test]
    fn test_ones_function() {
        let matlab = "o = ones(5);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("FLTARR(5) + 1"));
    }

    #[test]
    fn test_eye_function() {
        let matlab = "I = eye(4);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("IDENTITY(4)"));
    }

    #[test]
    fn test_linspace_function() {
        let matlab = "x = linspace(0, 10, 100);";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Should convert to FINDGEN expression
        assert!(result.contains("FINDGEN"));
    }

    #[test]
    fn test_array_with_range() {
        let matlab = "a = [1:5];";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        // Array containing a range
        assert!(result.contains("FINDGEN"));
    }

    #[test]
    fn test_array_element_operations() {
        let matlab = "result = a .* b;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        println!("Result: {}", result);
        // Element-wise multiply should convert to *
        // The actual format has spaces around operators
        assert!(result.contains("result = a * b") || result.contains("result = a  *  b"));
    }

    #[test]
    fn test_break_statement() {
        let matlab = "break;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("BREAK"));
    }

    #[test]
    fn test_continue_statement() {
        let matlab = "continue;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("CONTINUE"));
    }

    #[test]
    fn test_return_statement() {
        let matlab = "return;";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("RETURN"));
    }

    #[test]
    fn test_simple_switch() {
        let matlab = r#"switch x
            case 1
                y = 'one';
            case 2
                y = 'two';
            otherwise
                y = 'other';
        end"#;
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("CASE x OF"));
        assert!(result.contains("1: BEGIN"));
        assert!(result.contains("2: BEGIN"));
        assert!(result.contains("ELSE: BEGIN"));
        assert!(result.contains("ENDCASE"));
    }

    #[test]
    fn test_switch_with_cell_array() {
        let matlab = r#"switch x
            case {1, 2}
                y = 'one or two';
        end"#;
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("CASE x OF"));
        assert!(result.contains(": BEGIN"));
    }

    #[test]
    fn test_try_catch() {
        let matlab = r#"try
            result = risky_operation();
        catch err
            result = 0;
        end"#;
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("TRY block"));
        assert!(result.contains("CATCH block"));
    }

    #[test]
    fn test_for_loop_with_step() {
        let matlab = "for i = 1:2:10\n  disp(i);\nend";
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("for i ="));
        // Should have converted the range
        assert!(!result.contains("1:2:10"));
    }

    #[test]
    fn test_nested_control_flow() {
        let matlab = r#"for i = 1:5
            if i == 3
                continue;
            end
            disp(i);
        end"#;
        let result = transpile_matlab_to_xdl(matlab).unwrap();
        assert!(result.contains("for i"));
        assert!(result.contains("if"));
        assert!(result.contains("CONTINUE"));
    }
}

#[cfg(test)]
mod loop_and_random_regression_tests {
    use super::*;

    #[test]
    fn for_loop_bounds_keep_matlab_values() {
        let result = transpile_matlab_to_xdl("for i = 1:3\n  disp(i*0.1);\nend").unwrap();
        assert!(result.contains("for i = 1, 3"), "got: {result}");
        assert!(!result.contains("0, 2"), "bounds were shifted: {result}");
    }

    #[test]
    fn for_loop_step_keeps_matlab_values() {
        let result = transpile_matlab_to_xdl("for i = 1:2:9\n  disp(i);\nend").unwrap();
        assert!(result.contains("for i = 1, 9, 2"), "got: {result}");
    }

    #[test]
    fn randn_maps_to_normal_generator() {
        let result = transpile_matlab_to_xdl("y = randn(size(x));").unwrap();
        assert!(result.contains("RANDOMN"), "got: {result}");
        assert!(
            !result.contains("RANDOMU"),
            "randn must not be uniform: {result}"
        );
    }
}

#[cfg(test)]
mod plot_annotation_tests {
    use super::transpile_matlab_to_xdl;

    #[test]
    fn literal_labels_attach_to_the_immediately_preceding_plot() {
        let result = transpile_matlab_to_xdl(
            "x = [1, 2, 3];\ny = [2, 4, 8];\nplot(x, y);\ntitle('Response');\nxlabel('Time (s)');\nylabel('Amplitude');",
        )
        .unwrap();
        assert!(
            result.contains("PLOT, x, y, TITLE='Response', XTITLE='Time (s)', YTITLE='Amplitude'"),
            "got: {result}"
        );
    }

    #[test]
    fn labels_without_a_preceding_plot_are_rejected() {
        let error = transpile_matlab_to_xdl("title('Response');").unwrap_err();
        assert!(error.contains("must immediately follow"), "got: {error}");
    }
}
