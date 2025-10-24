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
        use std::sync::atomic::{AtomicUsize, Ordering};
        static STMT_EXEC_COUNT: AtomicUsize = AtomicUsize::new(0);

        for statement in &program.statements {
            let count = STMT_EXEC_COUNT.fetch_add(1, Ordering::SeqCst) + 1;

            // Log every 100th statement to track progress
            if count % 100 == 0 {
                eprintln!("[INTERPRETER] Executed {} statements so far...", count);
            }

            // Emergency stop if we exceed reasonable limit
            if count > 100000 {
                return Err(XdlError::RuntimeError(format!(
                    "Program exceeded maximum statement executions ({}). Possible infinite loop.",
                    count
                )));
            }

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
                        if let Some(_proc_def) = self.context.get_procedure(name) {
                            Err(XdlError::NotImplemented(
                                "User-defined procedures".to_string(),
                            ))
                        } else {
                            // Try calling standard library procedure
                            let mut arg_values = Vec::new();
                            for arg in args {
                                arg_values.push(self.evaluate_expression(arg)?);
                            }

                            // Evaluate keyword arguments
                            let mut keyword_values = HashMap::new();
                            for keyword in keywords {
                                if let Some(ref value_expr) = keyword.value {
                                    let value = self.evaluate_expression(value_expr)?;
                                    keyword_values.insert(keyword.name.clone(), value);
                                }
                            }

                            match self.evaluator.call_procedure_with_keywords(
                                name,
                                &arg_values,
                                &keyword_values,
                            ) {
                                Ok(_) => Ok(()),
                                Err(_) => Err(XdlError::RuntimeError(format!(
                                    "Unknown procedure: {}",
                                    name
                                ))),
                            }
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
        let mut iteration_count = 0;
        const MAX_ITERATIONS: usize = 1000000; // Safety limit

        while (step_i > 0 && current <= end_i) || (step_i < 0 && current >= end_i) {
            iteration_count += 1;
            if iteration_count > MAX_ITERATIONS {
                return Err(XdlError::RuntimeError(format!(
                    "For loop exceeded maximum iterations ({}). Loop variable '{}': {} to {} step {}",
                    MAX_ITERATIONS, variable, start_i, end_i, step_i
                )));
            }
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

        // Handle multi-dimensional arrays with comma-separated indices
        if indices.len() > 1 {
            // Check if this is a MultiDimArray with proper shape
            if let XdlValue::MultiDimArray { data, shape } = array_val {
                if indices.len() == 2 && shape.len() == 2 {
                    // 2D assignment: arr[i, j] = value
                    let i_index = match &indices[0] {
                        ArrayIndex::Single(expr) => {
                            let val = self.evaluate_expression(expr)?;
                            val.to_long()? as usize
                        }
                        _ => {
                            return Err(XdlError::NotImplemented(
                                "Range assignment not supported for multi-dimensional arrays"
                                    .to_string(),
                            ));
                        }
                    };

                    let j_index = match &indices[1] {
                        ArrayIndex::Single(expr) => {
                            let val = self.evaluate_expression(expr)?;
                            val.to_long()? as usize
                        }
                        _ => {
                            return Err(XdlError::NotImplemented(
                                "Range assignment not supported for multi-dimensional arrays"
                                    .to_string(),
                            ));
                        }
                    };

                    let nrows = shape[0];
                    let ncols = shape[1];

                    if i_index >= nrows || j_index >= ncols {
                        return Err(XdlError::RuntimeError(format!(
                            "Index [{}, {}] out of bounds for {}x{} array",
                            i_index, j_index, nrows, ncols
                        )));
                    }

                    let flat_index = i_index * ncols + j_index;
                    data[flat_index] = value.to_double()?;
                    return Ok(());
                }
            }

            // For nested arrays, navigate through them
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
                _ => Err(XdlError::RuntimeError(
                    "Multi-dimensional indexing requires nested array".to_string(),
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

    /// Get a reference to the context (for testing/debugging)
    pub fn context(&self) -> &Context {
        &self.context
    }
}

impl Default for Interpreter {
    fn default() -> Self {
        Self::new()
    }
}
