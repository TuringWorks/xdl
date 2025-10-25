//! System and utility functions

use std::env;
use std::process::Command;
use xdl_core::{XdlError, XdlResult, XdlValue};

/// Helper function to display information about a value (for HELP command)
fn display_value_info(value: &XdlValue) {
    match value {
        XdlValue::Undefined => {
            println!("Variable is UNDEFINED");
        }
        XdlValue::Byte(v) => {
            println!("       BYTE      = {}", v);
        }
        XdlValue::Int(v) => {
            println!("       INT       = {}", v);
        }
        XdlValue::Long(v) => {
            println!("       LONG      = {}", v);
        }
        XdlValue::Float(v) => {
            println!("       FLOAT     = {}", v);
        }
        XdlValue::Double(v) => {
            println!("       DOUBLE    = {}", v);
        }
        XdlValue::String(v) => {
            println!("       STRING    = '{}'", v);
            println!("       Length    = {}", v.len());
        }
        XdlValue::Array(arr) => {
            println!("       DOUBLE    = Array[{}]", arr.len());
            if arr.len() <= 10 {
                println!("       Values    = {:?}", arr);
            } else {
                println!("       First 5   = {:?}", &arr[0..5]);
                println!("       Last 5    = {:?}", &arr[arr.len() - 5..]);
            }
        }
        XdlValue::UInt(v) => {
            println!("       UINT      = {}", v);
        }
        XdlValue::ULong(v) => {
            println!("       ULONG     = {}", v);
        }
        XdlValue::Long64(v) => {
            println!("       LONG64    = {}", v);
        }
        XdlValue::ULong64(v) => {
            println!("       ULONG64   = {}", v);
        }
        XdlValue::Complex(v) => {
            println!("       COMPLEX   = ({}, {})", v.re, v.im);
        }
        XdlValue::DComplex(v) => {
            println!("       DCOMPLEX  = ({}, {})", v.re, v.im);
        }
        XdlValue::Pointer(v) => {
            println!("       POINTER   = <PTR_{:X}>", v);
        }
        XdlValue::ObjRef(v) => {
            println!("       OBJREF    = <OBJ_{:X}>", v);
        }
        XdlValue::NestedArray(rows) => {
            println!("       ARRAY     = NestedArray[{}]", rows.len());
        }
        XdlValue::MultiDimArray { data, shape } => {
            println!("       ARRAY     = MultiDimArray[{}]", data.len());
            println!("       Shape     = {:?}", shape);
        }
        XdlValue::PythonObject(id) => {
            println!("       PYTHON    = <{}>", id);
        }
    }
}

/// HELP procedure - displays information about variables and functions
pub fn help(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // General help
        println!("XDL Help - GNU Data Language Rust Implementation");
        println!("Usage:");
        println!("  HELP              - Show this general help");
        println!("  HELP, /PROCEDURES - List all available procedures");
        println!("  HELP, /FUNCTIONS  - List all available functions");
        println!("  HELP, item_name   - Show help for specific item (not yet implemented)");
        println!();
        println!("Total implemented: 58+ commands across all categories");
        println!("For complete listings, use HELP, /PROCEDURES or HELP, /FUNCTIONS");
    } else if args.len() == 1 {
        match &args[0] {
            XdlValue::String(keyword) => {
                // Check if this is a help keyword (starts with /) or is a known keyword
                if keyword.starts_with('/')
                    || keyword.to_uppercase() == "PROCEDURES"
                    || keyword.to_uppercase() == "FUNCTIONS"
                {
                    match keyword.to_uppercase().as_str() {
                        "/PROCEDURES" | "PROCEDURES" => {
                            println!("=== XDL PROCEDURES ===");
                            println!();
                            println!("Graphics Procedures:");
                            println!("  PLOT        - Create 2D line plots");
                            println!("  OPLOT       - Overplot on existing plot");
                            println!("  CONTOUR     - Generate contour plots");
                            println!("  SURFACE     - Create 3D surface plots");
                            println!("  WINDOW      - Create graphics window");
                            println!("  WSET        - Set current graphics window");
                            println!("  ERASE       - Clear graphics window");
                            println!("  DEVICE      - Set/query graphics device");
                            println!("  LOADCT      - Load color tables");
                            println!("  TVSCL       - Display scaled images");
                            println!("  AXIS        - Draw axes and tick marks");
                            println!();
                            println!("System Procedures:");
                            println!("  EXIT        - Terminate XDL session");
                            println!("  HELP        - Display help information");
                            println!("  PRINT       - Output to console");
                            println!("  CD          - Change directory");
                            println!("  SPAWN       - Execute system commands");
                            println!("  CALL_PROCEDURE - Call procedure dynamically");
                            println!("  DEFSYSV     - Define system variables");
                            println!("  @           - Execute batch files");
                            println!("  .COMPILE    - Compile .pro files");
                            println!("  .CONTINUE   - Continue debugger execution");
                            println!("  CATCH       - Set up error handling");
                            println!();
                            println!("File I/O Procedures:");
                            println!("  OPEN        - Open files for I/O");
                            println!("  CLOSE       - Close open files");
                            println!("  FREE_LUN    - Free logical unit numbers");
                        }
                        "/FUNCTIONS" | "FUNCTIONS" => {
                            println!("=== XDL FUNCTIONS ===");
                            println!();
                            println!("Mathematical Functions:");
                            println!("  SIN, COS, TAN       - Trigonometric functions");
                            println!("  ASIN, ACOS, ATAN    - Inverse trigonometric functions");
                            println!("  EXP                 - Exponential function");
                            println!("  ALOG, LN            - Natural logarithm");
                            println!("  ALOG10              - Base-10 logarithm");
                            println!("  SQRT                - Square root");
                            println!("  ABS                 - Absolute value");
                            println!("  FLOOR, CEIL, ROUND  - Rounding functions");
                            println!();
                            println!("Array Functions:");
                            println!("  FINDGEN     - Generate floating-point arrays");
                            println!("  INDGEN      - Generate integer arrays");
                            println!("  BYTARR      - Create byte arrays");
                            println!("  FLTARR      - Create floating-point arrays");
                            println!("  N_ELEMENTS  - Get array size");
                            println!("  WHERE       - Find array elements by condition");
                            println!();
                            println!("String Functions:");
                            println!("  STRLEN      - Get string length");
                            println!("  STRPOS      - Find substring position");
                            println!("  STRMID      - Extract substring");
                            println!("  STRUPCASE   - Convert to uppercase");
                            println!("  STRLOWCASE  - Convert to lowercase");
                            println!();
                            println!("File I/O Functions:");
                            println!("  GET_LUN     - Obtain logical unit number");
                            println!("  FILEPATH    - Locate files in search path");
                            println!("  READ_JPEG   - Read JPEG image files");
                            println!();
                            println!("Data Structure Functions:");
                            println!("  HASH        - Create hash tables");
                            println!();
                            println!("Python Integration Functions:");
                            println!("  PYTHON_IMPORT    - Import Python modules");
                            println!("  PYTHON_CALL      - Call Python functions");
                            println!("  PYTHON_CALL_KW   - Call Python functions with keywords");
                        }
                        _ => {
                            // Help for specific items
                            println!("Help for '{}' not yet implemented", keyword);
                            println!(
                                "Use HELP, /PROCEDURES or HELP, /FUNCTIONS for complete listings"
                            );
                        }
                    }
                } else {
                    // String value (not a keyword) - show its properties
                    display_value_info(&args[0]);
                }
            }
            // Show information about the variable/value
            value => {
                display_value_info(value);
            }
        }
    } else {
        println!("HELP: Too many arguments. Usage: HELP [keyword]");
    }
    Ok(XdlValue::Undefined)
}

