//! Parser-specific error types

use xdl_core::XdlError;

pub type ParseResult<T> = Result<T, XdlError>;

// Re-export core error types for convenience
pub use xdl_core::{XdlErrorContext, XdlResult};
