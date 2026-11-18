use crate::lexer::{lex, LineKind};
use std::collections::{HashMap, HashSet};

pub fn transpile_scilab_to_xdl(source: &str) -> Result<String, String> {
    let lines = lex(source)?;
    let statements: Vec<(usize, String)> = lines
        .into_iter()
        .filter_map(|line| match line.kind {
            LineKind::Empty => None,
            LineKind::Statement(text) => Some((line.number, text)),
        })
        .collect();
    let mut cursor = 0;
    let mut indexed_variables = HashMap::new();
    let mut function_definitions = Vec::new();
    let mut function_names = HashSet::new();
    let body = parse_block(
        &statements,
        &mut cursor,
        false,
        &mut indexed_variables,
        &mut function_definitions,
        &mut function_names,
    )?;
    if cursor != statements.len() {
        let (line, text) = &statements[cursor];
        return Err(format!("line {line}: unexpected '{text}'"));
    }
    Ok(format!("{}{}", function_definitions.concat(), body))
}

fn parse_block(
    lines: &[(usize, String)],
    cursor: &mut usize,
    nested: bool,
    indexed_variables: &mut HashMap<String, usize>,
    function_definitions: &mut Vec<String>,
    function_names: &mut HashSet<String>,
) -> Result<String, String> {
    let mut output = String::new();
    while *cursor < lines.len() {
        let (line_no, text) = &lines[*cursor];
        let lower = text.to_ascii_lowercase();
        if lower == "endwhile" {
            return Err(format!("line {line_no}: unexpected '{text}'"));
        }
        if lower.starts_with("function ") {
            if nested {
                return Err(format!(
                    "line {line_no}: Scilab function definitions cannot be nested"
                ));
            }
            let (name, function, consumed) = parse_function(lines, *cursor)?;
            let normalized_name = name.to_ascii_lowercase();
            if !function_names.insert(normalized_name) {
                return Err(format!(
                    "line {line_no}: duplicate Scilab function name '{name}'"
                ));
            }
            *cursor = consumed;
            function_definitions.push(function);
            continue;
        }
        if lower == "endfunction" {
            return Err(format!("line {line_no}: unexpected 'endfunction'"));
        }
        if matches!(lower.as_str(), "else" | "end") || lower.starts_with("elseif ") {
            if nested {
                break;
            }
            return Err(format!("line {line_no}: unexpected '{text}'"));
        }
        if lower.starts_with("if ") {
            let raw_condition = text.get(3..).unwrap_or("");
            let condition = raw_condition
                .get(..raw_condition.len().saturating_sub(5))
                .filter(|_| raw_condition.to_ascii_lowercase().ends_with(" then"))
                .ok_or_else(|| format!("line {line_no}: Scilab if requires 'then'"))?;
            validate_expression(condition, *line_no)?;
            *cursor += 1;
            output.push_str(&format!(
                "if {} then\n",
                translate_expression(condition, *line_no, indexed_variables)?
            ));
            output.push_str(&parse_block(
                lines,
                cursor,
                true,
                indexed_variables,
                function_definitions,
                function_names,
            )?);
            let mut saw_else = false;
            let mut lowered_elseif_count = 0usize;
            let mut consumed_end = false;
            while *cursor < lines.len() {
                let (branch_line, branch) = &lines[*cursor];
                let branch_lower = branch.to_ascii_lowercase();
                if branch_lower.starts_with("elseif ") {
                    if saw_else {
                        return Err(format!("line {branch_line}: elseif after else"));
                    }
                    let raw_condition = branch.get(7..).unwrap_or("");
                    let cond = raw_condition
                        .get(..raw_condition.len().saturating_sub(5))
                        .filter(|_| raw_condition.to_ascii_lowercase().ends_with(" then"))
                        .ok_or_else(|| {
                            format!("line {branch_line}: Scilab elseif requires 'then'")
                        })?;
                    validate_expression(cond, *branch_line)?;
                    *cursor += 1;
                    output.push_str(&format!(
                        "else\nif {} then\n",
                        translate_expression(cond, *branch_line, indexed_variables)?
                    ));
                    output.push_str(&parse_block(
                        lines,
                        cursor,
                        true,
                        indexed_variables,
                        function_definitions,
                        function_names,
                    )?);
                    lowered_elseif_count += 1;
                } else if branch_lower == "else" {
                    if saw_else {
                        return Err(format!("line {branch_line}: duplicate else"));
                    }
                    saw_else = true;
                    *cursor += 1;
                    output.push_str("else\n");
                    output.push_str(&parse_block(
                        lines,
                        cursor,
                        true,
                        indexed_variables,
                        function_definitions,
                        function_names,
                    )?);
                } else if branch_lower == "end" {
                    *cursor += 1;
                    consumed_end = true;
                    for _ in 0..lowered_elseif_count {
                        output.push_str("endif\n");
                    }
                    output.push_str("endif\n");
                    break;
                } else {
                    return Err(format!("line {branch_line}: expected elseif, else, or end"));
                }
            }
            if !consumed_end {
                return Err(format!("line {line_no}: missing 'end' for if block"));
            }
            continue;
        }
        if lower.starts_with("for ") {
            let (variable, start, step, stop) = parse_for_header(text, *line_no)?;
            *cursor += 1;
            output.push_str(&format!(
                "FOR {variable} = {start}, {stop}, {step} DO BEGIN\n"
            ));
            output.push_str(&parse_block(
                lines,
                cursor,
                true,
                indexed_variables,
                function_definitions,
                function_names,
            )?);
            if *cursor >= lines.len() || !lines[*cursor].1.eq_ignore_ascii_case("end") {
                return Err(format!("line {line_no}: missing 'end' for Scilab for loop"));
            }
            *cursor += 1;
            output.push_str("ENDFOR\n");
            continue;
        }
        if lower.starts_with("while ") {
            let condition = parse_while_header(text, *line_no)?;
            validate_expression(condition, *line_no)?;
            *cursor += 1;
            output.push_str(&format!(
                "while {} do begin\n",
                translate_expression(condition, *line_no, indexed_variables)?
            ));
            output.push_str(&parse_block(
                lines,
                cursor,
                true,
                indexed_variables,
                function_definitions,
                function_names,
            )?);
            if *cursor >= lines.len() || !lines[*cursor].1.eq_ignore_ascii_case("end") {
                return Err(format!(
                    "line {line_no}: missing 'end' for Scilab while loop"
                ));
            }
            *cursor += 1;
            output.push_str("endwhile\n");
            continue;
        }
        *cursor += 1;
        let statement = text.strip_suffix(';').unwrap_or(text).trim();
        if let Some(expression) = display_expression(statement) {
            validate_expression(expression, *line_no)?;
            output.push_str(&format!(
                "PRINT, {}\n",
                translate_expression(expression, *line_no, indexed_variables)?
            ));
            continue;
        }
        validate_assignment(statement, *line_no)?;
        let (lhs, rhs) = statement
            .split_once('=')
            .ok_or_else(|| format!("line {line_no}: expected assignment"))?;
        let name = lhs.trim();
        let translated = translate_expression(rhs.trim(), *line_no, indexed_variables)?;
        output.push_str(&format!("{name} = {translated}"));
        output.push('\n');
        if let Some(dimensions) = numeric_literal_dimensions(rhs.trim()) {
            indexed_variables.insert(name.to_string(), dimensions);
        } else {
            indexed_variables.remove(name);
        }
    }
    Ok(output)
}

