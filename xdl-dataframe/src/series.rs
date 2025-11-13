//! Series - a single column of data

use crate::error::{DataFrameError, DataFrameResult};
use std::collections::HashMap;
use xdl_core::XdlValue;

/// Series - represents a single column of data
#[derive(Debug, Clone)]
pub struct Series {
    data: Vec<XdlValue>,
}

impl Series {
    /// Create a new Series from a vector
    pub fn from_vec(data: Vec<XdlValue>) -> DataFrameResult<Self> {
        Ok(Self { data })
    }

    /// Get length
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get value at index
    pub fn get(&self, index: usize) -> DataFrameResult<&XdlValue> {
        self.data
            .get(index)
            .ok_or(DataFrameError::IndexOutOfBounds(index, self.data.len()))
    }

    /// Get data type as string
    pub fn dtype(&self) -> String {
        if self.data.is_empty() {
            return "undefined".to_string();
        }

        // Determine predominant type
        let first_type = format!("{:?}", self.data[0].gdl_type());
        first_type
    }

    /// Head - get first n elements
    pub fn head(&self, n: usize) -> DataFrameResult<Self> {
        let n = n.min(self.data.len());
        Ok(Self {
            data: self.data[..n].to_vec(),
        })
    }

    /// Tail - get last n elements
    pub fn tail(&self, n: usize) -> DataFrameResult<Self> {
        let n = n.min(self.data.len());
        let start = self.data.len() - n;
        Ok(Self {
            data: self.data[start..].to_vec(),
        })
    }

    /// Describe - get statistical summary for numeric series
    pub fn describe(&self) -> DataFrameResult<HashMap<String, f64>> {
        let nums: Vec<f64> = self
            .data
            .iter()
            .filter_map(|v| v.to_double().ok())
            .collect();

        if nums.is_empty() {
            return Err(DataFrameError::InvalidOperation(
                "Cannot describe non-numeric series".to_string(),
            ));
        }

        let mut stats = HashMap::new();
        stats.insert("count".to_string(), nums.len() as f64);

        let sum: f64 = nums.iter().sum();
        let mean = sum / nums.len() as f64;
        stats.insert("mean".to_string(), mean);

        let mut sorted = nums.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());

        stats.insert("min".to_string(), sorted[0]);
        stats.insert("max".to_string(), sorted[sorted.len() - 1]);

        // Median
        let mid = sorted.len() / 2;
        let median = if sorted.len().is_multiple_of(2) {
            (sorted[mid - 1] + sorted[mid]) / 2.0
        } else {
            sorted[mid]
        };
        stats.insert("median".to_string(), median);

        // Standard deviation
        let variance = nums.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / nums.len() as f64;
        stats.insert("std".to_string(), variance.sqrt());

        Ok(stats)
    }

    /// Sum of numeric values
    pub fn sum(&self) -> DataFrameResult<f64> {
        let sum: f64 = self.data.iter().filter_map(|v| v.to_double().ok()).sum();
        Ok(sum)
    }

    /// Mean of numeric values
    pub fn mean(&self) -> DataFrameResult<f64> {
        let nums: Vec<f64> = self
            .data
            .iter()
            .filter_map(|v| v.to_double().ok())
            .collect();

        if nums.is_empty() {
            return Err(DataFrameError::InvalidOperation(
                "Cannot compute mean of empty or non-numeric series".to_string(),
            ));
        }

        Ok(nums.iter().sum::<f64>() / nums.len() as f64)
    }

    /// Get unique values
    pub fn unique(&self) -> Vec<XdlValue> {
        let mut unique_values = Vec::new();
        let mut seen = std::collections::HashSet::new();

        for value in &self.data {
            let key = value.to_string_repr();
            if seen.insert(key) {
                unique_values.push(value.clone());
            }
        }

        unique_values
    }

    /// Count of values
    pub fn count(&self) -> usize {
        self.data.len()
    }

    /// Value counts - return counts of unique values
    pub fn value_counts(&self) -> HashMap<String, usize> {
        let mut counts = HashMap::new();

        for value in &self.data {
            let key = value.to_string_repr();
            *counts.entry(key).or_insert(0) += 1;
        }

        counts
    }

    /// Apply a function to each element
    pub fn map<F>(&self, f: F) -> DataFrameResult<Self>
    where
        F: Fn(&XdlValue) -> XdlValue,
    {
        let mapped_data: Vec<XdlValue> = self.data.iter().map(f).collect();
        Self::from_vec(mapped_data)
    }

    /// Filter elements based on predicate
    pub fn filter<F>(&self, predicate: F) -> DataFrameResult<Self>
    where
        F: Fn(&XdlValue) -> bool,
    {
        let filtered_data: Vec<XdlValue> =
            self.data.iter().filter(|v| predicate(v)).cloned().collect();
        Self::from_vec(filtered_data)
    }

    /// Get the underlying data vector
    pub fn data(&self) -> &[XdlValue] {
        &self.data
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_series_creation() {
        let data = vec![XdlValue::Long(1), XdlValue::Long(2), XdlValue::Long(3)];
        let series = Series::from_vec(data).unwrap();
        assert_eq!(series.len(), 3);
    }

    #[test]
    fn test_series_get() {
        let data = vec![XdlValue::Long(1), XdlValue::Long(2)];
        let series = Series::from_vec(data).unwrap();
        assert!(matches!(series.get(0), Ok(XdlValue::Long(1))));
    }

    #[test]
    fn test_series_sum() {
        let data = vec![
            XdlValue::Double(1.0),
            XdlValue::Double(2.0),
            XdlValue::Double(3.0),
        ];
        let series = Series::from_vec(data).unwrap();
        assert_eq!(series.sum().unwrap(), 6.0);
    }

    #[test]
    fn test_series_mean() {
        let data = vec![
            XdlValue::Double(1.0),
            XdlValue::Double(2.0),
            XdlValue::Double(3.0),
        ];
        let series = Series::from_vec(data).unwrap();
        assert_eq!(series.mean().unwrap(), 2.0);
    }
}
