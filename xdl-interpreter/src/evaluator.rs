//! Expression and statement evaluator

use std::collections::HashMap;

use crate::context::Context;
use crate::methods;
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

            Expression::Variable { name, .. } => {
                // Handle SELF as a special variable
                if name.eq_ignore_ascii_case("SELF") {
                    context.get_self()
                } else {
                    context.get_variable(name).cloned()
                }
            }

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

            Expression::Ternary {
                condition,
                if_true,
                if_false,
                ..
            } => {
                let cond_val = self.evaluate(condition, context)?;
                let is_true = self.to_bool(&cond_val);
                if is_true {
                    self.evaluate(if_true, context)
                } else {
                    self.evaluate(if_false, context)
                }
            }

            Expression::FunctionCall {
                name,
                args,
                keywords,
                ..
            } => {
                // User-defined function calls are handled by the Interpreter
                // (which intercepts them before calling the Evaluator)
                // Here we only handle built-in stdlib functions

                // Evaluate arguments
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate(arg, context)?);
                }

                // Evaluate keywords into a HashMap
                let mut keyword_values: HashMap<String, XdlValue> = HashMap::new();
                for kw in keywords {
                    let value = if let Some(ref expr) = kw.value {
                        self.evaluate(expr, context)?
                    } else {
                        // Flag-style keyword (e.g., /INDEX) - set to 1 (true)
                        XdlValue::Long(1)
                    };
                    keyword_values.insert(kw.name.to_uppercase(), value);
                }

                // Handle DataFrame functions that need Context access
                match name.to_uppercase().as_str() {
                    "XDLDATAFRAME_READ_CSV" => {
                        if arg_values.is_empty() {
                            return Err(XdlError::InvalidArgument(
                                "XDLDATAFRAME_READ_CSV requires a filename argument".to_string(),
                            ));
                        }

                        let filename = match &arg_values[0] {
                            XdlValue::String(s) => s.clone(),
                            _ => arg_values[0].to_string_repr(),
                        };

                        // Default delimiter is comma
                        let delimiter = if arg_values.len() > 1 {
                            match &arg_values[1] {
                                XdlValue::String(s) if !s.is_empty() => s.as_bytes()[0],
                                _ => b',',
                            }
                        } else {
                            b','
                        };

                        // Create CSV reader options
                        let options =
                            xdl_dataframe::CsvReaderOptions::default().with_delimiter(delimiter);

                        // Read the CSV file
                        let df = xdl_dataframe::read_csv(&filename, options).map_err(|e| {
                            XdlError::RuntimeError(format!("CSV read error: {}", e))
                        })?;

                        // Store DataFrame in Context and return ID
                        let id = context.store_dataframe(df);
                        Ok(XdlValue::DataFrame(id))
                    }
                    _ => {
                        // Call standard library function with keywords
                        self.stdlib
                            .call_function_with_keywords(name, &arg_values, &keyword_values)
                    }
                }
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

            Expression::StructRef { object, field, .. } => {
                let obj_val = self.evaluate(object, context)?;

                match obj_val {
                    XdlValue::Object(obj_id) => {
                        // Get the object instance
                        let obj_instance = context.get_object(obj_id)?;

                        // Get the field value
                        obj_instance.get_field(field).cloned().ok_or_else(|| {
                            XdlError::RuntimeError(format!("Object has no field named '{}'", field))
                        })
                    }
                    XdlValue::Struct(ref map) => {
                        // Handle regular structs
                        map.get(&field.to_uppercase()).cloned().ok_or_else(|| {
                            XdlError::RuntimeError(format!("Struct has no field named '{}'", field))
                        })
                    }
                    _ => Err(XdlError::TypeMismatch {
                        expected: "object or struct".to_string(),
                        actual: format!("{:?}", obj_val.gdl_type()),
                    }),
                }
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

                // Evaluate the object
                let obj_val = self.evaluate(object, context)?;

                // Evaluate method arguments for built-in type methods
                let mut arg_values = Vec::new();
                for arg in args {
                    arg_values.push(self.evaluate(arg, context)?);
                }

                // Dispatch based on object type
                match obj_val {
                    // DataFrame methods (use unevaluated args for context access)
                    XdlValue::DataFrame(id) => {
                        self.call_dataframe_method(id, method, args, context)
                    }

                    // User-defined object methods (use unevaluated args)
                    XdlValue::Object(obj_id) => {
                        self.call_user_method(obj_id, method, args, context)
                    }

                    // Array methods: arr->Sum(), arr->Mean(), arr->Sort(), etc.
                    XdlValue::Array(ref arr) => {
                        methods::call_array_method(arr, method, &arg_values)
                    }

                    // MultiDimArray methods: arr->Shape(), arr->Flatten(), etc.
                    XdlValue::MultiDimArray { ref data, ref shape } => {
                        methods::call_multidim_method(data, shape, method, &arg_values)
                    }

                    // NestedArray methods: matrix->NRows(), matrix->Flatten(), etc.
                    XdlValue::NestedArray(ref rows) => {
                        methods::call_nested_array_method(rows, method, &arg_values)
                    }

                    // String methods: str->ToUpper(), str->Length(), str->Contains(), etc.
                    XdlValue::String(ref s) => {
                        methods::call_string_method(s, method, &arg_values)
                    }

                    // Structs don't have methods - use dot notation for field access
                    XdlValue::Struct(ref _map) => Err(XdlError::TypeMismatch {
                        expected: "object with methods (use obj.field for struct field access)"
                            .to_string(),
                        actual: "Struct".to_string(),
                    }),

                    // Unsupported types
                    _ => Err(XdlError::TypeMismatch {
                        expected: "Array, String, DataFrame, or Object".to_string(),
                        actual: format!("{:?}", obj_val.gdl_type()),
                    }),
                }
            }

            Expression::ObjectNew {
                class_name,
                args,
                keywords,
                ..
            } => {
                // Handle empty OBJ_NEW() which returns NULL
                if class_name.is_empty() {
                    return Ok(XdlValue::Object(0));
                }

                // Get the class definition and clone the fields
                let (default_fields, has_init) = {
                    let class = context.get_class(class_name)?;
                    (class.fields.clone(), class.get_method("INIT").is_some())
                };

                // Create a new object instance with default fields
                let obj_id = context.create_object(class_name.clone(), &default_fields);

                // Evaluate constructor arguments
                let mut _arg_values = Vec::new();
                for arg in args {
                    _arg_values.push(self.evaluate(arg, context)?);
                }

                // Call Init method if it exists
                if has_init {
                    // TODO: Implement full method dispatch with SELF support
                    // For now, we'll skip calling Init
                    // When properly implemented, Init should be called with obj_id and args
                    // If Init returns 0, the object should be destroyed and NULL returned
                }

                // TODO: Handle keywords
                if !keywords.is_empty() {
                    return Err(XdlError::NotImplemented(
                        "OBJ_NEW keywords not yet supported".to_string(),
                    ));
                }

                Ok(XdlValue::Object(obj_id))
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

    /// Call a procedure from the standard library with keyword arguments
    pub fn call_procedure_with_keywords(
        &self,
        name: &str,
        args: &[XdlValue],
        keywords: &std::collections::HashMap<String, XdlValue>,
    ) -> XdlResult<XdlValue> {
        self.stdlib
            .call_procedure_with_keywords(name, args, keywords)
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
            // Handle MultiDimArray × MultiDimArray
            (
                MultiDimArray {
                    data: a,
                    shape: shape_a,
                },
                MultiDimArray {
                    data: b,
                    shape: shape_b,
                },
            ) => {
                if shape_a != shape_b {
                    return Err(XdlError::RuntimeError(format!(
                        "MultiDimArray dimensions must match for operations: {:?} vs {:?}",
                        shape_a, shape_b
                    )));
                }
                let result_data: Vec<f64> = match op {
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
                            "MultiDimArray operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(MultiDimArray {
                    data: result_data,
                    shape: shape_a.clone(),
                });
            }
            // Handle MultiDimArray × scalar
            (MultiDimArray { data: a, shape }, scalar) => {
                let s = self.to_double(scalar)?;
                let result_data: Vec<f64> = match op {
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
                            "MultiDimArray-scalar operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(MultiDimArray {
                    data: result_data,
                    shape: shape.clone(),
                });
            }
            // Handle scalar × MultiDimArray
            (scalar, MultiDimArray { data: a, shape }) => {
                let s = self.to_double(scalar)?;
                let result_data: Vec<f64> = match op {
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
                            "Scalar-MultiDimArray operation not implemented".to_string(),
                        ))
                    }
                };
                return Ok(MultiDimArray {
                    data: result_data,
                    shape: shape.clone(),
                });
            }
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

    /// Convert a XdlValue to boolean for ternary operator
    fn to_bool(&self, val: &XdlValue) -> bool {
        match val {
            XdlValue::Long(i) => *i != 0,
            XdlValue::Long64(i) => *i != 0,
            XdlValue::Int(i) => *i != 0,
            XdlValue::Byte(b) => *b != 0,
            XdlValue::Float(f) => *f != 0.0,
            XdlValue::Double(d) => *d != 0.0,
            XdlValue::String(s) => !s.is_empty(),
            XdlValue::Array(arr) => !arr.is_empty(),
            XdlValue::NestedArray(arr) => !arr.is_empty(),
            XdlValue::Undefined => false,
            _ => true, // Objects, structs, etc. are truthy
        }
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
        // Handle MultiDimArray with all indices at once
        if let XdlValue::MultiDimArray { data, shape } = array_val {
            return self.evaluate_multidim_index(data, shape, indices, context);
        }

        // Handle other array types by applying indices one at a time
        let mut current_val = array_val.clone();

        for index in indices {
            current_val = self.evaluate_single_index(&current_val, index, context)?;
        }

        Ok(current_val)
    }

    /// Evaluate multi-dimensional array indexing
    fn evaluate_multidim_index(
        &self,
        data: &[f64],
        shape: &[usize],
        indices: &[ArrayIndex],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        if indices.is_empty() {
            return Ok(XdlValue::MultiDimArray {
                data: data.to_vec(),
                shape: shape.to_vec(),
            });
        }

        // Check for slice extraction with wildcards (e.g., u[*, *, k])
        let has_wildcard = indices.iter().any(|idx| matches!(idx, ArrayIndex::All));

        if has_wildcard {
            return self.evaluate_multidim_slice(data, shape, indices, context);
        }

        // All indices are single values - extract a single element or sub-array
        let mut evaluated_indices = Vec::new();
        for idx in indices {
            match idx {
                ArrayIndex::Single(expr) => {
                    let val = self.evaluate(expr, context)?;
                    let i = val.to_long()?;
                    evaluated_indices.push(i);
                }
                ArrayIndex::Range { .. } => {
                    return self.evaluate_multidim_slice(data, shape, indices, context);
                }
                ArrayIndex::All => unreachable!(),
            }
        }

        // If fewer indices than dimensions, return a sub-array
        if evaluated_indices.len() < shape.len() {
            // For column-major, we need to extract a slice
            // Use the slice extraction with remaining dimensions as All
            let mut ranges: Vec<(usize, usize, usize)> = Vec::new();
            let mut result_shape = Vec::new();

            for (i, &idx) in evaluated_indices.iter().enumerate() {
                let actual_idx = if idx < 0 {
                    (shape[i] as i32 + idx) as usize
                } else {
                    idx as usize
                };
                if actual_idx >= shape[i] {
                    return Err(XdlError::RuntimeError(format!(
                        "Index {} out of bounds for dimension {} of size {}",
                        idx, i, shape[i]
                    )));
                }
                ranges.push((actual_idx, actual_idx + 1, 1));
            }

            // Add remaining dimensions as full ranges
            for &dim_size in shape.iter().skip(evaluated_indices.len()) {
                ranges.push((0, dim_size, 1));
                result_shape.push(dim_size);
            }

            let mut result_data = Vec::new();
            self.extract_slice_recursive(data, shape, &ranges, 0, 0, 1, &mut result_data);

            if result_shape.len() == 1 {
                return Ok(XdlValue::Array(result_data));
            }
            return Ok(XdlValue::MultiDimArray {
                data: result_data,
                shape: result_shape,
            });
        }

        // Full indexing - return single element
        if evaluated_indices.len() != shape.len() {
            return Err(XdlError::RuntimeError(format!(
                "Expected {} indices for {}-dimensional array, got {}",
                shape.len(),
                shape.len(),
                evaluated_indices.len()
            )));
        }

        // Calculate linear index (column-major order like IDL/GDL)
        // For shape [nx, ny, nz] and indices [i, j, k]:
        // linear_idx = i + j*nx + k*nx*ny
        let mut linear_idx = 0;
        let mut stride = 1;
        for i in 0..shape.len() {
            let idx = evaluated_indices[i];
            let actual_idx = if idx < 0 {
                (shape[i] as i32 + idx) as usize
            } else {
                idx as usize
            };
            if actual_idx >= shape[i] {
                return Err(XdlError::RuntimeError(format!(
                    "Index {} out of bounds for dimension {} of size {}",
                    idx, i, shape[i]
                )));
            }
            linear_idx += actual_idx * stride;
            stride *= shape[i];
        }

        if linear_idx >= data.len() {
            return Err(XdlError::RuntimeError(format!(
                "Computed index {} out of bounds for array of size {}",
                linear_idx,
                data.len()
            )));
        }

        Ok(XdlValue::Double(data[linear_idx]))
    }

    /// Evaluate multi-dimensional slice extraction (with wildcards or ranges)
    fn evaluate_multidim_slice(
        &self,
        data: &[f64],
        shape: &[usize],
        indices: &[ArrayIndex],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        // Build range for each dimension
        let mut ranges: Vec<(usize, usize, usize)> = Vec::new(); // (start, end, step)
        let mut result_shape = Vec::new();

        for (dim, idx) in indices.iter().enumerate() {
            if dim >= shape.len() {
                return Err(XdlError::RuntimeError(format!(
                    "Too many indices ({}) for {}-dimensional array",
                    indices.len(),
                    shape.len()
                )));
            }
            let dim_size = shape[dim];

            match idx {
                ArrayIndex::All => {
                    ranges.push((0, dim_size, 1));
                    result_shape.push(dim_size);
                }
                ArrayIndex::Single(expr) => {
                    let val = self.evaluate(expr, context)?;
                    let i = val.to_long()?;
                    let actual_idx = if i < 0 {
                        (dim_size as i32 + i) as usize
                    } else {
                        i as usize
                    };
                    if actual_idx >= dim_size {
                        return Err(XdlError::RuntimeError(format!(
                            "Index {} out of bounds for dimension {} of size {}",
                            i, dim, dim_size
                        )));
                    }
                    ranges.push((actual_idx, actual_idx + 1, 1));
                    // Single index collapses dimension - don't add to result_shape
                }
                ArrayIndex::Range { start, end, step } => {
                    let s = if let Some(e) = start {
                        self.evaluate(e, context)?.to_long()? as usize
                    } else {
                        0
                    };
                    let e = if let Some(e) = end {
                        (self.evaluate(e, context)?.to_long()? as usize + 1).min(dim_size)
                    } else {
                        dim_size
                    };
                    let st = if let Some(e) = step {
                        self.evaluate(e, context)?.to_long()? as usize
                    } else {
                        1
                    };
                    ranges.push((s, e, st));
                    let range_size = e.saturating_sub(s).div_ceil(st);
                    result_shape.push(range_size);
                }
            }
        }

        // Add remaining dimensions if not fully indexed
        for &dim_size in shape.iter().skip(indices.len()) {
            ranges.push((0, dim_size, 1));
            result_shape.push(dim_size);
        }

        // Extract data (start with stride=1 for column-major)
        let mut result_data = Vec::new();
        self.extract_slice_recursive(data, shape, &ranges, 0, 0, 1, &mut result_data);

        if result_shape.is_empty() {
            // Scalar result
            Ok(XdlValue::Double(result_data[0]))
        } else if result_shape.len() == 1 {
            Ok(XdlValue::Array(result_data))
        } else {
            Ok(XdlValue::MultiDimArray {
                data: result_data,
                shape: result_shape,
            })
        }
    }

    /// Recursively extract slice data (column-major order)
    #[allow(clippy::too_many_arguments, clippy::only_used_in_recursion)]
    fn extract_slice_recursive(
        &self,
        data: &[f64],
        shape: &[usize],
        ranges: &[(usize, usize, usize)],
        dim: usize,
        base_offset: usize,
        current_stride: usize,
        result: &mut Vec<f64>,
    ) {
        if dim >= ranges.len() {
            if base_offset < data.len() {
                result.push(data[base_offset]);
            }
            return;
        }

        let (start, end, step) = ranges[dim];
        // Column-major: stride for dimension i is product of shape[0..i]
        let next_stride = current_stride * shape[dim];

        let mut i = start;
        while i < end {
            let offset = base_offset + i * current_stride;
            self.extract_slice_recursive(data, shape, ranges, dim + 1, offset, next_stride, result);
            i += step;
        }
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

        // Use real Python integration when the python feature is enabled
        #[cfg(feature = "python")]
        {
            use xdl_stdlib::python;
            python::python_import(&[XdlValue::String(module_name)])
        }

        #[cfg(not(feature = "python"))]
        {
            // Return a mock Python module object when Python feature is disabled
            Ok(XdlValue::String(format!(
                "<Python module: {}> (Note: Build with --features python for real Python integration)",
                module_name
            )))
        }
    }

    /// Call a method on a DataFrame
    fn call_dataframe_method(
        &self,
        df_id: usize,
        method: &str,
        args: &[Expression],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        match method.to_uppercase().as_str() {
            // === Shape and size information ===
            "SHAPE" => {
                let df = context.get_dataframe(df_id)?;
                let rows = df.nrows() as f64;
                let cols = df.ncols() as f64;
                Ok(XdlValue::Array(vec![rows, cols]))
            }

            "NROWS" | "HEIGHT" | "LEN" | "LENGTH" => {
                let df = context.get_dataframe(df_id)?;
                Ok(XdlValue::Long(df.nrows() as i32))
            }

            "NCOLS" | "WIDTH" => {
                let df = context.get_dataframe(df_id)?;
                Ok(XdlValue::Long(df.ncols() as i32))
            }

            // === Column information ===
            "COLUMNNAMES" | "COLUMN_NAMES" | "COLUMNS" => {
                let df = context.get_dataframe(df_id)?;
                let names: Vec<XdlValue> = df
                    .column_names()
                    .iter()
                    .map(|s| XdlValue::String(s.clone()))
                    .collect();
                Ok(XdlValue::NestedArray(names))
            }

            // === Column access ===
            "COLUMN" | "COL" => {
                if args.is_empty() {
                    return Err(XdlError::RuntimeError(
                        "Column() requires a column name argument".to_string(),
                    ));
                }

                let col_name_val = self.evaluate(&args[0], context)?;
                let col_name = match col_name_val {
                    XdlValue::String(s) => s,
                    _ => col_name_val.to_string_repr(),
                };

                let df = context.get_dataframe(df_id)?;
                let series = df
                    .column(&col_name)
                    .map_err(|e| XdlError::RuntimeError(format!("Column error: {}", e)))?;

                // Convert series data to XdlValue::Array
                let data: Vec<f64> = series
                    .data()
                    .iter()
                    .map(|v| match v {
                        XdlValue::Long(n) => *n as f64,
                        XdlValue::Double(d) => *d,
                        XdlValue::Float(f) => *f as f64,
                        _ => 0.0,
                    })
                    .collect();

                Ok(XdlValue::Array(data))
            }

            // === Row access ===
            "ROW" => {
                if args.is_empty() {
                    return Err(XdlError::RuntimeError(
                        "Row() requires a row index argument".to_string(),
                    ));
                }

                let idx_val = self.evaluate(&args[0], context)?;
                let idx = idx_val.to_long()? as usize;

                let df = context.get_dataframe(df_id)?;
                let row_map = df
                    .row(idx)
                    .map_err(|e| XdlError::RuntimeError(format!("Row error: {}", e)))?;

                // Convert to struct
                let struct_map: std::collections::HashMap<String, XdlValue> = row_map
                    .into_iter()
                    .map(|(k, v)| (k.to_uppercase(), v))
                    .collect();
                Ok(XdlValue::Struct(struct_map))
            }

            // === Slicing ===
            "HEAD" => {
                let n = if args.is_empty() {
                    5
                } else {
                    self.evaluate(&args[0], context)?.to_long()? as usize
                };

                let df = context.get_dataframe(df_id)?;
                let head_df = df
                    .head(n)
                    .map_err(|e| XdlError::RuntimeError(format!("Head error: {}", e)))?;
                let new_id = context.store_dataframe(head_df);
                Ok(XdlValue::DataFrame(new_id))
            }

            "TAIL" => {
                let n = if args.is_empty() {
                    5
                } else {
                    self.evaluate(&args[0], context)?.to_long()? as usize
                };

                let df = context.get_dataframe(df_id)?;
                let tail_df = df
                    .tail(n)
                    .map_err(|e| XdlError::RuntimeError(format!("Tail error: {}", e)))?;
                let new_id = context.store_dataframe(tail_df);
                Ok(XdlValue::DataFrame(new_id))
            }

            // === Statistics ===
            "DESCRIBE" | "INFO" => {
                let df = context.get_dataframe(df_id)?;
                Ok(XdlValue::String(df.info()))
            }

            // === I/O ===
            "WRITECSV" | "WRITE_CSV" | "TOCSV" | "TO_CSV" => {
                if args.is_empty() {
                    return Err(XdlError::RuntimeError(
                        "WriteCSV() requires a filename argument".to_string(),
                    ));
                }

                let filename_val = self.evaluate(&args[0], context)?;
                let filename = match filename_val {
                    XdlValue::String(s) => s,
                    _ => filename_val.to_string_repr(),
                };

                let df = context.get_dataframe(df_id)?;
                xdl_dataframe::write_csv(df, &filename, b',')
                    .map_err(|e| XdlError::RuntimeError(format!("CSV write error: {}", e)))?;

                Ok(XdlValue::Undefined)
            }

            "TOJSON" | "TO_JSON" => {
                let df = context.get_dataframe(df_id)?;
                let json = df.to_json();
                Ok(XdlValue::String(format!("{:?}", json)))
            }

            // === Selection ===
            "SELECT" => {
                if args.is_empty() {
                    return Err(XdlError::RuntimeError(
                        "Select() requires column name arguments".to_string(),
                    ));
                }

                // Evaluate all column name arguments
                let mut col_names = Vec::new();
                for arg in args {
                    let val = self.evaluate(arg, context)?;
                    match val {
                        XdlValue::String(s) => col_names.push(s),
                        _ => col_names.push(val.to_string_repr()),
                    }
                }

                let col_refs: Vec<&str> = col_names.iter().map(|s| s.as_str()).collect();
                let df = context.get_dataframe(df_id)?;
                let selected = df
                    .select(&col_refs)
                    .map_err(|e| XdlError::RuntimeError(format!("Select error: {}", e)))?;
                let new_id = context.store_dataframe(selected);
                Ok(XdlValue::DataFrame(new_id))
            }

            // === Sorting ===
            "SORTBY" | "SORT_BY" | "SORT" => {
                if args.is_empty() {
                    return Err(XdlError::RuntimeError(
                        "SortBy() requires column name argument(s)".to_string(),
                    ));
                }

                // First arg is column name(s), optional second arg is ascending (default true)
                let col_val = self.evaluate(&args[0], context)?;
                let col_names = match col_val {
                    XdlValue::String(s) => vec![s],
                    _ => vec![col_val.to_string_repr()],
                };

                let ascending = if args.len() > 1 {
                    match self.evaluate(&args[1], context)? {
                        XdlValue::Long(n) => n != 0,
                        _ => true,
                    }
                } else {
                    true
                };

                let col_refs: Vec<&str> = col_names.iter().map(|s| s.as_str()).collect();
                let df = context.get_dataframe(df_id)?;
                let sorted = df
                    .sort_by(&col_refs, ascending)
                    .map_err(|e| XdlError::RuntimeError(format!("Sort error: {}", e)))?;
                let new_id = context.store_dataframe(sorted);
                Ok(XdlValue::DataFrame(new_id))
            }

            _ => Err(XdlError::NotImplemented(format!(
                "DataFrame method '{}'. Available: Shape, NRows, NCols, ColumnNames, \
                 Column, Row, Head, Tail, Describe, WriteCSV, ToJson, Select, SortBy",
                method
            ))),
        }
    }

    /// Call a user-defined method on an object
    fn call_user_method(
        &self,
        obj_id: usize,
        method_name: &str,
        args: &[Expression],
        context: &mut Context,
    ) -> XdlResult<XdlValue> {
        // Get the object to find its class
        let class_name = {
            let obj = context.get_object(obj_id)?;
            obj.class_name.clone()
        };

        // Get the class definition
        let class = context.get_class(&class_name)?;

        // Get the method definition
        let method = class
            .get_method(method_name)
            .ok_or_else(|| {
                XdlError::RuntimeError(format!(
                    "Class '{}' has no method '{}'",
                    class_name, method_name
                ))
            })?
            .clone();

        // Set SELF to point to this object
        context.set_self(obj_id);

        // Push new scope for method execution
        context.push_scope();

        // Evaluate arguments
        let mut arg_values = Vec::new();
        for arg_expr in args {
            arg_values.push(self.evaluate(arg_expr, context)?);
        }

        // Bind parameters to arguments
        for (i, param) in method.params.iter().enumerate() {
            if i < arg_values.len() {
                context.set_variable(param.name.clone(), arg_values[i].clone());
            } else if !param.optional {
                context.pop_scope()?;
                context.clear_self();
                return Err(XdlError::RuntimeError(format!(
                    "Method '{}' requires parameter '{}'",
                    method_name, param.name
                )));
            }
        }

        // Execute method body
        let mut result = XdlValue::Undefined;

        for stmt in &method.body {
            match self.evaluate_statement_in_context(stmt, context) {
                Ok(()) => continue,
                Err(XdlError::Return(val)) => {
                    result = val;
                    break;
                }
                Err(e) => {
                    context.pop_scope()?;
                    context.clear_self();
                    return Err(e);
                }
            }
        }

        // Pop method scope and clear SELF
        context.pop_scope()?;
        context.clear_self();

        Ok(result)
    }

    /// Helper to evaluate a statement (for use in method bodies)
    fn evaluate_statement_in_context(
        &self,
        _stmt: &xdl_parser::Statement,
        _context: &mut Context,
    ) -> XdlResult<()> {
        // This would need access to the interpreter's execute_statement method
        // For now, return an error indicating this needs to be implemented differently
        Err(XdlError::NotImplemented(
            "Statement execution in method context requires interpreter access".to_string(),
        ))
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

        context.set_variable("x".to_string(), XdlValue::Double(3.5));

        let expr = Expression::Variable {
            name: "x".to_string(),
            location: xdl_parser::Location::unknown(),
        };

        let result = evaluator.evaluate(&expr, &mut context).unwrap();
        assert_eq!(result, XdlValue::Double(3.5));
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
