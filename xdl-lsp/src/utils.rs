//! Utility functions for the LSP server

use tower_lsp::lsp_types::{Position, Range};

/// Convert byte offset to LSP Position
pub fn offset_to_position(text: &str, offset: usize) -> Position {
    let mut line = 0u32;
    let mut character = 0u32;

    for (i, c) in text.char_indices() {
        if i >= offset {
            break;
        }
        if c == '\n' {
            line += 1;
            character = 0;
        } else {
            character += 1;
        }
    }

    Position { line, character }
}

/// Convert LSP Position to byte offset
pub fn position_to_offset(text: &str, position: Position) -> Option<usize> {
    let mut current_line = 0u32;
    let mut current_char = 0u32;

    for (i, c) in text.char_indices() {
        if current_line == position.line && current_char == position.character {
            return Some(i);
        }
        if c == '\n' {
            if current_line == position.line {
                // Position is beyond end of line
                return Some(i);
            }
            current_line += 1;
            current_char = 0;
        } else {
            current_char += 1;
        }
    }

    // Position at end of file
    if current_line == position.line {
        Some(text.len())
    } else {
        None
    }
}

/// Create a Range from start and end byte offsets
pub fn offsets_to_range(text: &str, start: usize, end: usize) -> Range {
    Range {
        start: offset_to_position(text, start),
        end: offset_to_position(text, end),
    }
}

/// Check if a position is within a range
pub fn position_in_range(position: Position, range: Range) -> bool {
    if position.line < range.start.line || position.line > range.end.line {
        return false;
    }
    if position.line == range.start.line && position.character < range.start.character {
        return false;
    }
    if position.line == range.end.line && position.character > range.end.character {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_offset_to_position() {
        let text = "line1\nline2\nline3";

        assert_eq!(offset_to_position(text, 0), Position { line: 0, character: 0 });
        assert_eq!(offset_to_position(text, 3), Position { line: 0, character: 3 });
        assert_eq!(offset_to_position(text, 6), Position { line: 1, character: 0 });
        assert_eq!(offset_to_position(text, 8), Position { line: 1, character: 2 });
    }

    #[test]
    fn test_position_to_offset() {
        let text = "line1\nline2\nline3";

        assert_eq!(position_to_offset(text, Position { line: 0, character: 0 }), Some(0));
        assert_eq!(position_to_offset(text, Position { line: 0, character: 3 }), Some(3));
        assert_eq!(position_to_offset(text, Position { line: 1, character: 0 }), Some(6));
        assert_eq!(position_to_offset(text, Position { line: 1, character: 2 }), Some(8));
    }
}
