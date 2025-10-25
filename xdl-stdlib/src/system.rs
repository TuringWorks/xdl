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

/// SYSTIME - Get system time
/// SYSTIME([/SECONDS, /JULIAN])
/// Returns current system time as string or seconds since epoch
pub fn systime(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    // Get current time
    let now = SystemTime::now();
    let duration = now
        .duration_since(UNIX_EPOCH)
        .map_err(|e| XdlError::RuntimeError(format!("System time error: {}", e)))?;

    // Check if /SECONDS keyword is present (return as numeric)
    // For now, simple implementation returns seconds as default
    if args.is_empty() {
        // Return formatted string like "Sat Jan 25 08:30:00 2025"
        let secs = duration.as_secs();
        // Simple date formatting
        let datetime = format_unix_time(secs);
        Ok(XdlValue::String(datetime))
    } else {
        // Return seconds since epoch
        Ok(XdlValue::Double(duration.as_secs() as f64))
    }
}

/// Helper to format unix timestamp
fn format_unix_time(secs: u64) -> String {
    // Simple formatting - days since 1970
    let days = secs / 86400;
    let year = 1970 + (days / 365) as i32;
    let day_of_year = (days % 365) as i32;
    let _month = (day_of_year / 30).min(11) + 1;
    let day = (day_of_year % 30) + 1;

    let time_of_day = secs % 86400;
    let hours = time_of_day / 3600;
    let minutes = (time_of_day % 3600) / 60;
    let seconds = time_of_day % 60;

    format!(
        "Day {} {:02}:{:02}:{:02} {}",
        day, hours, minutes, seconds, year
    )
}

/// JULDAY - Convert calendar date to Julian day number
/// JULDAY(month, day, year [, hour, minute, second])
/// Returns Julian day number
pub fn julday(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "JULDAY: Expected at least 3 arguments (month, day, year)".to_string(),
        ));
    }

    let month = match &args[0] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let day = match &args[1] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let year = match &args[2] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    // Julian day calculation (simplified algorithm)
    // Based on standard astronomical formula
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;

    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;

    Ok(XdlValue::Double(jdn as f64))
}

/// CALDAT - Convert Julian day to calendar date
/// CALDAT(julian, month, day, year [, hour, minute, second])
/// Decomposes Julian day number into calendar components
pub fn caldat(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 4 {
        return Err(XdlError::InvalidArgument(
            "CALDAT: Expected at least 4 arguments (julian, month_var, day_var, year_var)"
                .to_string(),
        ));
    }

    let julian = match &args[0] {
        XdlValue::Double(v) => *v as i32,
        XdlValue::Float(v) => *v as i32,
        XdlValue::Long(v) => *v,
        XdlValue::Int(v) => *v as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Inverse Julian day calculation
    let a = julian + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = 100 * b + d - 4800 + m / 10;

    // In GDL, CALDAT modifies variables passed by reference
    // For now, return as array: [month, day, year]
    let result = vec![
        XdlValue::Long(month),
        XdlValue::Long(day),
        XdlValue::Long(year),
    ];

    Ok(XdlValue::NestedArray(result))
}

/// DAYOFYEAR - Get day of year from date
/// DAYOFYEAR(month, day, year)
pub fn dayofyear(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "DAYOFYEAR: Expected month, day, year".to_string(),
        ));
    }

    let month = match &args[0] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let day = match &args[1] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let year = match &args[2] {
        XdlValue::Int(v) => *v as i32,
        XdlValue::Long(v) => *v,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[2].gdl_type()),
            })
        }
    };

    // Days in each month
    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    if !(1..=12).contains(&month) {
        return Err(XdlError::InvalidArgument(
            "DAYOFYEAR: Month must be between 1 and 12".to_string(),
        ));
    }

    let mut doy = day;
    for &days_in_month in days_in_months.iter().take((month - 1) as usize) {
        doy += days_in_month;
    }

    Ok(XdlValue::Long(doy))
}

