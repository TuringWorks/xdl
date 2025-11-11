//! Expression and statement evaluator

use crate::context::Context;
use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_parser::{ArrayIndex, BinaryOp, Expression, UnaryOp};
use xdl_stdlib::StandardLibrary;

/// Expression evaluator with context
pub struct Evaluator {
    stdlib: StandardLibrary,
}

impl Evaluator {
    pub fn new() -> Self {
        Self {
            stdlib: StandardLibrary::new(),
        }
    }

    /// Evaluate an expression in the given context
    pub fn evaluate(&self, expr: &Expression, context: &mut Context) -> XdlResult<XdlValue> {
        match expr {
            Expression::Literal { value, .. } => Ok(value.clone()),

            Expression::Variable { name, .. } => context.get_variable(name).cloned(),

            Expression::SystemVariable { name, .. } => context.get_system_variable(name).cloned(),

            Expression::Binary {
                op, left, right, ..
            } => {
                let left_val = self.evaluate(left, context)?;
                let right_val = self.evaluate(right, context)?;
                self.evaluate_binary_op(*op, &left_val, &right_val)
            }

            Expression::Unary { op, expr, .. } => {
                let val = self.evaluate(expr, context)?;
                self.evaluate_unary_op(*op, &val)
            }

            Expression::FunctionCall {
                name,
                args,
                keywords,
                ..
            } => {
                // First check if it's a user-defined function
                if let Some(_func_def) = context.get_function(name) {
                    // TODO: Implement user-defined function calls
                    return Err(XdlError::NotImplemented(
                        "User-defined functions".to_string(),
                    ));
                }

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate(arg, context)?);
                }

                // TODO: Handle keywords
                if !keywords.is_empty() {
                    return Err(XdlError::NotImplemented("Function keywords".to_string()));
                }

                // Call standard library function
                self.stdlib.call_function(name, &arg_values)
            }

            Expression::ArrayDef { elements, .. } => {
                // Evaluate all elements
                let mut values = Vec::new();
                for element in elements {
                    let val = self.evaluate(element, context)?;
                    values.push(val);
                }

                // Check if all elements are arrays (nested array/matrix)
                let all_arrays = values.iter().all(|v| matches!(v, XdlValue::Array(_)));

                if all_arrays && !values.is_empty() {
                    // This is a nested array (matrix)
                    Ok(XdlValue::NestedArray(values))
                } else {
                    // Regular array - convert all to floats
                    let mut float_values = Vec::new();
                    for val in values {
                        float_values.push(self.to_double(&val)?);
                    }
                    Ok(XdlValue::Array(float_values))
                }
            }

            Expression::ArrayRef { array, indices, .. } => {
                let array_val = self.evaluate(array, context)?;
                self.evaluate_array_ref(&array_val, indices, context)
            }

            Expression::StructRef { object, .. } => {
                let _obj_val = self.evaluate(object, context)?;
                // TODO: Implement structure field access
                Err(XdlError::NotImplemented(
                    "Structure field access".to_string(),
                ))
            }

            Expression::MethodCall {
                object,
                method,
                args,
                ..
            } => {
                // Handle special cases for Python integration
                if let Expression::Variable { name, .. } = object.as_ref() {
                    if name == "Python" && method == "Import" {
                        return self.handle_python_import(args, context);
                    }
                }

                // Generic method call handling
                let _obj_val = self.evaluate(object, context)?;
                // TODO: Implement generic method calls
                Err(XdlError::NotImplemented("Method calls".to_string()))
            }

