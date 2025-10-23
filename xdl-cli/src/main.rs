//! # XDL CLI Application
//!
//! Command-line interface for the Extended Data Language (XDL) Rust implementation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{error, info};
use xdl_core::XdlValue;
use xdl_interpreter::Interpreter;

mod repl;

/// Extended Data Language (XDL) - Rust Implementation
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input XDL file to execute
    #[arg(value_name = "FILE")]
    file: Option<PathBuf>,

    /// Execute XDL command directly
    #[arg(short = 'e', long, value_name = "COMMAND")]
    execute: Option<String>,

    /// Start interactive REPL
    #[arg(short, long)]
    interactive: bool,

    /// Enable verbose output
    #[arg(short, long)]
    verbose: bool,

    /// Disable startup message
    #[arg(short = 'q', long)]
    quiet: bool,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse XDL file and show AST
    Parse {
        /// Input file
        file: PathBuf,
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Check XDL syntax
    Check {
        /// Input file
        file: PathBuf,
    },
    /// Run XDL tests
    Test {
        /// Test directory
        #[arg(default_value = "tests")]
        directory: PathBuf,
    },
    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();

    // Initialize tracing
    let subscriber = tracing_subscriber::fmt().finish();
    tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // Show startup message unless quiet
    if !cli.quiet && cli.file.is_none() && cli.execute.is_none() && cli.command.is_none() {
        print_startup_message();
    }

    match run(cli) {
        Ok(()) => process::exit(0),
        Err(e) => {
            error!("Error: {:#}", e);
            process::exit(1);
        }
    };
}

fn run(cli: Cli) -> Result<()> {
    match cli.command {
        Some(Commands::Parse { file, format }) => parse_file(&file, &format),
        Some(Commands::Check { file }) => check_file(&file),
        Some(Commands::Test { directory }) => run_tests(&directory),
        Some(Commands::Version) => {
            print_version();
            Ok(())
        }
        None => {
            if let Some(command) = cli.execute {
                execute_command(&command)
            } else if let Some(file) = cli.file {
                execute_file(&file)
            } else {
                // Start interactive REPL
                repl::start_repl()
            }
        }
    }
}

fn print_startup_message() {
    println!(
        "  XDL - Extended Data Language, Rust Version {}",
        env!("CARGO_PKG_VERSION")
    );
    println!("- For basic information type HELP,/INFO");
    println!();
}

fn print_version() {
    println!("XDL Rust Implementation");
    println!("Version: {}", env!("CARGO_PKG_VERSION"));
    // println!("Built with Rust {}", std::env::var("RUSTC_VERSION").unwrap_or_else(|_| "unknown".to_string()));
}

fn parse_file(file: &Path, format: &str) -> Result<()> {
    info!("Parsing file: {}", file.display());

    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    let ast = xdl_parser::parse_xdl(&content).with_context(|| "Failed to parse XDL code")?;

    match format {
        "json" => {
            // TODO: Implement JSON serialization when serde derives are added
            println!("JSON output not yet supported");
        }
        _ => {
            println!("{:#?}", ast);
        }
    }

    Ok(())
}

fn check_file(file: &Path) -> Result<()> {
    info!("Checking syntax of file: {}", file.display());

    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    match xdl_parser::parse_xdl(&content) {
        Ok(_) => {
            println!("✓ Syntax OK");
            Ok(())
        }
        Err(e) => {
            println!("✗ Syntax Error: {}", e);
            process::exit(1);
        }
    }
}

fn run_tests(directory: &Path) -> Result<()> {
    info!("Running tests in directory: {}", directory.display());

    // TODO: Implement test runner
    println!("Test runner not yet implemented");
    Ok(())
}

fn execute_command(command: &str) -> Result<()> {
    info!("Executing command: {}", command);

    let mut interpreter = Interpreter::new();

    // Try to parse as a statement first, then as an expression
    match xdl_parser::parse_xdl(command) {
        Ok(program) => {
            interpreter
                .execute_program(&program)
                .with_context(|| "Failed to execute program")?;
        }
        Err(_) => {
            // Try parsing as expression
            match xdl_parser::parse_expression(command) {
                Ok(expr) => {
                    let result = interpreter
                        .evaluate_expression(&expr)
                        .with_context(|| "Failed to evaluate expression")?;

                    // Print result if it's not undefined
                    match result {
                        XdlValue::Undefined => {}
                        _ => {
                            println!("{}", format_xdl_value(&result));
                        }
                    }
                }
                Err(e) => {
                    return Err(anyhow::anyhow!("Parse error: {}", e));
                }
            }
        }
    }

    Ok(())
}

fn execute_file(file: &Path) -> Result<()> {
    info!("Executing file: {}", file.display());

    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read file: {}", file.display()))?;

    // Check if this is a MATLAB .m file
    let xdl_code = if file.extension().and_then(|s| s.to_str()) == Some("m") {
        info!("Detected MATLAB .m file, transpiling to XDL");
        xdl_matlab::transpile_matlab_to_xdl(&content)
            .map_err(|e| anyhow::anyhow!("Failed to transpile MATLAB code: {}", e))?
    } else {
        content
    };

    let program = xdl_parser::parse_xdl(&xdl_code).with_context(|| "Failed to parse XDL code")?;

    let mut interpreter = Interpreter::new();
    interpreter
        .execute_program(&program)
        .with_context(|| "Failed to execute program")?;

    Ok(())
}

/// Format a XdlValue for display
fn format_xdl_value(value: &XdlValue) -> String {
    value.to_string_repr()
}
