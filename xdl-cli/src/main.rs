//! # XDL CLI Application
//!
//! Command-line interface for the Extended Data Language (XDL) Rust implementation.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process;
use std::time::{Instant, SystemTime, UNIX_EPOCH};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use serde::Deserialize;
use sha2::{Digest, Sha256};
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
    /// Replay the test directory recorded in a run manifest
    Replay {
        /// JSON run manifest emitted by `xdl test --manifest`
        manifest: PathBuf,
    },
    /// Show the current machine-readable compatibility baseline
    CompatibilityReport {
        /// Output format (text, json)
        #[arg(short, long, default_value = "text")]
        format: String,
        /// Scan a directory of MATLAB `.m` files and report transpilation outcomes
        #[arg(long)]
        corpus: Option<PathBuf>,
        /// Also execute successfully transpiled files through the XDL interpreter
        #[arg(long)]
        execute: bool,
        /// Fail if any corpus case lacks a passing declared reference
        #[arg(long)]
        require_reference: bool,
    },
    /// Validate and normalize an XDL model graph or versioned model document
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
        Some(Commands::Replay { manifest }) => replay_manifest(&manifest),
        Some(Commands::CompatibilityReport {
            format,
            corpus,
            execute,
            require_reference,
        }) => compatibility_report(&format, corpus.as_deref(), execute, require_reference),
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
    run_tests_with_expectations(directory, manifest_path, None)
}

fn run_tests_with_expectations(
    directory: &Path,
    manifest_path: Option<&Path>,
    expected_stdout: Option<&BTreeMap<PathBuf, String>>,
) -> Result<()> {
    info!("Running tests in directory: {}", directory.display());
    let run_started = Instant::now();
    let run_started_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow::anyhow!("system clock is before Unix epoch: {error}"))?
        .as_secs();

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

    let capture_output = manifest_path.is_some() || expected_stdout.is_some();
    let mut failures = Vec::new();
    let mut successful_outputs = Vec::new();
    for file in &files {
        let result = if capture_output {
            execute_file_with_output(file)
        } else {
            execute_file(file).map(|()| String::new())
        };
        match result {
            Ok(output) => {
                if capture_output {
                    print!("{output}");
                }
                let output_mismatch = expected_stdout
                    .and_then(|expected| expected.get(file))
                    .filter(|expected| expected.trim_end() != output.trim_end())
                    .map(|expected| {
                        format!(
                            "stdout mismatch: expected {:?}, got {:?}",
                            expected.trim_end(),
                            output.trim_end()
                        )
                    });
                if let Some(error) = output_mismatch {
                    println!("✗ {}: {error}", file.display());
                    failures.push((file.display().to_string(), error));
                } else {
                    if manifest_path.is_some() {
                        successful_outputs.push((file.clone(), output));
                    }
                    println!("✓ {}", file.display());
                }
            }
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
        write_test_manifest(
            path,
            directory,
            TestManifestData {
                status,
                files: &files,
                successful_outputs: &successful_outputs,
                failures: &failures,
                started_at: run_started_at,
                duration_ms: run_started
                    .elapsed()
                    .as_millis()
                    .try_into()
                    .unwrap_or(u64::MAX),
            },
        )?;
    }

    if failures.is_empty() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{} XDL test(s) failed", failures.len()))
    }
}

