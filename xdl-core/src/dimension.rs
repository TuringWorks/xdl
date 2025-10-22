//! XDL array dimensions and indexing

use crate::{XdlError, MAXRANK};
use serde::{Deserialize, Serialize};

/// XDL array dimension descriptor
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Dimension {
    dimensions: Vec<usize>,
}

impl Dimension {
    /// Create a scalar (0-dimensional)
    pub fn scalar() -> Self {
        Self { dimensions: vec![] }
    }

    /// Create from dimension vector
    pub fn from_vec(dims: Vec<usize>) -> Result<Self, XdlError> {
        if dims.len() > MAXRANK {
            return Err(XdlError::DimensionError(format!(
                "Too many dimensions: {} > {}",
                dims.len(),
                MAXRANK
            )));
        }

        if dims.contains(&0) {
            return Err(XdlError::DimensionError(
                "Zero dimensions not allowed".to_string(),
            ));
        }

        Ok(Self { dimensions: dims })
    }

    /// Create 1-dimensional array
    pub fn from_size(size: usize) -> Result<Self, XdlError> {
        if size == 0 {
            return Err(XdlError::DimensionError(
                "Zero size not allowed".to_string(),
            ));
        }
        Ok(Self {
            dimensions: vec![size],
        })
    }

    /// Get number of dimensions (rank)
    pub fn rank(&self) -> usize {
        self.dimensions.len()
    }

    /// Get dimension sizes
    pub fn dims(&self) -> &[usize] {
        &self.dimensions
    }

    /// Get specific dimension size
    pub fn dim(&self, index: usize) -> Option<usize> {
        self.dimensions.get(index).copied()
    }

    /// Calculate total number of elements
    pub fn n_elements(&self) -> usize {
        if self.dimensions.is_empty() {
            1 // scalar
        } else {
            self.dimensions.iter().product()
        }
    }

    /// Check if this is a scalar
    pub fn is_scalar(&self) -> bool {
        self.dimensions.is_empty()
    }

    /// Check if this is a vector (1D array)
    pub fn is_vector(&self) -> bool {
        self.dimensions.len() == 1
    }

    /// Convert multidimensional index to linear index
    pub fn linear_index(&self, indices: &[usize]) -> Result<usize, XdlError> {
        if indices.len() != self.dimensions.len() {
            return Err(XdlError::DimensionError(format!(
                "Index rank {} doesn't match array rank {}",
                indices.len(),
                self.dimensions.len()
            )));
        }

        let mut linear_idx = 0;
        let mut stride = 1;

        for (i, (&idx, &dim)) in indices.iter().zip(&self.dimensions).enumerate().rev() {
            if idx >= dim {
                return Err(XdlError::IndexError(format!(
                    "Index {} out of range for dimension {} (size {})",
                    idx, i, dim
                )));
            }
            linear_idx += idx * stride;
            stride *= dim;
        }

        Ok(linear_idx)
    }

    /// Convert linear index to multidimensional indices
    pub fn multi_index(&self, linear_idx: usize) -> Result<Vec<usize>, XdlError> {
        if linear_idx >= self.n_elements() {
            return Err(XdlError::IndexError(format!(
                "Linear index {} out of range for array with {} elements",
                linear_idx,
                self.n_elements()
            )));
        }

        if self.is_scalar() {
            return Ok(vec![]);
        }

        let mut indices = vec![0; self.dimensions.len()];
        let mut remaining = linear_idx;

        for i in (0..self.dimensions.len()).rev() {
            let dim_size = self.dimensions[i];
            indices[i] = remaining % dim_size;
            remaining /= dim_size;
        }

        Ok(indices)
    }

    /// Reform to new dimensions (like IDL REFORM)
    pub fn reform(&self, new_dims: Vec<usize>) -> Result<Self, XdlError> {
        let new_n_elements: usize = new_dims.iter().product();
        if new_n_elements != self.n_elements() {
            return Err(XdlError::DimensionError(format!(
                "Cannot reform array of {} elements to {} elements",
                self.n_elements(),
                new_n_elements
            )));
        }

        Self::from_vec(new_dims)
    }