fn parse_function(
    lines: &[(usize, String)],
    start: usize,
) -> Result<(String, String, usize), String> {
    let (line, header) = &lines[start];
    let invalid = || {
        format!(
            "line {line}: unsupported Scilab function header; expected `function output = name(input1, ..., input4)` or `function output = name()`"
        )
    };
    let declaration = header.get(9..).ok_or_else(invalid)?.trim();
    let (output_name, signature) = declaration.split_once('=').ok_or_else(invalid)?;
    let output_name = output_name.trim();
    if !is_identifier(output_name) {
        return Err(invalid());
    }
    let open = signature.find('(').ok_or_else(invalid)?;
    let close = signature.rfind(')').ok_or_else(invalid)?;
    if close != signature.len() - 1 || close < open {
        return Err(invalid());
    }
    let function_name = signature[..open].trim();
    let parameters = signature[open + 1..close].trim();
    if !is_identifier(function_name) {
        return Err(invalid());
    }
    let parameter_names: Vec<&str> = if parameters.is_empty() {
        Vec::new()
    } else {
        parameters.split(',').map(str::trim).collect()
    };
    if parameter_names.len() > 4 {
        return Err(invalid());
    }
    if parameter_names
        .iter()
        .any(|parameter| !is_identifier(parameter))
    {
        return Err(format!(
            "line {line}: malformed Scilab function parameter list"
        ));
    }
    let mut seen_parameters = std::collections::HashSet::with_capacity(parameter_names.len());
    for parameter in &parameter_names {
        if !seen_parameters.insert(*parameter) {
            return Err(format!(
                "line {line}: duplicate Scilab function parameter '{parameter}'"
            ));
        }
        if *parameter == output_name {
            return Err(format!(
                "line {line}: Scilab function output '{output_name}' cannot also be an input parameter"
            ));
        }
    }

    let signature = format!(
        "FUNCTION {function_name}{}\n",
        if parameter_names.is_empty() {
            String::new()
        } else {
            format!(", {}", parameter_names.join(", "))
        }
    );
    let mut body = signature;
    let mut output_assigned = false;
    let mut indexed_variables = HashMap::new();
    let mut cursor = start + 1;
    while cursor < lines.len() {
        let (body_line, text) = &lines[cursor];
        if text.eq_ignore_ascii_case("endfunction") {
            if !output_assigned {
                return Err(format!(
                    "line {body_line}: Scilab function must assign its output '{output_name}'"
                ));
            }
            body.push_str(&format!("RETURN, {output_name}\nENDFUNCTION\n"));
            return Ok((function_name.to_string(), body, cursor + 1));
        }
        let lower = text.to_ascii_lowercase();
        if lower == "end"
            || lower.starts_with("if ")
            || lower.starts_with("for ")
            || lower.starts_with("while ")
            || lower.starts_with("function ")
            || lower.starts_with("disp(")
        {
            return Err(format!(
                "line {body_line}: unsupported Scilab function body; only straight-line assignments are supported"
            ));
        }
        let statement = text.strip_suffix(';').unwrap_or(text).trim();
        validate_assignment(statement, *body_line).map_err(|_| {
            format!(
                "line {body_line}: unsupported Scilab function body; only straight-line assignments are supported"
            )
        })?;
        let (name, expression) = statement.split_once('=').ok_or_else(|| {
            format!(
                "line {body_line}: unsupported Scilab function body; only straight-line assignments are supported"
            )
        })?;
        let name = name.trim();
        let raw_expression = expression.trim();
        let expression = translate_expression(raw_expression, *body_line, &indexed_variables)?;
        body.push_str(&format!("{name} = {expression}\n"));
        if let Some(dimensions) = numeric_literal_dimensions(raw_expression) {
            indexed_variables.insert(name.to_string(), dimensions);
        } else {
            indexed_variables.remove(name);
        }
        if name == output_name {
            output_assigned = true;
        }
        cursor += 1;
    }
    Err(format!(
        "line {line}: missing 'endfunction' for Scilab function"
    ))
}

