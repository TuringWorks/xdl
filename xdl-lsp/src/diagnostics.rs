//! Diagnostics conversion from XDL errors to LSP diagnostics

use tower_lsp::lsp_types::{Diagnostic, DiagnosticSeverity, Position, Range};
use xdl_core::XdlError;

pub fn convert_error_to_diagnostics(error: &XdlError) -> Vec<Diagnostic> {
    match error {
        XdlError::ParseError {
            message,
            line,
            column,
        } => {
            vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: (*line as u32).saturating_sub(1),
                        character: *column as u32,
                    },
                    end: Position {
                        line: (*line as u32).saturating_sub(1),
                        character: (*column as u32) + 10,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: message.clone(),
                ..Default::default()
            }]
        }
        XdlError::SyntaxError(msg) => {
            vec![Diagnostic {
                range: Range {
                    start: Position {
                        line: 0,
                        character: 0,
                    },
                    end: Position {
                        line: 0,
                        character: 1,
                    },
                },
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: msg.clone(),
                ..Default::default()
            }]
        }
        XdlError::TypeMismatch { expected, actual } => {
            vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: format!("Type mismatch: expected {}, got {}", expected, actual),
                ..Default::default()
            }]
        }
        XdlError::VariableNotFound(name) => {
            vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: format!("Variable not found: {}", name),
                ..Default::default()
            }]
        }
        XdlError::FunctionNotFound(name) => {
            vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: format!("Function not found: {}", name),
                ..Default::default()
            }]
        }
        XdlError::ProcedureNotFound(name) => {
            vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: format!("Procedure not found: {}", name),
                ..Default::default()
            }]
        }
        _ => {
            vec![Diagnostic {
                range: Range::default(),
                severity: Some(DiagnosticSeverity::ERROR),
                source: Some("xdl".to_string()),
                message: format!("{}", error),
                ..Default::default()
            }]
        }
    }
}