fn replay_manifest(path: &Path) -> Result<()> {
    let json = fs::read_to_string(path)
        .with_context(|| format!("failed to read run manifest: {}", path.display()))?;
    let manifest = RunManifest::from_json(&json)
        .map_err(|error| anyhow::anyhow!("invalid run manifest: {error}"))?;
    let runner = manifest
        .configuration
        .get("runner")
        .map(String::as_str)
        .unwrap_or_default();
    if runner != "recursive-xdl-script-runner" {
        return Err(anyhow::anyhow!(
            "manifest runner '{runner}' is not replayable by xdl"
        ));
    }
    let recorded_directory = manifest
        .configuration
        .get("test_directory")
        .ok_or_else(|| anyhow::anyhow!("manifest does not record a test directory"))?;
    let recorded_directory = Path::new(recorded_directory);
    let directory = if recorded_directory.is_absolute() {
        recorded_directory.to_path_buf()
    } else {
        path.parent()
            .unwrap_or_else(|| Path::new("."))
            .join(recorded_directory)
    };
    if !directory.is_dir() && !directory.is_file() {
        return Err(anyhow::anyhow!(
            "recorded test directory does not exist: {}",
            directory.display()
        ));
    }
    let test_root = if directory.is_file() {
        directory.parent().unwrap_or_else(|| Path::new("."))
    } else {
        &directory
    };
    for artifact in &manifest.inputs {
        if artifact.kind != "xdl-test-source" {
            continue;
        }
        let recorded_path = Path::new(&artifact.path);
        let source = if recorded_path.is_absolute() {
            recorded_path.to_path_buf()
        } else {
            test_root.join(recorded_path)
        };
        let expected_hash = artifact.sha256.as_deref().ok_or_else(|| {
            anyhow::anyhow!("source artifact has no recorded sha256: {}", artifact.path)
        })?;
        let actual_hash = sha256_file(&source)?;
        if actual_hash != expected_hash {
            return Err(anyhow::anyhow!(
                "source hash mismatch for {}: expected {expected_hash}, got {actual_hash}",
                source.display()
            ));
        }
    }
    let manifest_root = path.parent().unwrap_or_else(|| Path::new("."));
    for artifact in &manifest.outputs {
        let recorded_path = Path::new(&artifact.path);
        let output = if recorded_path.is_absolute() {
            recorded_path.to_path_buf()
        } else {
            manifest_root.join(recorded_path)
        };
        let expected_hash = artifact.sha256.as_deref().ok_or_else(|| {
            anyhow::anyhow!("output artifact has no recorded sha256: {}", artifact.path)
        })?;
        let actual_hash = sha256_file(&output)?;
        if actual_hash != expected_hash {
            return Err(anyhow::anyhow!(
                "output artifact hash mismatch for {}: expected {expected_hash}, got {actual_hash}",
                output.display()
            ));
        }
    }
    let mut expected_stdout = BTreeMap::new();
    for artifact in &manifest.outputs {
        if artifact.kind != "xdl-test-stdout" {
            continue;
        }
        let artifact_path = Path::new(&artifact.path);
        let relative = artifact_path
            .strip_prefix("artifacts")
            .unwrap_or(artifact_path)
            .with_extension("xdl");
        let output_path = if artifact_path.is_absolute() {
            artifact_path.to_path_buf()
        } else {
            manifest_root.join(artifact_path)
        };
        let expected = fs::read_to_string(&output_path).with_context(|| {
            format!(
                "failed to read recorded stdout artifact: {}",
                output_path.display()
            )
        })?;
        expected_stdout.insert(test_root.join(relative), expected);
    }
    println!("Replaying manifest: {}", path.display());
    println!("Test directory: {}", directory.display());
    let replay_result = run_tests_with_expectations(&directory, None, Some(&expected_stdout));
    match (manifest.status, replay_result) {
        (RunStatus::Passed, Ok(())) => Ok(()),
        (RunStatus::Passed, Err(error)) => Err(error),
        (RunStatus::Failed, Err(_)) => {
            println!("Replay matched the recorded failed status.");
            Ok(())
        }
        (RunStatus::Failed, Ok(())) => Err(anyhow::anyhow!(
            "replay passed but manifest recorded a failed run"
        )),
        (RunStatus::InProgress, _) => Err(anyhow::anyhow!(
            "cannot replay a manifest with in_progress status"
        )),
    }
}

struct TestManifestData<'a> {
    status: RunStatus,
    files: &'a [PathBuf],
    successful_outputs: &'a [(PathBuf, String)],
    failures: &'a [(String, String)],
    started_at: u64,
    duration_ms: u64,
}

