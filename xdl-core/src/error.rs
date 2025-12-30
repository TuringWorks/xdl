//! XDL error types and handling

use thiserror::Error;

/// Main XDL error type
#[derive(Error, Debug, Clone, PartialEq)]
pub enum XdlError {
    #[error("Parse error: {message} at line {line}, column {column}")]
    ParseError {
        message: String,
        line: usize,
        column: usize,
    },

    #[error("Type mismatch: expected {expected}, got {actual}")]
    TypeMismatch { expected: String, actual: String },

    #[error("Dimension error: {0}")]
    DimensionError(String),

    #[error("Index error: {0}")]
    IndexError(String),

    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Variable not found: {0}")]
    VariableNotFound(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),

    #[error("Procedure not found: {0}")]
    ProcedureNotFound(String),

    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Math error: {0}")]
    MathError(String),

    #[error("Memory error: {0}")]
    MemoryError(String),

    #[error("System error: {0}")]
    SystemError(String),

    #[error("Compile error: {0}")]
    CompileError(String),

    #[error("Syntax error: {0}")]
    SyntaxError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Overflow error in {operation}")]
    Overflow { operation: String },

    #[error("Underflow error in {operation}")]
    Underflow { operation: String },

    #[error("Not implemented: {0}")]
    NotImplemented(String),

    #[error("Internal error: {0}")]
    InternalError(String),

    // Control flow errors (used for break/continue/return flow control)
    #[error("Break")]
    Break,

    #[error("Continue")]
    Continue,

    #[error("Return")]
    Return(crate::XdlValue),

    #[error("Goto: {0}")]
    Goto(String),

    #[error("Invalid assignment target")]
    InvalidAssignmentTarget,

    #[error("Invalid value: {0}")]
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
