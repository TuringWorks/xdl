//! RustPython integration for XDL
//!
//! This module provides an embedded Python interpreter using RustPython,
//! a Python implementation written entirely in Rust.
//!
//! # Limitations
//! RustPython does NOT support C extension modules like:
//! - NumPy, Pandas, scikit-learn, TensorFlow, etc.
//!
//! For those packages, use PyO3 with `--features python` instead.
//!
//! # Use Cases
//! - Running pure Python scripts
//! - Using Python's standard library
//! - Scripting and automation
//! - Educational purposes

use rustpython_vm as vm;
use rustpython_vm::convert::ToPyResult;
use rustpython_vm::Interpreter;
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

lazy_static::lazy_static! {
    /// Global RustPython interpreter
    static ref INTERPRETER: Mutex<Option<Interpreter>> = Mutex::new(None);
}

/// Initialize the RustPython interpreter
fn get_interpreter() -> XdlResult<()> {
    let mut interp = INTERPRETER.lock().unwrap();
    if interp.is_none() {
        *interp = Some(
            rustpython::InterpreterConfig::new()
                .init_stdlib()
                .interpreter(),
        );
    }
    Ok(())
}

/// RUSTPY_EXEC - Execute Python code
/// Usage: result = RUSTPY_EXEC(code)
///
/// Executes pure Python code and returns the result.
/// Note: NumPy, Pandas, and other C extensions are NOT available.
pub fn rustpy_exec(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "RUSTPY_EXEC requires 1 argument: code".to_string(),
        ));
    }

    let code = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => return Err(XdlError::RuntimeError("Code must be a string".to_string())),
    };

    get_interpreter()?;

    let interp = INTERPRETER.lock().unwrap();
    let interp = interp
        .as_ref()
        .ok_or_else(|| XdlError::RuntimeError("Failed to get interpreter".to_string()))?;

    interp.enter(|vm| {
        let scope = vm.new_scope_with_builtins();

        match vm.run_block_expr(scope.clone(), &code) {
            Ok(result) => {
                // Try to convert Python result to XdlValue
                if vm.is_none(&result) {
                    Ok(XdlValue::Undefined)
                } else if let Ok(s) = result.try_to_value::<String>(vm) {
                    Ok(XdlValue::String(s))
                } else if let Ok(i) = result.try_to_value::<i64>(vm) {
                    Ok(XdlValue::Long(i as i32))
                } else if let Ok(f) = result.try_to_value::<f64>(vm) {
                    Ok(XdlValue::Double(f))
                } else if let Ok(b) = result.try_to_value::<bool>(vm) {
                    Ok(XdlValue::Byte(if b { 1 } else { 0 }))
                } else if let Ok(list) = result.try_to_value::<Vec<i64>>(vm) {
                    Ok(XdlValue::Array(list.iter().map(|&x| x as f64).collect()))
                } else if let Ok(list) = result.try_to_value::<Vec<f64>>(vm) {
                    Ok(XdlValue::Array(list))
                } else if let Ok(list) = result.try_to_value::<Vec<String>>(vm) {
                    Ok(XdlValue::NestedArray(
                        list.into_iter().map(XdlValue::String).collect(),
                    ))
                } else {
                    // Return string representation
                    let repr = result
                        .repr(vm)
                        .map(|s| s.to_string())
                        .unwrap_or_else(|_| "<object>".to_string());
                    Ok(XdlValue::String(repr))
                }
            }
            Err(exc) => {
                let msg = exc
                    .to_pyresult(vm)
                    .and_then(|e| e.str(vm))
                    .map(|s| s.to_string())
                    .unwrap_or_else(|_| "Unknown Python error".to_string());
                Err(XdlError::RuntimeError(format!("Python error: {}", msg)))
            }
        }
    })
}

/// RUSTPY_EVAL - Evaluate a Python expression
/// Usage: result = RUSTPY_EVAL(expression)
pub fn rustpy_eval(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "RUSTPY_EVAL requires 1 argument: expression".to_string(),
        ));
    }

    // For simple expressions, wrap in eval-friendly format
    let expr = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Expression must be a string".to_string(),
            ))
        }
    };

    rustpy_exec(&[XdlValue::String(expr)])
}

