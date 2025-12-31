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

// ============================================================
// Additional Time & Date Functions (Phase 14 Completion)
// ============================================================

/// WEEKDAY - Return day of week (0=Sunday, 6=Saturday)
/// WEEKDAY(julian_day) or WEEKDAY(year, month, day)
pub fn weekday(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WEEKDAY: Expected at least 1 argument".to_string(),
        ));
    }

    let julian_day = if args.len() >= 3 {
        // Year, month, day provided
        let year = args[0].to_long()? as i32;
        let month = args[1].to_long()? as i32;
        let day = args[2].to_long()? as i32;

        // Calculate Julian day
        let a = (14 - month) / 12;
        let y = year + 4800 - a;
        let m = month + 12 * a - 3;
        day + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
    } else {
        args[0].to_long()? as i32
    };

    // Day of week from Julian day (0 = Sunday)
    let dow = (julian_day + 1) % 7;

    Ok(XdlValue::Long(dow))
}

/// BIN_DATE - Return date/time as binary array
/// BIN_DATE([julian_time])
pub fn bin_date(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let (year, month, day, hour, minute, second) = if args.is_empty() {
        // Current date/time
        let duration = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        let secs = duration.as_secs() as i64;

        // Convert to date components (simplified)
        let days = secs / 86400;
        let secs_in_day = secs % 86400;
        let hour = (secs_in_day / 3600) as i32;
        let minute = ((secs_in_day % 3600) / 60) as i32;
        let second = (secs_in_day % 60) as i32;

        // Calculate date from days since epoch
        let mut y = 1970;
        let mut remaining_days = days as i32;

        loop {
            let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
                366
            } else {
                365
            };
            if remaining_days < days_in_year {
                break;
            }
            remaining_days -= days_in_year;
            y += 1;
        }

        let is_leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
        let days_in_months = if is_leap {
            [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        } else {
            [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
        };

        let mut m = 0;
        while m < 12 && remaining_days >= days_in_months[m] {
            remaining_days -= days_in_months[m];
            m += 1;
        }

        (y, (m + 1) as i32, remaining_days + 1, hour, minute, second)
    } else {
        // Parse Julian time
        let jd = args[0].to_double()?;
        let z = (jd + 0.5).floor() as i32;
        let f = jd + 0.5 - z as f64;

        let alpha = ((z as f64 - 1867216.25) / 36524.25).floor() as i32;
        let a = z + 1 + alpha - alpha / 4;
        let b = a + 1524;
        let c = ((b as f64 - 122.1) / 365.25).floor() as i32;
        let d = (365.25 * c as f64).floor() as i32;
        let e = ((b - d) as f64 / 30.6001).floor() as i32;

        let day = b - d - (30.6001 * e as f64).floor() as i32;
        let month = if e < 14 { e - 1 } else { e - 13 };
        let year = if month > 2 { c - 4716 } else { c - 4715 };

        let hours = f * 24.0;
        let hour = hours.floor() as i32;
        let mins = (hours - hour as f64) * 60.0;
        let minute = mins.floor() as i32;
        let second = ((mins - minute as f64) * 60.0).floor() as i32;

        (year, month, day, hour, minute, second)
    };

    Ok(XdlValue::Array(vec![
        year as f64,
        month as f64,
        day as f64,
        hour as f64,
        minute as f64,
        second as f64,
    ]))
}

/// TIMESTAMP - Generate ISO 8601 timestamp string
pub fn timestamp(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    let secs = duration.as_secs();
    let millis = duration.subsec_millis();

    // Convert to date (simplified)
    let total_days = (secs / 86400) as i32;
    let secs_in_day = secs % 86400;

    let mut y = 1970;
    let mut remaining = total_days;
    loop {
        let days_in_year = if (y % 4 == 0 && y % 100 != 0) || y % 400 == 0 {
            366
        } else {
            365
        };
        if remaining < days_in_year {
            break;
        }
        remaining -= days_in_year;
        y += 1;
    }

    let is_leap = (y % 4 == 0 && y % 100 != 0) || y % 400 == 0;
    let days_in_months = if is_leap {
        [31, 29, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    } else {
        [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31]
    };

    let mut m = 0;
    while m < 12 && remaining >= days_in_months[m] {
        remaining -= days_in_months[m];
        m += 1;
    }

    let hour = secs_in_day / 3600;
    let minute = (secs_in_day % 3600) / 60;
    let second = secs_in_day % 60;

    let timestamp = format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02}.{:03}Z",
        y,
        m + 1,
        remaining + 1,
        hour,
        minute,
        second,
        millis
    );

    Ok(XdlValue::String(timestamp))
}

/// TIMEGEN - Generate array of time values
/// TIMEGEN(n, [start, [step]])
pub fn timegen(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "TIMEGEN: Expected at least 1 argument (count)".to_string(),
        ));
    }

    let n = args[0].to_long()? as usize;
    let start = if args.len() > 1 {
        args[1].to_double()?
    } else {
        0.0
    };
    let step = if args.len() > 2 {
        args[2].to_double()?
    } else {
        1.0
    };

    let result: Vec<f64> = (0..n).map(|i| start + (i as f64) * step).collect();

    Ok(XdlValue::Array(result))
}

