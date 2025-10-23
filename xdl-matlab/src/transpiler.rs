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
                TokenKind::Function | TokenKind::For | TokenKind::While | TokenKind::If => {
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

        // Convert MATLAB 1-based to XDL 0-based
        let xdl_range = self.convert_range(&range_expr)?;

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

    fn transpile_statement(&mut self) -> Result<(), String> {
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
                TokenKind::Identifier(name) => {
                    // Map MATLAB constants to XDL system variables
                    let mapped_name = match name.as_str() {
                        "pi" => "!PI",
                        "e" => "!E",
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
                        if args.len() >= 1 {
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
                                expr.push_str(&imag_part);
                            } else {
                                // Use real part for now
                                expr.push_str(&real_part);
                            }
                        } else if args.len() == 1 {
                            // Just real part
                            expr.push_str(&args[0]);
                        } else {
                            expr.push_str("/* complex() needs 1-2 args */");
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
                TokenKind::LeftBracket => {
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

        expr.trim().to_string()
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
                format!("/* unhandled range expression */")
            }
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
}
