//! XDL array data structures and operations

use crate::{Dimension, GdlData, GdlType, XdlError, XdlResult, XdlValue};
use ndarray::{ArrayD, IxDyn};
use num_complex::{Complex32, Complex64};
use std::fmt;

/// Generic XDL array container
#[derive(Debug, Clone)]
pub struct GdlArray<T> {
    data: ArrayD<T>,
    dimensions: Dimension,
    gdl_type: GdlType,
}

impl<T> GdlArray<T>
where
    T: Clone + Default + fmt::Debug + Send + Sync + 'static,
{
    /// Create new array with given dimensions
    pub fn new(dimensions: Dimension, gdl_type: GdlType) -> Result<Self, XdlError>
    where
        T: Default,
    {
        let shape: Vec<usize> = if dimensions.is_scalar() {
            vec![]
        } else {
            dimensions.dims().to_vec()
        };

        let data = ArrayD::default(IxDyn(&shape));

        Ok(Self {
            data,
            dimensions,
            gdl_type,
        })
    }

    /// Create array from data vector
    pub fn from_vec(
        data: Vec<T>,
        dimensions: Dimension,
        gdl_type: GdlType,
    ) -> Result<Self, XdlError> {
        if data.len() != dimensions.n_elements() {
            return Err(XdlError::DimensionError(format!(
                "Data length {} doesn't match dimension size {}",
                data.len(),
                dimensions.n_elements()
            )));
        }

        let shape: Vec<usize> = if dimensions.is_scalar() {
            vec![]
        } else {
            dimensions.dims().to_vec()
        };

        let array_data = ArrayD::from_shape_vec(IxDyn(&shape), data)
            .map_err(|e| XdlError::DimensionError(format!("Shape error: {}", e)))?;

        Ok(Self {
            data: array_data,
            dimensions,
            gdl_type,
        })
    }

    /// Create scalar array
    pub fn scalar(value: T, gdl_type: GdlType) -> Self {
        let data = ArrayD::from_elem(IxDyn(&[]), value);
        Self {
            data,
            dimensions: Dimension::scalar(),
            gdl_type,
        }
    }

    /// Get element at index
    pub fn get(&self, indices: &[usize]) -> Result<&T, XdlError> {
        if self.dimensions.is_scalar() && indices.is_empty() {
            return Ok(&self.data[IxDyn(&[])]);
        }

        if indices.len() != self.dimensions.rank() {
            return Err(XdlError::IndexError(format!(
                "Index rank {} doesn't match array rank {}",
                indices.len(),
                self.dimensions.rank()
            )));
        }

        let ix = IxDyn(indices);
        self.data
            .get(ix)
            .ok_or_else(|| XdlError::IndexError(format!("Index {:?} out of bounds", indices)))
    }

    /// Set element at index
    pub fn set(&mut self, indices: &[usize], value: T) -> Result<(), XdlError> {
        if self.dimensions.is_scalar() && indices.is_empty() {
            self.data[IxDyn(&[])] = value;
            return Ok(());
        }

        if indices.len() != self.dimensions.rank() {
            return Err(XdlError::IndexError(format!(
                "Index rank {} doesn't match array rank {}",
                indices.len(),
                self.dimensions.rank()
            )));
        }

        let ix = IxDyn(indices);
        if let Some(elem) = self.data.get_mut(ix) {
            *elem = value;
            Ok(())
        } else {
            Err(XdlError::IndexError(format!(
                "Index {:?} out of bounds",
                indices
            )))
        }
    }

    /// Get linear element access
    pub fn get_linear(&self, index: usize) -> Result<&T, XdlError> {
        if index >= self.dimensions.n_elements() {
            return Err(XdlError::IndexError(format!(
                "Linear index {} out of range",
                index
            )));
        }

        // Convert to multi-dimensional index
        let multi_idx = self.dimensions.multi_index(index)?;
        self.get(&multi_idx)
    }

    /// Set linear element access
    pub fn set_linear(&mut self, index: usize, value: T) -> Result<(), XdlError> {
        if index >= self.dimensions.n_elements() {
            return Err(XdlError::IndexError(format!(
                "Linear index {} out of range",
                index
            )));
        }

        let multi_idx = self.dimensions.multi_index(index)?;
        self.set(&multi_idx, value)
    }

    /// Transpose array
    pub fn transpose(&self, axes: Option<&[usize]>) -> Result<Self, XdlError>
    where
        T: Clone,
    {
        let new_dims = self.dimensions.transpose(axes)?;
        let new_data = if let Some(ax) = axes {
            self.data.clone().permuted_axes(ax).to_owned()
        } else {
            // Default transpose (reverse axes)
            let reversed_axes: Vec<usize> = (0..self.dimensions.rank()).rev().collect();
            self.data
                .clone()
                .permuted_axes(reversed_axes.as_slice())
                .to_owned()
        };

        Ok(Self {
            data: new_data,
            dimensions: new_dims,
            gdl_type: self.gdl_type,
        })
    }

    /// Reform array to new shape
    pub fn reform(&self, new_dims: Vec<usize>) -> Result<Self, XdlError>
    where
        T: Clone,
    {
        let new_dimensions = self.dimensions.reform(new_dims)?;
        let shape: Vec<usize> = if new_dimensions.is_scalar() {
            vec![]
        } else {
            new_dimensions.dims().to_vec()
        };

        let new_data = self
            .data
            .clone()
            .into_shape(IxDyn(&shape))
            .map_err(|e| XdlError::DimensionError(format!("Reform error: {}", e)))?;

        Ok(Self {
            data: new_data,
            dimensions: new_dimensions,
            gdl_type: self.gdl_type,
        })
    }

    /// Get dimensions
    pub fn dimensions(&self) -> &Dimension {
        &self.dimensions
    }

    /// Get XDL type
    pub fn gdl_type(&self) -> GdlType {
        self.gdl_type
    }

    /// Get raw data slice (for linear access)
    pub fn as_slice(&self) -> Option<&[T]> {
        self.data.as_slice()
    }

    /// Convert to vector
    pub fn to_vec(&self) -> Vec<T>
    where
        T: Clone,
    {
        if let Some(slice) = self.as_slice() {
            slice.to_vec()
        } else {
            self.data.iter().cloned().collect()
        }
    }
}

