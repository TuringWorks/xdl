//! # XDL Interpreter
//!
//! Interpreter engine for executing XDL/IDL Abstract Syntax Trees.

pub mod context;
pub mod evaluator;
pub mod methods;
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

/// An index whose expression has been evaluated.
enum ResolvedIndex {
    Single(i64),
    Range {
        start: Option<i64>,
        end: Option<i64>,
        step: Option<i64>,
    },
    All,
}

/// Resolve a possibly-negative index (counted from the end) against `len`.
/// Out of range is an error, never a clamped element.
fn resolve_position(raw: i64, len: usize) -> XdlResult<usize> {
    let position = if raw < 0 { len as i64 + raw } else { raw };
    usize::try_from(position)
        .ok()
        .filter(|&p| p < len)
        .ok_or_else(|| {
            XdlError::RuntimeError(format!(
                "Index {} out of bounds for array of length {}",
                raw, len
            ))
        })
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
        // Pre-scan statements to build label map for GOTO support
        let label_map = self.build_label_map(&program.statements);

        // Execute statements with index-based control for GOTO
        let mut index = 0;
        while index < program.statements.len() {
            let statement = &program.statements[index];
            match self.execute_statement(statement) {
                Ok(()) => {
                    index += 1;
                }
                Err(XdlError::Goto(label)) => {
                    // Jump to the label
                    if let Some(&target_index) = label_map.get(&label.to_uppercase()) {
                        index = target_index;
                    } else {
                        return Err(XdlError::RuntimeError(format!(
                            "Label not found: {}",
                            label
                        )));
                    }
                }
                Err(e) => return Err(e),
            }
        }
        Ok(())
    }

    /// Build a map of label names to statement indices for GOTO support
    fn build_label_map(&self, statements: &[Statement]) -> HashMap<String, usize> {
        let mut label_map = HashMap::new();
        for (index, stmt) in statements.iter().enumerate() {
            if let Statement::Label { name, .. } = stmt {
                label_map.insert(name.to_uppercase(), index);
            }
        }
        label_map
    }

    pub fn evaluate_expression(&mut self, expr: &Expression) -> XdlResult<XdlValue> {
        // Resolve all user-defined function calls in the expression tree
        let resolved = self.resolve_user_functions(expr)?;
        self.evaluator.evaluate(&resolved, &mut self.context)
    }

    /// Recursively resolve user-defined function calls in an expression
    fn resolve_user_functions(&mut self, expr: &Expression) -> XdlResult<Expression> {
        match expr {
            Expression::FunctionCall {
                name,
                args,
                keywords,
                location,
            } => {
                // Check if this is a user-defined function
                if let Some(func_def) = self.context.get_function(name).cloned() {
                    // First resolve any nested user function calls in arguments
                    let resolved_args: Vec<Expression> = args
                        .iter()
                        .map(|arg| self.resolve_user_functions(arg))
                        .collect::<XdlResult<Vec<_>>>()?;

                    // Call the user function and get the result
                    let result =
                        self.call_user_function(name, &resolved_args, keywords, &func_def)?;

                    // Return the result as a literal expression
                    Ok(Expression::Literal {
                        value: result,
                        location: location.clone(),
                    })
                } else {
                    // Not a user-defined function - resolve arguments and return as-is
                    let resolved_args: Vec<Expression> = args
                        .iter()
                        .map(|arg| self.resolve_user_functions(arg))
                        .collect::<XdlResult<Vec<_>>>()?;

                    Ok(Expression::FunctionCall {
                        name: name.clone(),
                        args: resolved_args,
                        keywords: keywords.clone(),
                        location: location.clone(),
                    })
                }
            }
            Expression::Binary {
                op,
                left,
                right,
                location,
            } => {
                let resolved_left = self.resolve_user_functions(left)?;
                let resolved_right = self.resolve_user_functions(right)?;
                Ok(Expression::Binary {
                    op: *op,
                    left: Box::new(resolved_left),
                    right: Box::new(resolved_right),
                    location: location.clone(),
                })
            }
            Expression::Unary { op, expr, location } => {
                let resolved = self.resolve_user_functions(expr)?;
                Ok(Expression::Unary {
                    op: *op,
                    expr: Box::new(resolved),
                    location: location.clone(),
                })
            }
            Expression::Ternary {
                condition,
                if_true,
                if_false,
                location,
            } => {
                let resolved_cond = self.resolve_user_functions(condition)?;
                let resolved_true = self.resolve_user_functions(if_true)?;
                let resolved_false = self.resolve_user_functions(if_false)?;
                Ok(Expression::Ternary {
                    condition: Box::new(resolved_cond),
                    if_true: Box::new(resolved_true),
                    if_false: Box::new(resolved_false),
                    location: location.clone(),
                })
            }
            Expression::ArrayRef {
                array,
                indices,
                location,
            } => {
                let resolved_array = self.resolve_user_functions(array)?;
                // Note: indices might contain expressions too, but for simplicity we skip those
                Ok(Expression::ArrayRef {
                    array: Box::new(resolved_array),
                    indices: indices.clone(),
                    location: location.clone(),
                })
            }
            Expression::ArrayDef { elements, location } => {
                let resolved_elements: Vec<Expression> = elements
                    .iter()
                    .map(|e| self.resolve_user_functions(e))
                    .collect::<XdlResult<Vec<_>>>()?;
                Ok(Expression::ArrayDef {
                    elements: resolved_elements,
                    location: location.clone(),
                })
            }
            Expression::MethodCall {
                object,
                method,
                args,
                keywords,
                location,
            } => {
                let resolved_object = self.resolve_user_functions(object)?;
                let resolved_args: Vec<Expression> = args
                    .iter()
                    .map(|arg| self.resolve_user_functions(arg))
                    .collect::<XdlResult<Vec<_>>>()?;
                Ok(Expression::MethodCall {
                    object: Box::new(resolved_object),
                    method: method.clone(),
                    args: resolved_args,
                    keywords: keywords.clone(),
                    location: location.clone(),
                })
            }
            // For other expression types, return as-is
            _ => Ok(expr.clone()),
        }
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

            Statement::Common { .. } | Statement::CompileOpt { .. } => {
                // These statements are mostly compile-time directives, ignore for now
                Ok(())
            }

            Statement::Label { .. } => {
                // Labels are markers for GOTO - no execution action needed
                Ok(())
            }

            Statement::Goto { label, .. } => {
                // Signal a jump to the specified label
                Err(XdlError::Goto(label.clone()))
            }

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
        let Expression::Variable { name, .. } = array_expr else {
            return Err(XdlError::NotImplemented(
                "Nested array element assignment".to_string(),
            ));
        };

        // Evaluate every index expression before borrowing the variable, so the
        // element write mutates the stored array in place. Cloning the whole array
        // per element write made an N-element fill cost O(N^2).
        self.context.get_variable(name)?;
        let resolved = self.resolve_indices(indices)?;
        let array_val = self.context.get_variable_mut(name)?;
        Self::write_element(array_val, &resolved, value)
    }

    /// Evaluate assignment indices before borrowing the destination variable.
    fn resolve_indices(
        &mut self,
        indices: &[xdl_parser::ArrayIndex],
    ) -> XdlResult<Vec<ResolvedIndex>> {
        use xdl_parser::ArrayIndex;
        indices
            .iter()
            .map(|index| match index {
                ArrayIndex::Single(expr) => {
                    let value = self.evaluate_expression(expr)?;
                    Ok(ResolvedIndex::Single(value.to_long()? as i64))
                }
                ArrayIndex::Range { start, end, step } => {
                    let evaluate_bound = |this: &mut Self, expression: &Option<Box<Expression>>| {
                        expression
                            .as_ref()
                            .map(|value| {
                                this.evaluate_expression(value)
                                    .map(|value| value.to_long().map(i64::from))?
                            })
                            .transpose()
                    };
                    Ok(ResolvedIndex::Range {
                        start: evaluate_bound(self, start)?,
                        end: evaluate_bound(self, end)?,
                        step: evaluate_bound(self, step)?,
                    })
                }
                ArrayIndex::All => Ok(ResolvedIndex::All),
            })
            .collect()
    }

    /// Write one element into an array value. Pure: no evaluation, no scope access.
    fn write_element(
        array_val: &mut XdlValue,
        indices: &[ResolvedIndex],
        value: XdlValue,
    ) -> XdlResult<()> {
        let single = |index: &ResolvedIndex| match index {
            ResolvedIndex::Single(raw) => Ok(*raw),
            ResolvedIndex::Range { .. } | ResolvedIndex::All => Err(XdlError::NotImplemented(
                "Range indexing in multi-dimensional assignment".to_string(),
            )),
        };

        match indices {
            [] => Err(XdlError::RuntimeError(
                "No index provided for array assignment".to_string(),
            )),
            [only] => match array_val {
                XdlValue::Array(arr) => match only {
                    ResolvedIndex::Single(raw) => {
                        let position = resolve_position(*raw, arr.len())?;
                        arr[position] = value.to_double()?;
                        Ok(())
                    }
                    ResolvedIndex::Range { .. } | ResolvedIndex::All => {
                        Self::write_f64_range(arr, only, value)
                    }
                },
                XdlValue::MultiDimArray { data, .. } => match only {
                    ResolvedIndex::Single(raw) => {
                        // A single index addresses the flat, column-major element.
                        let position = resolve_position(*raw, data.len())?;
                        data[position] = value.to_double()?;
                        Ok(())
                    }
                    ResolvedIndex::Range { .. } | ResolvedIndex::All => {
                        Self::write_f64_range(data, only, value)
                    }
                },
                XdlValue::NestedArray(rows) => {
                    let position = resolve_position(single(only)?, rows.len())?;
                    rows[position] = value;
                    Ok(())
                }
                _ => Err(XdlError::RuntimeError(
                    "Cannot index non-array value".to_string(),
                )),
            },
            [first, rest @ ..] => match array_val {
                XdlValue::NestedArray(rows) => {
                    if matches!(first, ResolvedIndex::Single(_)) {
                        let position = resolve_position(single(first)?, rows.len())?;
                        Self::write_element(&mut rows[position], rest, value)
                    } else {
                        if let [row_index, column_index] = indices {
                            if matches!(
                                column_index,
                                ResolvedIndex::Range { .. } | ResolvedIndex::All
                            ) {
                                return Self::write_numeric_matrix_block(
                                    rows,
                                    row_index,
                                    column_index,
                                    value,
                                );
                            }
                        }
                        let (start, end, step, selected_len) =
                            Self::range_bounds(first, rows.len())?;
                        let values = Self::numeric_values(value)?;
                        let expands_across_rows = values.len() == 1;
                        if !expands_across_rows && values.len() != selected_len {
                            return Err(XdlError::RuntimeError(format!(
                                "Matrix assignment selects {} rows but source has {} elements",
                                selected_len,
                                values.len()
                            )));
                        }
                        let scalar_value = expands_across_rows.then_some(values[0]);
                        if !expands_across_rows
                            && rest.iter().any(|index| {
                                matches!(index, ResolvedIndex::Range { .. } | ResolvedIndex::All)
                            })
                        {
                            return Err(XdlError::NotImplemented(
                                "Matrix range assignment from nonscalar nested selections is not supported"
                                    .to_string(),
                            ));
                        }
                        if rest.len() != 1 {
                            return Err(XdlError::NotImplemented(
                                "Nested matrix range assignment supports one remaining subscript"
                                    .to_string(),
                            ));
                        }
                        for row in (start..end).step_by(step) {
                            match (&rows[row], &rest[0]) {
                                (XdlValue::Array(values), ResolvedIndex::Single(index)) => {
                                    resolve_position(*index, values.len())?;
                                }
                                (
                                    XdlValue::Array(values),
                                    ResolvedIndex::Range { .. } | ResolvedIndex::All,
                                ) => {
                                    Self::range_bounds(&rest[0], values.len())?;
                                }
                                _ => {
                                    return Err(XdlError::NotImplemented(
                                        "Nested matrix range assignment requires flat numeric rows"
                                            .to_string(),
                                    ));
                                }
                            }
                        }
                        let mut source = values.into_iter();
                        for (offset, row) in (start..end).step_by(step).enumerate() {
                            let value = if let Some(scalar) = scalar_value {
                                scalar
                            } else {
                                // The source length was checked against selected_len.
                                source.next().ok_or_else(|| {
                                    XdlError::RuntimeError(format!(
                                        "Missing source value for selected row {offset}"
                                    ))
                                })?
                            };
                            Self::write_element(&mut rows[row], rest, XdlValue::Double(value))?;
                        }
                        Ok(())
                    }
                }
                XdlValue::MultiDimArray { data, shape } => {
                    if indices.len() > shape.len() {
                        return Err(XdlError::RuntimeError(format!(
                            "Too many indices: array has {} dimensions",
                            shape.len()
                        )));
                    }
                    // Column-major (IDL order). Each index is bounds-checked against
                    // its own dimension; a flat-only check would let x[2,0] on a
                    // 2x2 array silently write x[0,1].
                    let mut linear = 0usize;
                    let mut stride = 1usize;
                    for (dim, index) in indices.iter().enumerate() {
                        let extent = shape[dim];
                        let raw = single(index)?;
                        let position = usize::try_from(raw)
                            .ok()
                            .filter(|&p| p < extent)
                            .ok_or_else(|| {
                                XdlError::RuntimeError(format!(
                                    "Index {} out of bounds for dimension {} of size {}",
                                    raw, dim, extent
                                ))
                            })?;
                        linear += position * stride;
                        stride *= extent;
                    }
                    data[linear] = value.to_double()?;
                    Ok(())
                }
                _ => Err(XdlError::RuntimeError(
                    "Multi-dimensional indexing requires nested array or multi-dimensional array"
                        .to_string(),
                )),
            },
        }
    }

    /// Assign a rectangular block in a nested numeric matrix. A scalar fills
    /// the whole block; a matrix source must have exactly the selected shape.
    /// Flat vectors are intentionally not reshaped because their orientation
    /// would otherwise be ambiguous for a two-dimensional destination.
    fn write_numeric_matrix_block(
        rows: &mut [XdlValue],
        row_index: &ResolvedIndex,
        column_index: &ResolvedIndex,
        value: XdlValue,
    ) -> XdlResult<()> {
        let matrix_width = rows
            .first()
            .and_then(|row| match row {
                XdlValue::Array(values) => Some(values.len()),
                _ => None,
            })
            .filter(|width| *width > 0)
            .ok_or_else(|| {
                XdlError::InvalidArgument(
                    "Matrix block assignment requires a nonempty rectangular numeric matrix"
                        .to_string(),
                )
            })?;
        if rows
            .iter()
            .any(|row| !matches!(row, XdlValue::Array(values) if values.len() == matrix_width))
        {
            return Err(XdlError::InvalidArgument(
                "Matrix block assignment does not support ragged or nonnumeric rows".to_string(),
            ));
        }

        let (row_start, row_end, row_step, selected_rows) =
            Self::matrix_axis_bounds(row_index, rows.len())?;
        let (column_start, column_end, column_step, selected_columns) =
            Self::matrix_axis_bounds(column_index, matrix_width)?;
        let scalar = match value {
            XdlValue::Array(values) if values.len() == 1 => Some(values[0]),
            XdlValue::NestedArray(source_rows) => {
                let source_width = source_rows
                    .first()
                    .and_then(|row| match row {
                        XdlValue::Array(values) => Some(values.len()),
                        _ => None,
                    })
                    .filter(|width| *width > 0)
                    .ok_or_else(|| {
                        XdlError::InvalidArgument(
                            "Matrix block source must be a nonempty rectangular numeric matrix"
                                .to_string(),
                        )
                    })?;
                if source_rows.iter().any(
                    |row| !matches!(row, XdlValue::Array(values) if values.len() == source_width),
                ) {
                    return Err(XdlError::InvalidArgument(
                        "Matrix block source does not support ragged or nonnumeric rows"
                            .to_string(),
                    ));
                }
                if source_rows.len() != selected_rows || source_width != selected_columns {
                    return Err(XdlError::RuntimeError(format!(
                        "Matrix block assignment selects {}x{} elements but source is {}x{}",
                        selected_rows,
                        selected_columns,
                        source_rows.len(),
                        source_width
                    )));
                }
                let source = source_rows
                    .iter()
                    .map(|row| match row {
                        XdlValue::Array(values) => Ok(values.clone()),
                        _ => Err(XdlError::InvalidArgument(
                            "Matrix block source does not support nonnumeric rows".to_string(),
                        )),
                    })
                    .collect::<XdlResult<Vec<_>>>()?;
                for (target_row, source_row) in
                    (row_start..row_end).step_by(row_step).zip(source.iter())
                {
                    let Some(XdlValue::Array(target_values)) = rows.get_mut(target_row) else {
                        return Err(XdlError::InvalidArgument(
                            "Matrix block assignment requires rectangular numeric rows".to_string(),
                        ));
                    };
                    for (target_column, source_value) in (column_start..column_end)
                        .step_by(column_step)
                        .zip(source_row.iter())
                    {
                        let Some(target) = target_values.get_mut(target_column) else {
                            return Err(XdlError::RuntimeError(
                                "Matrix block column index is out of bounds".to_string(),
                            ));
                        };
                        *target = *source_value;
                    }
                }
                return Ok(());
            }
            other => {
                let values = Self::numeric_values(other)?;
                if values.len() != 1 {
                    return Err(XdlError::RuntimeError(format!(
                        "Matrix block assignment requires a scalar or a {}x{} matrix source, got {} values",
                        selected_rows, selected_columns, values.len()
                    )));
                }
                Some(values[0])
            }
        };
        if let Some(scalar) = scalar {
            for target_row in (row_start..row_end).step_by(row_step) {
                let Some(XdlValue::Array(target_values)) = rows.get_mut(target_row) else {
                    return Err(XdlError::InvalidArgument(
                        "Matrix block assignment requires rectangular numeric rows".to_string(),
                    ));
                };
                for target_column in (column_start..column_end).step_by(column_step) {
                    let Some(target) = target_values.get_mut(target_column) else {
                        return Err(XdlError::RuntimeError(
                            "Matrix block column index is out of bounds".to_string(),
                        ));
                    };
                    *target = scalar;
                }
            }
        }
        Ok(())
    }

    fn matrix_axis_bounds(
        index: &ResolvedIndex,
        length: usize,
    ) -> XdlResult<(usize, usize, usize, usize)> {
        match index {
            ResolvedIndex::Single(raw) => {
                let position = resolve_position(*raw, length)?;
                Ok((position, position + 1, 1, 1))
            }
            ResolvedIndex::Range { .. } | ResolvedIndex::All => Self::range_bounds(index, length),
        }
    }

    fn write_f64_range(
        target: &mut [f64],
        index: &ResolvedIndex,
        value: XdlValue,
    ) -> XdlResult<()> {
        let (start, end, step, selected_len) = Self::range_bounds(index, target.len())?;
        let values = Self::numeric_values(value)?;
        if values.len() == 1 {
            for position in (start..end).step_by(step) {
                target[position] = values[0];
            }
        } else if values.len() == selected_len {
            for (offset, value) in values.into_iter().enumerate() {
                target[start + offset * step] = value;
            }
        } else {
            return Err(XdlError::RuntimeError(format!(
                "Array assignment has {} destination elements but {} source elements",
                selected_len,
                values.len()
            )));
        }
        Ok(())
    }

    fn range_bounds(
        index: &ResolvedIndex,
        length: usize,
    ) -> XdlResult<(usize, usize, usize, usize)> {
        let (start, end, step) = match index {
            ResolvedIndex::Range { start, end, step } => (
                start.unwrap_or(0),
                end.unwrap_or(length as i64),
                step.unwrap_or(1),
            ),
            ResolvedIndex::All => (0, length as i64, 1),
            ResolvedIndex::Single(_) => {
                return Err(XdlError::RuntimeError("Expected a range index".to_string()));
            }
        };
        let start = usize::try_from(start).map_err(|_| {
            XdlError::RuntimeError("Array assignment range start must be non-negative".to_string())
        })?;
        let end = usize::try_from(end).map_err(|_| {
            XdlError::RuntimeError("Array assignment range end must be non-negative".to_string())
        })?;
        let step = usize::try_from(step)
            .ok()
            .filter(|&step| step > 0)
            .ok_or_else(|| {
                XdlError::RuntimeError("Array assignment range step must be positive".to_string())
            })?;
        if start > length || end > length {
            return Err(XdlError::RuntimeError(format!(
                "Array assignment range {}:{} exceeds array length {}",
                start, end, length
            )));
        }
        let selected_len = if start < end {
            1 + (end - 1 - start) / step
        } else {
            0
        };
        Ok((start, end, step, selected_len))
    }

    fn numeric_values(value: XdlValue) -> XdlResult<Vec<f64>> {
        match value {
            XdlValue::Array(values) => Ok(values),
            XdlValue::MultiDimArray { data, .. } => Ok(data),
            XdlValue::NestedArray(_) => Err(XdlError::NotImplemented(
                "Nested-array sources in flat range assignment are not supported".to_string(),
            )),
            scalar => Ok(vec![scalar.to_double()?]),
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

    /// Call a user-defined function
    fn call_user_function(
        &mut self,
        name: &str,
        args: &[Expression],
        keywords: &[xdl_parser::Keyword],
        func_def: &context::FunctionDef,
    ) -> XdlResult<XdlValue> {
        // Evaluate arguments in current scope
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

        // Push a new scope for the function execution
        self.context.push_scope();

        // Bind positional parameters to arguments
        for (i, param) in func_def.params.iter().enumerate() {
            if i < arg_values.len() {
                self.context
                    .set_variable(param.name.clone(), arg_values[i].clone());
            } else if !param.optional {
                self.context.pop_scope()?;
                return Err(XdlError::RuntimeError(format!(
                    "Missing required parameter '{}' for function '{}'",
                    param.name, name
                )));
            }
        }

        // Bind keyword parameters
        for keyword_decl in &func_def.keywords {
            let key = keyword_decl.name.clone();
            let value_opt = keyword_map
                .get(&key)
                .or_else(|| keyword_map.get(&key.to_uppercase()));
            if let Some(value) = value_opt {
                self.context.set_variable(key, value.clone());
            }
        }

        // Execute function body and capture return value
        let mut result = XdlValue::Undefined;
        for stmt in &func_def.body {
            match self.execute_statement(stmt) {
                Ok(()) => continue,
                Err(XdlError::Return(val)) => {
                    result = val;
                    break;
                }
                Err(e) => {
                    self.context.pop_scope()?;
                    return Err(e);
                }
            }
        }

        // Pop the function scope
        self.context.pop_scope()?;

        Ok(result)
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

#[cfg(test)]
mod array_element_access_tests {
    use super::*;

    /// Run a program and return the interpreter, so tests can inspect variables.
    /// No unwrap/expect: the panic ratchet counts test code too.
    fn run(source: &str) -> XdlResult<Interpreter> {
        let program = xdl_parser::parse_xdl(source)
            .map_err(|e| XdlError::RuntimeError(format!("parse: {e}")))?;
        let mut interpreter = Interpreter::new();
        interpreter.execute_program(&program)?;
        Ok(interpreter)
    }

    fn variable(interpreter: &Interpreter, name: &str) -> XdlResult<XdlValue> {
        interpreter.context.get_variable(name).cloned()
    }

    #[test]
    fn multidim_writes_land_in_column_major_slots() -> XdlResult<()> {
        // 2x3 array: w[i,j] is flat index i + 2*j.
        let interpreter = run("w = FLTARR(2,3)\nw[1,2] = 7.0\nw[0,1] = 2.0\n")?;
        assert!(matches!(
            variable(&interpreter, "w")?,
            XdlValue::MultiDimArray { ref data, .. }
                if data == &[0.0, 0.0, 2.0, 0.0, 0.0, 7.0]
        ));
        Ok(())
    }

    #[test]
    fn out_of_range_subscript_is_an_error_not_a_neighbouring_element() -> XdlResult<()> {
        // x[2,0] on a 2x2 array has flat index 2, which is x[0,1]. It must be rejected.
        let result = run("x = FLTARR(2,2)\nx[2,0] = 5.0\n");
        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn single_subscript_on_multidim_reads_flat_element() -> XdlResult<()> {
        let interpreter = run("w = FLTARR(2,3)\nw[1,2] = 7.0\nv = w[5]\n")?;
        assert!(matches!(variable(&interpreter, "v")?, XdlValue::Double(v) if v == 7.0));
        Ok(())
    }

    #[test]
    fn nested_loop_fill_counts_every_element() -> XdlResult<()> {
        let interpreter = run("a = FLTARR(3,2,2)\n\
             FOR i = 0, 2 DO BEGIN\n\
               FOR j = 0, 1 DO BEGIN\n\
                 FOR k = 0, 1 DO BEGIN\n\
                   a[i,j,k] = 1.0\n\
                 ENDFOR\n\
               ENDFOR\n\
             ENDFOR\n\
             t = TOTAL(a)\n")?;
        assert!(matches!(variable(&interpreter, "t")?, XdlValue::Double(v) if v == 12.0));
        Ok(())
    }

    #[test]
    fn element_read_matches_write_on_one_dimensional_array() -> XdlResult<()> {
        let interpreter = run("x = FINDGEN(5)\ny = x[-1]\nz = x[0]\n")?;
        assert!(matches!(variable(&interpreter, "y")?, XdlValue::Double(v) if v == 4.0));
        assert!(matches!(variable(&interpreter, "z")?, XdlValue::Double(v) if v == 0.0));
        Ok(())
    }

    #[test]
    fn range_assignment_writes_matching_vector_values() -> XdlResult<()> {
        let interpreter = run("x = [10, 20, 30, 40]\nx[1:3] = [99, 88]\n")?;
        assert!(matches!(
            variable(&interpreter, "x")?,
            XdlValue::Array(ref values) if values == &[10.0, 99.0, 88.0, 40.0]
        ));
        Ok(())
    }

    #[test]
    fn range_assignment_expands_scalar_over_vector_selection() -> XdlResult<()> {
        let interpreter = run("x = [10, 20, 30, 40]\nx[1:4:2] = 7\n")?;
        assert!(matches!(
            variable(&interpreter, "x")?,
            XdlValue::Array(ref values) if values == &[10.0, 7.0, 30.0, 7.0]
        ));
        Ok(())
    }

    #[test]
    fn linear_range_assignment_uses_flat_storage_for_shaped_arrays() -> XdlResult<()> {
        let interpreter = run("w = FLTARR(2,3)\nw[1:4] = [5, 6, 7]\n")?;
        assert!(matches!(
            variable(&interpreter, "w")?,
            XdlValue::MultiDimArray { ref data, .. }
                if data == &[0.0, 5.0, 6.0, 7.0, 0.0, 0.0]
        ));
        Ok(())
    }

    #[test]
    fn nested_matrix_column_range_assignment_preserves_rows() -> XdlResult<()> {
        let interpreter = run("matrix = [[1, 2, 3], [4, 5, 6]]\nmatrix[0:2, 0] = [11, 22]\n")?;
        assert!(matches!(
            variable(&interpreter, "matrix")?,
            XdlValue::NestedArray(ref rows)
                if matches!(&rows[0], XdlValue::Array(values) if values == &[11.0, 2.0, 3.0])
                    && matches!(&rows[1], XdlValue::Array(values) if values == &[22.0, 5.0, 6.0])
        ));
        Ok(())
    }

    #[test]
    fn nested_matrix_block_assignment_preserves_source_shape() -> XdlResult<()> {
        let interpreter = run("matrix = [[1, 2, 3], [4, 5, 6], [7, 8, 9]]\n\
             matrix[0:2, 1:3] = [[20, 30], [50, 60]]\n")?;
        assert!(matches!(
            variable(&interpreter, "matrix")?,
            XdlValue::NestedArray(ref rows)
                if matches!(&rows[0], XdlValue::Array(values) if values == &[1.0, 20.0, 30.0])
                    && matches!(&rows[1], XdlValue::Array(values) if values == &[4.0, 50.0, 60.0])
                    && matches!(&rows[2], XdlValue::Array(values) if values == &[7.0, 8.0, 9.0])
        ));
        Ok(())
    }

    #[test]
    fn nested_matrix_block_assignment_expands_scalar() -> XdlResult<()> {
        let interpreter = run("matrix = [[1, 2, 3], [4, 5, 6]]\nmatrix[0:2, 1:3] = 9\n")?;
        assert!(matches!(
            variable(&interpreter, "matrix")?,
            XdlValue::NestedArray(ref rows)
                if matches!(&rows[0], XdlValue::Array(values) if values == &[1.0, 9.0, 9.0])
                    && matches!(&rows[1], XdlValue::Array(values) if values == &[4.0, 9.0, 9.0])
        ));
        Ok(())
    }

    #[test]
    fn nested_matrix_block_assignment_accepts_scalar_row_with_range_column() -> XdlResult<()> {
        let interpreter = run("matrix = [[1, 2, 3], [4, 5, 6]]\nmatrix[0, 1:3] = [8, 9]\n")?;
        assert!(matches!(
            variable(&interpreter, "matrix")?,
            XdlValue::NestedArray(ref rows)
                if matches!(&rows[0], XdlValue::Array(values) if values == &[1.0, 8.0, 9.0])
                    && matches!(&rows[1], XdlValue::Array(values) if values == &[4.0, 5.0, 6.0])
        ));
        Ok(())
    }

    #[test]
    fn nested_matrix_block_assignment_rejects_shape_and_ragged_rows() {
        assert!(run("matrix = [[1, 2, 3], [4, 5, 6]]\nmatrix[0:2, 1:3] = [[8, 9]]\n").is_err());
        assert!(run("matrix = [[1, 2], [3]]\nmatrix[0:2, 0:1] = 5\n").is_err());
        assert!(run("matrix = [[1, 2], [3, 4]]\nmatrix[0:2, 0:2] = [[5, 6], [7]]\n").is_err());
    }

    #[test]
    fn range_assignment_rejects_shape_mismatch_and_out_of_bounds() -> XdlResult<()> {
        assert!(run("x = [1, 2, 3]\nx[0:2] = [8, 9, 10]\n").is_err());
        assert!(run("x = [1, 2, 3]\nx[2:4] = 0\n").is_err());
        Ok(())
    }

    #[test]
    fn flat_range_assignment_rejects_nested_source_shape() -> XdlResult<()> {
        assert!(run("x = FLTARR(4)\ny = [[1, 2], [3, 4]]\nx[0:4] = y\n").is_err());
        Ok(())
    }
}
