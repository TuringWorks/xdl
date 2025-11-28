//! Document state management and parsing

use ropey::Rope;
use tower_lsp::lsp_types::Diagnostic;
use xdl_parser::ast::Program;

use crate::diagnostics;

#[derive(Debug)]
pub struct DocumentState {
    pub content: Rope,
    #[allow(dead_code)]
    pub version: i32,
    pub ast: Option<Program>,
    pub diagnostics: Vec<Diagnostic>,
}

impl DocumentState {
    pub fn parse(text: String, version: i32) -> Self {
        let content = Rope::from_str(&text);

        // Parse the document
        let parse_result = xdl_parser::parse_xdl(&text);

        let (ast, diagnostics) = match parse_result {
            Ok(program) => (Some(program), Vec::new()),
            Err(err) => {
                let diags = diagnostics::convert_error_to_diagnostics(&err);
                (None, diags)
            }
        };

        Self {
            content,
            version,
            ast,
            diagnostics,
        }
    }

    pub fn get_word_at_position(&self, line: u32, character: u32) -> Option<String> {
        let line_idx = line as usize;
        if line_idx >= self.content.len_lines() {
            return None;
        }

        let line_str = self.content.line(line_idx).to_string();
        let char_idx = character as usize;

        if char_idx >= line_str.len() {
            return None;
        }

        // Find word boundaries
        let chars: Vec<char> = line_str.chars().collect();

        // Find start of word
        let mut start = char_idx;
        while start > 0 && is_word_char(chars[start - 1]) {
            start -= 1;
        }

        // Check if we're starting with ! (system variable)
        if start > 0 && chars[start - 1] == '!' {
            start -= 1;
        }

        // Find end of word
        let mut end = char_idx;
        while end < chars.len() && is_word_char(chars[end]) {
            end += 1;
        }

        if start == end {
            return None;
        }

        Some(chars[start..end].iter().collect())
    }

    pub fn get_line(&self, line: u32) -> Option<String> {
        let line_idx = line as usize;
        if line_idx >= self.content.len_lines() {
            return None;
        }
        Some(self.content.line(line_idx).to_string())
    }
}

fn is_word_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
