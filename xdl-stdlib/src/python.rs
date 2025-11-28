//! Python integration module
//!
//! This module provides real Python execution capabilities using PyO3

use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use std::collections::HashMap;
use tracing::info;
use xdl_core::{XdlResult, XdlValue};

/// Python module manager with simpler gil handling
pub struct PythonManager {
    /// Imported modules cache (just module names for now)
    modules: HashMap<String, String>,
    /// Stored Python objects by ID
    objects: HashMap<String, Py<PyAny>>,
    /// Counter for generating unique object IDs
    object_counter: usize,
}

impl PythonManager {
    pub fn new() -> XdlResult<Self> {
        // Initialize Python interpreter if not already done
        // This is safe to call multiple times
        let _ = Python::initialize();

        // Verify Python is working
        Python::attach(|py| {
            let version = py.version_info();
            info!(
                "Python version: {}.{}.{}",
                version.major, version.minor, version.patch
            );
        });

        Ok(Self {
            modules: HashMap::new(),
            objects: HashMap::new(),
            object_counter: 0,
        })
    }

    /// Import a Python module
    pub fn import_module(&mut self, module_name: &str) -> XdlResult<String> {
        let module_key = format!("module_{}", self.modules.len());

        // Test that the module can be imported
        Python::attach(|py| match py.import(module_name) {
            Ok(_) => {
                self.modules
                    .insert(module_key.clone(), module_name.to_string());
                Ok(module_key)
            }
            Err(e) => Err(xdl_core::XdlError::RuntimeError(format!(
                "Failed to import Python module '{}': {}",
                module_name, e
            ))),
        })
    }

    /// Call a method on a Python module
    pub fn call_method(
        &mut self,
        module_key: &str,
        method_name: &str,
        args: &[XdlValue],
        kwargs: &[(String, XdlValue)],
    ) -> XdlResult<XdlValue> {
        let module_name = self
            .modules
            .get(module_key)
            .ok_or_else(|| {
                xdl_core::XdlError::RuntimeError(format!("Python module not found: {}", module_key))
            })?
            .clone(); // Clone the module name to avoid borrow issues

        Python::attach(|py| {
            let module = py.import(module_name.as_str()).map_err(|e| {
                xdl_core::XdlError::RuntimeError(format!(
                    "Failed to re-import module '{}': {}",
                    module_name, e
                ))
            })?;

            // Convert XdlValue arguments to Python objects
            let py_args: Vec<Py<PyAny>> = args
                .iter()
                .map(|arg| self.gdl_to_python(py, arg))
                .collect::<Result<Vec<_>, _>>()?;

            // Create kwargs dictionary
            let py_kwargs = PyDict::new(py);
            for (key, value) in kwargs {
                let py_value = self.gdl_to_python(py, value)?;
                py_kwargs.set_item(key, py_value).map_err(|e| {
                    xdl_core::XdlError::RuntimeError(format!("Failed to set kwargs: {}", e))
                })?;
            }

            // Call the method
            let result = if py_kwargs.is_empty() {
                module.call_method1(method_name, PyTuple::new(py, py_args).unwrap())
            } else {
                module.call_method(
                    method_name,
                    PyTuple::new(py, py_args).unwrap(),
                    Some(&py_kwargs),
                )
            };

            match result {
                Ok(py_result) => {
                    // Convert Python result back to XdlValue
                    self.python_to_gdl(py, py_result.unbind())
                }
                Err(e) => Err(xdl_core::XdlError::RuntimeError(format!(
                    "Python method call failed: {}",
                    e
                ))),
            }
        })
    }

    /// Store a Python object and return its ID
    fn store_object(&mut self, _py: Python, py_obj: Py<PyAny>) -> String {
        let object_id = format!("pyobj_{}", self.object_counter);
        self.object_counter += 1;
        self.objects.insert(object_id.clone(), py_obj);
        object_id
    }

    /// Retrieve a stored Python object
    fn get_object(&self, object_id: &str) -> XdlResult<&Py<PyAny>> {
        self.objects.get(object_id).ok_or_else(|| {
            xdl_core::XdlError::RuntimeError(format!("Python object not found: {}", object_id))
        })
    }

    /// Get string representation of a Python object
    pub fn object_to_string(&self, object_id: &str) -> XdlResult<String> {
        Python::attach(|py| {
            let py_obj = self.get_object(object_id)?;
            let bound = py_obj.bind(py);

            let py_str = bound.str().map_err(|e| {
                xdl_core::XdlError::RuntimeError(format!("Python str() failed: {}", e))
            })?;

            let rust_str = py_str.to_str().map_err(|e| {
                xdl_core::XdlError::RuntimeError(format!("String conversion failed: {}", e))
            })?;

            Ok(rust_str.to_string())
        })
    }

