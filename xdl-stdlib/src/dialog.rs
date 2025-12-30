//! Dialog functions for user interaction
//!
//! These functions provide dialog boxes for user input and file selection.
//! In CLI mode, they use terminal-based prompts as fallbacks.

use std::io::{self, Write};
use xdl_core::{XdlError, XdlResult, XdlValue};

/// DIALOG_MESSAGE - Display a message dialog
/// IDL syntax: result = DIALOG_MESSAGE(message [, /ERROR] [, /INFORMATION] [, /QUESTION] [, TITLE=title])
/// Returns 1 for OK, 0 for Cancel (for question dialogs)
pub fn dialog_message(args: &[XdlValue], keywords: &std::collections::HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    if args.is_empty() {
        return Err(XdlError::InvalidArgument(
            "DIALOG_MESSAGE: Expected message argument".to_string(),
        ));
    }

    let message = match &args[0] {
        XdlValue::String(s) => s.clone(),
        _ => args[0].to_string_repr(),
    };

    // Get optional title
    let title = keywords
        .get("TITLE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Message".to_string());

    // Check dialog type keywords
    let is_error = keywords.contains_key("ERROR");
    let is_question = keywords.contains_key("QUESTION");
    let is_info = keywords.contains_key("INFORMATION") || keywords.contains_key("INFO");

    // Determine dialog type prefix
    let prefix = if is_error {
        "[ERROR]"
    } else if is_question {
        "[QUESTION]"
    } else if is_info {
        "[INFO]"
    } else {
        "[MESSAGE]"
    };

    // In CLI mode, print to stdout and wait for input if it's a question
    println!("\n{} {}", prefix, title);
    println!("{}", "=".repeat(title.len() + prefix.len() + 1));
    println!("{}", message);

    if is_question {
        print!("\nPress Y for Yes, N for No: ");
        io::stdout().flush().ok();

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_ok() {
            let response = input.trim().to_uppercase();
            if response.starts_with('Y') {
                return Ok(XdlValue::Long(1)); // Yes
            }
        }
        Ok(XdlValue::Long(0)) // No or Cancel
    } else {
        println!("\n[Press Enter to continue]");
        let mut input = String::new();
        io::stdin().read_line(&mut input).ok();
        Ok(XdlValue::Long(1)) // OK
    }
}

/// DIALOG_PICKFILE - Display a file picker dialog
/// IDL syntax: result = DIALOG_PICKFILE([DEFAULT_EXTENSION=ext] [, FILTER=filter] [, /DIRECTORY] [, /WRITE] [, TITLE=title])
/// Returns the selected file path as a string, or empty string if cancelled
pub fn dialog_pickfile(args: &[XdlValue], keywords: &std::collections::HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    // Get optional parameters
    let title = keywords
        .get("TITLE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Select File".to_string());

    let default_ext = keywords.get("DEFAULT_EXTENSION").and_then(|v| match v {
        XdlValue::String(s) => Some(s.clone()),
        _ => None,
    });

    let filter = keywords.get("FILTER").and_then(|v| match v {
        XdlValue::String(s) => Some(s.clone()),
        _ => None,
    });

    let is_directory = keywords.contains_key("DIRECTORY");
    let is_write = keywords.contains_key("WRITE");

    // Get path from first argument if provided
    let default_path = if !args.is_empty() {
        match &args[0] {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        }
    } else {
        keywords.get("PATH").and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
    };

    // In CLI mode, prompt for file path
    println!("\n{}", title);
    println!("{}", "=".repeat(title.len()));

    if is_directory {
        println!("Enter directory path:");
    } else if is_write {
        println!("Enter file path to save:");
    } else {
        println!("Enter file path to open:");
    }

    if let Some(path) = &default_path {
        println!("(Default: {})", path);
    }
    if let Some(ext) = &default_ext {
        println!("(Default extension: {})", ext);
    }
    if let Some(f) = &filter {
        println!("(Filter: {})", f);
    }

    print!("> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let path = input.trim();
        if path.is_empty() {
            // Return default path or empty string
            if let Some(default) = default_path {
                return Ok(XdlValue::String(default));
            }
            return Ok(XdlValue::String(String::new()));
        }

        // Add default extension if needed
        let mut result_path = path.to_string();
        if let Some(ext) = default_ext {
            if !result_path.contains('.') {
                result_path = format!("{}.{}", result_path, ext);
            }
        }

        return Ok(XdlValue::String(result_path));
    }

    Ok(XdlValue::String(String::new()))
}

/// DIALOG_PRINTERSETUP - Display printer setup dialog (placeholder)
/// In CLI mode, this just returns default printer info
pub fn dialog_printersetup(_args: &[XdlValue], _keywords: &std::collections::HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    // In CLI mode, return a placeholder indicating no printer setup available
    println!("\n[PRINTER SETUP]");
    println!("Printer setup dialog not available in CLI mode.");
    println!("Default: System default printer");

    // Return a struct-like value with printer info
    Ok(XdlValue::String("default".to_string()))
}

/// DIALOG_READ_TEXT - Read text input from user via dialog
/// IDL syntax: result = DIALOG_READ_TEXT([prompt] [, TITLE=title] [, DEFAULT=default])
pub fn dialog_read_text(args: &[XdlValue], keywords: &std::collections::HashMap<String, XdlValue>) -> XdlResult<XdlValue> {
    let prompt = if !args.is_empty() {
        match &args[0] {
            XdlValue::String(s) => s.clone(),
            _ => "Enter text:".to_string(),
        }
    } else {
        "Enter text:".to_string()
    };

    let title = keywords
        .get("TITLE")
        .and_then(|v| match v {
            XdlValue::String(s) => Some(s.clone()),
            _ => None,
        })
        .unwrap_or_else(|| "Input".to_string());

    let default = keywords.get("DEFAULT").and_then(|v| match v {
        XdlValue::String(s) => Some(s.clone()),
        _ => None,
    });

    println!("\n{}", title);
    println!("{}", "=".repeat(title.len()));
    println!("{}", prompt);
    if let Some(def) = &default {
        println!("(Default: {})", def);
    }

    print!("> ");
    io::stdout().flush().ok();

    let mut input = String::new();
    if io::stdin().read_line(&mut input).is_ok() {
        let text = input.trim();
        if text.is_empty() {
            if let Some(def) = default {
                return Ok(XdlValue::String(def));
            }
        }
        return Ok(XdlValue::String(text.to_string()));
    }

    Ok(XdlValue::String(default.unwrap_or_default()))
}
