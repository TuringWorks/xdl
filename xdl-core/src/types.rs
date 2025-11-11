//! XDL value types and data representations

use crate::{Dimension, GdlType, XdlError};
use num_complex::{Complex32, Complex64};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// XDL scalar value types
#[derive(Debug, Clone, PartialEq, Default)]
pub enum XdlValue {
    #[default]
    Undefined,
    Byte(u8),
    Int(i16),
    Long(i32),
    Float(f32),
    Double(f64),
    Complex(Complex32),
    DComplex(Complex64),
    String(String),
    UInt(u16),
    ULong(u32),
    Long64(i64),
    ULong64(u64),
    Pointer(usize),
    ObjRef(usize),
    Array(Vec<f64>),            // Simple 1D array representation
    NestedArray(Vec<XdlValue>), // Nested arrays (matrices, etc.)
    MultiDimArray {
        // Multi-dimensional array with shape
        data: Vec<f64>,
        shape: Vec<usize>, // Dimensions: [rows, cols] for 2D, [depth, rows, cols] for 3D
    },
    PythonObject(String), // Opaque reference to Python object (stored by ID)
}

impl XdlValue {
    /// Get the XDL type of this value
    pub fn gdl_type(&self) -> GdlType {
        match self {
            XdlValue::Undefined => GdlType::Undefined,
            XdlValue::Byte(_) => GdlType::Byte,
            XdlValue::Int(_) => GdlType::Int,
            XdlValue::Long(_) => GdlType::Long,
            XdlValue::Float(_) => GdlType::Float,
            XdlValue::Double(_) => GdlType::Double,
            XdlValue::Complex(_) => GdlType::Complex,
            XdlValue::DComplex(_) => GdlType::DComplex,
            XdlValue::String(_) => GdlType::String,
            XdlValue::UInt(_) => GdlType::UInt,
            XdlValue::ULong(_) => GdlType::ULong,
            XdlValue::Long64(_) => GdlType::Long64,
            XdlValue::ULong64(_) => GdlType::ULong64,
            XdlValue::Pointer(_) => GdlType::Pointer,
            XdlValue::ObjRef(_) => GdlType::ObjRef,
            XdlValue::Array(_) => GdlType::Float, // Arrays default to float type for now
            XdlValue::NestedArray(_) => GdlType::Float, // Nested arrays also default to float
            XdlValue::MultiDimArray { .. } => GdlType::Float, // Multi-dim arrays are float
            XdlValue::PythonObject(_) => GdlType::ObjRef, // Python objects are object references
        }
    }

