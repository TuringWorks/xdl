//! # XDL Interpreter
//!
//! Interpreter engine for executing XDL/IDL Abstract Syntax Trees.

pub mod context;
pub mod evaluator;
pub mod runtime;

use std::cell::RefCell;
use std::collections::HashMap;
use std::io::Write;
use std::rc::Rc;
use xdl_core::{XdlError, XdlResult, XdlValue};
use xdl_parser::{Expression, Program, Statement};

use crate::context::Context;
use crate::evaluator::Evaluator;

/// XDL Interpreter with execution context and evaluator
pub struct Interpreter {
    context: Context,
    evaluator: Evaluator,
    output: Rc<RefCell<dyn Write>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            context: Context::new(),
            evaluator: Evaluator::new(),
            output: Rc::new(RefCell::new(std::io::stdout())),
        }
    }

    /// Create a new interpreter with custom output writer
    pub fn with_output(output: Rc<RefCell<dyn Write>>) -> Self {
        Self {
            context: Context::new(),
            evaluator: Evaluator::new(),
            output,
        }
    }

    pub fn execute_program(&mut self, program: &Program) -> XdlResult<()> {
        for statement in &program.statements {
            self.execute_statement(statement)?;
        }
        Ok(())
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> XdlResult<XdlValue> {
        self.evaluator.evaluate(expr, &mut self.context)
    }

    /// Get all variables from the interpreter's context
    pub fn get_variables(&self) -> HashMap<String, &XdlValue> {
        self.context.get_all_variables()
    }

    pub fn execute_statement(&mut self, stmt: &Statement) -> XdlResult<()> {
        match stmt {
            Statement::Assignment { target, value, .. } => {
                let val = self.evaluate_expression(value)?;

                match target {
                    Expression::Variable { name, .. } => {
                        self.context.set_variable(name.clone(), val);
                        Ok(())
                    }
                    Expression::SystemVariable { name, .. } => {
                        // Handle system variable assignment: !pi = 3.14
                        self.context.set_system_variable(name.clone(), val);
                        Ok(())
                    }
                    Expression::ArrayRef { array, indices, .. } => {
                        // Handle array element assignment: arr[i] = value
                        self.execute_array_assignment(array, indices, val)
                    }
                    _ => Err(XdlError::NotImplemented(
                        "Complex assignment targets".to_string(),
                    )),
                }
            }

            Statement::Expression { expr, .. } => {
                // Evaluate expression and optionally print result
                let result = self.evaluate_expression(expr)?;

                // In REPL mode, we typically want to show the result
                // This is a simple implementation - in a real XDL, this would depend on context
                match result {
                    XdlValue::Undefined => {} // Don't print undefined
                    _ => {
                        if let Ok(mut out) = self.output.try_borrow_mut() {
                            let _ = writeln!(out, "{}", self.format_value(&result));
                        }
                    }
                }
                Ok(())
            }

            Statement::ProcedureCall {
                name,
                args,
                keywords,
                ..
            } => {
                // Handle built-in procedures like PRINT
                match name.to_uppercase().as_str() {
                    "PRINT" => {
                        // Evaluate all arguments first to avoid borrow conflicts
                        let mut values = Vec::new();
                        for arg in args {
                            let value = self.evaluate_expression(arg)?;
                            values.push(self.format_value(&value));
                        }

                        // Now write to output
                        if let Ok(mut out) = self.output.try_borrow_mut() {
                            for (i, formatted) in values.iter().enumerate() {
                                if i > 0 {
                                    let _ = write!(out, " ");
                                }
                                let _ = write!(out, "{}", formatted);
                            }
                            let _ = writeln!(out);
                        }
                        Ok(())
                    }
                    "EXIT" => {
                        std::process::exit(0);
                    }
                    _ => {
                        // Check for user-defined procedures
                        if let Some(proc_def) = self.context.get_procedure(name).cloned() {
                            self.call_user_procedure(name, args, keywords, &proc_def)?;
                            Ok(())
                        } else {
                            // Try calling standard library procedure
                            let mut arg_values = Vec::new();
                            for arg in args {
                                arg_values.push(self.evaluate_expression(arg)?);
                            }

                            // Evaluate keyword arguments
                            let mut keyword_map = HashMap::new();
                            for keyword in keywords {
                                if let Some(value_expr) = &keyword.value {
                                    let value = self.evaluate_expression(value_expr)?;
                                    keyword_map.insert(keyword.name.to_uppercase(), value);
                                }
                            }

                            self.evaluator.call_procedure_with_keywords(
                                name,
                                &arg_values,
                                &keyword_map,
                            )?;
                            Ok(())
                        }
                    }
                }
            }

            Statement::FunctionDef {
                name,
                params,
                keywords,
                body,
                ..
            } => {
                use crate::context::FunctionDef;
                let func_def = FunctionDef {
                    params: params.clone(),
                    keywords: keywords.clone(),
                    body: body.clone(),
                };
                self.context.define_function(name.clone(), func_def);
                Ok(())
            }

            Statement::ProcedureDef {
                name,
                params,
                keywords,
                body,
                ..
            } => {
                use crate::context::ProcedureDef;
                let proc_def = ProcedureDef {
                    params: params.clone(),
                    keywords: keywords.clone(),
                    body: body.clone(),
                };
                self.context.define_procedure(name.clone(), proc_def);
                Ok(())
            }

            Statement::For {
                variable,
                start,
                end,
                step,
                body,
                ..
            } => self.execute_for_loop(variable, start, end, step, body),

            Statement::If {
                condition,
                then_block,
                else_block,
                ..
            } => {
                let cond_val = self.evaluate_expression(condition)?;

                if !cond_val.is_zero() {
                    for stmt in then_block {
                        self.execute_statement(stmt)?;
                    }
                } else if let Some(else_stmts) = else_block {
                    for stmt in else_stmts {
                        self.execute_statement(stmt)?;
                    }
                }
                Ok(())
            }

            Statement::While {
                condition, body, ..
            } => self.execute_while_loop(condition, body),

            Statement::Repeat {
                body, condition, ..
            } => self.execute_repeat_loop(body, condition),

            Statement::Foreach {
                variable,
                iterable,
                index_var,
                body,
                ..
            } => self.execute_foreach_loop(variable, iterable, index_var.as_deref(), body),

            Statement::Break { .. } => Err(XdlError::Break),

            Statement::Continue { .. } => Err(XdlError::Continue),

            Statement::Return { value, .. } => {
                let return_val = if let Some(expr) = value {
                    self.evaluate_expression(expr)?
                } else {
                    XdlValue::Undefined
                };
                Err(XdlError::Return(return_val))
            }

            Statement::Common { .. } | Statement::CompileOpt { .. } | Statement::Label { .. } => {
                // These statements are mostly compile-time directives, ignore for now
                Ok(())
            }

            Statement::Goto { .. } => Err(XdlError::NotImplemented("GOTO statements".to_string())),

            Statement::Case {
                expr,
                branches,
                else_block,
                ..
            } => self.execute_case_statement(expr, branches, else_block, false),

            Statement::Switch {
                expr,
                branches,
                else_block,
                ..
            } => self.execute_case_statement(expr, branches, else_block, true),

            // Object-oriented programming
            Statement::ClassDefinition { name, body, .. } => {
                self.execute_class_definition(name, body)
            }

            Statement::MethodDefinition {
                class_name,
                method_name,
                is_function,
                params,
                keywords,
                body,
                ..
            } => self.execute_method_definition(
                class_name,
                method_name,
                *is_function,
                params,
                keywords,
                body,
            ),

            Statement::ObjectDestroy { objects, .. } => self.execute_obj_destroy(objects),
        }
    }

    /// Format a XdlValue for display
    fn format_value(&self, value: &XdlValue) -> String {
        value.to_string_repr()
    }

    /// Execute a for loop
    fn execute_for_loop(
        &mut self,
        variable: &str,
        start: &Expression,
        end: &Expression,
        step: &Option<Expression>,
        body: &[Statement],
    ) -> XdlResult<()> {
        let start_val = self.evaluate_expression(start)?;
        let end_val = self.evaluate_expression(end)?;
        let step_val = if let Some(step_expr) = step {
            self.evaluate_expression(step_expr)?
        } else {
            XdlValue::Long(1)
        };

        let start_i = start_val.to_double()? as i64;
        let end_i = end_val.to_double()? as i64;
        let step_i = step_val.to_double()? as i64;

        if step_i == 0 {
            return Err(XdlError::RuntimeError("Zero step in for loop".to_string()));
        }

        let mut current = start_i;

        while (step_i > 0 && current <= end_i) || (step_i < 0 && current >= end_i) {
            // Set loop variable
            self.context
                .set_variable(variable.to_string(), XdlValue::Long(current as i32));

            // Execute loop body
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(()) => continue,
                    Err(XdlError::Break) => return Ok(()),
                    Err(XdlError::Continue) => break,
                    Err(e) => return Err(e),
                }
            }

            current += step_i;
        }

        Ok(())
    }

    /// Execute a while loop
    fn execute_while_loop(&mut self, condition: &Expression, body: &[Statement]) -> XdlResult<()> {
        loop {
            let cond_val = self.evaluate_expression(condition)?;
            if cond_val.is_zero() {
                break;
            }

            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(()) => continue,
                    Err(XdlError::Break) => return Ok(()),
                    Err(XdlError::Continue) => break,
                    Err(e) => return Err(e),
                }
            }
        }
        Ok(())
    }

    /// Execute a repeat loop
    fn execute_repeat_loop(&mut self, body: &[Statement], condition: &Expression) -> XdlResult<()> {
        loop {
            // Execute body first (do-while semantics)
            for stmt in body {
                match self.execute_statement(stmt) {
                    Ok(()) => continue,
                    Err(XdlError::Break) => return Ok(()),
                    Err(XdlError::Continue) => {
                        // Check condition before continuing
                        let cond_val = self.evaluate_expression(condition)?;
                        if !cond_val.is_zero() {
                            return Ok(());
                        }
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }

            // Check exit condition
            let cond_val = self.evaluate_expression(condition)?;
            if !cond_val.is_zero() {
                break;
            }
        }
        Ok(())
    }

    /// Execute a CASE or SWITCH statement
    /// CASE: Executes the first matching branch and exits (no fall-through)
    /// SWITCH: Same behavior in XDL (unlike C, XDL SWITCH doesn't fall-through)
    fn execute_case_statement(
        &mut self,
        expr: &Expression,
        branches: &[xdl_parser::CaseBranch],
        else_block: &Option<Vec<Statement>>,
        _is_switch: bool, // For future: SWITCH might support fall-through
    ) -> XdlResult<()> {
        // Evaluate the switch expression
        let switch_val = self.evaluate_expression(expr)?;

        // Try each branch
        let mut matched = false;
        for branch in branches {
            // Check if any value in the branch matches
            for case_expr in &branch.values {
                let case_val = self.evaluate_expression(case_expr)?;

                // Compare values
                if self.values_equal(&switch_val, &case_val) {
                    // Execute branch body
                    for stmt in &branch.body {
                        match self.execute_statement(stmt) {
                            Ok(()) => continue,
                            Err(XdlError::Break) => return Ok(()), // BREAK exits the CASE
                            Err(e) => return Err(e),
                        }
                    }
                    matched = true;
                    break;
                }
            }

            if matched {
                break;
            }
        }

        // Execute else block if no match
        if !matched {
            if let Some(else_stmts) = else_block {
                for stmt in else_stmts {
                    self.execute_statement(stmt)?;
                }
            }
        }

        Ok(())
    }

    /// Compare two XdlValues for equality
    fn values_equal(&self, a: &XdlValue, b: &XdlValue) -> bool {
        match (a, b) {
            // Numeric comparisons - convert to double for comparison
            (XdlValue::Byte(x), XdlValue::Byte(y)) => x == y,
            (XdlValue::Int(x), XdlValue::Int(y)) => x == y,
            (XdlValue::Long(x), XdlValue::Long(y)) => x == y,
            (XdlValue::Long64(x), XdlValue::Long64(y)) => x == y,
            (XdlValue::UInt(x), XdlValue::UInt(y)) => x == y,
            (XdlValue::ULong(x), XdlValue::ULong(y)) => x == y,
            (XdlValue::ULong64(x), XdlValue::ULong64(y)) => x == y,
            (XdlValue::Float(x), XdlValue::Float(y)) => (x - y).abs() < f32::EPSILON,
            (XdlValue::Double(x), XdlValue::Double(y)) => (x - y).abs() < f64::EPSILON,
            (XdlValue::String(x), XdlValue::String(y)) => x == y,
            // Cross-type numeric comparisons
            _ => {
                if let (Ok(x), Ok(y)) = (a.to_double(), b.to_double()) {
                    (x - y).abs() < f64::EPSILON
                } else if let (XdlValue::String(x), _) = (a, b) {
                    // String to string comparison
                    if let XdlValue::String(y) = b {
                        x == y
                    } else {
                        false
                    }
                } else {
                    false
                }
            }
        }
    }

    /// Execute a foreach loop
    fn execute_foreach_loop(
        &mut self,
        variable: &str,
        iterable: &Expression,
        index_var: Option<&str>,
        body: &[Statement],
    ) -> XdlResult<()> {
        let iterable_val = self.evaluate_expression(iterable)?;

        match iterable_val {
            XdlValue::Array(arr) => {
                for (index, element) in arr.iter().enumerate() {
                    // Set loop variable to current element
                    self.context
                        .set_variable(variable.to_string(), XdlValue::Double(*element));

                    // Set index variable if provided
                    if let Some(idx_var) = index_var {
                        self.context
                            .set_variable(idx_var.to_string(), XdlValue::Long(index as i32));
                    }

                    // Execute body
                    for stmt in body {
                        match self.execute_statement(stmt) {
                            Ok(()) => continue,
                            Err(XdlError::Break) => return Ok(()),
                            Err(XdlError::Continue) => break,
                            Err(e) => return Err(e),
                        }
                    }
                }
                Ok(())
            }
            _ => Err(XdlError::RuntimeError(
                "FOREACH requires an array".to_string(),
            )),
        }
    }

    /// Execute array element assignment: arr[i] = value
    fn execute_array_assignment(
        &mut self,
        array_expr: &Expression,
        indices: &[xdl_parser::ArrayIndex],
        value: XdlValue,
    ) -> XdlResult<()> {
        // For now, only support single variable array assignments like: arr[i] = value
        // Multi-dimensional like matrix[i][j] = value would require recursive handling

        if let Expression::Variable { name, .. } = array_expr {
            // Get the current array
            let mut array_val = self.context.get_variable(name)?.clone();

            // Modify the array at the specified index
            self.modify_array_element(&mut array_val, indices, value)?;

            // Store the modified array back
            self.context.set_variable(name.clone(), array_val);
            Ok(())
        } else {
            Err(XdlError::NotImplemented(
                "Nested array element assignment".to_string(),
            ))
        }
    }

    /// Modify an array element at the given indices
    fn modify_array_element(
        &mut self,
        array_val: &mut XdlValue,
        indices: &[xdl_parser::ArrayIndex],
        value: XdlValue,
    ) -> XdlResult<()> {
        use xdl_parser::ArrayIndex;

        if indices.is_empty() {
            return Err(XdlError::RuntimeError(
                "No index provided for array assignment".to_string(),
            ));
        }

        // Handle nested arrays (multi-dimensional)
        if indices.len() > 1 {
            // For multi-dimensional access, we need to navigate through nested arrays
            match array_val {
                XdlValue::NestedArray(rows) => {
                    // Get the first index
                    match &indices[0] {
                        ArrayIndex::Single(expr) => {
                            let index_val = self.evaluate_expression(expr)?;
                            let raw_index = index_val.to_long()?;

                            // Handle negative indices
                            let index = if raw_index < 0 {
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

                            if index >= rows.len() {
                                return Err(XdlError::RuntimeError(format!(
                                    "Index {} out of bounds for array of length {}",
                                    raw_index,
                                    rows.len()
                                )));
                            }

                            // Recursively handle remaining indices
                            self.modify_array_element(&mut rows[index], &indices[1..], value)?;
                            Ok(())
                        }
                        _ => Err(XdlError::NotImplemented(
                            "Range indexing in multi-dimensional assignment".to_string(),
                        )),
                    }
                }
                XdlValue::MultiDimArray { data, shape } => {
                    // Handle multi-dimensional array indexing
                    // Compute linear index from multi-dimensional indices
                    match &indices[0] {
                        ArrayIndex::Single(expr) => {
                            let index_val = self.evaluate_expression(expr)?;
                            let idx0 = index_val.to_long()? as usize;

                            if indices.len() == 1 {
                                // Single index on multi-dim array (treats as flat)
                                if idx0 >= data.len() {
                                    return Err(XdlError::RuntimeError(format!(
                                        "Index {} out of bounds for array of size {}",
                                        idx0,
                                        data.len()
                                    )));
                                }
                                data[idx0] = value.to_double()?;
                                Ok(())
                            } else {
                                // Multi-dimensional indexing - compute linear index
                                let mut linear_idx = idx0;
                                let mut stride = 1;

                                // Calculate strides (column-major order like IDL/GDL)
                                for i in 1..indices.len() {
                                    match &indices[i] {
                                        ArrayIndex::Single(expr) => {
                                            let index_val = self.evaluate_expression(expr)?;
                                            let idx = index_val.to_long()? as usize;

                                            if i >= shape.len() {
                                                return Err(XdlError::RuntimeError(format!(
                                                    "Too many indices: array has {} dimensions",
                                                    shape.len()
                                                )));
                                            }

                                            stride *= shape[i - 1];
                                            linear_idx += idx * stride;
                                        }
                                        _ => {
                                            return Err(XdlError::NotImplemented(
                                                "Range indexing in multi-dimensional arrays"
                                                    .to_string(),
                                            ))
                                        }
                                    }
                                }

                                if linear_idx >= data.len() {
                                    return Err(XdlError::RuntimeError(format!(
                                        "Index out of bounds: computed linear index {} >= {}",
                                        linear_idx,
                                        data.len()
                                    )));
                                }

                                data[linear_idx] = value.to_double()?;
                                Ok(())
                            }
                        }
                        _ => Err(XdlError::NotImplemented(
                            "Range indexing in multi-dimensional assignment".to_string(),
                        )),
                    }
                }
                _ => Err(XdlError::RuntimeError(
                    "Multi-dimensional indexing requires nested array or multi-dimensional array"
                        .to_string(),
                )),
            }
        } else {
            // Single index assignment
            match array_val {
                XdlValue::Array(arr) => {
                    match &indices[0] {
                        ArrayIndex::Single(expr) => {
                            // Get the index
                            let index_val = self.evaluate_expression(expr)?;
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

                            // Set the value
                            arr[index] = value.to_double()?;
                            Ok(())
                        }
                        ArrayIndex::Range { .. } => Err(XdlError::NotImplemented(
                            "Range assignment not supported".to_string(),
                        )),
                        ArrayIndex::All => Err(XdlError::NotImplemented(
                            "All-element assignment not supported".to_string(),
                        )),
                    }
                }
                XdlValue::NestedArray(rows) => match &indices[0] {
                    ArrayIndex::Single(expr) => {
                        let index_val = self.evaluate_expression(expr)?;
                        let raw_index = index_val.to_long()?;

                        let index = if raw_index < 0 {
                            let len = rows.len() as i32;
                            (len + raw_index).max(0) as usize
                        } else {
                            raw_index as usize
                        };

                        if index >= rows.len() {
                            return Err(XdlError::RuntimeError(format!(
                                "Index out of bounds: {}",
                                raw_index
                            )));
                        }

                        rows[index] = value;
                        Ok(())
                    }
                    _ => Err(XdlError::NotImplemented(
                        "Range/all assignment on nested arrays".to_string(),
                    )),
                },
                _ => Err(XdlError::RuntimeError(
                    "Cannot index non-array value".to_string(),
                )),
            }
        }
    }

    /// Call a user-defined procedure
    fn call_user_procedure(
        &mut self,
        name: &str,
        args: &[Expression],
        keywords: &[xdl_parser::Keyword],
        proc_def: &context::ProcedureDef,
    ) -> XdlResult<()> {
        // Evaluate all arguments
        let mut arg_values = Vec::new();
        for arg in args {
            arg_values.push(self.evaluate_expression(arg)?);
        }

        // Evaluate keyword arguments
        let mut keyword_map = HashMap::new();
        for keyword in keywords {
            if let Some(value_expr) = &keyword.value {
                let value = self.evaluate_expression(value_expr)?;
                keyword_map.insert(keyword.name.to_uppercase(), value);
            }
        }

        // Push a new scope for the procedure execution
        self.context.push_scope();

        // Bind positional parameters to arguments
        for (i, param) in proc_def.params.iter().enumerate() {
            if i < arg_values.len() {
                self.context
                    .set_variable(param.name.clone(), arg_values[i].clone());
            } else if !param.optional {
                self.context.pop_scope()?;
                return Err(XdlError::RuntimeError(format!(
                    "Missing required parameter '{}' for procedure '{}'",
                    param.name, name
                )));
            }
        }

        // Bind keyword parameters
        for keyword_decl in &proc_def.keywords {
            let key = keyword_decl.name.clone();
            // Check both the original case and uppercase in the keyword_map
            let value_opt = keyword_map
                .get(&key)
                .or_else(|| keyword_map.get(&key.to_uppercase()));
            if let Some(value) = value_opt {
                self.context.set_variable(key, value.clone());
            }
            // If no value provided, the keyword is undefined (IDL behavior)
        }

        // Execute procedure body
        let mut result = Ok(());
        for stmt in &proc_def.body {
            match self.execute_statement(stmt) {
                Ok(()) => continue,
                Err(XdlError::Return(_)) => {
                    // Procedures can have RETURN statements (without values)
                    break;
                }
                Err(e) => {
                    result = Err(e);
                    break;
                }
            }
        }

        // Pop the procedure scope
        self.context.pop_scope()?;

        result
    }

    /// Get a reference to the context (for testing/debugging)
    pub fn context(&self) -> &Context {
        &self.context
    }

    /// Execute a class definition (PRO ClassName__define)
    fn execute_class_definition(&mut self, name: &str, body: &[Statement]) -> XdlResult<()> {
        use crate::context::ClassDef;

        // Create new class definition
        let mut class_def = ClassDef::new(name.to_string());

        // Execute the body to extract structure definitions
        // In IDL, the class __define procedure typically contains a structure definition
        // that specifies the class fields
        for stmt in body {
            // Look for structure definitions or other initialization
            // For now, we'll support basic field initialization via assignments
            if let Statement::Assignment {
                target:
                    Expression::Variable {
                        name: field_name, ..
                    },
                value,
                ..
            } = stmt
            {
                let field_value = self.evaluate_expression(value)?;
                class_def
                    .fields
                    .insert(field_name.to_uppercase(), field_value);
            }
            // Also execute any other statements in the body
            self.execute_statement(stmt)?;
        }

        // Store the class definition
        self.context.define_class(name.to_string(), class_def);
        Ok(())
    }

    /// Execute a method definition (PRO/FUNCTION ClassName::MethodName)
    fn execute_method_definition(
        &mut self,
        class_name: &str,
        method_name: &str,
        is_function: bool,
        params: &[xdl_parser::Parameter],
        keywords: &[xdl_parser::KeywordDecl],
        body: &[Statement],
    ) -> XdlResult<()> {
        use crate::context::MethodDef;

        // Create method definition
        let method = MethodDef {
            is_function,
            params: params.to_vec(),
            keywords: keywords.to_vec(),
            body: body.to_vec(),
        };

        // Get or create the class
        let class = if let Ok(cls) = self.context.get_class_mut(class_name) {
            cls
        } else {
            // If class doesn't exist yet, create it
            // This can happen if methods are defined before __define
            use crate::context::ClassDef;
            let class_def = ClassDef::new(class_name.to_string());
            self.context.define_class(class_name.to_string(), class_def);
            self.context.get_class_mut(class_name)?
        };

        // Add method to the class
        class.add_method(method_name.to_string(), method);
        Ok(())
    }

    /// Execute OBJ_DESTROY statement
    fn execute_obj_destroy(&mut self, objects: &[Expression]) -> XdlResult<()> {
        for obj_expr in objects {
            let obj_val = self.evaluate_expression(obj_expr)?;

            // Extract object ID
            let obj_id = match obj_val {
                XdlValue::Object(id) => id,
                _ => {
                    return Err(XdlError::TypeMismatch {
                        expected: "object".to_string(),
                        actual: format!("{:?}", obj_val.gdl_type()),
                    })
                }
            };

            // Skip NULL objects
            if obj_id == 0 {
                continue;
            }

            // Get object to find class name for Cleanup method
            let class_name = {
                let obj = self.context.get_object(obj_id)?;
                obj.class_name.clone()
            };

            // Try to call Cleanup method if it exists
            if let Ok(class) = self.context.get_class(&class_name) {
                if class.get_method("CLEANUP").is_some() {
                    // Call the Cleanup method
                    // For now, we'll execute it without parameters
                    // TODO: Implement full method dispatch with SELF support
                    let _ = self.execute_user_method(obj_id, "CLEANUP", &[], &[]);
                }
            }

            // Remove the object from storage
            self.context.remove_object(obj_id)?;
        }

        Ok(())
    }

    /// Execute a user-defined method on an object (helper for method dispatch)
    fn execute_user_method(
        &mut self,
        _obj_id: usize,
        _method_name: &str,
        _args: &[Expression],
        _keywords: &[xdl_parser::Keyword],
    ) -> XdlResult<XdlValue> {
        // TODO: Implement full method dispatch with SELF support
        // For now, return a placeholder
        Err(XdlError::NotImplemented(
            "User-defined method dispatch not yet implemented".to_string(),
        ))
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
