//! # XDL REPL (Read-Eval-Print Loop)
//!
//! Interactive command-line interface for XDL.

use anyhow::Result;
use rustyline::{error::ReadlineError, Editor};
use tracing::{error, info};
use xdl_core::XdlValue;
use xdl_interpreter::Interpreter;

pub fn start_repl() -> Result<()> {
    info!("Starting XDL REPL");

    let mut rl = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    let mut interpreter = Interpreter::new();

    // Load history if it exists
    let history_file = dirs::home_dir().unwrap_or_default().join(".gdl_history");
    let _ = rl.load_history(&history_file);

    println!("XDL Interactive Session");
    println!("Type '.help' for help, '.quit' to exit");

    loop {
        let readline = rl.readline("XDL> ");

        match readline {
            Ok(line) => {
                let trimmed = line.trim();

                if trimmed.is_empty() {
                    continue;
                }

                let _ = rl.add_history_entry(line.as_str());

                // Handle special REPL commands
                match trimmed {
                    ".quit" | ".exit" => {
                        println!("Goodbye!");
                        break;
                    }
                    ".help" => {
                        print_help();
                        continue;
                    }
                    ".version" => {
                        println!("XDL Rust Implementation v{}", env!("CARGO_PKG_VERSION"));
                        continue;
                    }
                    _ => {}
                }

                // Try to parse and execute the XDL command
                match execute_repl_command(trimmed, &mut interpreter) {
                    Ok(result) => {
                        if let Some(output) = result {
                            println!("{}", output);
                        }
                    }
                    Err(e) => {
                        error!("Error: {}", e);
                    }
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("CTRL-D");
                break;
            }
            Err(err) => {
                error!("Error: {:?}", err);
                break;
            }
        }
    }

    // Save history
    let _ = rl.save_history(&history_file);

    Ok(())
}

fn print_help() {
    println!("XDL REPL Commands:");
    println!("  .help     - Show this help message");
    println!("  .version  - Show version information");
    println!("  .quit     - Exit the REPL");
    println!("  .exit     - Exit the REPL");
    println!();
    println!("Enter any XDL expression or statement to execute it.");
    println!("Examples:");
    println!("  print, 'Hello, World!'");
    println!("  x = 42");
    println!("  y = sin(findgen(100) * !pi / 50)");
}

fn execute_repl_command(command: &str, interpreter: &mut Interpreter) -> Result<Option<String>> {
    // Try to parse as a statement first, then as an expression
    match xdl_parser::parse_xdl(command) {
        Ok(program) => {
            interpreter.execute_program(&program)?;
            Ok(None) // Program execution handles its own output
        }
        Err(_) => {
            // Try parsing as expression
            match xdl_parser::parse_expression(command) {
                Ok(expr) => {
                    let result = interpreter.evaluate_expression(&expr)?;

                    // Return result for display, unless it's undefined
                    match result {
                        XdlValue::Undefined => Ok(None),
                        _ => Ok(Some(format_xdl_value(&result))),
                    }
                }
                Err(e) => Err(anyhow::anyhow!("Parse error: {}", e)),
            }
        }
    }
}

/// Format a XdlValue for display
fn format_xdl_value(value: &XdlValue) -> String {
    value.to_string_repr()
}
