//! # XDL CLI Application
//!
//! Command-line interface for the Extended Data Language (XDL) Rust implementation.

use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tracing::{error, info};
use xdl_artifacts::{
    CompatibilityEntry, CompatibilityInventory, CompatibilityStatus, RunManifest, RunStatus,
    TestSummary,
};
use xdl_core::XdlValue;
use xdl_interpreter::Interpreter;
use xdl_matlab::transpile_matlab_to_xdl;

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

    /// Write visualizations to HTML files in the working directory instead of opening windows
    #[arg(long)]
    headless: bool,

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
        /// Write a truthful run summary manifest to this path
        #[arg(long)]
        manifest: Option<PathBuf>,
    },
    /// Show the current machine-readable compatibility baseline
    CompatibilityReport {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Validate and normalize a versioned XDL model document
    ModelCheck {
        /// JSON model document
        file: PathBuf,
        /// Rewrite the validated document as canonical current-version JSON
        #[arg(long)]
        migrate: bool,
        /// Render the validated document as SVG
        #[arg(long)]
        svg: bool,
    },
    /// Show version information
    Version,
}

fn main() {
    let cli = Cli::parse();
    xdl_core::headless::set_headless(cli.headless);

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
        Some(Commands::Test {
            directory,
            manifest,
        }) => run_tests(&directory, manifest.as_deref()),
        Some(Commands::CompatibilityReport { format }) => compatibility_report(&format),
        Some(Commands::ModelCheck { file, migrate, svg }) => check_model_file(&file, migrate, svg),
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

fn run_tests(directory: &Path, manifest_path: Option<&Path>) -> Result<()> {
    info!("Running tests in directory: {}", directory.display());

    let mut files = Vec::new();
    collect_xdl_files(directory, &mut files)
        .with_context(|| format!("Failed to discover tests in {}", directory.display()))?;
    files.sort();

    if files.is_empty() {
        return Err(anyhow::anyhow!(
            "no .xdl test files found in {}",
            directory.display()
        ));
    }

    let mut failures = Vec::new();
    for file in &files {
        match execute_file(file) {
            Ok(()) => println!("✓ {}", file.display()),
            Err(error) => {
                println!("✗ {}: {error:#}", file.display());
                failures.push((file.display().to_string(), error.to_string()));
            }
        }
    }

    println!(
        "\nTest summary: {} passed, {} failed",
        files.len() - failures.len(),
        failures.len()
    );

    let status = if failures.is_empty() {
        RunStatus::Passed
    } else {
        RunStatus::Failed
    };

    if let Some(path) = manifest_path {
        write_test_manifest(path, directory, status, files.len(), failures.len())?;
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{} XDL test(s) failed", failures.len()))
    }
}

fn write_test_manifest(
    path: &Path,
    directory: &Path,
    status: RunStatus,
    discovered: usize,
    failed: usize,
) -> Result<()> {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow::anyhow!("system clock is before Unix epoch: {error}"))?;
    let run_id = format!("xdl-test-{}-{}", process::id(), now.as_nanos());
    let mut manifest = RunManifest::new(run_id, "xdl", &directory.display().to_string());
    manifest.status = status;
    manifest.started_at = Some(format!("unix:{}", now.as_secs()));
    manifest.configuration.insert(
        "test_directory".to_string(),
        directory.display().to_string(),
    );
    manifest.configuration.insert(
        "runner".to_string(),
        "recursive-xdl-script-runner".to_string(),
    );
    manifest.environment.insert(
        "xdl_cli_version".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    manifest.test_summary = Some(TestSummary {
        discovered,
        passed: discovered - failed,
        failed,
    });
    manifest.warnings.push(
        "This manifest contains test counts only; file hashes, output artifacts, timeouts, and numerical evidence are not collected by this runner.".to_string(),
    );

    let json = manifest
        .to_json_pretty()
        .with_context(|| "failed to validate test run manifest")?;
    fs::write(path, json)
        .with_context(|| format!("failed to write test run manifest: {}", path.display()))?;
    println!("Manifest: {}", path.display());
    Ok(())
}

fn collect_xdl_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(directory)
        .with_context(|| format!("Failed to read test directory: {}", directory.display()))?;

    for entry in entries {
        let entry = entry.with_context(|| {
            format!("Failed to inspect test directory: {}", directory.display())
        })?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("Failed to inspect {}", path.display()))?;

        if file_type.is_dir() {
            collect_xdl_files(&path, files)?;
        } else if file_type.is_file()
            && path.extension().and_then(|ext| ext.to_str()) == Some("xdl")
        {
            files.push(path);
        }
    }

    Ok(())
}