fn is_identifier(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        && !name.chars().next().is_some_and(|ch| ch.is_ascii_digit())
}

fn parse_while_header(text: &str, line: usize) -> Result<&str, String> {
    let unsupported = || {
        format!(
            "line {line}: unsupported Scilab while header; expected `while expression` with optional same-line `do` or `then`"
        )
    };
    let raw = text.get(6..).ok_or_else(unsupported)?.trim();
    let lower = raw.to_ascii_lowercase();
    let condition = ["do", "then"]
        .iter()
        .find_map(|keyword| {
            lower
                .strip_suffix(keyword)
                .filter(|prefix| prefix.len() < raw.len())
                .map(|prefix| raw[..prefix.len()].trim_end())
        })
        .unwrap_or(raw);
    if condition.is_empty()
        || condition.contains(',')
        || condition.contains(';')
        || condition.to_ascii_lowercase().contains(" else")
    {
        return Err(unsupported());
    }
    Ok(condition)
}

fn parse_for_header(text: &str, line: usize) -> Result<(String, String, String, String), String> {
    let unsupported = || {
        format!(
            "line {line}: unsupported Scilab for header; expected `for i = start:stop` or `for i = start:step:stop` with signed 32-bit integer literal bounds"
        )
    };
    let header = text.get(4..).ok_or_else(unsupported)?.trim();
    let (variable, range) = header.split_once('=').ok_or_else(unsupported)?;
    let variable = variable.trim();
    if variable.is_empty()
        || !variable
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        || variable
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit())
    {
        return Err(unsupported());
    }
    let parts: Vec<_> = range.split(':').map(str::trim).collect();
    let (start, step, stop) = match parts.as_slice() {
        [start, stop] => (*start, "1", *stop),
        [start, step, stop] => (*start, *step, *stop),
        _ => return Err(unsupported()),
    };
    let start_value = start.parse::<i32>().map_err(|_| unsupported())?;
    let step_value = step.parse::<i32>().map_err(|_| unsupported())?;
    let stop_value = stop.parse::<i32>().map_err(|_| unsupported())?;
    if step_value == 0 {
        return Err(format!("line {line}: Scilab for loop step must be nonzero"));
    }
    let iterations = if (step_value > 0 && start_value <= stop_value)
        || (step_value < 0 && start_value >= stop_value)
    {
        (i64::from(stop_value) - i64::from(start_value))
            .abs()
            .checked_div(i64::from(step_value).abs())
            .and_then(|count| count.checked_add(1))
            .ok_or_else(|| format!("line {line}: invalid Scilab for loop range"))?
    } else {
        0
    };
    if iterations > 1_000_000 {
        return Err(format!(
            "line {line}: Scilab for loop exceeds the 1,000,000 iteration limit"
        ));
    }
    Ok((
        variable.to_string(),
        start_value.to_string(),
        step_value.to_string(),
        stop_value.to_string(),
    ))
}