fn write_test_manifest(path: &Path, directory: &Path, data: TestManifestData<'_>) -> Result<()> {
    let TestManifestData {
        status,
        files,
        successful_outputs,
        failures,
        started_at,
        duration_ms,
    } = data;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| anyhow::anyhow!("system clock is before Unix epoch: {error}"))?;
    let run_id = format!("xdl-test-{}-{}", process::id(), now.as_nanos());
    let mut manifest = RunManifest::new(run_id, "xdl", directory.display().to_string());
    let source_root = if directory.is_file() {
        directory.parent().unwrap_or_else(|| Path::new("."))
    } else {
        directory
    };
    let manifest_root = path.parent().unwrap_or_else(|| Path::new("."));
    let recorded_directory = match directory.strip_prefix(manifest_root) {
        Ok(relative) if !relative.as_os_str().is_empty() => relative.display().to_string(),
        Ok(_) => ".".to_string(),
        Err(_) => {
            // A relative input such as `examples/foo.xdl` cannot be replayed
            // relative to an unrelated absolute manifest path. Preserve an
            // absolute source location in that case instead of creating a
            // path that only works from the original process directory.
            if directory.is_absolute() {
                directory.display().to_string()
            } else {
                std::env::current_dir()?
                    .join(directory)
                    .display()
                    .to_string()
            }
        }
    };
    manifest.status = status;
    manifest.started_at = Some(format!("unix:{started_at}"));
    manifest.completed_at = Some(format!("unix:{}", now.as_secs()));
    manifest.duration_ms = Some(duration_ms);
    manifest
        .configuration
        .insert("test_directory".to_string(), recorded_directory);
    manifest.configuration.insert(
        "runner".to_string(),
        "recursive-xdl-script-runner".to_string(),
    );
    manifest.environment.insert(
        "xdl_cli_version".to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    manifest.test_summary = Some(TestSummary {
        discovered: files.len(),
        passed: files.len() - failures.len(),
        failed: failures.len(),
    });
    manifest.inputs = files
        .iter()
        .map(|file| {
            let relative = file.strip_prefix(source_root).unwrap_or(file);
            Ok(xdl_artifacts::RunArtifact {
                kind: "xdl-test-source".to_string(),
                path: relative.display().to_string(),
                sha256: Some(sha256_file(file)?),
            })
        })
        .collect::<Result<Vec<_>>>()?;
    let artifact_root = path
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .join("artifacts");
    for (file, output) in successful_outputs {
        let relative = file.strip_prefix(source_root).unwrap_or(file);
        let artifact_path = artifact_root.join(relative).with_extension("stdout");
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create output artifact directory: {}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, output).with_context(|| {
            format!(
                "failed to write output artifact: {}",
                artifact_path.display()
            )
        })?;
        manifest.outputs.push(xdl_artifacts::RunArtifact {
            kind: "xdl-test-stdout".to_string(),
            path: artifact_path
                .strip_prefix(manifest_root)
                .unwrap_or(&artifact_path)
                .display()
                .to_string(),
            sha256: Some(sha256_file(&artifact_path)?),
        });
    }
    for (file, diagnostic) in failures {
        let source = Path::new(file);
        let relative = source.strip_prefix(source_root).unwrap_or(source);
        let artifact_path = artifact_root.join(relative).with_extension("failure.txt");
        if let Some(parent) = artifact_path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!(
                    "failed to create failure artifact directory: {}",
                    parent.display()
                )
            })?;
        }
        fs::write(&artifact_path, diagnostic).with_context(|| {
            format!(
                "failed to write failure artifact: {}",
                artifact_path.display()
            )
        })?;
        manifest.outputs.push(xdl_artifacts::RunArtifact {
            kind: "xdl-test-failure".to_string(),
            path: artifact_path
                .strip_prefix(manifest_root)
                .unwrap_or(&artifact_path)
                .display()
                .to_string(),
            sha256: Some(sha256_file(&artifact_path)?),
        });
    }
    manifest.warnings.push(
        "This runner records source hashes, successful stdout artifacts, failure diagnostics, and test counts; timeouts and numerical evidence are not collected yet.".to_string(),
    );

    let json = manifest
        .to_json_pretty()
        .with_context(|| "failed to validate test run manifest")?;
    fs::write(path, json)
        .with_context(|| format!("failed to write test run manifest: {}", path.display()))?;
    println!("Manifest: {}", path.display());
    Ok(())
}

fn sha256_file(path: &Path) -> Result<String> {
    let mut file = fs::File::open(path)
        .with_context(|| format!("failed to open test source for hashing: {}", path.display()))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0_u8; 8192];
    loop {
        let read = file
            .read(&mut buffer)
            .with_context(|| format!("failed to hash test source: {}", path.display()))?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("sha256:{:x}", hasher.finalize()))
}

fn collect_xdl_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    if directory.is_file() {
        if directory.extension().and_then(|ext| ext.to_str()) == Some("xdl") {
            files.push(directory.to_path_buf());
            return Ok(());
        }
        return Err(anyhow::anyhow!(
            "test path is a file but not an .xdl script: {}",
            directory.display()
        ));
    }
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

