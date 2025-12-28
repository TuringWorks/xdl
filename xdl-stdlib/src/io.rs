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

// ============================================================
// File System Functions
// ============================================================

/// FILE_TEST - Test file attributes
/// FILE_TEST(path [, /DIRECTORY] [, /READ] [, /WRITE] [, /EXECUTABLE])
/// Returns 1 if file exists (or matches criteria), 0 otherwise
pub fn file_test(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_TEST: Expected path argument".to_string(),
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

    let path = Path::new(path);

    // Check basic existence
    if !path.exists() {
        return Ok(XdlValue::Long(0));
    }

    // Check optional flags (simplified - check if second arg is non-zero for directory test)
    if args.len() > 1 {
        let check_directory = match &args[1] {
            XdlValue::Long(n) => *n != 0,
            XdlValue::Int(n) => *n != 0,
            _ => false,
        };

        if check_directory && !path.is_dir() {
            return Ok(XdlValue::Long(0));
        }
    }

    Ok(XdlValue::Long(1))
}

/// FILE_INFO - Get file information
/// Returns structure with file metadata
pub fn file_info(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::time::UNIX_EPOCH;

    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_INFO: Expected path argument".to_string(),
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

    let path = Path::new(path);

    if !path.exists() {
        return Ok(XdlValue::Struct(std::collections::HashMap::new()));
    }

    let metadata = std::fs::metadata(path).map_err(|e| {
        XdlError::RuntimeError(format!("FILE_INFO: Cannot get metadata: {}", e))
    })?;

    let mut info = std::collections::HashMap::new();

    // File size
    info.insert("SIZE".to_string(), XdlValue::Long64(metadata.len() as i64));

    // Is directory
    info.insert(
        "DIRECTORY".to_string(),
        XdlValue::Long(if metadata.is_dir() { 1 } else { 0 }),
    );

    // Is regular file
    info.insert(
        "REGULAR".to_string(),
        XdlValue::Long(if metadata.is_file() { 1 } else { 0 }),
    );

    // Modification time (seconds since epoch)
    if let Ok(mtime) = metadata.modified() {
        if let Ok(duration) = mtime.duration_since(UNIX_EPOCH) {
            info.insert("MTIME".to_string(), XdlValue::Double(duration.as_secs_f64()));
        }
    }

    // Permissions (read/write)
    let perms = metadata.permissions();
    info.insert(
        "READONLY".to_string(),
        XdlValue::Long(if perms.readonly() { 1 } else { 0 }),
    );

    Ok(XdlValue::Struct(info))
}

/// FILE_SEARCH - Search for files matching a pattern
/// FILE_SEARCH(pattern [, path])
pub fn file_search(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_SEARCH: Expected pattern argument".to_string(),
        ));
    }

    let pattern = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let search_path = if args.len() > 1 {
        match &args[1] {
            XdlValue::String(s) => s.clone(),
            _ => ".".to_string(),
        }
    } else {
        ".".to_string()
    };

    // Simple glob implementation
    let mut matches = Vec::new();
    let search_dir = Path::new(&search_path);

    if let Ok(entries) = std::fs::read_dir(search_dir) {
        for entry in entries.flatten() {
            let filename = entry.file_name().to_string_lossy().to_string();
            // Simple wildcard matching
            if simple_glob_match(pattern, &filename) {
                matches.push(XdlValue::String(
                    entry.path().to_string_lossy().to_string(),
                ));
            }
        }
    }

    Ok(XdlValue::NestedArray(matches))
}

// Simple glob matching (supports * and ?)
fn simple_glob_match(pattern: &str, text: &str) -> bool {
    let mut p_chars = pattern.chars().peekable();
    let mut t_chars = text.chars().peekable();

    while let Some(pc) = p_chars.next() {
        match pc {
            '*' => {
                // Match any sequence
                if p_chars.peek().is_none() {
                    return true; // * at end matches everything
                }
                // Try matching rest of pattern at each position
                let rest_pattern: String = p_chars.collect();
                let rest_text: String = t_chars.collect();
                for i in 0..=rest_text.len() {
                    if simple_glob_match(&rest_pattern, &rest_text[i..]) {
                        return true;
                    }
                }
                return false;
            }
            '?' => {
                // Match any single character
                if t_chars.next().is_none() {
                    return false;
                }
            }
            c => {
                // Match literal character
                if t_chars.next() != Some(c) {
                    return false;
                }
            }
        }
    }

    t_chars.next().is_none()
}

/// FILE_MKDIR - Create directory
pub fn file_mkdir(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_MKDIR: Expected path argument".to_string(),
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

    std::fs::create_dir_all(path).map_err(|e| {
        XdlError::RuntimeError(format!("FILE_MKDIR: Failed to create {}: {}", path, e))
    })?;

    Ok(XdlValue::Long(1))
}

