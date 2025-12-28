//! System and utility functions

use std::env;
use std::process::Command;
use xdl_core::{XdlError, XdlResult, XdlValue};

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
                        println!("Use HELP, /PROCEDURES or HELP, /FUNCTIONS for complete listings");
                    }
                }
            }
            _ => {
                println!("HELP: Invalid argument type. Use string keywords like '/PROCEDURES' or '/FUNCTIONS'");
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

/// WAIT - Pause execution for specified seconds
/// Usage: WAIT, seconds
pub fn wait(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::RuntimeError(
            "WAIT requires at least one argument (seconds)".to_string(),
        ));
    }

    let seconds = match &args[0] {
        XdlValue::Long(n) => *n as f64,
        XdlValue::Double(f) => *f,
        XdlValue::Float(f) => *f as f64,
        _ => {
            return Err(XdlError::RuntimeError(
                "WAIT requires a numeric argument".to_string(),
            ))
        }
    };

    if seconds < 0.0 {
        return Err(XdlError::RuntimeError(
            "WAIT time cannot be negative".to_string(),
        ));
    }

    // Sleep for the specified duration
    std::thread::sleep(std::time::Duration::from_secs_f64(seconds));

    Ok(XdlValue::Undefined)
}

/// STOP - Halt program execution
/// Usage: STOP [, message]
pub fn stop(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let message = if args.is_empty() {
        "% Stop encountered: Execution halted".to_string()
    } else {
        match &args[0] {
            XdlValue::String(s) => format!("% Stop encountered: {}", s),
            _ => format!("% Stop encountered: {:?}", args[0]),
        }
    };

    // In GDL/IDL, STOP enters an interactive debug mode
    // For now, we'll print the message and return an error to halt execution
    eprintln!("{}", message);
    eprintln!("% Execution halted.");

    Err(XdlError::RuntimeError(
        "Execution stopped by STOP command".to_string(),
    ))
}

// ============================================================
// Time Functions
// ============================================================

/// SYSTIME - Get current system time
/// SYSTIME() - Returns current time as string
/// SYSTIME(1) - Returns seconds since Unix epoch
/// SYSTIME(0, julian) - Converts Julian date to calendar string
pub fn systime(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let julian_mode = if !args.is_empty() {
        match &args[0] {
            XdlValue::Long(n) => *n == 1,
            XdlValue::Int(n) => *n == 1,
            _ => false,
        }
    } else {
        false
    };

    let now = SystemTime::now();
    let since_epoch = now.duration_since(UNIX_EPOCH).unwrap();

    if julian_mode {
        // Return seconds since Unix epoch
        Ok(XdlValue::Double(since_epoch.as_secs_f64()))
    } else {
        // Return formatted time string
        let secs = since_epoch.as_secs();

        // Simple UTC time formatting (basic implementation)
        let days_since_1970 = secs / 86400;
        let time_of_day = secs % 86400;
        let hours = time_of_day / 3600;
        let minutes = (time_of_day % 3600) / 60;
        let seconds = time_of_day % 60;

        // Calculate date from days since 1970 (simplified, ignoring leap years correctly for demo)
        let year = 1970 + days_since_1970 / 365;
        let day_of_year = days_since_1970 % 365;
        let month = day_of_year / 30 + 1;
        let day = day_of_year % 30 + 1;

        let months = ["Jan", "Feb", "Mar", "Apr", "May", "Jun",
                      "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];
        let month_name = months.get(month as usize % 12).unwrap_or(&"???");

        Ok(XdlValue::String(format!(
            "{} {:02} {:02}:{:02}:{:02} {}",
            month_name, day, hours, minutes, seconds, year
        )))
    }
}

/// JULDAY - Calculate Julian day number
/// JULDAY(month, day, year [, hour, minute, second])
pub fn julday(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "JULDAY: Expected at least 3 arguments (month, day, year)".to_string(),
        ));
    }

    let month = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let day = match &args[1] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let year = match &args[2] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => return Err(XdlError::TypeMismatch {
            expected: "integer".to_string(),
            actual: format!("{:?}", args[2].gdl_type()),
        }),
    };

    // Optional hour, minute, second
    let hour = if args.len() > 3 {
        match &args[3] {
            XdlValue::Long(n) => *n as f64,
            XdlValue::Double(n) => *n,
            _ => 0.0,
        }
    } else { 0.0 };

    let minute = if args.len() > 4 {
        match &args[4] {
            XdlValue::Long(n) => *n as f64,
            XdlValue::Double(n) => *n,
            _ => 0.0,
        }
    } else { 0.0 };

    let second = if args.len() > 5 {
        match &args[5] {
            XdlValue::Long(n) => *n as f64,
            XdlValue::Double(n) => *n,
            _ => 0.0,
        }
    } else { 0.0 };

    // Julian day calculation (based on standard algorithm)
    let a = (14 - month) / 12;
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;

    let jdn = day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045;

    // Add fractional day for time
    let fraction = (hour - 12.0) / 24.0 + minute / 1440.0 + second / 86400.0;

    Ok(XdlValue::Double(jdn as f64 + fraction))
}

