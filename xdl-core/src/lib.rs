//! # XDL Core
//!
//! Core data structures and types for the Extended Data Language (XDL) Rust implementation.
//! This module provides the fundamental building blocks for XDL data types and operations.

pub mod array;
pub mod dimension;
pub mod error;
pub mod types;

use serde::{Deserialize, Serialize};
use std::fmt;

pub use array::*;
pub use dimension::*;
pub use error::*;
pub use types::*;

/// Maximum number of dimensions supported by XDL arrays
pub const MAXRANK: usize = 8;

/// XDL data types enumeration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GdlType {
    Undefined,
    Byte,
    Int,
    Long,
    Float,
    Double,
    Complex,
    DComplex,
    String,
    Struct,
    Pointer,
    ObjRef,
    UInt,
    ULong,
    Long64,
    ULong64,
}

impl GdlType {
    /// Returns the size in bytes of this type
    pub fn size(self) -> usize {
        match self {
            GdlType::Undefined => 0,
            GdlType::Byte => 1,
            GdlType::Int => 2,
            GdlType::Long => 4,
            GdlType::Float => 4,
            GdlType::Double => 8,
            GdlType::Complex => 8,
            GdlType::DComplex => 16,
            GdlType::String => std::mem::size_of::<String>(),
            GdlType::Struct => 0, // Variable size
            GdlType::Pointer => std::mem::size_of::<usize>(),
            GdlType::ObjRef => std::mem::size_of::<usize>(),
            GdlType::UInt => 2,
            GdlType::ULong => 4,
            GdlType::Long64 => 8,
            GdlType::ULong64 => 8,
        }
    }

    /// Returns true if this is a numeric type
    pub fn is_numeric(self) -> bool {
        matches!(
            self,
            GdlType::Byte
                | GdlType::Int
                | GdlType::Long
                | GdlType::Float
                | GdlType::Double
                | GdlType::Complex
                | GdlType::DComplex
                | GdlType::UInt
                | GdlType::ULong
                | GdlType::Long64
                | GdlType::ULong64
        )
    }

    /// Returns true if this is an integer type
    pub fn is_integer(self) -> bool {
        matches!(
            self,
            GdlType::Byte
                | GdlType::Int
                | GdlType::Long
                | GdlType::UInt
                | GdlType::ULong
                | GdlType::Long64
                | GdlType::ULong64
        )
    }

    /// Returns true if this is a floating point type
    pub fn is_float(self) -> bool {
        matches!(self, GdlType::Float | GdlType::Double)
    }

    /// Returns true if this is a complex type
    pub fn is_complex(self) -> bool {
        matches!(self, GdlType::Complex | GdlType::DComplex)
    }
}

impl fmt::Display for GdlType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            GdlType::Undefined => "UNDEFINED",
            GdlType::Byte => "BYTE",
            GdlType::Int => "INT",
            GdlType::Long => "LONG",
            GdlType::Float => "FLOAT",
            GdlType::Double => "DOUBLE",
            GdlType::Complex => "COMPLEX",
            GdlType::DComplex => "DCOMPLEX",
            GdlType::String => "STRING",
            GdlType::Struct => "STRUCT",
            GdlType::Pointer => "POINTER",
            GdlType::ObjRef => "OBJREF",
            GdlType::UInt => "UINT",
            GdlType::ULong => "ULONG",
            GdlType::Long64 => "LONG64",
            GdlType::ULong64 => "ULONG64",
        };
        write!(f, "{}", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdl_type_sizes() {
        assert_eq!(GdlType::Byte.size(), 1);
        assert_eq!(GdlType::Int.size(), 2);
        assert_eq!(GdlType::Long.size(), 4);
        assert_eq!(GdlType::Float.size(), 4);
        assert_eq!(GdlType::Double.size(), 8);
        assert_eq!(GdlType::Complex.size(), 8);
        assert_eq!(GdlType::DComplex.size(), 16);
    }

    #[test]
    fn test_type_predicates() {
        assert!(GdlType::Float.is_numeric());
        assert!(GdlType::Int.is_integer());
        assert!(GdlType::Float.is_float());
        assert!(GdlType::Complex.is_complex());
        assert!(!GdlType::String.is_numeric());
    }
}