/// DAYOFYEAR - Return day of year (1-366)
pub fn dayofyear(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 3 {
        return Err(XdlError::InvalidArgument(
            "DAYOFYEAR: Expected 3 arguments (year, month, day)".to_string(),
        ));
    }

    let year = args[0].to_long()? as i32;
    let month = args[1].to_long()? as i32;
    let day = args[2].to_long()? as i32;

    let is_leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
    let days_before_month = if is_leap {
        [0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335]
    } else {
        [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
    };

    let doy = if month >= 1 && month <= 12 {
        days_before_month[(month - 1) as usize] + day
    } else {
        day
    };

    Ok(XdlValue::Long(doy))
}

/// JS2JD - Convert Julian seconds to Julian date
pub fn js2jd(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "JS2JD: Expected 1 argument (julian_seconds)".to_string(),
        ));
    }

    let js = args[0].to_double()?;

    // Julian seconds since J2000.0 (2000-01-01 12:00:00 TT)
    // J2000.0 = JD 2451545.0
    let jd = js / 86400.0 + 2451545.0;

    Ok(XdlValue::Double(jd))
}

// ============================================================
// Additional System & Control Functions (Phase 18)
// ============================================================

/// MEMORY - Return memory usage information
pub fn memory(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Return placeholder memory information
    // Real implementation would query system memory
    let _ = args;
    Ok(XdlValue::Array(vec![
        1024.0 * 1024.0 * 100.0, // Heap usage estimate (100MB)
        1024.0 * 1024.0 * 1024.0, // Available memory (1GB)
    ]))
}

/// EXIT - Exit the XDL session
pub fn exit_session(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let code = if !args.is_empty() {
        args[0].to_long()? as i32
    } else {
        0
    };

    println!("EXIT: Session exit requested with code {}", code);
    // In a real implementation, this would trigger session termination
    Ok(XdlValue::Undefined)
}

/// RETALL - Return to top level
pub fn retall(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    println!("RETALL: Return to top level");
    Ok(XdlValue::Undefined)
}

/// ROUTINE_INFO - Query routine information
pub fn routine_info(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ROUTINE_INFO: Expected routine name argument".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => args[0].to_string_repr(),
    };

    // Return placeholder information
    Ok(XdlValue::String(format!("Routine '{}' information not available", name)))
}

/// MESSAGE - Print a message/error
pub fn message(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Ok(XdlValue::Undefined);
    }

    let msg = args[0].to_string_repr();
    eprintln!("MESSAGE: {}", msg);

    Ok(XdlValue::Undefined)
}

/// ON_ERROR - Set error handling mode
pub fn on_error(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ON_ERROR: Expected error handling mode".to_string(),
        ));
    }

    let mode = args[0].to_long()?;
    println!("ON_ERROR: Error handling mode set to {}", mode);

    Ok(XdlValue::Undefined)
}

/// EXECUTE - Execute a string as XDL code (placeholder)
pub fn execute(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "EXECUTE: Expected code string argument".to_string(),
        ));
    }

    let code = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    println!("EXECUTE: Would execute: {}", code);
    // In a real implementation, this would parse and execute the code
    Ok(XdlValue::Undefined)
}

/// N_PARAMS - Return number of parameters passed to routine
pub fn n_params(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // This would need to be called from within a procedure/function context
    // Return 0 as placeholder
    Ok(XdlValue::Long(0))
}

/// SCOPE_VARNAME - Return names of variables at specified scope level
/// IDL syntax: result = SCOPE_VARNAME([level])
/// Returns an array of variable names at the specified scope level
pub fn scope_varname(args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Level 0 = current scope, 1 = caller, etc.
    let _level = if args.is_empty() {
        0
    } else {
        args[0].to_long().unwrap_or(0)
    };

    // This would need access to the interpreter context to be fully implemented
    // Return an empty array as placeholder
    Ok(XdlValue::NestedArray(vec![]))
}

/// SCOPE_LEVEL - Return the current scope level
/// Returns 0 for main level, 1 for first level procedure, etc.
pub fn scope_level(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // This would need access to the interpreter context to return actual scope level
    // Return 0 (main level) as placeholder
    Ok(XdlValue::Long(0))
}

/// SCOPE_TRACEBACK - Return array of routine names in call stack
/// Returns a string array with the call trace from current routine to main
pub fn scope_traceback(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // This would need access to the interpreter context to get call stack
    // Return a single element "$MAIN$" as placeholder
    Ok(XdlValue::NestedArray(vec![XdlValue::String(
        "$MAIN$".to_string(),
    )]))
}

/// PATH_SEP - Return the platform-specific path separator
pub fn path_sep(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Ok(XdlValue::String(std::path::MAIN_SEPARATOR.to_string()))
}