fn compatibility_report(
    format: &str,
    corpus: Option<&Path>,
    execute: bool,
    require_reference: bool,
) -> Result<()> {
    if execute && corpus.is_none() {
        return Err(anyhow::anyhow!(
            "--execute requires --corpus because the baseline has no source files"
        ));
    }
    let inventory = match corpus {
        Some(directory) => corpus_compatibility_inventory(directory, execute)?,
        None => baseline_compatibility_inventory(),
    };
    if require_reference {
        if corpus.is_none() {
            return Err(anyhow::anyhow!(
                "--require-reference requires --corpus because the baseline has no corpus files"
            ));
        }
        let incomplete: Vec<&str> = inventory
            .entries
            .iter()
            .filter(|entry| entry.status != CompatibilityStatus::Verified)
            .map(|entry| entry.capability.as_str())
            .collect();
        if !incomplete.is_empty() {
            return Err(anyhow::anyhow!(
                "{} corpus case(s) lack passing declared references: {}",
                incomplete.len(),
                incomplete.join(", ")
            ));
        }
    }
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
            if execute {
                println!(
                    "\nThis report measures transpilation and XDL execution only; runtime semantics and numerical equivalence are not independently verified."
                );
            } else if corpus.is_some() {
                println!(
                    "\nThis report measures transpilation only; runtime and numerical equivalence are not verified."
                );
            } else {
                println!("\nThis is a curated baseline, not a generated conformance percentage.");
            }
        }
        other => {
            return Err(anyhow::anyhow!(
                "unsupported compatibility report format '{other}'; expected text or json"
            ));
        }
    }
    Ok(())
}

fn corpus_compatibility_inventory(
    directory: &Path,
    execute: bool,
) -> Result<CompatibilityInventory> {
    let mut files = Vec::new();
    collect_matlab_files(directory, &mut files).with_context(|| {
        format!(
            "Failed to discover MATLAB corpus in {}",
            directory.display()
        )
    })?;
    files.sort();

    if files.is_empty() {
        return Err(anyhow::anyhow!(
            "no .m files found in MATLAB corpus {}",
            directory.display()
        ));
    }

    let mut entries = Vec::with_capacity(files.len());
    for file in files {
        let relative = file
            .strip_prefix(directory)
            .unwrap_or(&file)
            .display()
            .to_string();
        let source = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read MATLAB corpus file: {}", file.display()))?;
        let (status, notes, evidence) = match transpile_matlab_to_xdl(&source) {
            Ok(xdl_code) if execute => match execute_transpiled_source(&xdl_code) {
                Ok((output, variables)) => corpus_reference_result(&file, &output, &variables),
                Err(error) => (
                    CompatibilityStatus::Unsupported,
                    format!("Transpilation passed but XDL execution failed: {error}"),
                    vec![format!("transpile: {}", file.display())],
                ),
            },
            Ok(_) => (
                CompatibilityStatus::Partial,
                "Transpilation passed; runtime and numerical equivalence remain unverified."
                    .to_string(),
                vec![format!("transpile: {}", file.display())],
            ),
            Err(error) => (
                CompatibilityStatus::Unsupported,
                format!("Transpilation failed: {error}"),
                Vec::new(),
            ),
        };
        entries.push(CompatibilityEntry {
            id: format!("matlab.corpus.{relative}"),
            surface: "matlab-transpiler".to_string(),
            capability: relative,
            status,
            evidence,
            notes: Some(notes),
        });
    }

    Ok(CompatibilityInventory::new(
        format!("xdl-cli-matlab-corpus:{}", directory.display()),
        entries,
    ))
}

fn execute_transpiled_source(source: &str) -> Result<(String, BTreeMap<String, XdlValue>)> {
    let program = xdl_parser::parse_xdl(source)
        .map_err(|error| anyhow::anyhow!("transpiled XDL parse failed: {error}"))?;
    let buffer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let output: Rc<RefCell<dyn std::io::Write>> =
        Rc::new(RefCell::new(SharedOutput(buffer.clone())));
    let mut interpreter = Interpreter::with_output(output);
    interpreter
        .execute_program(&program)
        .map_err(|error| anyhow::anyhow!("transpiled XDL execution failed: {error}"))?;
    let bytes = buffer.borrow().clone();
    let output = String::from_utf8(bytes)
        .map_err(|error| anyhow::anyhow!("interpreter output was not UTF-8: {error}"))?;
    let variables = interpreter
        .get_variables()
        .into_iter()
        .map(|(name, value)| (name, value.clone()))
        .collect();
    Ok((output, variables))
}

