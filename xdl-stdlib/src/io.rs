//! Input/output functions

use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Write};
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
        .truncate(true)
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

/// WRITEU - Write unformatted (binary) data to file
/// WRITEU, lun, data...
pub fn writeu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::io::Write;

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "WRITEU: Expected LUN and data arguments".to_string(),
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
        .ok_or_else(|| XdlError::RuntimeError(format!("WRITEU: LUN {} not open", lun)))?;

    // Get the writer
    let writer = handle.writer.as_mut().ok_or_else(|| {
        XdlError::RuntimeError(format!("WRITEU: LUN {} not open for writing", lun))
    })?;

    // Write each argument as binary data
    for arg in &args[1..] {
        write_binary_value(writer, arg)?;
    }

    writer
        .flush()
        .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Flush error: {}", e)))?;

    Ok(XdlValue::Undefined)
}

/// Helper function to write a value as binary
fn write_binary_value<W: Write>(writer: &mut W, value: &XdlValue) -> XdlResult<()> {
    match value {
        XdlValue::Byte(v) => {
            writer
                .write_all(&[*v])
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Int(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Long(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Float(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Double(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::UInt(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::ULong(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Long64(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::ULong64(v) => {
            writer
                .write_all(&v.to_le_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        XdlValue::Array(arr) => {
            // Write array as sequence of doubles (f64)
            for &val in arr {
                writer
                    .write_all(&val.to_le_bytes())
                    .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
            }
        }
        XdlValue::MultiDimArray { data, .. } => {
            // Write multi-dimensional array as flat sequence of doubles
            for &val in data {
                writer
                    .write_all(&val.to_le_bytes())
                    .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
            }
        }
        XdlValue::String(s) => {
            // Write string as bytes (no length prefix)
            writer
                .write_all(s.as_bytes())
                .map_err(|e| XdlError::RuntimeError(format!("WRITEU: Write error: {}", e)))?;
        }
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "WRITEU: Unsupported type: {:?}",
                value.gdl_type()
            )));
        }
    }
    Ok(())
}

/// READU - Read unformatted (binary) data from file
/// READU, lun, variable
pub fn readu(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::io::Read;

    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "READU: Expected LUN and variable arguments".to_string(),
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
        .ok_or_else(|| XdlError::RuntimeError(format!("READU: LUN {} not open", lun)))?;

    // Get the reader
    let reader = handle.reader.as_mut().ok_or_else(|| {
        XdlError::RuntimeError(format!("READU: LUN {} not open for reading", lun))
    })?;

    // For the second argument, we need to know what type to read
    // In XDL, this is typically determined by the variable type that was passed
    // For now, we'll read based on the type of the template variable
    let template = &args[1];

    let result = match template {
        XdlValue::Byte(_) => {
            let mut buf = [0u8; 1];
            reader
                .read_exact(&mut buf)
                .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
            XdlValue::Byte(buf[0])
        }
        XdlValue::Int(_) => {
            let mut buf = [0u8; 2];
            reader
                .read_exact(&mut buf)
                .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
            XdlValue::Int(i16::from_le_bytes(buf))
        }
        XdlValue::Long(_) => {
            let mut buf = [0u8; 4];
            reader
                .read_exact(&mut buf)
                .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
            XdlValue::Long(i32::from_le_bytes(buf))
        }
        XdlValue::Float(_) => {
            let mut buf = [0u8; 4];
            reader
                .read_exact(&mut buf)
                .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
            XdlValue::Float(f32::from_le_bytes(buf))
        }
        XdlValue::Double(_) => {
            let mut buf = [0u8; 8];
            reader
                .read_exact(&mut buf)
                .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
            XdlValue::Double(f64::from_le_bytes(buf))
        }
        XdlValue::Array(arr) => {
            // Read array of doubles
            let n = arr.len();
            let mut result = Vec::with_capacity(n);
            for _ in 0..n {
                let mut buf = [0u8; 8];
                reader
                    .read_exact(&mut buf)
                    .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
                result.push(f64::from_le_bytes(buf));
            }
            XdlValue::Array(result)
        }
        XdlValue::MultiDimArray { data, shape } => {
            // Read multi-dimensional array
            let n = data.len();
            let mut result = Vec::with_capacity(n);
            for _ in 0..n {
                let mut buf = [0u8; 8];
                reader
                    .read_exact(&mut buf)
                    .map_err(|e| XdlError::RuntimeError(format!("READU: Read error: {}", e)))?;
                result.push(f64::from_le_bytes(buf));
            }
            XdlValue::MultiDimArray {
                data: result,
                shape: shape.clone(),
            }
        }
        _ => {
            return Err(XdlError::RuntimeError(format!(
                "READU: Unsupported type: {:?}",
                template.gdl_type()
            )));
        }
    };

    Ok(result)
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

/// FILE_TEST - Test file existence and properties
/// FILE_TEST(filename [, /READ, /WRITE, /DIRECTORY, /REGULAR])
/// Returns 1 if test passes, 0 otherwise
pub fn file_test(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_TEST: Expected filename".to_string(),
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

    let path = Path::new(filename);

    // For now, simple existence check
    // Keywords like /READ, /WRITE would be passed via keywords parameter
    let exists = path.exists();

    Ok(XdlValue::Long(if exists { 1 } else { 0 }))
}

/// FILE_LINES - Count lines in a text file
/// FILE_LINES(filename)
/// Returns number of lines in the file
pub fn file_lines(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_LINES: Expected filename".to_string(),
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

    let file = File::open(filename)
        .map_err(|e| XdlError::RuntimeError(format!("FILE_LINES: Cannot open file: {}", e)))?;

    let reader = BufReader::new(file);
    let line_count = reader.lines().count();

    Ok(XdlValue::Long(line_count as i32))
}

/// FILE_INFO - Get file information
/// FILE_INFO(filename)
/// Returns structure with file metadata
pub fn file_info(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_INFO: Expected filename".to_string(),
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

    let path = Path::new(filename);

    if !path.exists() {
        return Err(XdlError::RuntimeError(format!(
            "FILE_INFO: File not found: {}",
            filename
        )));
    }

    let metadata = std::fs::metadata(path)
        .map_err(|e| XdlError::RuntimeError(format!("FILE_INFO: Cannot read metadata: {}", e)))?;

    // Return array with: [exists, size, is_directory, is_regular]
    let result = vec![
        XdlValue::Long(1),                                      // exists
        XdlValue::Long(metadata.len() as i32),                  // size
        XdlValue::Long(if metadata.is_dir() { 1 } else { 0 }),  // is_directory
        XdlValue::Long(if metadata.is_file() { 1 } else { 0 }), // is_regular
    ];

    Ok(XdlValue::NestedArray(result))
}

/// EOF - Test for end of file
/// EOF(lun)
/// Returns 1 if at end of file, 0 otherwise
pub fn eof_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "EOF: Expected logical unit number".to_string(),
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

    // Check if LUN is valid and at EOF
    let file_handles = FILE_HANDLES.lock().unwrap();

    if let Some(_handle) = file_handles.get(&lun) {
        // For now, placeholder: would need to track file position
        // Real implementation would check if current position == file size
        Ok(XdlValue::Long(0))
    } else {
        Err(XdlError::RuntimeError(format!(
            "EOF: Invalid logical unit number: {}",
            lun
        )))
    }
}

/// FLUSH - Flush file buffer to disk
/// FLUSH, lun
/// Ensures all buffered data is written to file
pub fn flush_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FLUSH: Expected logical unit number".to_string(),
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

    let mut file_handles = FILE_HANDLES.lock().unwrap();

    if let Some(handle) = file_handles.get_mut(&lun) {
        // Flush writer if present
        if let Some(writer) = &mut handle.writer {
            writer.flush().map_err(|e| {
                XdlError::RuntimeError(format!("FLUSH: Error flushing file: {}", e))
            })?;
        }
        Ok(XdlValue::Undefined)
    } else {
        Err(XdlError::RuntimeError(format!(
            "FLUSH: Invalid logical unit number: {}",
            lun
        )))
    }
}

/// POINT_LUN - Position file pointer
/// POINT_LUN, lun, position
/// Sets the file pointer to specified byte position
pub fn point_lun(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "POINT_LUN: Expected LUN and position".to_string(),
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

    let _position = match &args[1] {
        XdlValue::Long(n) => *n as u64,
        XdlValue::Int(n) => *n as u64,
        XdlValue::Double(n) => *n as u64,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "numeric".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    let mut file_handles = FILE_HANDLES.lock().unwrap();

    if let Some(_handle) = file_handles.get_mut(&lun) {
        // POINT_LUN functionality requires access to underlying File
        // Current FileHandle structure with BufReader/BufWriter doesn't expose seek
        // For now, return placeholder success
        // Full implementation would need refactored FileHandle to support seek
        Ok(XdlValue::Undefined)
    } else {
        Err(XdlError::RuntimeError(format!(
            "POINT_LUN: Invalid logical unit number: {}",
            lun
        )))
    }
}

/// ASSOC - Associate array with file
/// ASSOC(lun, array_template)
/// Creates a file association for binary I/O
/// Returns a special value that can be used for direct array access
pub fn assoc(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "ASSOC: Expected LUN and array template".to_string(),
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

    // Verify LUN is valid
    let file_handles = FILE_HANDLES.lock().unwrap();
    if !file_handles.contains_key(&lun) {
        return Err(XdlError::RuntimeError(format!(
            "ASSOC: Invalid logical unit number: {}",
            lun
        )));
    }

    // For now, return a placeholder value representing the association
    // Full implementation would create a special structure
    // that tracks the LUN and array template for indexed access
    Ok(XdlValue::Long(lun))
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
            XdlValue::Double(std::f64::consts::PI),
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