/// RUSTPY_CALL - Call a Python function with arguments
/// Usage: result = RUSTPY_CALL(func_def, func_name, arg1, arg2, ...)
///
/// Example:
///   code = "def add(a, b): return a + b"
///   result = RUSTPY_CALL(code, "add", 1, 2)  ; Returns 3
pub fn rustpy_call(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::RuntimeError(
            "RUSTPY_CALL requires at least 2 arguments: func_def, func_name".to_string(),
        ));
    }

    let func_def = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Function definition must be a string".to_string(),
            ))
        }
    };

    let func_name = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Function name must be a string".to_string(),
            ))
        }
    };

    // Build argument list
    let py_args: Vec<String> = args[2..]
        .iter()
        .map(|v| match v {
            XdlValue::Long(n) => n.to_string(),
            XdlValue::Int(n) => n.to_string(),
            XdlValue::Float(f) => f.to_string(),
            XdlValue::Double(d) => d.to_string(),
            XdlValue::String(s) => format!("\"{}\"", s.replace('\"', "\\\"")),
            XdlValue::Byte(b) => {
                if *b != 0 {
                    "True".to_string()
                } else {
                    "False".to_string()
                }
            }
            XdlValue::Array(arr) => format!(
                "[{}]",
                arr.iter()
                    .map(|x| x.to_string())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
            _ => "None".to_string(),
        })
        .collect();

    let call_code = format!("{}\n{}({})", func_def, func_name, py_args.join(", "));

    rustpy_exec(&[XdlValue::String(call_code)])
}

/// RUSTPY_IMPORT - Import a Python module (standard library only)
/// Usage: RUSTPY_IMPORT, module_name
///
/// Note: Only standard library modules are available.
/// C extensions (numpy, pandas, etc.) will NOT work.
pub fn rustpy_import(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "RUSTPY_IMPORT requires 1 argument: module_name".to_string(),
        ));
    }

    let module_name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::RuntimeError(
                "Module name must be a string".to_string(),
            ))
        }
    };

    // Check for unsupported modules
    let unsupported = [
        "numpy",
        "pandas",
        "scipy",
        "sklearn",
        "tensorflow",
        "torch",
        "cv2",
        "PIL",
    ];
    if unsupported.iter().any(|&m| module_name.starts_with(m)) {
        return Err(XdlError::RuntimeError(format!(
            "Module '{}' requires C extensions and is not supported by RustPython. \
             Use PyO3 with --features python instead, or use the native Rust alternatives: \
             Polars (dataframes), Linfa (ml), ndarray (arrays).",
            module_name
        )));
    }

    let import_code = format!("import {}", module_name);
    rustpy_exec(&[XdlValue::String(import_code)])?;
    Ok(XdlValue::String(format!("Imported {}", module_name)))
}

/// RUSTPY_VERSION - Get RustPython version info
/// Usage: version = RUSTPY_VERSION()
pub fn rustpy_version(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let code = "import sys; f'{sys.version} (RustPython)'";
    rustpy_exec(&[XdlValue::String(code.to_string())])
}

/// RUSTPY_STDLIB - List available standard library modules
/// Usage: modules = RUSTPY_STDLIB()
pub fn rustpy_stdlib(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let modules = vec![
        "math",
        "random",
        "json",
        "re",
        "datetime",
        "collections",
        "itertools",
        "functools",
        "operator",
        "string",
        "textwrap",
        "struct",
        "codecs",
        "io",
        "os",
        "sys",
        "time",
        "calendar",
        "hashlib",
        "base64",
        "binascii",
        "copy",
        "types",
        "abc",
        "contextlib",
        "decimal",
        "fractions",
        "statistics",
        "cmath",
    ];
    Ok(XdlValue::NestedArray(
        modules
            .iter()
            .map(|s| XdlValue::String(s.to_string()))
            .collect(),
    ))
}