/// JS2JD - Convert Julian seconds to Julian date
/// JS2JD(julian_seconds)
pub fn js2jd(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "JS2JD: Expected Julian seconds".to_string(),
        ));
    }

    let js = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Julian seconds are seconds since 12h Jan 1, 4713 BC
    // Convert to Julian days
    let jd = js / 86400.0;

    Ok(XdlValue::Double(jd))
}

/// MESSAGE - Print message or error
/// MESSAGE(text [, /INFORMATIONAL, /CONTINUE])
pub fn message(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "MESSAGE: Expected message text".to_string(),
        ));
    }

    let text = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Simple implementation - just print the message
    eprintln!("% {}", text);
    Ok(XdlValue::Undefined)
}

/// ON_ERROR - Set error handling mode (placeholder)
/// ON_ERROR(mode)
pub fn on_error(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ON_ERROR: Expected error mode".to_string(),
        ));
    }

    let _mode = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Placeholder - error handling mode not fully implemented
    Ok(XdlValue::Undefined)
}

/// MEMORY - Return memory usage information
/// MEMORY([/CURRENT, /HIGHWATER, /L64])
pub fn memory(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Placeholder implementation - return mock memory info
    // In full implementation, would query actual memory usage
    let memory_info = vec![
        XdlValue::Long64(1024 * 1024 * 100), // Current memory (100MB mock)
        XdlValue::Long64(1024 * 1024 * 200), // High water mark (200MB mock)
    ];

    Ok(XdlValue::NestedArray(memory_info))
}

/// EXIT - Exit XDL session
/// EXIT([status])
pub fn exit(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let status = if args.is_empty() {
        0
    } else {
        match &args[0] {
            XdlValue::Long(n) => *n,
            XdlValue::Int(n) => *n as i32,
            _ => 0,
        }
    };

    // Return special exit signal
    // Interpreter should catch this and terminate
    Err(XdlError::RuntimeError(format!("EXIT:{}", status)))
}

/// STOP - Halt program execution (debugging)
pub fn stop(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    eprintln!("% Program stopped - STOP");
    // In full implementation, would enter debugger
    Ok(XdlValue::Undefined)
}

/// RETALL - Return from all levels
pub fn retall(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Signal to return to top level
    Err(XdlError::RuntimeError("RETALL".to_string()))
}

/// ROUTINE_INFO - Get information about routines
/// ROUTINE_INFO([name] [, /FUNCTIONS, /PROCEDURES])
pub fn routine_info(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // Return list of all loaded routines (placeholder)
        let routines = vec![
            XdlValue::String("PLOT".to_string()),
            XdlValue::String("PRINT".to_string()),
            XdlValue::String("HELP".to_string()),
        ];
        return Ok(XdlValue::NestedArray(routines));
    }

    // Get info about specific routine
    match &args[0] {
        XdlValue::String(name) => {
            // Placeholder - return mock info
            let info = vec![
                XdlValue::String(name.clone()),
                XdlValue::String("compiled".to_string()),
            ];
            Ok(XdlValue::NestedArray(info))
        }
        _ => Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    }
}

