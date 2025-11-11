//! Input/output functions

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::Mutex;
use xdl_core::{XdlError, XdlResult, XdlValue};

// File handle storage
struct FileHandle {
    reader: Option<BufReader<File>>,
    writer: Option<BufWriter<File>>,
    #[allow(dead_code)]
    mode: FileMode,
}

#[allow(dead_code)]
enum FileMode {
    Read,
    Write,
    Update, // Read/Write
}

// Global file handle management
static LUN_COUNTER: Mutex<i32> = Mutex::new(10); // Start from LUN 10
lazy_static::lazy_static! {
    static ref FILE_HANDLES: Mutex<HashMap<i32, FileHandle>> = Mutex::new(HashMap::new());
}

pub fn print(_args: &[XdlValue]) -> XdlResult<XdlValue> {
    // Print all arguments separated by spaces
    let output = _args
        .iter()
        .map(|arg| arg.to_string_repr())
        .collect::<Vec<_>>()
        .join(" ");

    println!("{}", output);

    // PRINT is a procedure, not a function, so it doesn't return a value
    // But we need to return something for the function interface
    Ok(XdlValue::Undefined)
}

/// GET_LUN - Get a logical unit number for file operations
pub fn get_lun(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if !args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "GET_LUN: No arguments expected".to_string(),
        ));
    }

    let mut counter = LUN_COUNTER.lock().unwrap();
    let lun = *counter;
    *counter += 1;

    Ok(XdlValue::Long(lun))
}

/// FREE_LUN - Free a logical unit number
pub fn free_lun(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "FREE_LUN: Expected 1 argument (LUN)".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    println!("Freed LUN {}", lun);
    Ok(XdlValue::Undefined)
}

/// FILEPATH - Locate a file in the search path
pub fn filepath(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "FILEPATH: Expected 1 argument (filename)".to_string(),
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

    // Simple implementation - check if file exists in current directory
    if Path::new(filename).exists() {
        Ok(XdlValue::String(filename.clone()))
    } else {
        // TODO: Search in XDL path
        Ok(XdlValue::String("".to_string()))
    }
}

