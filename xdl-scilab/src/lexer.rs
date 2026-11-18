//! Scilab line lexer. Strings protect comment delimiters and percent constants.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub number: usize,
    pub kind: LineKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LineKind {
    Empty,
    Statement(String),
}

/// Remove Scilab `//` and `/* ... */` comments and translate known `%` constants.
pub fn lex(source: &str) -> Result<Vec<Line>, String> {
    let mut in_block_comment = false;
    let mut lines = Vec::new();
    for (index, source_line) in source.lines().enumerate() {
        let number = index + 1;
        let statement = scan_line(source_line, number, &mut in_block_comment)?;
        lines.push(Line {
            number,
            kind: if statement.trim().is_empty() {
                LineKind::Empty
            } else {
                LineKind::Statement(statement.trim().to_string())
            },
        });
    }
    if in_block_comment {
        return Err(format!(
            "line {}: unterminated Scilab block comment",
            lines.len()
        ));
    }
    Ok(lines)
}

fn scan_line(line: &str, number: usize, in_block_comment: &mut bool) -> Result<String, String> {
    let chars: Vec<char> = line.chars().collect();
    let mut out = String::with_capacity(line.len());
    let mut i = 0;
    while i < chars.len() {
        if *in_block_comment {
            if chars[i] == '*' && chars.get(i + 1) == Some(&'/') {
                *in_block_comment = false;
                i += 2;
            } else {
                i += 1;
            }
            continue;
        }

        let ch = chars[i];
        if ch == '\'' && is_transpose_suffix(&out) {
            let start = transpose_operand_start(&out)
                .ok_or_else(|| format!("line {number}: unsupported Scilab transpose operand"))?;
            out.insert_str(start, "TRANSPOSE(");
            out.push(')');
            i += 1;
            continue;
        }
        if ch == '\'' || ch == '"' {
            let quote = ch;
            out.push(ch);
            i += 1;
            let mut closed = false;
            while i < chars.len() {
                let current = chars[i];
                out.push(current);
                i += 1;
                if current == quote {
                    if i < chars.len() && chars[i] == quote {
                        out.push(chars[i]);
                        i += 1;
                    } else {
                        closed = true;
                        break;
                    }
                } else if current == '\\' && i < chars.len() {
                    out.push(chars[i]);
                    i += 1;
                }
            }
            if !closed {
                return Err(format!("line {number}: unterminated string literal"));
            }
            continue;
        }

        if ch == '%' {
            let end = (i + 1..chars.len())
                .find(|&j| !(chars[j].is_ascii_alphanumeric() || chars[j] == '_'))
                .unwrap_or(chars.len());
            let form: String = chars[i..end].iter().collect();
            match form.as_str() {
                "%pi" => out.push_str("!PI"),
                "%e" => out.push_str("!E"),
                "%t" | "%T" => out.push('1'),
                "%f" | "%F" => out.push('0'),
                _ => {
                    return Err(format!(
                        "line {number}: unsupported Scilab percent form {form}"
                    ))
                }
            }
            i = end;
            continue;
        }
        if ch == '/' && chars.get(i + 1) == Some(&'/') {
            break;
        }
        if ch == '/' && chars.get(i + 1) == Some(&'*') {
            *in_block_comment = true;
            i += 2;
            continue;
        }
        out.push(ch);
        i += 1;
    }
    Ok(out)
}

fn is_transpose_suffix(output: &str) -> bool {
    output
        .chars()
        .rev()
        .find(|ch| !ch.is_whitespace())
        .is_some_and(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | ']' | ')'))
}

fn transpose_operand_start(output: &str) -> Option<usize> {
    let chars: Vec<(usize, char)> = output.char_indices().collect();
    let end = chars.iter().rposition(|(_, ch)| !ch.is_whitespace())?;
    let last = chars[end].1;
    let start_index = if last == ']' {
        let mut depth = 0usize;
        (0..=end).rev().find(|&index| match chars[index].1 {
            ']' => {
                depth += 1;
                false
            }
            '[' => {
                depth = depth.saturating_sub(1);
                depth == 0
            }
            _ => false,
        })?
    } else if last == ')' {
        let mut depth = 0usize;
        (0..=end).rev().find(|&index| match chars[index].1 {
            ')' => {
                depth += 1;
                false
            }
            '(' => {
                depth = depth.saturating_sub(1);
                depth == 0
            }
            _ => false,
        })?
    } else {
        let mut index = end;
        while index > 0 && (chars[index - 1].1.is_ascii_alphanumeric() || chars[index - 1].1 == '_')
        {
            index -= 1;
        }
        index
    };
    Some(chars[start_index].0)
}