/// FILE_DELETE - Delete file or directory
pub fn file_delete(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FILE_DELETE: Expected path argument".to_string(),
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

    let path_obj = Path::new(path);

    if path_obj.is_dir() {
        std::fs::remove_dir_all(path).map_err(|e| {
            XdlError::RuntimeError(format!("FILE_DELETE: Failed to delete {}: {}", path, e))
        })?;
    } else {
        std::fs::remove_file(path).map_err(|e| {
            XdlError::RuntimeError(format!("FILE_DELETE: Failed to delete {}: {}", path, e))
        })?;
    }

    Ok(XdlValue::Long(1))
}

/// FILE_COPY - Copy file
pub fn file_copy(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "FILE_COPY: Expected source and destination arguments".to_string(),
        ));
    }

    let source = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let dest = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    std::fs::copy(source, dest).map_err(|e| {
        XdlError::RuntimeError(format!("FILE_COPY: Failed to copy {} to {}: {}", source, dest, e))
    })?;

    Ok(XdlValue::Long(1))
}

/// FILE_MOVE - Move/rename file
pub fn file_move(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "FILE_MOVE: Expected source and destination arguments".to_string(),
        ));
    }

    let source = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let dest = match &args[1] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[1].gdl_type()),
            })
        }
    };

    std::fs::rename(source, dest).map_err(|e| {
        XdlError::RuntimeError(format!("FILE_MOVE: Failed to move {} to {}: {}", source, dest, e))
    })?;

    Ok(XdlValue::Long(1))
}

/// EOF - Check for end of file
/// Returns 1 if at EOF, 0 otherwise
pub fn eof_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    use std::io::BufRead;

    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "EOF: Expected LUN argument".to_string(),
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
        .ok_or_else(|| XdlError::RuntimeError(format!("EOF: LUN {} not open", lun)))?;

    // Get the reader
    let reader = handle.reader.as_mut().ok_or_else(|| {
        XdlError::RuntimeError(format!("EOF: LUN {} not open for reading", lun))
    })?;

    // Check if at EOF by peeking at buffer
    let at_eof = reader.fill_buf().map_or(true, |buf| buf.is_empty());

    Ok(XdlValue::Long(if at_eof { 1 } else { 0 }))
}

/// FLUSH - Flush file buffer
pub fn flush_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "FLUSH: Expected LUN argument".to_string(),
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
        .ok_or_else(|| XdlError::RuntimeError(format!("FLUSH: LUN {} not open", lun)))?;

    // Get the writer
    if let Some(writer) = handle.writer.as_mut() {
        writer.flush().map_err(|e| {
            XdlError::RuntimeError(format!("FLUSH: Failed to flush: {}", e))
        })?;
    }

    Ok(XdlValue::Undefined)
}

/// CD - Change current directory
pub fn cd_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        // Return current directory
        let cwd = std::env::current_dir()
            .map_err(|e| XdlError::RuntimeError(format!("CD: Cannot get current dir: {}", e)))?;
        return Ok(XdlValue::String(cwd.to_string_lossy().to_string()));
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

    std::env::set_current_dir(path)
        .map_err(|e| XdlError::RuntimeError(format!("CD: Cannot change to {}: {}", path, e)))?;

    Ok(XdlValue::Undefined)
}

/// GETENV - Get environment variable
pub fn getenv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "GETENV: Expected variable name argument".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    match std::env::var(name) {
        Ok(value) => Ok(XdlValue::String(value)),
        Err(_) => Ok(XdlValue::String("".to_string())),
    }
}

/// SETENV - Set environment variable
pub fn setenv(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.len() < 2 {
        return Err(XdlError::InvalidArgument(
            "SETENV: Expected variable name and value arguments".to_string(),
        ));
    }

    let name = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let value = match &args[1] {
        XdlValue::String(s) => s.clone(),
        other => other.to_string_repr(),
    };

    std::env::set_var(name, value);

    Ok(XdlValue::Undefined)
}

/// SPAWN - Execute external command
pub fn spawn_func(args: &[XdlValue]) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "SPAWN: Expected command argument".to_string(),
        ));
    }

    let cmd = match &args[0] {
        XdlValue::String(s) => s,
        _ => {
            return Err(XdlError::TypeMismatch {
                expected: "string".to_string(),
                actual: format!("{:?}", args[0].gdl_type()),
            })
        }
    };

    let output = std::process::Command::new(if cfg!(target_os = "windows") { "cmd" } else { "sh" })
        .arg(if cfg!(target_os = "windows") { "/C" } else { "-c" })
        .arg(cmd)
        .output()
        .map_err(|e| XdlError::RuntimeError(format!("SPAWN: Failed to execute: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    // Return combined output
    let result = if stderr.is_empty() {
        stdout
    } else if stdout.is_empty() {
        stderr
    } else {
        format!("{}\n{}", stdout, stderr)
    };

    Ok(XdlValue::String(result.trim().to_string()))
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
