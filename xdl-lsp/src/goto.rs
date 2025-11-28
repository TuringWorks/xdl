//! Go-to-definition and find references

use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Position, Range, Url};
use xdl_parser::ast::{Expression, Statement};

use crate::document::DocumentState;

pub fn goto_definition(
    doc: &DocumentState,
    position: Position,
    uri: &Url,
) -> Option<GotoDefinitionResponse> {
    let word = doc.get_word_at_position(position.line, position.character)?;

    if let Some(ref ast) = doc.ast {
        for stmt in &ast.statements {
            if let Some(location) = find_definition_in_statement(stmt, &word) {
                return Some(GotoDefinitionResponse::Scalar(Location {
                    uri: uri.clone(),
                    range: location,
                }));
            }
        }
    }

    None
}

pub fn find_references(
    doc: &DocumentState,
    position: Position,
    uri: &Url,
) -> Option<Vec<Location>> {
    let word = doc.get_word_at_position(position.line, position.character)?;

    let mut references = Vec::new();

    if let Some(ref ast) = doc.ast {
        for stmt in &ast.statements {
            find_references_in_statement(stmt, &word, uri, &mut references);
        }
    }

    if references.is_empty() {
        None
    } else {
        Some(references)
    }
}

fn find_definition_in_statement(stmt: &Statement, target: &str) -> Option<Range> {
    match stmt {
        Statement::FunctionDef { name, location, .. } => {
            if name.eq_ignore_ascii_case(target) {
                return Some(location_to_range(location));
            }
        }
        Statement::ProcedureDef { name, location, .. } => {
            if name.eq_ignore_ascii_case(target) {
                return Some(location_to_range(location));
            }
        }
        Statement::Assignment { target: expr, location, .. } => {
            if let Expression::Variable { name, .. } = expr {
                if name.eq_ignore_ascii_case(target) {
                    return Some(location_to_range(location));
                }
            }
        }
        Statement::For { variable, location, .. } => {
            if variable.eq_ignore_ascii_case(target) {
                return Some(location_to_range(location));
            }
        }
        Statement::Foreach { variable, location, .. } => {
            if variable.eq_ignore_ascii_case(target) {
                return Some(location_to_range(location));
            }
        }
        Statement::Label { name, location } => {
            if name.eq_ignore_ascii_case(target) {
                return Some(location_to_range(location));
            }
        }
        _ => {}
    }

    None
}

fn find_references_in_statement(stmt: &Statement, target: &str, uri: &Url, refs: &mut Vec<Location>) {
    match stmt {
        Statement::FunctionDef { name, body, location, .. } => {
            if name.eq_ignore_ascii_case(target) {
                refs.push(Location {
                    uri: uri.clone(),
                    range: location_to_range(location),
                });
            }
            for s in body {
                find_references_in_statement(s, target, uri, refs);
            }
        }
        Statement::ProcedureDef { name, body, location, .. } => {
            if name.eq_ignore_ascii_case(target) {
                refs.push(Location {
                    uri: uri.clone(),
                    range: location_to_range(location),
                });
            }
            for s in body {
                find_references_in_statement(s, target, uri, refs);
            }
        }
        Statement::Assignment { target: expr, value, location } => {
            if let Expression::Variable { name, .. } = expr {
                if name.eq_ignore_ascii_case(target) {
                    refs.push(Location {
                        uri: uri.clone(),
                        range: location_to_range(location),
                    });
                }
            }
            find_references_in_expression(value, target, uri, refs);
        }
        Statement::If { condition, then_block, else_block, .. } => {
            find_references_in_expression(condition, target, uri, refs);
            for s in then_block {
                find_references_in_statement(s, target, uri, refs);
            }
            if let Some(else_stmts) = else_block {
                for s in else_stmts {
                    find_references_in_statement(s, target, uri, refs);
                }
            }
        }
        Statement::For { variable, start, end, step, body, location } => {
            if variable.eq_ignore_ascii_case(target) {
                refs.push(Location {
                    uri: uri.clone(),
                    range: location_to_range(location),
                });
            }
            find_references_in_expression(start, target, uri, refs);
            find_references_in_expression(end, target, uri, refs);
            if let Some(s) = step {
                find_references_in_expression(s, target, uri, refs);
            }
            for s in body {
                find_references_in_statement(s, target, uri, refs);
            }
        }
        Statement::While { condition, body, .. } => {
            find_references_in_expression(condition, target, uri, refs);
            for s in body {
                find_references_in_statement(s, target, uri, refs);
            }
        }
        Statement::ProcedureCall { args, .. } => {
            for arg in args {
                find_references_in_expression(arg, target, uri, refs);
            }
        }
        Statement::Expression { expr, .. } => {
            find_references_in_expression(expr, target, uri, refs);
        }
        Statement::Return { value, .. } => {
            if let Some(v) = value {
                find_references_in_expression(v, target, uri, refs);
            }
        }
        _ => {}
    }
}

fn find_references_in_expression(expr: &Expression, target: &str, uri: &Url, refs: &mut Vec<Location>) {
    match expr {
        Expression::Variable { name, location } => {
            if name.eq_ignore_ascii_case(target) {
                refs.push(Location {
                    uri: uri.clone(),
                    range: location_to_range(location),
                });
            }
        }
        Expression::FunctionCall { name, args, location, .. } => {
            if name.eq_ignore_ascii_case(target) {
                refs.push(Location {
                    uri: uri.clone(),
                    range: location_to_range(location),
                });
            }
            for arg in args {
                find_references_in_expression(arg, target, uri, refs);
            }
        }
        Expression::Binary { left, right, .. } => {
            find_references_in_expression(left, target, uri, refs);
            find_references_in_expression(right, target, uri, refs);
        }
        Expression::Unary { expr, .. } => {
            find_references_in_expression(expr, target, uri, refs);
        }
        Expression::ArrayRef { array, indices, .. } => {
            find_references_in_expression(array, target, uri, refs);
            for idx in indices {
                match idx {
                    xdl_parser::ast::ArrayIndex::Single(e) => {
                        find_references_in_expression(e, target, uri, refs);
                    }
                    xdl_parser::ast::ArrayIndex::Range { start, end, step } => {
                        if let Some(s) = start {
                            find_references_in_expression(s, target, uri, refs);
                        }
                        if let Some(e) = end {
                            find_references_in_expression(e, target, uri, refs);
                        }
                        if let Some(st) = step {
                            find_references_in_expression(st, target, uri, refs);
                        }
                    }
                    xdl_parser::ast::ArrayIndex::All => {}
                }
            }
        }
        Expression::MethodCall { object, args, .. } => {
            find_references_in_expression(object, target, uri, refs);
            for arg in args {
                find_references_in_expression(arg, target, uri, refs);
            }
        }
        Expression::StructRef { object, .. } => {
            find_references_in_expression(object, target, uri, refs);
        }
        Expression::Ternary { condition, if_true, if_false, .. } => {
            find_references_in_expression(condition, target, uri, refs);
            find_references_in_expression(if_true, target, uri, refs);
            find_references_in_expression(if_false, target, uri, refs);
        }
        Expression::ArrayDef { elements, .. } => {
            for elem in elements {
                find_references_in_expression(elem, target, uri, refs);
            }
        }
        _ => {}
    }
}

fn location_to_range(location: &xdl_parser::ast::Location) -> Range {
    Range {
        start: Position {
            line: location.line.saturating_sub(1) as u32,
            character: location.column as u32,
        },
        end: Position {
            line: location.line.saturating_sub(1) as u32,
            character: (location.column + 10) as u32,
        },
    }
}