    /// Convert XdlValue to Python object
    fn gdl_to_python(&self, py: Python, value: &XdlValue) -> XdlResult<Py<PyAny>> {
        match value {
            XdlValue::Long(n) => Ok(n.into_pyobject(py).unwrap().into_any().unbind()),
            XdlValue::Double(f) => Ok(f.into_pyobject(py).unwrap().into_any().unbind()),
            XdlValue::Float(f) => Ok((*f as f64).into_pyobject(py).unwrap().into_any().unbind()),
            XdlValue::String(s) => {
                // Check if this is a module key or Python object ID
                if s.starts_with("pyobj_") {
                    // This is a Python object reference, retrieve it
                    let py_obj = self.get_object(s)?;
                    Ok(py_obj.clone_ref(py).into_any())
                } else {
                    // Regular string
                    Ok(s.into_pyobject(py).unwrap().into_any().unbind())
                }
            }
            XdlValue::PythonObject(object_id) => {
                let py_obj = self.get_object(object_id)?;
                Ok(py_obj.clone_ref(py).into_any())
            }
            _ => Err(xdl_core::XdlError::RuntimeError(format!(
                "Cannot convert XdlValue to Python: {:?}",
                value
            ))),
        }
    }

    /// Convert Python object to XdlValue
    fn python_to_gdl(&mut self, py: Python, py_obj: Py<PyAny>) -> XdlResult<XdlValue> {
        let bound_obj = py_obj.bind(py);

        // Try to handle different Python types
        if let Ok(val) = bound_obj.extract::<i64>() {
            Ok(XdlValue::Long(val as i32))
        } else if let Ok(val) = bound_obj.extract::<f64>() {
            Ok(XdlValue::Double(val))
        } else if let Ok(val) = bound_obj.extract::<String>() {
            Ok(XdlValue::String(val))
        } else if bound_obj.is_none() {
            Ok(XdlValue::Undefined)
        } else {
            // For complex types (arrays, objects, etc.), store and return reference
            let object_id = self.store_object(py, py_obj);
            Ok(XdlValue::PythonObject(object_id))
        }
    }
}

impl Default for PythonManager {
    fn default() -> Self {
        Self::new().expect("Failed to initialize Python manager")
    }
}

// Thread-local storage for Python manager
thread_local! {
    static PYTHON_MANAGER: std::cell::RefCell<Option<PythonManager>> = const { std::cell::RefCell::new(None) };
}

/// Get or create the Python manager
fn with_python_manager<F, R>(f: F) -> XdlResult<R>
where
    F: FnOnce(&mut PythonManager) -> XdlResult<R>,
{
    PYTHON_MANAGER.with(|manager_cell| {
        let mut manager_opt = manager_cell.borrow_mut();
        if manager_opt.is_none() {
            *manager_opt = Some(PythonManager::new()?);
        }
        f(manager_opt.as_mut().unwrap())
    })
}

/// Import a Python module
pub fn python_import(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(xdl_core::XdlError::RuntimeError(
            "PYTHON_IMPORT requires exactly one argument".to_string(),
        ));
    }

    let module_name = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(xdl_core::XdlError::RuntimeError(
                "PYTHON_IMPORT argument must be a string".to_string(),
            ))
        }
    };

    with_python_manager(|manager| {
        let module_key = manager.import_module(module_name)?;
        Ok(XdlValue::String(module_key))
    })
}

/// Call a Python method
pub fn python_call(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(xdl_core::XdlError::RuntimeError(
            "PYTHON_CALL requires at least 2 arguments: module and method name".to_string(),
        ));
    }

    let module_key = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(xdl_core::XdlError::RuntimeError(
                "PYTHON_CALL first argument must be a Python module".to_string(),
            ))
        }
    };

    let method_name = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(xdl_core::XdlError::RuntimeError(
                "PYTHON_CALL second argument must be a method name".to_string(),
            ))
        }
    };

    let method_args = &args[2..];

    with_python_manager(|manager| manager.call_method(module_key, method_name, method_args, &[]))
}

/// Call a Python method with keyword arguments
pub fn python_call_kw(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(xdl_core::XdlError::RuntimeError(
            "PYTHON_CALL_KW requires at least 2 arguments: module and method name".to_string(),
        ));
    }

    let module_key = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(xdl_core::XdlError::RuntimeError(
                "PYTHON_CALL_KW first argument must be a Python module".to_string(),
            ))
        }
    };

    let method_name = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(xdl_core::XdlError::RuntimeError(
                "PYTHON_CALL_KW second argument must be a method name".to_string(),
            ))
        }
    };

    // Parse arguments into positional and keyword arguments
    let mut positional_args = Vec::new();
    let mut keyword_args = Vec::new();

    for arg in &args[2..] {
        if let XdlValue::String(s) = arg {
            if let Some(eq_pos) = s.find('=') {
                let key = s[..eq_pos].to_string();
                let value_str = &s[eq_pos + 1..];

                // Try to parse the value
                let value = if let Ok(n) = value_str.parse::<i32>() {
                    XdlValue::Long(n)
                } else if let Ok(f) = value_str.parse::<f64>() {
                    XdlValue::Double(f)
                } else {
                    XdlValue::String(value_str.to_string())
                };

                keyword_args.push((key, value));
            } else {
                positional_args.push(arg.clone());
            }
        } else {
            positional_args.push(arg.clone());
        }
    }

    with_python_manager(|manager| {
        manager.call_method(module_key, method_name, &positional_args, &keyword_args)
    })
}