// Implementations for specific types
pub type ByteArray = GdlArray<u8>;
pub type IntArray = GdlArray<i16>;
pub type LongArray = GdlArray<i32>;
pub type FloatArray = GdlArray<f32>;
pub type DoubleArray = GdlArray<f64>;
pub type ComplexArray = GdlArray<Complex32>;
pub type DComplexArray = GdlArray<Complex64>;
pub type StringArray = GdlArray<String>;

// Implement GdlData trait for typed arrays
macro_rules! impl_gdl_data {
    ($array_type:ty, $gdl_type:expr) => {
        impl GdlData for $array_type {
            fn gdl_type(&self) -> GdlType {
                $gdl_type
            }

            fn dimensions(&self) -> &Dimension {
                &self.dimensions
            }

            fn n_elements(&self) -> usize {
                self.dimensions.n_elements()
            }

            fn size_bytes(&self) -> usize {
                self.n_elements() * $gdl_type.size()
            }

            fn clone_boxed(&self) -> Box<dyn GdlData> {
                Box::new(self.clone())
            }

            fn to_string_repr(&self) -> String {
                if self.dimensions.is_scalar() {
                    format!("{:?}", self.get(&[]).unwrap())
                } else {
                    format!("Array[{}]: {}", self.gdl_type(), self.dimensions)
                }
            }
        }
    };
}

impl_gdl_data!(ByteArray, GdlType::Byte);
impl_gdl_data!(IntArray, GdlType::Int);
impl_gdl_data!(LongArray, GdlType::Long);
impl_gdl_data!(FloatArray, GdlType::Float);
impl_gdl_data!(DoubleArray, GdlType::Double);
impl_gdl_data!(ComplexArray, GdlType::Complex);
impl_gdl_data!(DComplexArray, GdlType::DComplex);
impl_gdl_data!(StringArray, GdlType::String);

/// Helper functions for creating arrays from XdlValue
impl GdlArray<f64> {
    /// Create from XdlValue (converting to double)
    pub fn from_gdl_value(value: &XdlValue) -> XdlResult<Self> {
        let double_val = value.to_double()?;
        Ok(Self::scalar(double_val, GdlType::Double))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_array() {
        let arr = FloatArray::scalar(3.5, GdlType::Float);
        assert!(arr.dimensions().is_scalar());
        assert_eq!(arr.n_elements(), 1);
        assert_eq!(*arr.get(&[]).unwrap(), 3.5);
    }

    #[test]
    fn test_vector_array() {
        let dim = Dimension::from_size(5).unwrap();
        let data = vec![1, 2, 3, 4, 5];
        let arr = LongArray::from_vec(data, dim, GdlType::Long).unwrap();

        assert!(arr.dimensions().is_vector());
        assert_eq!(arr.n_elements(), 5);
        assert_eq!(*arr.get(&[2]).unwrap(), 3);
    }

    #[test]
    fn test_multi_dim_array() {
        let dim = Dimension::from_vec(vec![2, 3]).unwrap();
        let data = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let arr = DoubleArray::from_vec(data, dim, GdlType::Double).unwrap();

        assert_eq!(arr.dimensions().rank(), 2);
        assert_eq!(arr.n_elements(), 6);
        assert_eq!(*arr.get(&[1, 2]).unwrap(), 6.0);
    }

    #[test]
    fn test_linear_indexing() {
        let dim = Dimension::from_vec(vec![2, 3]).unwrap();
        let data = vec![1, 2, 3, 4, 5, 6];
        let mut arr = LongArray::from_vec(data, dim, GdlType::Long).unwrap();

        assert_eq!(*arr.get_linear(5).unwrap(), 6);
        arr.set_linear(0, 10).unwrap();
        assert_eq!(*arr.get(&[0, 0]).unwrap(), 10);
    }

    #[test]
    fn test_transpose() {
        let dim = Dimension::from_vec(vec![2, 3]).unwrap();
        let data = vec![1, 2, 3, 4, 5, 6];
        let arr = LongArray::from_vec(data, dim, GdlType::Long).unwrap();

        let transposed = arr.transpose(None).unwrap();
        assert_eq!(transposed.dimensions().dims(), &[3, 2]);
    }

    #[test]
    fn test_reform() {
        let dim = Dimension::from_vec(vec![2, 3]).unwrap();
        let data = vec![1, 2, 3, 4, 5, 6];
        let arr = LongArray::from_vec(data, dim, GdlType::Long).unwrap();

        let reformed = arr.reform(vec![3, 2]).unwrap();
        assert_eq!(reformed.dimensions().dims(), &[3, 2]);
        assert_eq!(reformed.n_elements(), 6);
    }
}