#[derive(Debug, Deserialize)]
struct NumericReference {
    #[serde(default = "default_numeric_tolerance")]
    abs_tolerance: f64,
    #[serde(default)]
    provenance: Option<String>,
    values: BTreeMap<String, Vec<f64>>,
}

fn default_numeric_tolerance() -> f64 {
    1e-9
}

fn corpus_reference_result(
    file: &Path,
    output: &str,
    variables: &BTreeMap<String, XdlValue>,
) -> (CompatibilityStatus, String, Vec<String>) {
    let mut evidence = vec![
        format!("transpile: {}", file.display()),
        format!("execute: {}", file.display()),
    ];
    let mut failures = Vec::new();
    let stdout_reference = PathBuf::from(format!("{}.expected", file.display()));
    if stdout_reference.is_file() {
        match fs::read_to_string(&stdout_reference) {
            Ok(expected) if output.trim_end() == expected.trim_end() => {
                evidence.push(format!("stdout-reference: {}", stdout_reference.display()));
            }
            Ok(_) => failures.push(format!(
                "observed output did not match {}",
                stdout_reference.display()
            )),
            Err(error) => failures.push(format!(
                "could not read {}: {error}",
                stdout_reference.display()
            )),
        }
    }

    let numeric_reference = PathBuf::from(format!("{}.expected.json", file.display()));
    if numeric_reference.is_file() {
        evidence.push(format!(
            "numeric-reference: {}",
            numeric_reference.display()
        ));
        match fs::read_to_string(&numeric_reference)
            .map_err(|error| error.to_string())
            .and_then(|json| {
                serde_json::from_str::<NumericReference>(&json).map_err(|error| error.to_string())
            })
            .and_then(|reference| {
                if let Some(provenance) = reference.provenance.as_deref() {
                    if provenance.trim().is_empty() {
                        return Err("numeric reference provenance must not be empty".to_string());
                    }
                    evidence.push(format!(
                        "numeric-reference-provenance: {}",
                        provenance.trim()
                    ));
                }
                compare_numeric_reference(&reference, variables)
            }) {
            Ok(()) => {}
            Err(error) => failures.push(format!("numeric reference failed: {error}")),
        }
    }

    if !failures.is_empty() {
        (CompatibilityStatus::Failed, failures.join("; "), evidence)
    } else if evidence.len() > 2 {
        (
            CompatibilityStatus::Verified,
            "Transpilation, XDL execution, and declared observed-output/numeric references passed; broader runtime and numerical equivalence remain unverified.".to_string(),
            evidence,
        )
    } else {
        (
            CompatibilityStatus::Partial,
            "Transpilation and XDL execution passed; no observed-output or numeric reference was supplied.".to_string(),
            evidence,
        )
    }
}

fn compare_numeric_reference(
    reference: &NumericReference,
    variables: &BTreeMap<String, XdlValue>,
) -> Result<(), String> {
    if !reference.abs_tolerance.is_finite() || reference.abs_tolerance < 0.0 {
        return Err("absolute tolerance must be finite and non-negative".to_string());
    }
    for (name, expected) in &reference.values {
        let actual = variables
            .get(name)
            .ok_or_else(|| format!("variable '{name}' was not produced"))?;
        let actual = numeric_values(actual)
            .ok_or_else(|| format!("variable '{name}' is not a supported numeric value"))?;
        if actual.len() != expected.len() {
            return Err(format!(
                "variable '{name}' length {} != expected {}",
                actual.len(),
                expected.len()
            ));
        }
        for (index, (actual, expected)) in actual.iter().zip(expected).enumerate() {
            let matches = if actual.is_nan() && expected.is_nan() {
                true
            } else if actual.is_infinite() || expected.is_infinite() {
                actual == expected
            } else {
                (actual - expected).abs() <= reference.abs_tolerance
            };
            if !matches {
                return Err(format!(
                    "variable '{name}' element {index}: actual {actual} != expected {expected} within {}",
                    reference.abs_tolerance
                ));
            }
        }
    }
    Ok(())
}