    /// Transpose dimensions
    pub fn transpose(&self, perm: Option<&[usize]>) -> Result<Self, XdlError> {
        if self.is_scalar() {
            return Ok(self.clone());
        }

        let perm = if let Some(p) = perm {
            if p.len() != self.dimensions.len() {
                return Err(XdlError::DimensionError(
                    "Permutation length doesn't match array rank".to_string(),
                ));
            }
            p.to_vec()
        } else {
            // Default: reverse order
            (0..self.dimensions.len()).rev().collect()
        };

        // Check permutation validity
        let mut check = vec![false; self.dimensions.len()];
        for &p in &perm {
            if p >= self.dimensions.len() {
                return Err(XdlError::DimensionError(
                    "Invalid permutation index".to_string(),
                ));
            }
            if check[p] {
                return Err(XdlError::DimensionError(
                    "Duplicate in permutation".to_string(),
                ));
            }
            check[p] = true;
        }

        let new_dims = perm.iter().map(|&i| self.dimensions[i]).collect();
        Ok(Self {
            dimensions: new_dims,
        })
    }
}

impl Default for Dimension {
    fn default() -> Self {
        Self::scalar()
    }
}

impl std::fmt::Display for Dimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_scalar() {
            write!(f, "scalar")
        } else {
            write!(
                f,
                "[{}]",
                self.dimensions
                    .iter()
                    .map(|d| d.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_dimension() {
        let dim = Dimension::scalar();
        assert!(dim.is_scalar());
        assert_eq!(dim.rank(), 0);
        assert_eq!(dim.n_elements(), 1);
    }

    #[test]
    fn test_vector_dimension() {
        let dim = Dimension::from_size(10).unwrap();
        assert!(dim.is_vector());
        assert_eq!(dim.rank(), 1);
        assert_eq!(dim.n_elements(), 10);
        assert_eq!(dim.dim(0), Some(10));
    }

    #[test]
    fn test_multi_dimension() {
        let dim = Dimension::from_vec(vec![3, 4, 5]).unwrap();
        assert_eq!(dim.rank(), 3);
        assert_eq!(dim.n_elements(), 60);
        assert_eq!(dim.dims(), &[3, 4, 5]);
    }

    #[test]
    fn test_indexing() {
        let dim = Dimension::from_vec(vec![3, 4]).unwrap();

        // Test linear index conversion
        assert_eq!(dim.linear_index(&[0, 0]).unwrap(), 0);
        assert_eq!(dim.linear_index(&[2, 3]).unwrap(), 11);

        // Test multi index conversion
        assert_eq!(dim.multi_index(0).unwrap(), vec![0, 0]);
        assert_eq!(dim.multi_index(11).unwrap(), vec![2, 3]);
    }

    #[test]
    fn test_reform() {
        let dim = Dimension::from_vec(vec![3, 4]).unwrap();
        let reformed = dim.reform(vec![2, 6]).unwrap();
        assert_eq!(reformed.dims(), &[2, 6]);
        assert_eq!(reformed.n_elements(), 12);
    }

    #[test]
    fn test_transpose() {
        let dim = Dimension::from_vec(vec![3, 4, 5]).unwrap();
        let transposed = dim.transpose(None).unwrap();
        assert_eq!(transposed.dims(), &[5, 4, 3]);

        let custom_transpose = dim.transpose(Some(&[1, 0, 2])).unwrap();
        assert_eq!(custom_transpose.dims(), &[4, 3, 5]);
    }

    #[test]
    fn test_error_cases() {
        // Zero dimension
        assert!(Dimension::from_vec(vec![3, 0, 5]).is_err());

        // Too many dimensions
        assert!(Dimension::from_vec(vec![1; MAXRANK + 1]).is_err());

        // Invalid reform
        let dim = Dimension::from_size(10).unwrap();
        assert!(dim.reform(vec![3, 4]).is_err()); // 12 != 10
    }
}