fn compatibility_report(format: &str) -> Result<()> {
    let inventory = baseline_compatibility_inventory();
    match format {
        "json" => println!(
            "{}",
            inventory
                .to_json_pretty()
                .with_context(|| "failed to serialize compatibility inventory")?
        ),
        "text" => {
            println!(
                "Compatibility baseline (schema {}, generated by {})",
                inventory.schema_version, inventory.generated_by
            );
            for entry in inventory.entries {
                println!(
                    "- {:<12} {:<10} {}",
                    compatibility_status_name(entry.status),
                    entry.surface,
                    entry.capability
                );
            }
            println!("\nThis is a curated baseline, not a generated conformance percentage.");
        }
        other => {
            return Err(anyhow::anyhow!(
                "unsupported compatibility report format '{other}'; expected text or json"
            ));
        }
    }
    Ok(())
}

fn baseline_compatibility_inventory() -> CompatibilityInventory {
    CompatibilityInventory::new(
        "xdl-cli-baseline-2026-11-12",
        vec![
            compatibility_entry(
                "matlab.array-literals",
                "matlab",
                "array literals and basic matrix literals",
                CompatibilityStatus::Partial,
                "xdl-matlab focused tests",
                "Advanced indexing, broadcasting, and object semantics remain incomplete.",
            ),
            compatibility_entry(
                "matlab.control-flow",
                "matlab",
                "if/for/while/switch/try-catch control flow",
                CompatibilityStatus::Partial,
                "xdl-matlab focused tests",
                "The tested syntax subset is not a complete MATLAB error-semantics contract.",
            ),
            compatibility_entry(
                "octave.core-language",
                "octave",
                "GNU Octave language and package compatibility",
                CompatibilityStatus::Unsupported,
                "",
                "No Octave conformance corpus or runtime bridge is currently wired.",
            ),
            compatibility_entry(
                "scientific-io.formats",
                "xdl",
                "HDF5, NetCDF, FITS, MAT, Arrow, Parquet, and Zarr workflows",
                CompatibilityStatus::Partial,
                "repository feature flags and I/O tests",
                "Some paths remain feature-gated or placeholder-based.",
            ),
            compatibility_entry(
                "graphics.plotting",
                "xdl",
                "2D/3D plotting and chart backends",
                CompatibilityStatus::Partial,
                "graphics and chart tests",
                "Plot type, keyword, platform, and backend coverage varies.",
            ),
            compatibility_entry(
                "gpu.backends",
                "xdl",
                "accelerated numerical execution",
                CompatibilityStatus::Experimental,
                "xdl-amp backend tests",
                "Availability and operation coverage vary; CPU fallback must be reported.",
            ),
            compatibility_entry(
                "model.graph",
                "simulink-xcos",
                "typed block/signal model graph",
                CompatibilityStatus::Partial,
                "xdl-model tests",
                "Schema and validation foundations exist; full editor/library parity does not.",
            ),
            compatibility_entry(
                "simulation.scheduler",
                "simulink-xcos",
                "deterministic multi-rate and solver foundation",
                CompatibilityStatus::Partial,
                "xdl-sim tests",
                "Continuous, hybrid, DAE, event, and physical-model breadth remains incomplete.",
            ),
            compatibility_entry(
                "run-manifest.contract",
                "xdl",
                "versioned run summary manifest",
                CompatibilityStatus::Verified,
                "xdl-artifacts tests and xdl test --manifest",
                "Hashes, output capture, timeouts, and replay are not yet implemented.",
            ),
            compatibility_entry(
                "vibecoder.scientific-workspace",
                "vibecoder",
                "native variable, plot, model, and artifact panels",
                CompatibilityStatus::Unsupported,
                "",
                "The cross-application protocol is not yet connected in this repository.",
            ),
        ],
    )
}

fn compatibility_entry(
    id: &str,
    surface: &str,
    capability: &str,
    status: CompatibilityStatus,
    evidence: &str,
    notes: &str,
) -> CompatibilityEntry {
    CompatibilityEntry {
        id: id.to_string(),
        surface: surface.to_string(),
        capability: capability.to_string(),
        status,
        evidence: (!evidence.is_empty())
            .then(|| evidence.to_string())
            .into_iter()
            .collect(),
        notes: Some(notes.to_string()),
    }
}

fn compatibility_status_name(status: CompatibilityStatus) -> &'static str {
    match status {
        CompatibilityStatus::Verified => "verified",
        CompatibilityStatus::Partial => "partial",
        CompatibilityStatus::Experimental => "experimental",
        CompatibilityStatus::Delegated => "delegated",
        CompatibilityStatus::Unsupported => "unsupported",
    }
}

fn check_model_file(file: &Path, migrate: bool, svg: bool) -> Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read model document: {}", file.display()))?;
    let document = xdl_model::ModelDocument::from_json(&content)
        .with_context(|| format!("Invalid model document: {}", file.display()))?;
    if svg {
        println!("{}", document.to_svg()?);
    } else if migrate {
        println!("{}", document.to_json()?);
    } else {
        println!(
            "✓ Model OK: {} blocks, {} connections, schema {}",
            document.model.blocks.len(),
            document.model.connections.len(),
            document.schema_version
        );
    }
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

    // Check if file is a MATLAB file and transpile it
    let xdl_code = if file.extension().and_then(|s| s.to_str()) == Some("m") {
        info!("Detected MATLAB file, transpiling...");
        transpile_matlab_to_xdl(&content)
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