/// ADD_SLASH - Add trailing path separator if not present
pub fn add_slash(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "ADD_SLASH: Expected path argument".to_string(),
        ));
    }

    let path = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let sep = std::path::MAIN_SEPARATOR;
    if path.ends_with(sep) || path.ends_with('/') || path.ends_with('\\') {
        Ok(XdlValue::String(path))
    } else {
        Ok(XdlValue::String(format!("{}{}", path, sep)))
    }
}

/// GET_SCREEN_SIZE - Return screen dimensions [width, height]
pub fn get_screen_size(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Default screen size (common resolution)
    // In a real GUI implementation, this would query the display
    #[cfg(target_os = "macos")]
    {
        // Try to get actual screen size on macOS
        if let Ok(output) = Command::new("system_profiler")
            .args(["SPDisplaysDataType"])
            .output()
        {
            let stdout = String::from_utf8_lossy(&output.stdout);
            // Parse resolution from output (simplified)
            if stdout.contains("Resolution:") {
                // Default to common MacBook resolution if parsing fails
                return Ok(XdlValue::Array(vec![1920.0, 1080.0]));
            }
        }
    }

    // Default fallback
    Ok(XdlValue::Array(vec![1920.0, 1080.0]))
}

/// GETENV_ALL - Get all environment variables as a struct-like array
pub fn getenv_all(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let vars: Vec<XdlValue> = env::vars()
        .map(|(k, v)| XdlValue::String(format!("{}={}", k, v)))
        .collect();
    Ok(XdlValue::NestedArray(vars))
}

/// SETENV - Set an environment variable
pub fn setenv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SETENV: Expected name and value arguments".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let value = match &args[1] {
        XdlValue::String(s) => s.clone(),
        _ => format!("{:?}", args[1]),
    };

    env::set_var(&name, &value);
    Ok(XdlValue::Undefined)
}

/// UNSETENV - Remove an environment variable
pub fn unsetenv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "UNSETENV: Expected name argument".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    env::remove_var(&name);
    Ok(XdlValue::Undefined)
}

/// CPU - Return CPU information
pub fn cpu(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let num_cpus = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);

    Ok(XdlValue::Long(num_cpus as i32))
}

/// HOSTNAME - Return the system hostname
pub fn hostname(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("hostname").output() {
            let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
            return Ok(XdlValue::String(name));
        }
    }

    Ok(XdlValue::String("localhost".to_string()))
}

/// TEMPORARY - Create a unique temporary filename
pub fn temporary(args: &[XdlValue]) -> XdlResult<XdlValue> {
    let prefix = if !args.is_empty() {
        match &args[0] {
            XdlValue::String(s) => s.clone(),
            _ => "xdl_temp".to_string(),
        }
    } else {
        "xdl_temp".to_string()
    };

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();

    let temp_dir = env::temp_dir();
    let filename = format!("{}_{}.tmp", prefix, timestamp);
    let path = temp_dir.join(filename);

    Ok(XdlValue::String(path.to_string_lossy().to_string()))
}

/// SLEEP - Pause execution for specified seconds
pub fn sleep(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SLEEP: Expected seconds argument".to_string(),
        ));
    }

    let seconds = match &args[0] {
        XdlValue::Float(f) => *f as f64,
        XdlValue::Double(d) => *d,
        XdlValue::Int(i) => *i as f64,
        XdlValue::Long(l) => *l as f64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    std::thread::sleep(std::time::Duration::from_secs_f64(seconds));
    Ok(XdlValue::Undefined)
}

/// VERSION - Return XDL version information
pub fn version(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Return version as a structure-like string array
    let version_info = vec![
        XdlValue::String("XDL".to_string()),
        XdlValue::String("0.1.7".to_string()),
        XdlValue::String(env::consts::OS.to_string()),
        XdlValue::String(env::consts::ARCH.to_string()),
    ];
    Ok(XdlValue::NestedArray(version_info))
}

/// PLATFORM - Return platform information
pub fn platform(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    let platform = format!("{}-{}", env::consts::OS, env::consts::ARCH);
    Ok(XdlValue::String(platform))
}

/// IS_WINDOWS - Check if running on Windows
pub fn is_windows(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Ok(XdlValue::Int(if cfg!(target_os = "windows") { 1 } else { 0 }))
}

/// IS_MACOS - Check if running on macOS
pub fn is_macos(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Ok(XdlValue::Int(if cfg!(target_os = "macos") { 1 } else { 0 }))
}

/// IS_LINUX - Check if running on Linux
pub fn is_linux(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    Ok(XdlValue::Int(if cfg!(target_os = "linux") { 1 } else { 0 }))
}

/// WHICH - Find executable in PATH (like Unix 'which' command)
pub fn which(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "WHICH: Expected program name".to_string(),
        ));
    }

    let program = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    #[cfg(unix)]
    {
        if let Ok(output) = Command::new("which").arg(&program).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                return Ok(XdlValue::String(path));
            }
        }
    }

    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("where").arg(&program).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout)
                    .lines()
                    .next()
                    .unwrap_or("")
                    .to_string();
                return Ok(XdlValue::String(path));
            }
        }
    }

    Ok(XdlValue::String(String::new()))
}