/// CD procedure - change directory
pub fn cd(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "CD: Expected 1 argument (directory path)".to_string(),
        ));
    }

    let path = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    match env::set_current_dir(path) {
        Ok(()) => {
            println!("Changed directory to: {}", path);
            Ok(XdlValue::Undefined)
        }
        Err(e) => Err(XdlError::RuntimeError(format!(
            "CD: Failed to change directory: {}",
            e
        ))),
    }
}

/// SPAWN procedure - execute system command
pub fn spawn(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "SPAWN: Expected 1 argument (command)".to_string(),
        ));
    }

    let command = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    match Command::new("sh").arg("-c").arg(command).status() {
        Ok(status) => {
            if status.success() {
                Ok(XdlValue::Undefined)
            } else {
                Err(XdlError::RuntimeError(format!(
                    "SPAWN: Command failed with exit code: {:?}",
                    status.code()
                )))
            }
        }
        Err(e) => Err(XdlError::RuntimeError(format!(
            "SPAWN: Failed to execute command: {}",
            e
        ))),
    }
}

/// CALL_PROCEDURE - dynamically call a procedure
pub fn call_procedure(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "CALL_PROCEDURE: Expected procedure name".to_string(),
        ));
    }

    let proc_name = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // TODO: Implement dynamic procedure calling
    println!("CALL_PROCEDURE: {} (not yet implemented)", proc_name);
    Ok(XdlValue::Undefined)
}

/// DEFSYSV - define system variable
pub fn defsysv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "DEFSYSV: Expected variable name and value".to_string(),
        ));
    }

    let var_name = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // TODO: Implement system variable storage
    println!(
        "DEFSYSV: Defined system variable {} (placeholder)",
        var_name
    );
    Ok(XdlValue::Undefined)
}

/// Execute batch file - @filename
pub fn execute_batch(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "@: Expected filename argument".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // TODO: Implement batch file execution
    println!("@: Executing batch file {} (not yet implemented)", filename);
    Ok(XdlValue::Undefined)
}

/// Compile and execute .pro file - .COMPILE
pub fn compile_pro(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            ".COMPILE: Expected .pro filename".to_string(),
        ));
    }

    let filename = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // TODO: Implement .pro file compilation
    println!(".COMPILE: Compiling {} (not yet implemented)", filename);
    Ok(XdlValue::Undefined)
}

/// Continue program execution - .CONTINUE
pub fn continue_execution(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !args.is_empty() {
        return Err(XdlError::InvalidArgument(
            ".CONTINUE: No arguments expected".to_string(),
        ));
    }

    // TODO: Implement debugger continue
    println!(".CONTINUE: Continuing execution (debugger not yet implemented)");
    Ok(XdlValue::Undefined)
}

/// Error handler - CATCH
pub fn catch_error(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // TODO: Implement error handling
    println!("CATCH: Error handling not yet implemented");
    Ok(XdlValue::Undefined)
}

/// WAIT procedure - pause execution for specified seconds
pub fn wait(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WAIT: Expected number of seconds to wait".to_string(),
        ));
    }

    let seconds = match &args[0] {
        XdlValue::Int(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Double(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if seconds > 0.0 {
        std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
    }

    Ok(XdlValue::Undefined)
}