/// OPENR - Open file for reading
/// OPENR, lun, filename
pub fn openr(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "OPENR: Expected LUN and filename".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let filename = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Open file for reading
    let file = File::open(filename).map_err(|e| {
        XdlError::RuntimeError(format!("OPENR: Failed to open {}: {}", filename, e))
    })?;

    let handle = FileHandle {
        reader: Some(BufReader::new(file)),
        writer: None,
        mode: FileMode::Read,
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    handles.insert(lun, handle);

    Ok(XdlValue::Undefined)
}

/// OPENW - Open file for writing (create/truncate)
/// OPENW, lun, filename
pub fn openw(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "OPENW: Expected LUN and filename".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let filename = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Open file for writing (create or truncate)
    let file = File::create(filename).map_err(|e| {
        XdlError::RuntimeError(format!("OPENW: Failed to create {}: {}", filename, e))
    })?;

    let handle = FileHandle {
        reader: None,
        writer: Some(BufWriter::new(file)),
        mode: FileMode::Write,
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    handles.insert(lun, handle);

    Ok(XdlValue::Undefined)
}

/// OPENU - Open file for read/write (update)
/// OPENU, lun, filename
pub fn openu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "OPENU: Expected LUN and filename".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let filename = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    // Open file for read/write
    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(filename)
        .map_err(|e| {
            XdlError::RuntimeError(format!("OPENU: Failed to open {}: {}", filename, e))
        })?;

    // For update mode, we can't have both reader and writer on same file simultaneously
    // For now, just provide writer capability
    let handle = FileHandle {
        reader: None,
        writer: Some(BufWriter::new(file)),
        mode: FileMode::Update,
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    handles.insert(lun, handle);

    Ok(XdlValue::Undefined)
}

/// CLOSE - Close a file
pub fn close_file(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() != 1 {
        return Err(XdlError::InvalidArgument(
            "CLOSE: Expected LUN argument".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    if handles.remove(&lun).is_some() {
        Ok(XdlValue::Undefined)
    } else {
        Err(XdlError::RuntimeError(format!(
            "CLOSE: LUN {} not open",
            lun
        )))
    }
}

/// READF - Read formatted data from file
/// READF, lun, variable
/// For now, reads a line as string
pub fn readf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::io::BufRead;

    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "READF: Expected at least LUN argument".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    let handle = handles
        .get_mut(&lun)
        .ok_or_else(|| XdlError::RuntimeError(format!("READF: LUN {} not open", lun)))?;

    // Get the reader
    let reader = handle.reader.as_mut().ok_or_else(|| {
        XdlError::RuntimeError(format!("READF: LUN {} not open for reading", lun))
    })?;

    // Read a line from file
    let mut line = String::new();
    let bytes_read = reader
        .read_line(&mut line)
        .map_err(|e| XdlError::RuntimeError(format!("READF: Read error: {}", e)))?;

    if bytes_read == 0 {
        // EOF
        return Ok(XdlValue::String("".to_string()));
    }

    // Remove trailing newline
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }

    Ok(XdlValue::String(line))
}

/// WRITEF - Write formatted data to file
/// WRITEF, lun, data...
pub fn writef(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "WRITEF: Expected LUN and data arguments".to_string(),
        ));
    }

    let lun = match &args[0] {
        XdlValue::Long(n) => *n,
        XdlValue::Int(n) => *n as i32,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "integer".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let mut handles = FILE_HANDLES.lock().unwrap();
    let handle = handles
        .get_mut(&lun)
        .ok_or_else(|| XdlError::RuntimeError(format!("WRITEF: LUN {} not open", lun)))?;

    // Get the writer
    let writer = handle.writer.as_mut().ok_or_else(|| {
        XdlError::RuntimeError(format!("WRITEF: LUN {} not open for writing", lun))
    })?;

    // Write all arguments (except LUN) separated by spaces
    for (i, arg) in args[1..].iter().enumerate() {
        if i > 0 {
            write!(writer, " ")
                .map_err(|e| XdlError::RuntimeError(format!("WRITEF: Write error: {}", e)))?;
        }
        write!(writer, "{}", arg.to_string_repr())
            .map_err(|e| XdlError::RuntimeError(format!("WRITEF: Write error: {}", e)))?;
    }
    writeln!(writer).map_err(|e| XdlError::RuntimeError(format!("WRITEF: Write error: {}", e)))?;

    writer
        .flush()
        .map_err(|e| XdlError::RuntimeError(format!("WRITEF: Flush error: {}", e)))?;

    Ok(XdlValue::Undefined)
}

/// PRINTF - Print formatted data to file (alias for WRITEF)
/// PRINTF, lun, data...
pub fn printf(args: &[XdlValue]) -> XdlResult<XdlValue> {
    writef(args)
}

/// OPEN - Generic open (for compatibility, redirects to OPENU)
pub fn open_file(args: &[XdlValue]) -> XdlResult<XdlValue> {
    openu(args)
}

/// READ_JPEG - Read JPEG image file
pub fn read_jpeg(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "READ_JPEG: Expected filename".to_string(),
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

    // TODO: Implement JPEG reading with image crate
    if Path::new(filename).exists() {
        println!(
            "READ_JPEG: Read {} (placeholder - returns dimensions)",
            filename
        );
        // Return placeholder dimensions
        Ok(XdlValue::String(format!(
            "JPEG dimensions from {}",
            filename
        )))
    } else {
        Err(XdlError::RuntimeError(format!(
            "READ_JPEG: File not found: {}",
            filename
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_single_value() {
        let args = vec![XdlValue::Long(42)];
        let result = print(&args);
        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), XdlValue::Undefined));
    }

    #[test]
    fn test_print_multiple_values() {
        let args = vec![
            XdlValue::Long(42),
            XdlValue::Double(3.5),
            XdlValue::String("hello".to_string()),
        ];
        let result = print(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_no_args() {
        let args = vec![];
        let result = print(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_print_string() {
        let args = vec![XdlValue::String("test string".to_string())];
        let result = print(&args);
        assert!(result.is_ok());
    }
}