    /// Convert to string representation
    pub fn to_string_repr(&self) -> String {
        match self {
            XdlValue::Undefined => "!NULL".to_string(),
            XdlValue::Byte(v) => v.to_string(),
            XdlValue::Int(v) => v.to_string(),
            XdlValue::Long(v) => v.to_string(),
            XdlValue::Float(v) => {
                if v.is_finite() {
                    format!("{:.6}", v)
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        "Inf".to_string()
                    } else {
                        "-Inf".to_string()
                    }
                } else {
                    "NaN".to_string()
                }
            }
            XdlValue::Double(v) => {
                if v.is_finite() {
                    format!("{:.15}", v)
                } else if v.is_infinite() {
                    if v.is_sign_positive() {
                        "Inf".to_string()
                    } else {
                        "-Inf".to_string()
                    }
                } else {
                    "NaN".to_string()
                }
            }
            XdlValue::Complex(v) => format!("({}, {})", v.re, v.im),
            XdlValue::DComplex(v) => format!("({}, {})", v.re, v.im),
            XdlValue::String(s) => s.clone(),
            XdlValue::UInt(v) => v.to_string(),
            XdlValue::ULong(v) => v.to_string(),
            XdlValue::Long64(v) => v.to_string(),
            XdlValue::ULong64(v) => v.to_string(),
            XdlValue::Pointer(v) => format!("<PTR_{:X}>", v),
            XdlValue::ObjRef(v) => format!("<OBJ_{:X}>", v),
            XdlValue::Array(arr) => {
                if arr.len() <= 10 {
                    format!(
                        "[{}]",
                        arr.iter()
                            .map(|x| format!("{:.6}", x))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    format!(
                        "[{:.6}, {:.6}, ..., {:.6}] ({})",
                        arr[0],
                        arr[1],
                        arr[arr.len() - 1],
                        arr.len()
                    )
                }
            }
            XdlValue::NestedArray(rows) => {
                format!(
                    "[{}]",
                    rows.iter()
                        .map(|row| row.to_string_repr())
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            XdlValue::MultiDimArray { data, shape } => {
                let dims_str = shape
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join("x");
                if data.len() <= 6 {
                    format!(
                        "Array[{}]: [{}]",
                        dims_str,
                        data.iter()
                            .map(|x| format!("{:.3}", x))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                } else {
                    format!(
                        "Array[{}]: [{:.3}, {:.3}, ..., {:.3}] ({})",
                        dims_str,
                        data[0],
                        data[1],
                        data[data.len() - 1],
                        data.len()
                    )
                }
            }
            XdlValue::PythonObject(id) => {
                // Return a placeholder - actual string conversion happens in the stdlib layer
                format!("<Python:{}>", id)
            }
        }
    }

    /// Convert to double precision value (if numeric)
    pub fn to_double(&self) -> Result<f64, XdlError> {
        match self {
            XdlValue::Byte(v) => Ok(*v as f64),
            XdlValue::Int(v) => Ok(*v as f64),
            XdlValue::Long(v) => Ok(*v as f64),
            XdlValue::Float(v) => Ok(*v as f64),
            XdlValue::Double(v) => Ok(*v),
            XdlValue::UInt(v) => Ok(*v as f64),
            XdlValue::ULong(v) => Ok(*v as f64),
            XdlValue::Long64(v) => Ok(*v as f64),
            XdlValue::ULong64(v) => Ok(*v as f64),
            XdlValue::Array(arr) => {
                if arr.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(arr[0]) // Return first element for scalar operations
                }
            }
            XdlValue::MultiDimArray { data, .. } => {
                if data.is_empty() {
                    Ok(0.0)
                } else {
                    Ok(data[0])
                }
            }
            _ => Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: self.gdl_type().to_string(),
            }),
        }
    }

    /// Convert to integer value (if convertible)
    pub fn to_long(&self) -> Result<i32, XdlError> {
        match self {
            XdlValue::Byte(v) => Ok(*v as i32),
            XdlValue::Int(v) => Ok(*v as i32),
            XdlValue::Long(v) => Ok(*v),
            XdlValue::Float(v) => Ok(*v as i32),
            XdlValue::Double(v) => Ok(*v as i32),
            XdlValue::UInt(v) => Ok(*v as i32),
            XdlValue::ULong(v) => Ok(*v as i32),
            XdlValue::Long64(v) => Ok(*v as i32),
            XdlValue::ULong64(v) => Ok(*v as i32),
            XdlValue::Array(arr) => {
                if arr.is_empty() {
                    Ok(0)
                } else {
                    Ok(arr[0] as i32) // Return first element for scalar operations
                }
            }
            XdlValue::MultiDimArray { data, .. } => {
                if data.is_empty() {
                    Ok(0)
                } else {
                    Ok(data[0] as i32)
                }
            }
            _ => Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: self.gdl_type().to_string(),
            }),
        }
    }

    /// Create a multi-dimensional array from data and shape
    pub fn from_multidim(data: Vec<f64>, shape: Vec<usize>) -> Result<Self, XdlError> {
        let expected_size: usize = shape.iter().product();
        if data.len() != expected_size {
            return Err(XdlError::InvalidArgument(format!(
                "Data size {} does not match shape {:?} (expected {})",
                data.len(),
                shape,
                expected_size
            )));
        }
        Ok(XdlValue::MultiDimArray { data, shape })
    }

    /// Get shape of multi-dimensional array (if applicable)
    pub fn shape(&self) -> Option<Vec<usize>> {
        match self {
            XdlValue::Array(arr) => Some(vec![arr.len()]),
            XdlValue::MultiDimArray { shape, .. } => Some(shape.clone()),
            _ => None,
        }
    }

    /// Get data slice for multi-dimensional array
    pub fn as_slice(&self) -> Option<&[f64]> {
        match self {
            XdlValue::Array(arr) => Some(arr),
            XdlValue::MultiDimArray { data, .. } => Some(data),
            _ => None,
        }
    }

    /// Get total number of elements
    pub fn n_elements(&self) -> usize {
        match self {
            XdlValue::Array(arr) => arr.len(),
            XdlValue::MultiDimArray { data, .. } => data.len(),
            _ => 1,
        }
    }

    /// Check if value is zero
    pub fn is_zero(&self) -> bool {
        match self {
            XdlValue::Byte(v) => *v == 0,
            XdlValue::Int(v) => *v == 0,
            XdlValue::Long(v) => *v == 0,
            XdlValue::Float(v) => *v == 0.0,
            XdlValue::Double(v) => *v == 0.0,
            XdlValue::Complex(v) => v.re == 0.0 && v.im == 0.0,
            XdlValue::DComplex(v) => v.re == 0.0 && v.im == 0.0,
            XdlValue::String(s) => s.is_empty(),
            XdlValue::UInt(v) => *v == 0,
            XdlValue::ULong(v) => *v == 0,
            XdlValue::Long64(v) => *v == 0,
            XdlValue::ULong64(v) => *v == 0,
            XdlValue::Array(arr) => arr.is_empty() || arr.iter().all(|&x| x == 0.0),
            XdlValue::NestedArray(rows) => rows.is_empty() || rows.iter().all(|r| r.is_zero()),
            XdlValue::MultiDimArray { data, .. } => {
                data.is_empty() || data.iter().all(|&x| x == 0.0)
            }
            XdlValue::PythonObject(_) => false, // Python objects are never considered zero
            _ => false,
        }
    }
}

