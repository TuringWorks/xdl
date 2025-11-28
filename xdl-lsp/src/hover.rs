//! Hover information provider

use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind, Position};

use crate::document::DocumentState;
use crate::symbols::SymbolTable;

pub fn provide_hover(
    doc: &DocumentState,
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<Hover> {
    let word = doc.get_word_at_position(position.line, position.character)?;

    // Check if it's a system variable
    if word.starts_with('!') {
        if let Some(info) = symbol_table.get_system_variable(&word) {
            return Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!(
                        "**System Variable** `!{}`\n\n**Type:** `{}`\n\n{}",
                        info.name, info.type_info, info.documentation
                    ),
                }),
                range: None,
            });
        }
    }

    // Check built-in functions
    if let Some(info) = symbol_table.get_function(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**Function** `{}`\n\n```xdl\n{}\n```\n\n**Returns:** `{}`\n\n{}",
                    info.name, info.signature, info.return_type, info.documentation
                ),
            }),
            range: None,
        });
    }

    // Check built-in procedures
    if let Some(info) = symbol_table.get_procedure(&word) {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!(
                    "**Procedure** `{}`\n\n```xdl\n{}\n```\n\n{}",
                    info.name, info.signature, info.documentation
                ),
            }),
            range: None,
        });
    }

    // Check for keywords
    let keyword_info = get_keyword_info(&word.to_uppercase());
    if let Some((keyword, description)) = keyword_info {
        return Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value: format!("**Keyword** `{}`\n\n{}", keyword, description),
            }),
            range: None,
        });
    }

    // Check user-defined symbols in AST
    if let Some(ref ast) = doc.ast {
        for stmt in &ast.statements {
            if let Some(hover) = check_statement_for_symbol(stmt, &word) {
                return Some(hover);
            }
        }
    }

    None
}

fn get_keyword_info(keyword: &str) -> Option<(&'static str, &'static str)> {
    match keyword {
        "IF" => Some(("IF", "Conditional statement. Syntax: `IF condition THEN statement [ELSE statement]`")),
        "THEN" => Some(("THEN", "Introduces the statement(s) to execute when IF condition is true")),
        "ELSE" => Some(("ELSE", "Introduces the statement(s) to execute when IF condition is false")),
        "ENDIF" => Some(("ENDIF", "Marks the end of a multi-line IF statement")),
        "FOR" => Some(("FOR", "Counted loop. Syntax: `FOR var = start, end [, step] DO statement`")),
        "ENDFOR" => Some(("ENDFOR", "Marks the end of a FOR loop block")),
        "FOREACH" => Some(("FOREACH", "Iterator loop. Syntax: `FOREACH element, array [, index] DO statement`")),
        "WHILE" => Some(("WHILE", "Conditional loop. Syntax: `WHILE condition DO statement`")),
        "ENDWHILE" => Some(("ENDWHILE", "Marks the end of a WHILE loop block")),
        "REPEAT" => Some(("REPEAT", "Post-condition loop. Syntax: `REPEAT statement UNTIL condition`")),
        "UNTIL" => Some(("UNTIL", "Terminates REPEAT loop when condition becomes true")),
        "DO" => Some(("DO", "Introduces the body of a loop")),
        "BEGIN" => Some(("BEGIN", "Starts a compound statement block")),
        "END" => Some(("END", "Ends a compound statement block")),
        "BREAK" => Some(("BREAK", "Exit from the innermost loop")),
        "CONTINUE" => Some(("CONTINUE", "Skip to the next iteration of the innermost loop")),
        "RETURN" => Some(("RETURN", "Return from function/procedure, optionally with a value")),
        "FUNCTION" => Some(("FUNCTION", "Define a function. Syntax: `FUNCTION name, param1, param2, ...`")),
        "PRO" | "PROCEDURE" => Some(("PRO/PROCEDURE", "Define a procedure. Syntax: `PRO name, param1, param2, ...`")),
        "CASE" => Some(("CASE", "Multi-way branch. Syntax: `CASE expr OF value1: stmt1 ... ENDCASE`")),
        "SWITCH" => Some(("SWITCH", "Multi-way branch with fall-through. Syntax: `SWITCH expr OF value1: stmt1 ... ENDSWITCH`")),
        "COMMON" => Some(("COMMON", "Declare shared variables. Syntax: `COMMON block_name, var1, var2, ...`")),
        "AND" => Some(("AND", "Logical AND operator")),
        "OR" => Some(("OR", "Logical OR operator")),
        "NOT" => Some(("NOT", "Logical NOT operator")),
        "XOR" => Some(("XOR", "Logical exclusive OR operator")),
        "EQ" => Some(("EQ", "Equal comparison operator")),
        "NE" => Some(("NE", "Not equal comparison operator")),
        "LT" => Some(("LT", "Less than comparison operator")),
        "GT" => Some(("GT", "Greater than comparison operator")),
        "LE" => Some(("LE", "Less than or equal comparison operator")),
        "GE" => Some(("GE", "Greater than or equal comparison operator")),
        "MOD" => Some(("MOD", "Modulo (remainder) operator")),
        _ => None,
    }
}

fn check_statement_for_symbol(
    stmt: &xdl_parser::ast::Statement,
    target_name: &str,
) -> Option<Hover> {
    use xdl_parser::ast::Statement;

    match stmt {
        Statement::FunctionDef { name, params, .. } => {
            if name.eq_ignore_ascii_case(target_name) {
                let param_str = params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**User Function** `{}`\n\n```xdl\nFUNCTION {}, {}\n```",
                            name, name, param_str
                        ),
                    }),
                    range: None,
                });
            }
        }
        Statement::ProcedureDef { name, params, .. } => {
            if name.eq_ignore_ascii_case(target_name) {
                let param_str = params
                    .iter()
                    .map(|p| p.name.clone())
                    .collect::<Vec<_>>()
                    .join(", ");
                return Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: format!(
                            "**User Procedure** `{}`\n\n```xdl\nPRO {}, {}\n```",
                            name, name, param_str
                        ),
                    }),
                    range: None,
                });
            }
        }
        _ => {}
    }

    None
}