fn numeric_literal_dimensions(text: &str) -> Option<usize> {
    if !(text.starts_with('[') && text.ends_with(']')) {
        return None;
    }
    let content = &text[1..text.len().checked_sub(1)?];
    if content.contains(';') {
        Some(2)
    } else if content.contains(',')
        && content
            .split(',')
            .all(|value| is_numeric_literal(value.trim()))
    {
        Some(1)
    } else {
        None
    }
}

fn translate_expression(
    text: &str,
    line: usize,
    indexed_variables: &HashMap<String, usize>,
) -> Result<String, String> {
    let operators = translate_operators(text);
    let chars: Vec<char> = operators.chars().collect();
    let mut output = String::with_capacity(operators.len());
    let mut index = 0;
    while index < chars.len() {
        let ch = chars[index];
        if ch == '\'' || ch == '"' {
            let quote = ch;
            output.push(ch);
            index += 1;
            while index < chars.len() {
                let current = chars[index];
                output.push(current);
                index += 1;
                if current == quote {
                    if index < chars.len() && chars[index] == quote {
                        output.push(chars[index]);
                        index += 1;
                    } else {
                        break;
                    }
                }
            }
            continue;
        }
        if ch.is_ascii_alphabetic() || ch == '_' {
            let start = index;
            index += 1;
            while index < chars.len()
                && (chars[index].is_ascii_alphanumeric() || chars[index] == '_')
            {
                index += 1;
            }
            let name: String = chars[start..index].iter().collect();
            let mut open = index;
            while open < chars.len() && chars[open].is_whitespace() {
                open += 1;
            }
            if let Some(&dimensions) = indexed_variables.get(&name) {
                if chars.get(open) != Some(&'(') {
                    output.push_str(&name);
                    continue;
                }
                let mut depth = 0usize;
                let close = (open..chars.len())
                    .find(|&position| match chars[position] {
                        '(' => {
                            depth += 1;
                            false
                        }
                        ')' => {
                            depth -= 1;
                            depth == 0
                        }
                        _ => false,
                    })
                    .ok_or_else(|| {
                        format!("line {line}: indexed read requires integer literal indices")
                    })?;
                let args: String = chars[open + 1..close].iter().collect();
                let indices: Vec<&str> = args.split(',').map(str::trim).collect();
                if indices.len() != dimensions {
                    return Err(format!("line {line}: Scilab {dimensions}D literal reads require {dimensions} positive integer literal index{}", if dimensions == 1 { "" } else { "es" }));
                }
                let translated_indices: Vec<String> = indices
                    .iter()
                    .map(|value| {
                        if dimensions == 1 && value.contains(':') {
                            translate_vector_range(name.as_str(), value, line, indexed_variables)
                        } else if dimensions == 1 {
                            if is_numeric_literal(value) {
                                let literal = value.parse::<f64>().map_err(|_| {
                                    format!("line {line}: Scilab vector index must be a positive integer")
                                })?;
                                if literal < 1.0
                                    || literal.fract() != 0.0
                                    || literal > i32::MAX as f64 + 1.0
                                {
                                    return Err(format!(
                                        "line {line}: Scilab vector index must be a positive integer representable by XDL"
                                    ));
                                }
                                return Ok(((literal - 1.0) as i32).to_string());
                            }
                            validate_expression(value, line)?;
                            Ok(format!(
                                "SCILAB_INDEX({})",
                                translate_expression(value, line, indexed_variables)?
                            ))
                        } else {
                            let parsed = value.parse::<usize>().map_err(|_| {
                                format!("line {line}: Scilab indexed reads require positive integer literal indices or a positive-literal vector range")
                            })?;
                            if parsed == 0 {
                                return Err(format!(
                                    "line {line}: Scilab indices are one-based and must be positive"
                                ));
                            }
                            Ok((parsed - 1).to_string())
                        }
                    })
                    .collect::<Result<_, _>>()?;
                if dimensions == 1 && indices[0].contains(':') {
                    output.push_str(&translated_indices[0]);
                } else {
                    output.push_str(&name);
                    output.push('[');
                    output.push_str(&translated_indices.join(", "));
                    output.push(']');
                }
                index = close + 1;
                continue;
            }
            output.push_str(&name);
            continue;
        }
        output.push(ch);
        index += 1;
    }
    Ok(output)
}