            _ => Err(XdlError::NotImplemented(format!(
                "Expression type: {:?}",
                expr
            ))),
        }
    }

    /// Call a procedure from the standard library
    pub fn call_procedure(&self, name: &str, args: &[XdlValue]) -> XdlResult<XdlValue> {
        self.stdlib.call_procedure(name, args)
    }

    /// Evaluate binary operations
    pub fn evaluate_binary_op(
        &self,
        op: BinaryOp,
        left: &XdlValue,
        right: &XdlValue,
    ) -> XdlResult<XdlValue> {
        use BinaryOp::*;
        use XdlValue::*;

        // Handle array operations
        match (left, right) {
            (Array(a), Array(b)) => {
                if a.len() != b.len() {
                    return Err(XdlError::RuntimeError(
                        "Array dimensions must match for operations".to_string(),
                    ));
                }
                let result: Vec<f64> = match op {
                    Add => a.iter().zip(b.iter()).map(|(x, y)| x + y).collect(),
                    Subtract => a.iter().zip(b.iter()).map(|(x, y)| x - y).collect(),
                    Multiply => a.iter().zip(b.iter()).map(|(x, y)| x * y).collect(),
                    Divide => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if *y == 0.0 { f64::NAN } else { x / y })
                        .collect(),
                    Power => a.iter().zip(b.iter()).map(|(x, y)| x.powf(*y)).collect(),
                    Modulo => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if *y == 0.0 { f64::NAN } else { x % y })
                        .collect(),
                    // Comparison operators for arrays
                    Equal => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| {
                            if (x - y).abs() < f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    NotEqual => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| {
                            if (x - y).abs() >= f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    Less => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if x < y { 1.0 } else { 0.0 })
                        .collect(),
                    Greater => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if x > y { 1.0 } else { 0.0 })
                        .collect(),
                    LessEqual => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if x <= y { 1.0 } else { 0.0 })
                        .collect(),
                    GreaterEqual => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if x >= y { 1.0 } else { 0.0 })
                        .collect(),
                    And => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if *x != 0.0 && *y != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    Or => a
                        .iter()
                        .zip(b.iter())
                        .map(|(x, y)| if *x != 0.0 || *y != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    _ => {
                        return Err(XdlError::NotImplemented(
                            "Array operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(Array(result));
            }
            (Array(a), scalar) => {
                let s = self.to_double(scalar)?;
                let result: Vec<f64> = match op {
                    Add => a.iter().map(|x| x + s).collect(),
                    Subtract => a.iter().map(|x| x - s).collect(),
                    Multiply => a.iter().map(|x| x * s).collect(),
                    Divide => a
                        .iter()
                        .map(|x| if s == 0.0 { f64::NAN } else { x / s })
                        .collect(),
                    Power => a.iter().map(|x| x.powf(s)).collect(),
                    Modulo => a
                        .iter()
                        .map(|x| if s == 0.0 { f64::NAN } else { x % s })
                        .collect(),
                    // Comparison operators
                    Equal => a
                        .iter()
                        .map(|x| {
                            if (x - s).abs() < f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    NotEqual => a
                        .iter()
                        .map(|x| {
                            if (x - s).abs() >= f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    Less => a.iter().map(|x| if x < &s { 1.0 } else { 0.0 }).collect(),
                    Greater => a.iter().map(|x| if x > &s { 1.0 } else { 0.0 }).collect(),
                    LessEqual => a.iter().map(|x| if x <= &s { 1.0 } else { 0.0 }).collect(),
                    GreaterEqual => a.iter().map(|x| if x >= &s { 1.0 } else { 0.0 }).collect(),
                    And => a
                        .iter()
                        .map(|x| if *x != 0.0 && s != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    Or => a
                        .iter()
                        .map(|x| if *x != 0.0 || s != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    _ => {
                        return Err(XdlError::NotImplemented(
                            "Array-scalar operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(Array(result));
            }
            (scalar, Array(a)) => {
                let s = self.to_double(scalar)?;
                let result: Vec<f64> = match op {
                    Add => a.iter().map(|x| s + x).collect(),
                    Subtract => a.iter().map(|x| s - x).collect(),
                    Multiply => a.iter().map(|x| s * x).collect(),
                    Divide => a
                        .iter()
                        .map(|x| if *x == 0.0 { f64::NAN } else { s / x })
                        .collect(),
                    Power => a.iter().map(|x| s.powf(*x)).collect(),
                    Modulo => a
                        .iter()
                        .map(|x| if *x == 0.0 { f64::NAN } else { s % x })
                        .collect(),
                    // Comparison operators
                    Equal => a
                        .iter()
                        .map(|x| {
                            if (s - x).abs() < f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    NotEqual => a
                        .iter()
                        .map(|x| {
                            if (s - x).abs() >= f64::EPSILON {
                                1.0
                            } else {
                                0.0
                            }
                        })
                        .collect(),
                    Less => a.iter().map(|x| if &s < x { 1.0 } else { 0.0 }).collect(),
                    Greater => a.iter().map(|x| if &s > x { 1.0 } else { 0.0 }).collect(),
                    LessEqual => a.iter().map(|x| if &s <= x { 1.0 } else { 0.0 }).collect(),
                    GreaterEqual => a.iter().map(|x| if &s >= x { 1.0 } else { 0.0 }).collect(),
                    And => a
                        .iter()
                        .map(|x| if s != 0.0 && *x != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    Or => a
                        .iter()
                        .map(|x| if s != 0.0 || *x != 0.0 { 1.0 } else { 0.0 })
                        .collect(),
                    _ => {
                        return Err(XdlError::NotImplemented(
                            "Scalar-array operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(Array(result));
            }
            _ => {} // Continue with scalar operations
        }

        match op {
            Add => match (left, right) {
                (Long(a), Long(b)) => Ok(Long(a + b)),
                (Long(a), Double(b)) => Ok(Double(*a as f64 + b)),
                (Double(a), Long(b)) => Ok(Double(a + *b as f64)),
                (Double(a), Double(b)) => Ok(Double(a + b)),
                (Float(a), Float(b)) => Ok(Float(a + b)),
                (String(a), String(b)) => Ok(String(format!("{}{}", a, b))),
                _ => self.try_numeric_conversion(left, right, |a, b| a + b),
            },

            Subtract => match (left, right) {
                (Long(a), Long(b)) => Ok(Long(a - b)),
                (Long(a), Double(b)) => Ok(Double(*a as f64 - b)),
                (Double(a), Long(b)) => Ok(Double(a - *b as f64)),
                (Double(a), Double(b)) => Ok(Double(a - b)),
                (Float(a), Float(b)) => Ok(Float(a - b)),
                _ => self.try_numeric_conversion(left, right, |a, b| a - b),
            },

            Multiply => match (left, right) {
                (Long(a), Long(b)) => Ok(Long(a * b)),
                (Long(a), Double(b)) => Ok(Double(*a as f64 * b)),
                (Double(a), Long(b)) => Ok(Double(a * *b as f64)),
                (Double(a), Double(b)) => Ok(Double(a * b)),
                (Float(a), Float(b)) => Ok(Float(a * b)),
                _ => self.try_numeric_conversion(left, right, |a, b| a * b),
            },

            Divide => match (left, right) {
                (Long(a), Long(b)) => {
                    if *b == 0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Long(a / b))
                    }
                }
                (Long(a), Double(b)) => {
                    if *b == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(*a as f64 / b))
                    }
                }
                (Double(a), Long(b)) => {
                    if *b == 0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a / *b as f64))
                    }
                }
                (Double(a), Double(b)) => {
                    if *b == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a / b))
                    }
                }
                _ => {
                    let a_f64 = self.to_double(left)?;
                    let b_f64 = self.to_double(right)?;
                    if b_f64 == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a_f64 / b_f64))
                    }
                }
            },

            Power => {
                let base = self.to_double(left)?;
                let exp = self.to_double(right)?;
                Ok(Double(base.powf(exp)))
            }

            Modulo => match (left, right) {
                (Long(a), Long(b)) => {
                    if *b == 0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Long(a % b))
                    }
                }
                (Long(a), Double(b)) => {
                    if *b == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double((*a as f64) % b))
                    }
                }
                (Double(a), Long(b)) => {
                    if *b == 0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a % (*b as f64)))
                    }
                }
                (Double(a), Double(b)) => {
                    if *b == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a % b))
                    }
                }
                _ => {
                    let a_f64 = self.to_double(left)?;
                    let b_f64 = self.to_double(right)?;
                    if b_f64 == 0.0 {
                        Err(XdlError::DivisionByZero)
                    } else {
                        Ok(Double(a_f64 % b_f64))
                    }
                }
            },

            // Comparison operators
            Equal => Ok(Long(if self.values_equal(left, right)? {
                1
            } else {
                0
            })),
            NotEqual => Ok(Long(if !self.values_equal(left, right)? {
                1
            } else {
                0
            })),
            Less => Ok(Long(if self.compare_values(left, right)? < 0 {
                1
            } else {
                0
            })),
            Greater => Ok(Long(if self.compare_values(left, right)? > 0 {
                1
            } else {
                0
            })),
            LessEqual => Ok(Long(if self.compare_values(left, right)? <= 0 {
                1
            } else {
                0
            })),
            GreaterEqual => Ok(Long(if self.compare_values(left, right)? >= 0 {
                1
            } else {
                0
            })),

            // Logical operators (XDL uses 0/1 for false/true)
            And => {
                let a_bool = !left.is_zero();
                let b_bool = !right.is_zero();
                Ok(Long(if a_bool && b_bool { 1 } else { 0 }))
            }
            Or => {
                let a_bool = !left.is_zero();
                let b_bool = !right.is_zero();
                Ok(Long(if a_bool || b_bool { 1 } else { 0 }))
            }

            _ => Err(XdlError::NotImplemented(format!(
                "Binary operator: {:?}",
                op
            ))),
        }
    }

    /// Evaluate unary operations
    fn evaluate_unary_op(&self, op: UnaryOp, val: &XdlValue) -> XdlResult<XdlValue> {
        use UnaryOp::*;
        use XdlValue::*;

        match op {
            Plus => Ok(val.clone()),
            Minus => match val {
                Long(v) => Ok(Long(-v)),
                Double(v) => Ok(Double(-v)),
                Float(v) => Ok(Float(-v)),
                Array(arr) => {
                    let result: Vec<f64> = arr.iter().map(|&x| -x).collect();
                    Ok(Array(result))
                }
                _ => {
                    let num_val = self.to_double(val)?;
                    Ok(Double(-num_val))
                }
            },
            Not => Ok(Long(if val.is_zero() { 1 } else { 0 })),
            _ => Err(XdlError::NotImplemented(format!(
                "Unary operator: {:?}",
                op
            ))),
        }
    }

    /// Try to convert both values to f64 and apply operation
    fn try_numeric_conversion<F>(
        &self,
        left: &XdlValue,
        right: &XdlValue,
        op: F,
    ) -> XdlResult<XdlValue>
    where
        F: Fn(f64, f64) -> f64,
    {
        let left_f64 = self.to_double(left)?;
        let right_f64 = self.to_double(right)?;
        Ok(XdlValue::Double(op(left_f64, right_f64)))
    }

    /// Convert a XdlValue to f64
    fn to_double(&self, val: &XdlValue) -> XdlResult<f64> {
        val.to_double()
    }

    /// Check if two values are equal
    fn values_equal(&self, left: &XdlValue, right: &XdlValue) -> XdlResult<bool> {
        use XdlValue::*;

        Ok(match (left, right) {
            (Long(a), Long(b)) => a == b,
            (Double(a), Double(b)) => (a - b).abs() < f64::EPSILON,
            (Float(a), Float(b)) => (a - b).abs() < f32::EPSILON,
            (String(a), String(b)) => a == b,
            (Undefined, Undefined) => true,
            // Try numeric conversion for mixed types
            _ => {
                if let (Ok(a), Ok(b)) = (self.to_double(left), self.to_double(right)) {
                    (a - b).abs() < f64::EPSILON
                } else {
                    false
                }
            }
        })
    }

    /// Compare two values (-1, 0, 1)
    fn compare_values(&self, left: &XdlValue, right: &XdlValue) -> XdlResult<i32> {
        use XdlValue::*;

        match (left, right) {
            (Long(a), Long(b)) => Ok(a.cmp(b) as i32),
            (Double(a), Double(b)) => {
                Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32)
            }
            (Float(a), Float(b)) => {
                Ok(a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal) as i32)
            }
            (String(a), String(b)) => Ok(a.cmp(b) as i32),
            // Try numeric conversion for mixed types
            _ => {
                let a_f64 = self.to_double(left)?;
                let b_f64 = self.to_double(right)?;
                Ok(a_f64
                    .partial_cmp(&b_f64)
                    .unwrap_or(std::cmp::Ordering::Equal) as i32)
            }
        }
    }

    /// Evaluate array indexing and slicing
    fn evaluate_array_ref(
        &self,
        array_val: &XdlValue,
        indices: &[ArrayIndex],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        // Handle multi-dimensional indexing by applying indices one at a time
        let mut current_val = array_val.clone();

        for index in indices {
            current_val = self.evaluate_single_index(&current_val, index, context)?;
        }

        Ok(current_val)
    }

    /// Evaluate a single index operation
    fn evaluate_single_index(
        &self,
        array_val: &XdlValue,
        index: &ArrayIndex,
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        // Handle nested arrays (matrices)
        if let XdlValue::NestedArray(rows) = array_val {
            match index {
                ArrayIndex::Single(expr) => {
                    let index_val = self.evaluate(expr, context)?;
                    let raw_index = index_val.to_long()?;

                    // Handle negative indices
                    let idx = if raw_index < 0 {
                        let len = rows.len() as i32;
                        let positive_idx = len + raw_index;
                        if positive_idx < 0 {
                            return Err(XdlError::RuntimeError(format!(
                                "Index {} out of bounds for array of length {}",
                                raw_index,
                                rows.len()
                            )));
                        }
                        positive_idx as usize
                    } else {
                        raw_index as usize
                    };

                    if idx >= rows.len() {
                        return Err(XdlError::RuntimeError(format!(
                            "Index {} out of bounds for array of length {}",
                            raw_index,
                            rows.len()
                        )));
                    }

                    return Ok(rows[idx].clone());
                }
                _ => {
                    return Err(XdlError::NotImplemented(
                        "Range indexing on nested arrays not yet supported".to_string(),
                    ));
                }
            }
        }

        let arr = match array_val {
            XdlValue::Array(a) => a,
            _ => {
                return Err(XdlError::RuntimeError(
                    "Cannot index non-array value".to_string(),
                ))
            }
        };

        match index {
            ArrayIndex::Single(expr) => {
                // Single element access: arr[i] or arr[-i]
                let index_val = self.evaluate(expr, context)?;
                let raw_index = index_val.to_long()?;

                // Handle negative indices
                let index = if raw_index < 0 {
                    let len = arr.len() as i32;
                    let positive_idx = len + raw_index;
                    if positive_idx < 0 {
                        return Err(XdlError::RuntimeError(format!(
                            "Index {} out of bounds for array of length {}",
                            raw_index,
                            arr.len()
                        )));
                    }
                    positive_idx as usize
                } else {
                    raw_index as usize
                };

                if index >= arr.len() {
                    return Err(XdlError::RuntimeError(format!(
                        "Index {} out of bounds for array of length {}",
                        raw_index,
                        arr.len()
                    )));
                }

                Ok(XdlValue::Double(arr[index]))
            }

            ArrayIndex::Range { start, end, step } => {
                // Range access: arr[start:end] or arr[start:end:step]
                let start_idx = if let Some(s) = start {
                    let val = self.evaluate(s, context)?;
                    val.to_long()? as usize
                } else {
                    0
                };

                let end_idx = if let Some(e) = end {
                    let val = self.evaluate(e, context)?;
                    let idx = val.to_long()? as usize;
                    idx.min(arr.len())
                } else {
                    arr.len()
                };

                let step_val = if let Some(s) = step {
                    let val = self.evaluate(s, context)?;
                    val.to_long()?
                } else {
                    1
                };

                if step_val == 0 {
                    return Err(XdlError::RuntimeError(
                        "Array slice step cannot be zero".to_string(),
                    ));
                }

                if step_val < 0 {
                    return Err(XdlError::NotImplemented(
                        "Negative step in array slicing".to_string(),
                    ));
                }

                // Extract slice
                let mut result = Vec::new();
                let mut i = start_idx;
                while i < end_idx && i < arr.len() {
                    result.push(arr[i]);
                    i += step_val as usize;
                }

                Ok(XdlValue::Array(result))
            }

            ArrayIndex::All => {
                // Return entire array
                Ok(array_val.clone())
            }
        }
    }

    /// Handle Python.Import() method calls
    fn handle_python_import(
        &self,
        args: &[Expression],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        if args.len() != 1 {
            return Err(XdlError::RuntimeError(
                "Python.Import() requires exactly one argument".to_string(),
            ));
        }

        let module_name_expr = &args[0];
        let module_name_val = self.evaluate(module_name_expr, context)?;

        let module_name = match module_name_val {
            XdlValue::String(s) => s,
            _ => {
                return Err(XdlError::RuntimeError(
                    "Python.Import() argument must be a string".to_string(),
                ))
            }
        };

        // For now, return a mock Python module object
        // In a real implementation, this would interface with Python via FFI
        Ok(XdlValue::String(format!(
            "<Python module: {}>",
            module_name
        )))
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_literal_evaluation() {
        let evaluator = Evaluator::new();
        let mut context = Context::new();

        let expr = Expression::Literal {
            value: XdlValue::Long(42),
            location: xdl_parser::Location::unknown(),
        };

        let result = evaluator.evaluate(&expr, &mut context).unwrap();
        assert_eq!(result, XdlValue::Long(42));
    }

    #[test]
    fn test_binary_arithmetic() {
        let evaluator = Evaluator::new();
        let mut context = Context::new();

        let expr = Expression::Binary {
            op: BinaryOp::Add,
            left: Box::new(Expression::Literal {
                value: XdlValue::Long(2),
                location: xdl_parser::Location::unknown(),
            }),
            right: Box::new(Expression::Literal {
                value: XdlValue::Long(3),
                location: xdl_parser::Location::unknown(),
            }),
            location: xdl_parser::Location::unknown(),
        };

        let result = evaluator.evaluate(&expr, &mut context).unwrap();
        assert_eq!(result, XdlValue::Long(5));
    }

    #[test]
    fn test_variable_lookup() {
        let evaluator = Evaluator::new();
        let mut context = Context::new();

        context.set_variable("x".to_string(), XdlValue::Double(3.14));

        let expr = Expression::Variable {
            name: "x".to_string(),
            location: xdl_parser::Location::unknown(),
        };

        let result = evaluator.evaluate(&expr, &mut context).unwrap();
        assert_eq!(result, XdlValue::Double(3.14));
    }

    #[test]
    fn test_system_variable() {
        let evaluator = Evaluator::new();
        let mut context = Context::new();

        let expr = Expression::SystemVariable {
            name: "PI".to_string(),
            location: xdl_parser::Location::unknown(),
        };

        let result = evaluator.evaluate(&expr, &mut context).unwrap();
        match result {
            XdlValue::Double(val) => assert!((val - std::f64::consts::PI).abs() < 1e-10),
            _ => panic!("PI should be a Double"),
        }
    }
}