/// XDL structure field descriptor
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructField {
    pub name: String,
    pub gdl_type: GdlType,
    pub dimensions: Option<Dimension>,
}

/// XDL structure definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StructDef {
    pub name: Option<String>,
    pub fields: Vec<StructField>,
    pub size: usize,
}

impl StructDef {
    pub fn new(name: Option<String>, fields: Vec<StructField>) -> Self {
        let size = fields
            .iter()
            .map(|f| {
                let element_size = f.gdl_type.size();
                if let Some(ref dim) = f.dimensions {
                    element_size * dim.n_elements()
                } else {
                    element_size
                }
            })
            .sum();

        Self { name, fields, size }
    }

    pub fn find_field(&self, name: &str) -> Option<usize> {
        self.fields
            .iter()
            .position(|f| f.name.eq_ignore_ascii_case(name))
    }
}

/// XDL structure instance
#[derive(Debug, Clone)]
pub struct GdlStruct {
    pub definition: StructDef,
    pub fields: HashMap<String, Box<dyn GdlData>>,
}

/// Trait for all XDL data containers
pub trait GdlData: std::fmt::Debug + Send + Sync {
    fn gdl_type(&self) -> GdlType;
    fn dimensions(&self) -> &Dimension;
    fn n_elements(&self) -> usize;
    fn size_bytes(&self) -> usize;
    fn clone_boxed(&self) -> Box<dyn GdlData>;
    fn to_string_repr(&self) -> String;
}

// We need to implement Clone manually for Box<dyn GdlData>
impl Clone for Box<dyn GdlData> {
    fn clone(&self) -> Self {
        self.clone_boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gdl_value_types() {
        assert_eq!(XdlValue::Byte(42).gdl_type(), GdlType::Byte);
        assert_eq!(XdlValue::Float(3.5).gdl_type(), GdlType::Float);
        assert_eq!(
            XdlValue::String("hello".to_string()).gdl_type(),
            GdlType::String
        );
    }

    #[test]
    fn test_value_conversions() {
        let val = XdlValue::Int(42);
        assert_eq!(val.to_double().unwrap(), 42.0);
        assert_eq!(val.to_long().unwrap(), 42);
    }

    #[test]
    fn test_struct_def() {
        let fields = vec![
            StructField {
                name: "x".to_string(),
                gdl_type: GdlType::Float,
                dimensions: None,
            },
            StructField {
                name: "y".to_string(),
                gdl_type: GdlType::Double,
                dimensions: None,
            },
        ];

        let struct_def = StructDef::new(Some("Point".to_string()), fields);
        assert_eq!(struct_def.size, 12); // 4 + 8 bytes
        assert_eq!(struct_def.find_field("x"), Some(0));
        assert_eq!(struct_def.find_field("Y"), Some(1)); // Case insensitive
    }
}