fn translate_vector_range(
    vector: &str,
    range: &str,
    line: usize,
    indexed_variables: &HashMap<String, usize>,
) -> Result<String, String> {
    let parts: Vec<_> = range.split(':').map(str::trim).collect();
    let unsupported = || {
        format!(
            "line {line}: Scilab vector ranges require positive integer literal start, end, and optional step"
        )
    };
    let (start, step, end) = match parts.as_slice() {
        [start, end] => (*start, None, *end),
        [start, step, end] => (*start, Some(*step), *end),
        _ => return Err(unsupported()),
    };
    let lower_component = |value: &str| -> Result<String, String> {
        if is_numeric_literal(value) {
            let integer = value
                .parse::<usize>()
                .ok()
                .filter(|number| *number > 0 && *number <= i32::MAX as usize + 1);
            return integer
                .map(|number| number.to_string())
                .ok_or_else(unsupported);
        }
        validate_expression(value, line)?;
        Ok(translate_expression(value, line, indexed_variables)?)
    };
    let start = lower_component(start)?;
    let end = lower_component(end)?;
    let step = step.map(lower_component).transpose()?;
    let mut args = vec![
        vector.to_string(),
        start,
        step.unwrap_or_else(|| "1".to_string()),
        end,
    ];
    Ok(format!(
        "SCILAB_INDEX_RANGE({})",
        args.drain(..).collect::<Vec<_>>().join(", ")
    ))
}

fn display_expression(statement: &str) -> Option<&str> {
    let name = statement.get(..4)?;
    if !name.eq_ignore_ascii_case("disp")
        || statement.chars().nth(4)? != '('
        || !statement.ends_with(')')
    {
        return None;
    }
    let expression = &statement[5..statement.len() - 1];
    let mut depth = 0_i32;
    let mut quote = None;
    for (index, ch) in expression.char_indices() {
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => quote = Some(ch),
            '(' => depth += 1,
            ')' => depth -= 1,
            _ => (),
        }
        if depth < 0 || (ch == ')' && depth == 0 && index + ch.len_utf8() != expression.len()) {
            return None;
        }
    }
    (quote.is_none() && depth == 0 && !expression.trim().is_empty()).then_some(expression.trim())
}

fn validate_assignment(text: &str, line: usize) -> Result<(), String> {
    let Some((lhs, rhs)) = text.split_once('=') else {
        return Err(format!(
            "line {line}: unsupported Scilab syntax '{text}' (expected assignment)"
        ));
    };
    if lhs.trim().contains('(') {
        return Err(format!(
            "line {line}: indexed assignment targets are unsupported in the Scilab subset"
        ));
    }
    if lhs.trim().is_empty()
        || !lhs
            .trim()
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        || lhs
            .trim()
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_digit())
    {
        return Err(format!(
            "line {line}: unsupported assignment target '{}'",
            lhs.trim()
        ));
    }
    validate_expression(rhs, line)
}

fn validate_expression(text: &str, line: usize) -> Result<(), String> {
    validate_numeric_matrix_literals(text, line)?;
    let mut depth = 0_i32;
    let mut in_string = None;
    for ch in text.chars() {
        if let Some(quote) = in_string {
            if ch == quote {
                in_string = None;
            }
            continue;
        }
        match ch {
            '\'' | '"' => in_string = Some(ch),
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth < 0 {
                    return Err(format!("line {line}: unbalanced expression"));
                }
            }
            ':' if depth > 0 => (),
            c if c.is_ascii_alphanumeric()
                || c.is_whitespace()
                || "_+-*/^.=<>!&|~[],;".contains(c) =>
            {
                ()
            }
            _ => {
                return Err(format!(
                    "line {line}: unsupported character '{ch}' in expression"
                ))
            }
        }
    }
    if in_string.is_some() || depth != 0 || text.trim().is_empty() {
        return Err(format!("line {line}: malformed expression"));
    }
    Ok(())
}