/// CALDAT - Convert Julian date to calendar date
/// CALDAT, julian, month, day, year [, hour, minute, second]
pub fn caldat(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "CALDAT: Expected Julian date argument".to_string(),
        ));
    }

    let julian = match &args[0] {
        XdlValue::Long(n) => *n as f64,
        XdlValue::Double(n) => *n,
        XdlValue::Float(n) => *n as f64,
        _ => return Err(XdlError::TypeMismatch {
            expected: "numeric".to_string(),
            actual: format!("{:?}", args[0].gdl_type()),
        }),
    };

    let jdn = julian.floor() as i32;
    let fraction = julian - jdn as f64;

    // Reverse Julian day calculation
    let a = jdn + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (b * 146097) / 4;
    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let day = e - (153 * m + 2) / 5 + 1;
    let month = m + 3 - 12 * (m / 10);
    let year = b * 100 + d - 4800 + m / 10;

    // Calculate time from fraction
    let hours = ((fraction + 0.5) * 24.0) % 24.0;
    let minutes = hours.fract() * 60.0;
    let seconds = minutes.fract() * 60.0;

    // Return as array [month, day, year, hour, minute, second]
    Ok(XdlValue::Array(vec![
        month as f64,
        day as f64,
        year as f64,
        hours.floor(),
        minutes.floor(),
        seconds.floor(),
    ]))
}

// Global variable for TIC/TOC timing
static mut TIC_TIME: Option<std::time::Instant> = None;

/// TIC - Start timer
pub fn tic(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    unsafe {
        TIC_TIME = Some(std::time::Instant::now());
    }
    Ok(XdlValue::Undefined)
}

/// TOC - Stop timer and return elapsed time
pub fn toc(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    unsafe {
        match TIC_TIME {
            Some(start) => {
                let elapsed = start.elapsed().as_secs_f64();
                println!("Elapsed time: {:.6} seconds", elapsed);
                Ok(XdlValue::Double(elapsed))
            }
            None => Err(XdlError::RuntimeError(
                "TOC: No corresponding TIC call found".to_string(),
            )),
        }
    }
}

// ============================================================
// Structure Functions
// ============================================================

/// N_TAGS - Get number of tags in a structure
pub fn n_tags(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "N_TAGS: Expected structure argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Struct(map) => Ok(XdlValue::Long(map.len() as i32)),
        _ => Ok(XdlValue::Long(0)),
    }
}

/// TAG_NAMES - Get array of tag names from a structure
pub fn tag_names(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TAG_NAMES: Expected structure argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Struct(map) => {
            let names: Vec<XdlValue> = map
                .keys()
                .map(|k| XdlValue::String(k.clone()))
                .collect();
            Ok(XdlValue::NestedArray(names))
        }
        _ => Ok(XdlValue::NestedArray(vec![])),
    }
}

/// SIZE - Get size information about a variable
pub fn size_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SIZE: Expected variable argument".to_string(),
        ));
    }

    match &args[0] {
        XdlValue::Array(arr) => {
            // [1, n_elements, type_code, n_elements]
            Ok(XdlValue::Array(vec![
                1.0, // 1 dimension
                arr.len() as f64,
                5.0, // float type code
                arr.len() as f64,
            ]))
        }
        XdlValue::MultiDimArray { data, shape } => {
            let mut result = vec![shape.len() as f64];
            for dim in shape {
                result.push(*dim as f64);
            }
            result.push(5.0); // float type code
            result.push(data.len() as f64);
            Ok(XdlValue::Array(result))
        }
        XdlValue::String(s) => {
            Ok(XdlValue::Array(vec![1.0, s.len() as f64, 7.0, s.len() as f64]))
        }
        _ => Ok(XdlValue::Array(vec![0.0, 0.0, 0.0, 0.0])),
    }
}

/// ISA - Check if variable is of specified type
pub fn isa(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "ISA: Expected variable and type name arguments".to_string(),
        ));
    }

    let type_name = match &args[1] {
        XdlValue::String(s) => s.to_uppercase(),
        _ => return Err(XdlError::TypeMismatch {
            expected: "string".to_string(),
            actual: format!("{:?}", args[1].gdl_type()),
        }),
    };

    let matches = match &args[0] {
        XdlValue::Array(_) | XdlValue::MultiDimArray { .. } => {
            type_name == "ARRAY" || type_name == "NUMBER" || type_name == "NUMERIC"
        }
        XdlValue::String(_) => type_name == "STRING",
        XdlValue::Long(_) => type_name == "LONG" || type_name == "NUMBER" || type_name == "INTEGER",
        XdlValue::Int(_) => type_name == "INT" || type_name == "NUMBER" || type_name == "INTEGER",
        XdlValue::Float(_) => type_name == "FLOAT" || type_name == "NUMBER",
        XdlValue::Double(_) => type_name == "DOUBLE" || type_name == "NUMBER",
        XdlValue::Byte(_) => type_name == "BYTE" || type_name == "NUMBER" || type_name == "INTEGER",
        XdlValue::Complex(_) => type_name == "COMPLEX" || type_name == "NUMBER",
        XdlValue::Struct(_) => type_name == "STRUCT" || type_name == "STRUCTURE",
        _ => false,
    };

    Ok(XdlValue::Long(if matches { 1 } else { 0 }))
}
