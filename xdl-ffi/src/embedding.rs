//! # XDL Embedding API
//!
//! C-compatible FFI bindings for embedding XDL in other applications.

use std::ffi::CStr;
use std::os::raw::{c_char, c_double, c_int};
use xdl_stdlib::StandardLibrary;

/// Opaque handle to XDL context
#[repr(C)]
pub struct XdlContext {
    stdlib: StandardLibrary,
}

/// Result codes for XDL operations
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub enum XdlResultCode {
    Success = 0,
    Error = 1,
}

/// Initialize XDL context
#[no_mangle]
pub extern "C" fn xdl_init() -> *mut XdlContext {
    let stdlib = StandardLibrary::new();
    let context = Box::new(XdlContext { stdlib });
    Box::into_raw(context)
}

/// Clean up XDL context
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn xdl_cleanup(context: *mut XdlContext) {
    if !context.is_null() {
        unsafe {
            let _ = Box::from_raw(context);
        }
    }
}

/// Call XDL function with scalar arguments
#[no_mangle]
#[allow(clippy::not_unsafe_ptr_arg_deref)]
pub extern "C" fn xdl_call_function(
    context: *mut XdlContext,
    func_name: *const c_char,
    args: *const c_double,
    nargs: c_int,
) -> c_double {
    if context.is_null() || func_name.is_null() {
        return 0.0;
    }

    let _context = unsafe { &mut *context };
    let func_str = match unsafe { CStr::from_ptr(func_name) }.to_str() {
        Ok(s) => s,
        Err(_) => return 0.0,
    };

    // For now, just handle sin function as an example
    if func_str == "sin" && nargs == 1 && !args.is_null() {
        let arg = unsafe { *args };
        return arg.sin();
    }

    0.0
}