fn validate_numeric_matrix_literals(text: &str, line: usize) -> Result<(), String> {
    let chars: Vec<char> = text.chars().collect();
    let mut index = 0;
    let mut quote = None;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(delimiter) = quote {
            if ch == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            index += 1;
            continue;
        }
        if ch != '[' {
            index += 1;
            continue;
        }
        let start = index + 1;
        let Some(relative_end) = chars[start..].iter().position(|&item| item == ']') else {
            return Err(format!("line {line}: unterminated numeric matrix literal"));
        };
        let end = start + relative_end;
        let content: String = chars[start..end].iter().collect();
        if content.contains(';') {
            let rows: Vec<Vec<&str>> = content
                .split(';')
                .map(|row| row.split(',').map(str::trim).collect())
                .collect();
            let width = rows.first().map_or(0, Vec::len);
            if rows.len() < 2
                || width == 0
                || rows.iter().any(|row| {
                    row.len() != width || row.iter().any(|value| !is_numeric_literal(value))
                })
            {
                return Err(format!(
                    "line {line}: matrix literals must have rectangular rows of numeric comma-separated values"
                ));
            }
        }
        index = end + 1;
    }
    Ok(())
}

fn is_numeric_literal(value: &str) -> bool {
    let value = value.trim();
    if value.is_empty() {
        return false;
    }
    let unsigned = value.strip_prefix(['+', '-']).unwrap_or(value);
    if unsigned.is_empty() {
        return false;
    }
    unsigned.parse::<f64>().is_ok_and(f64::is_finite)
}

fn lower_matrix_literals(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    let mut quote = None;
    while index < chars.len() {
        let ch = chars[index];
        if let Some(delimiter) = quote {
            output.push(ch);
            if ch == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        if matches!(ch, '\'' | '"') {
            quote = Some(ch);
            output.push(ch);
            index += 1;
            continue;
        }
        if ch != '[' {
            output.push(ch);
            index += 1;
            continue;
        }
        let start = index + 1;
        let Some(relative_end) = chars[start..].iter().position(|&item| item == ']') else {
            output.push(ch);
            index += 1;
            continue;
        };
        let end = start + relative_end;
        let content: String = chars[start..end].iter().collect();
        if content.contains(';') {
            let rows = content
                .split(';')
                .map(|row| {
                    format!(
                        "[{}]",
                        row.split(',').map(str::trim).collect::<Vec<_>>().join(", ")
                    )
                })
                .collect::<Vec<_>>();
            output.push('[');
            output.push_str(&rows.join(", "));
            output.push(']');
        } else {
            output.extend(chars[index..=end].iter());
        }
        index = end + 1;
    }
    output
}

fn translate_operators(text: &str) -> String {
    let lowered = lower_matrix_literals(text);
    let chars: Vec<char> = lowered.chars().collect();
    let mut output = String::with_capacity(text.len());
    let mut index = 0;
    while index < chars.len() {
        if chars[index] == '\'' || chars[index] == '"' {
            let quote = chars[index];
            output.push(quote);
            index += 1;
            while index < chars.len() {
                let ch = chars[index];
                output.push(ch);
                index += 1;
                if ch == quote {
                    if index < chars.len() && chars[index] == quote {
                        output.push(chars[index]);
                        index += 1;
                    } else {
                        break;
                    }
                }
            }
            continue;
        }
        let pair = chars
            .get(index..index + 2)
            .map(|s| s.iter().collect::<String>());
        let translated = match pair.as_deref() {
            Some("==") => Some("EQ"),
            Some("~=") => Some("NE"),
            Some(">=") => Some("GE"),
            Some("<=") => Some("LE"),
            _ => None,
        };
        if let Some(operator) = translated {
            output.push_str(operator);
            index += 2;
            continue;
        }
        match chars[index] {
            '>' => output.push_str("GT"),
            '<' => output.push_str("LT"),
            ch => output.push(ch),
        }
        index += 1;
    }
    output
}
