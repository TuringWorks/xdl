//! XDL error types and handling

use thiserror::Error;

/// Main XDL error type
#[derive(Error, Debug, Clone, PartialEq)]
pub enum XdlError {
    #[error("{}", crate::messages::render("parse_error", &[("message", &.message), ("line", &.line), ("column", &.column)]))]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("{}", crate::messages::render("type_mismatch", &[("expected", &.expected), ("actual", &.actual)]))]
    TypeMismatch { expected: String, actual: String },

    #[error("{}", crate::messages::render("dimension_error", &[("value", &.0)]))]
    DimensionError(String),

    #[error("{}", crate::messages::render("index_error", &[("value", &.0)]))]
    IndexError(String),

    #[error("{}", crate::messages::render("runtime_error", &[("value", &.0)]))]
    RuntimeError(String),

    #[error("{}", crate::messages::render("variable_not_found", &[("value", &.0)]))]
    VariableNotFound(String),

    #[error("{}", crate::messages::render("function_not_found", &[("value", &.0)]))]
    FunctionNotFound(String),

    #[error("{}", crate::messages::render("procedure_not_found", &[("value", &.0)]))]
    ProcedureNotFound(String),

    #[error("{}", crate::messages::render("invalid_argument", &[("value", &.0)]))]
    InvalidArgument(String),

    #[error("{}", crate::messages::render("io_error", &[("value", &.0)]))]
    IoError(String),

    #[error("{}", crate::messages::render("math_error", &[("value", &.0)]))]
    MathError(String),

    #[error("{}", crate::messages::render("memory_error", &[("value", &.0)]))]
    MemoryError(String),

    #[error("{}", crate::messages::render("system_error", &[("value", &.0)]))]
    SystemError(String),

    #[error("{}", crate::messages::render("compile_error", &[("value", &.0)]))]
    CompileError(String),

    #[error("{}", crate::messages::render("syntax_error", &[("value", &.0)]))]
    SyntaxError(String),

    #[error("{}", crate::messages::render("file_not_found", &[("value", &.0)]))]
    FileNotFound(String),

    #[error("{}", crate::messages::render("division_by_zero", &[]))]
    DivisionByZero,

    #[error("{}", crate::messages::render("overflow_error", &[("operation", &.operation)]))]
    Overflow { operation: String },

    #[error("{}", crate::messages::render("underflow_error", &[("operation", &.operation)]))]
    Underflow { operation: String },

    #[error("{}", crate::messages::render("not_implemented", &[("value", &.0)]))]
    NotImplemented(String),

    #[error("{}", crate::messages::render("internal_error", &[("value", &.0)]))]
    InternalError(String),

    // Control flow errors (used for break/continue/return flow control)
    #[error("{}", crate::messages::render("break", &[]))]
    Break,

    #[error("{}", crate::messages::render("continue", &[]))]
    Continue,

    #[error("{}", crate::messages::render("return", &[]))]
    Return(crate::XdlValue),

    #[error("{}", crate::messages::render("goto", &[("value", &.0)]))]
    Goto(String),

    #[error("{}", crate::messages::render("invalid_assignment_target", &[]))]
    InvalidAssignmentTarget,

    #[error("{}", crate::messages::render("invalid_value", &[("value", &.0)]))]
    InvalidValue(String),
}

impl From<std::io::Error> for XdlError {
    fn from(err: std::io::Error) -> Self {
        XdlError::IoError(err.to_string())
    }
}

// Generic error conversion for plotters errors
impl<T> From<plotters::drawing::DrawingAreaErrorKind<T>> for XdlError
where
    T: std::error::Error + Send + Sync + 'static,
{
    fn from(err: plotters::drawing::DrawingAreaErrorKind<T>) -> Self {
        XdlError::SystemError(format!("Drawing error: {}", err))
    }
}

/// Result type for XDL operations
pub type XdlResult<T> = Result<T, XdlError>;

/// Helper trait for creating error contexts
pub trait XdlErrorContext<T> {
    fn gdl_context(self, context: &str) -> XdlResult<T>;
}

impl<T> XdlErrorContext<T> for Option<T> {
    fn gdl_context(self, context: &str) -> XdlResult<T> {
        self.ok_or_else(|| XdlError::RuntimeError(context.to_string()))
    }
}

impl<T, E: std::error::Error> XdlErrorContext<T> for Result<T, E> {
    fn gdl_context(self, context: &str) -> XdlResult<T> {
        self.map_err(|e| XdlError::RuntimeError(format!("{}: {}", context, e)))
    }
}

#[cfg(test)]
mod message_text_tests {
    use super::*;

    /// Each variant must render the exact text it rendered before the message
    /// templates moved to resources/messages.txt. These literals are the
    /// original `#[error]` strings.
    #[test]
    fn rendered_text_is_unchanged() {
        let cases: Vec<(XdlError, &str)> = vec![
            (
                XdlError::ParseError {
                    message: "unexpected token".into(),
                    line: 3,
                    column: 7,
                },
                "Parse error: unexpected token at line 3, column 7",
            ),
            (
                XdlError::TypeMismatch {
                    expected: "FLOAT".into(),
                    actual: "STRING".into(),
                },
                "Type mismatch: expected FLOAT, got STRING",
            ),
            (XdlError::DimensionError("x".into()), "Dimension error: x"),
            (XdlError::IndexError("x".into()), "Index error: x"),
            (XdlError::RuntimeError("x".into()), "Runtime error: x"),
            (
                XdlError::VariableNotFound("x".into()),
                "Variable not found: x",
            ),
            (
                XdlError::FunctionNotFound("x".into()),
                "Function not found: x",
            ),
            (
                XdlError::ProcedureNotFound("x".into()),
                "Procedure not found: x",
            ),
            (XdlError::InvalidArgument("x".into()), "Invalid argument: x"),
            (XdlError::IoError("x".into()), "IO error: x"),
            (XdlError::MathError("x".into()), "Math error: x"),
            (XdlError::MemoryError("x".into()), "Memory error: x"),
            (XdlError::SystemError("x".into()), "System error: x"),
            (XdlError::CompileError("x".into()), "Compile error: x"),
            (XdlError::SyntaxError("x".into()), "Syntax error: x"),
            (XdlError::FileNotFound("x".into()), "File not found: x"),
            (XdlError::DivisionByZero, "Division by zero"),
            (
                XdlError::Overflow {
                    operation: "add".into(),
                },
                "Overflow error in add",
            ),
            (
                XdlError::Underflow {
                    operation: "add".into(),
                },
                "Underflow error in add",
            ),
            (XdlError::NotImplemented("x".into()), "Not implemented: x"),
            (XdlError::InternalError("x".into()), "Internal error: x"),
            (XdlError::Break, "Break"),
            (XdlError::Continue, "Continue"),
            (XdlError::Goto("x".into()), "Goto: x"),
            (
                XdlError::InvalidAssignmentTarget,
                "Invalid assignment target",
            ),
            (XdlError::InvalidValue("x".into()), "Invalid value: x"),
        ];
        for (err, expected) in cases {
            assert_eq!(err.to_string(), expected);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = XdlError::TypeMismatch {
            expected: "FLOAT".to_string(),
            actual: "STRING".to_string(),
        };
        assert_eq!(err.to_string(), "Type mismatch: expected FLOAT, got STRING");
    }

    #[test]
    fn test_error_context() {
        let none_val: Option<i32> = None;
        let result = none_val.gdl_context("test operation");
        assert!(matches!(result, Err(XdlError::RuntimeError(_))));
    }
}
