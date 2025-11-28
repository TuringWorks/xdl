//! Completion provider for XDL

use tower_lsp::lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, Documentation, InsertTextFormat,
    MarkupContent, MarkupKind, Position,
};

use crate::document::DocumentState;
use crate::symbols::SymbolTable;

pub fn provide_completions(
    doc: &DocumentState,
    position: Position,
    symbol_table: &SymbolTable,
) -> Option<CompletionResponse> {
    let line = doc.get_line(position.line)?;
    let char_idx = position.character as usize;

    // Check what triggered completion
    let trigger_char = if char_idx > 0 {
        line.chars().nth(char_idx - 1)
    } else {
        None
    };

    let items = match trigger_char {
        Some('!') => {
            // System variable completion
            symbol_table
                .system_variables
                .values()
                .map(|info| CompletionItem {
                    label: format!("!{}", info.name),
                    kind: Some(CompletionItemKind::CONSTANT),
                    detail: Some(info.type_info.clone()),
                    documentation: Some(Documentation::MarkupContent(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: info.documentation.clone(),
                    })),
                    ..Default::default()
                })
                .collect()
        }
        _ => {
            // Get prefix for filtering
            let prefix = get_word_prefix(&line, char_idx).to_uppercase();

            let mut items = Vec::new();

            // Keywords
            let keywords = vec![
                ("IF", "if condition then"),
                ("THEN", "then clause"),
                ("ELSE", "else clause"),
                ("ENDIF", "end if statement"),
                ("FOR", "for loop"),
                ("ENDFOR", "end for loop"),
                ("FOREACH", "foreach loop"),
                ("WHILE", "while loop"),
                ("ENDWHILE", "end while loop"),
                ("REPEAT", "repeat loop"),
                ("UNTIL", "until condition"),
                ("DO", "do statement"),
                ("BREAK", "break from loop"),
                ("CONTINUE", "continue to next iteration"),
                ("RETURN", "return from function"),
                ("FUNCTION", "define function"),
                ("ENDFUNCTION", "end function"),
                ("PRO", "define procedure"),
                ("PROCEDURE", "define procedure"),
                ("ENDPRO", "end procedure"),
                ("BEGIN", "begin block"),
                ("END", "end block"),
                ("CASE", "case statement"),
                ("ENDCASE", "end case"),
                ("SWITCH", "switch statement"),
                ("ENDSWITCH", "end switch"),
                ("OF", "case of"),
                ("COMMON", "common block"),
                ("COMPILE_OPT", "compiler options"),
                ("FORWARD_FUNCTION", "forward function declaration"),
                ("GOTO", "goto label"),
                ("ON_IOERROR", "I/O error handler"),
                ("AND", "logical and"),
                ("OR", "logical or"),
                ("NOT", "logical not"),
                ("XOR", "logical xor"),
                ("EQ", "equal comparison"),
                ("NE", "not equal comparison"),
                ("LT", "less than"),
                ("GT", "greater than"),
                ("LE", "less than or equal"),
                ("GE", "greater than or equal"),
                ("MOD", "modulo operator"),
            ];

            for (kw, desc) in keywords {
                if prefix.is_empty() || kw.starts_with(&prefix) {
                    items.push(CompletionItem {
                        label: kw.to_string(),
                        kind: Some(CompletionItemKind::KEYWORD),
                        detail: Some(desc.to_string()),
                        ..Default::default()
                    });
                }
            }

            // Built-in functions
            for (name, info) in &symbol_table.builtin_functions {
                if prefix.is_empty() || name.starts_with(&prefix) {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail: Some(format!("Returns: {}", info.return_type)),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("```\n{}\n```\n\n{}", info.signature, info.documentation),
                        })),
                        insert_text: Some(format!("{}($0)", name)),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
            }

            // Built-in procedures
            for (name, info) in &symbol_table.builtin_procedures {
                if prefix.is_empty() || name.starts_with(&prefix) {
                    items.push(CompletionItem {
                        label: name.clone(),
                        kind: Some(CompletionItemKind::METHOD),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: format!("```\n{}\n```\n\n{}", info.signature, info.documentation),
                        })),
                        insert_text: Some(format!("{}, $0", name)),
                        insert_text_format: Some(InsertTextFormat::SNIPPET),
                        ..Default::default()
                    });
                }
            }

            // System variables (without ! prefix for regular completion)
            for (name, info) in &symbol_table.system_variables {
                let full_name = format!("!{}", name);
                if prefix.is_empty() || full_name.starts_with(&prefix) || name.starts_with(&prefix)
                {
                    items.push(CompletionItem {
                        label: full_name.clone(),
                        kind: Some(CompletionItemKind::CONSTANT),
                        detail: Some(info.type_info.clone()),
                        documentation: Some(Documentation::MarkupContent(MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: info.documentation.clone(),
                        })),
                        ..Default::default()
                    });
                }
            }

            items
        }
    };

    if items.is_empty() {
        None
    } else {
        Some(CompletionResponse::Array(items))
    }
}

fn get_word_prefix(line: &str, char_idx: usize) -> String {
    let chars: Vec<char> = line.chars().collect();
    let mut start = char_idx;

    while start > 0 {
        let c = chars[start - 1];
        if c.is_alphanumeric() || c == '_' || c == '!' {
            start -= 1;
        } else {
            break;
        }
    }

    chars[start..char_idx].iter().collect()
}