fn numeric_values(value: &XdlValue) -> Option<Vec<f64>> {
    match value {
        XdlValue::Byte(value) => Some(vec![f64::from(*value)]),
        XdlValue::Int(value) => Some(vec![f64::from(*value)]),
        XdlValue::Long(value) => Some(vec![f64::from(*value)]),
        XdlValue::Float(value) => Some(vec![f64::from(*value)]),
        XdlValue::Double(value) => Some(vec![*value]),
        XdlValue::UInt(value) => Some(vec![f64::from(*value)]),
        XdlValue::ULong(value) => Some(vec![f64::from(*value)]),
        XdlValue::Long64(value) => Some(vec![*value as f64]),
        XdlValue::ULong64(value) => Some(vec![*value as f64]),
        XdlValue::Array(values) => Some(values.clone()),
        XdlValue::MultiDimArray { data, .. } => Some(data.clone()),
        XdlValue::NestedArray(values) => values
            .iter()
            .map(numeric_values)
            .collect::<Option<Vec<Vec<f64>>>>()
            .map(|values| values.into_iter().flatten().collect()),
        _ => None,
    }
}

struct SharedOutput(Rc<RefCell<Vec<u8>>>);

impl Write for SharedOutput {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(bytes);
        Ok(bytes.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn collect_matlab_files(directory: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
    let entries = fs::read_dir(directory)
        .with_context(|| format!("Failed to read MATLAB corpus: {}", directory.display()))?;

    for entry in entries {
        let entry = entry
            .with_context(|| format!("Failed to inspect MATLAB corpus: {}", directory.display()))?;
        let path = entry.path();
        let file_type = entry
            .file_type()
            .with_context(|| format!("Failed to inspect {}", path.display()))?;

        if file_type.is_dir() {
            collect_matlab_files(&path, files)?;
        } else if file_type.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("m")
        {
            files.push(path);
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
        CompatibilityStatus::Failed => "failed",
    }
}

fn check_model_file(file: &Path, migrate: bool, svg: bool) -> Result<()> {
    let content = fs::read_to_string(file)
        .with_context(|| format!("Failed to read model document: {}", file.display()))?;
    let document = match xdl_model::ModelDocument::from_json(&content) {
        Ok(document) => document,
        Err(document_error) => {
            let model: xdl_model::Model = serde_json::from_str(&content).with_context(|| {
                format!(
                    "Invalid model document or raw model graph: {} (document error: {})",
                    file.display(),
                    document_error
                )
            })?;
            let layout = xdl_model::ModelLayout::from_model(&model).with_context(|| {
                format!("Unable to auto-layout raw model graph: {}", file.display())
            })?;
            xdl_model::ModelDocument::new(model, layout)
                .with_context(|| format!("Invalid raw model graph: {}", file.display()))?
        }
    };
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
    // Print what the program printed, including output produced before a
    // runtime error. Dropping it on error hid everything a failing script had
    // already said.
    let (output, result) = run_file_capturing(file);
    use std::io::Write;
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(output.as_bytes())?;
    stdout.flush()?;
    result
}

fn execute_file_with_output(file: &Path) -> Result<String> {
    let (output, result) = run_file_capturing(file);
    result.map(|()| output)
}

/// Run a file and return everything it printed, together with the outcome.
/// Output is returned even when execution fails.
fn run_file_capturing(file: &Path) -> (String, Result<()>) {
    let buffer = Rc::new(RefCell::new(Vec::<u8>::new()));
    let result = run_file_into(file, &buffer);
    let text = match String::from_utf8(buffer.borrow().clone()) {
        Ok(text) => text,
        Err(e) => {
            return (
                String::new(),
                result.and(Err(anyhow::anyhow!(
                    "output from {} was not UTF-8: {}",
                    file.display(),
                    e
                ))),
            )
        }
    };
    (text, result)
}

fn run_file_into(file: &Path, buffer: &Rc<RefCell<Vec<u8>>>) -> Result<()> {
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

    let output: Rc<RefCell<dyn std::io::Write>> =
        Rc::new(RefCell::new(SharedOutput(buffer.clone())));
    let mut interpreter = Interpreter::with_output(output);
    interpreter
        .execute_program(&program)
        .with_context(|| "Failed to execute program")?;
    Ok(())
}

/// Format a XdlValue for display
fn format_xdl_value(value: &XdlValue) -> String {
    value.to_string_repr()
}
