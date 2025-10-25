//! MATLAB to XDL Transpiler
//!
//! Converts MATLAB syntax to XDL-compatible code

use crate::function_map::get_xdl_function;
use crate::lexer::{Lexer, Token, TokenKind};

pub struct Transpiler {
    tokens: Vec<Token>,
    position: usize,
    output: String,
    indent_level: usize,
    // Subplot state for tiledlayout
    subplot_rows: usize,
    subplot_cols: usize,
    current_tile: usize,
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
                TokenKind::Identifier(_) => {
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

        // Emit XDL function
        self.emit_line(&format!("FUNCTION {}", func_name));
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

        self.indent_level -= 1;
        self.emit_line("END");

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
        let range_expr = self.collect_expression_until_newline();

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

        let condition = self.collect_expression_until_newline();

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

        let condition = self.collect_expression_until_newline();

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
        let switch_expr = self.collect_expression_until_newline();

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
                self.emit_line("RETURN");
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
                "xlabel" | "ylabel" | "title" | "legend" | "grid" | "zlabel" => {
                    // These should be converted to PLOT keywords, but for now ignore
                    let cmd_name = name.clone();
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
                        "; ({} command - use PLOT keywords: title=, xtitle=, ytitle=)",
                        cmd_name
                    ));
                    return Ok(());
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

        let expr = self.collect_expression_until_newline();
        if !expr.trim().is_empty() {
            self.emit_line(&expr);
        }
        Ok(())
    }

    fn collect_expression_until_newline(&mut self) -> String {
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

                        // Convert 1-based to 0-based if it's a simple number
                        if let Ok(num) = index_expr.trim().parse::<i32>() {
                            expr.push_str(&format!("{}", num - 1));
                        } else {
                            expr.push_str(&format!("({}) - 1", index_expr));
                        }

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
                        // Both rand and randn map to RANDOMU for now (normal distribution not yet implemented)
                        let xdl_func = "RANDOMU";
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

                    if has_colon && paren_depth == 0 {
                        // This is a range expression like (0:L-1) or (1:2:10)
                        // Parse the range and convert to FINDGEN
                        let range_expr = self.parse_range_expression(&range_tokens);
                        expr.push_str(&range_expr);
                    } else {
                        // Not a range, restore position and just add the paren
                        self.position = start_pos;
                        self.advance();
                        expr.push('(');
                    }
                    continue;
                }
                TokenKind::ElementMultiply => expr.push_str(" * "),
                TokenKind::ElementDivide => expr.push_str(" / "),
                TokenKind::ElementPower => expr.push_str(" ^ "),
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

                // Check if RHS is primarily a colon expression (not complex)
                // Simple heuristic: count colons vs other operators
                let colon_count = rhs.matches(" : ").count();
                let has_other_ops = rhs.contains(" + ")
                    || rhs.contains(" * ")
                    || rhs.contains(" / ")
                    || rhs.contains("(")
                    || rhs.contains("[");

                if colon_count > 0 && !has_other_ops {
                    // This looks like a simple range: convert it
                    let converted = self.convert_range_to_findgen(rhs);
                    return format!("{} = {}", lhs, converted);
                }
            }
        }

        expr_trimmed.to_string()
    }

    fn convert_range(&self, range: &str) -> Result<String, String> {
        // Convert MATLAB range like 1:10 to XDL 0, 9
        // Or 1:2:10 to 0, 9, 2

        let parts: Vec<&str> = range.split(':').collect();

        match parts.len() {
            2 => {
                // start:end -> (start-1), (end-1)
                let start: i32 = parts[0].trim().parse().map_err(|_| "Invalid range start")?;
                let end: i32 = parts[1].trim().parse().map_err(|_| "Invalid range end")?;
                Ok(format!("{}, {}", start - 1, end - 1))
            }
            3 => {
                // start:step:end -> (start-1), (end-1), step
                let start: i32 = parts[0].trim().parse().map_err(|_| "Invalid range start")?;
                let step: i32 = parts[1].trim().parse().map_err(|_| "Invalid range step")?;
                let end: i32 = parts[2].trim().parse().map_err(|_| "Invalid range end")?;
                Ok(format!("{}, {}, {}", start - 1, end - 1, step))
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
                    if !current_element.trim().is_empty()
                        && current_element
                            .trim()
                            .chars()
                            .all(|c| c.is_ascii_digit() || c == '.' || c == '-')
                    {
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