/// BIN_DATE - Convert system time to date/time array
/// BIN_DATE(time_value)
/// Returns [year, month, day, hour, minute, second]
pub fn bin_date(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "BIN_DATE: Expected time value in seconds since epoch".to_string(),
        ));
    }

    let secs = match &args[0] {
        XdlValue::Double(v) => *v as u64,
        XdlValue::Float(v) => *v as u64,
        XdlValue::Long(v) => *v as u64,
        XdlValue::Int(v) => *v as u64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    // Calculate components from Unix timestamp
    const SECS_PER_DAY: u64 = 86400;
    const SECS_PER_HOUR: u64 = 3600;
    const SECS_PER_MIN: u64 = 60;

    let days_since_epoch = secs / SECS_PER_DAY;
    let time_of_day = secs % SECS_PER_DAY;

    let hours = time_of_day / SECS_PER_HOUR;
    let minutes = (time_of_day % SECS_PER_HOUR) / SECS_PER_MIN;
    let seconds = time_of_day % SECS_PER_MIN;

    // Simplified Gregorian calendar calculation
    let mut year = 1970;
    let mut remaining_days = days_since_epoch as i32;

    loop {
        let days_in_year = if is_leap_year(year) { 366 } else { 365 };
        if remaining_days < days_in_year {
            break;
        }
        remaining_days -= days_in_year;
        year += 1;
    }

    let days_in_months = if is_leap_year(year) {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut month = 1;
    for &days_in_month in &days_in_months {
        if remaining_days < days_in_month {
            break;
        }
        remaining_days -= days_in_month;
        month += 1;
    }
    let day = remaining_days + 1;

    let result = vec![
        XdlValue::Long(year),
        XdlValue::Long(month),
        XdlValue::Long(day),
        XdlValue::Long(hours as i32),
        XdlValue::Long(minutes as i32),
        XdlValue::Long(seconds as i32),
    ];

    Ok(XdlValue::NestedArray(result))
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

/// TIMESTAMP - Generate timestamp string
/// TIMESTAMP([/DATE, /TIME, /DATETIME])
pub fn timestamp(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now();
    let duration = now
        .duration_since(UNIX_EPOCH)
        .map_err(|e| XdlError::RuntimeError(format!("System time error: {}", e)))?;

    let secs = duration.as_secs();

    // Parse as date/time components
    let bin = bin_date(&[XdlValue::Double(secs as f64)])?;
    let components = match bin {
        XdlValue::NestedArray(ref v) => v,
        _ => {
            return Err(XdlError::RuntimeError(
                "Invalid date conversion".to_string(),
            ))
        }
    };

    let year = match components[0] {
        XdlValue::Long(v) => v,
        _ => 0,
    };
    let month = match components[1] {
        XdlValue::Long(v) => v,
        _ => 0,
    };
    let day = match components[2] {
        XdlValue::Long(v) => v,
        _ => 0,
    };
    let hour = match components[3] {
        XdlValue::Long(v) => v,
        _ => 0,
    };
    let minute = match components[4] {
        XdlValue::Long(v) => v,
        _ => 0,
    };
    let second = match components[5] {
        XdlValue::Long(v) => v,
        _ => 0,
    };

    // Default: datetime format
    let timestamp = if args.is_empty() {
        format!(
            "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    } else {
        // Simple implementation - can be extended with keywords
        format!(
            "{:04}-{:02}-{:02} {:02}:{:02}:{:02}",
            year, month, day, hour, minute, second
        )
    };

    Ok(XdlValue::String(timestamp))
}

/// TIMEGEN - Generate array of time values
/// TIMEGEN(start, [final, step])
pub fn timegen(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TIMEGEN: Expected at least start time".to_string(),
        ));
    }

    let start = match &args[0] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    if args.len() == 1 {
        // Return array from 0 to start-1
        let n = start as usize;
        let result: Vec<XdlValue> = (0..n).map(|i| XdlValue::Double(i as f64)).collect();
        return Ok(XdlValue::NestedArray(result));
    }

    let final_val = match &args[1] {
        XdlValue::Double(v) => *v,
        XdlValue::Float(v) => *v as f64,
        XdlValue::Long(v) => *v as f64,
        XdlValue::Int(v) => *v as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let step = if args.len() > 2 {
        match &args[2] {
            XdlValue::Double(v) => *v,
            XdlValue::Float(v) => *v as f64,
            XdlValue::Long(v) => *v as f64,
            XdlValue::Int(v) => *v as f64,
            _ => {
                return Err(XdlError::TypeMismatch {
                    expected: "numeric".to_string(),
                    actual: format!("{:?}", args[2].gdl_type()),
                })
            }
        }
    } else {
        1.0
    };

    if step == 0.0 {
        return Err(XdlError::InvalidArgument(
            "TIMEGEN: Step cannot be zero".to_string(),
        ));
    }

    let mut result = Vec::new();
    let mut current = start;

    if step > 0.0 {
        while current <= final_val {
            result.push(XdlValue::Double(current));
            current += step;
        }
    } else {
        while current >= final_val {
            result.push(XdlValue::Double(current));
            current += step;
        }
    }

    Ok(XdlValue::NestedArray(result))
}
