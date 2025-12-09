//! This module contains the main logic for the cargo-script CLI tool.
//!
//! It parses the command-line arguments and executes the appropriate commands.
use crate::commands::{init::init_script_file, script::run_script, Commands, script::Scripts, show::show_scripts, completions::generate_completions, validate::{validate_scripts, print_validation_results}};
use crate::error::CargoScriptError;
use std::fs;
use clap::{Parser, CommandFactory};
use colored::*;

/// Command-line arguments structure for the cargo-script CLI tool.
#[derive(Parser, Debug)]
#[command(name = "cargo-script")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Optional path to the Scripts.toml file.
    #[arg(long, default_value = "Scripts.toml", global = true)]
    scripts_path: String,
}

/// Run function that handles errors gracefully.
pub fn run_with_error_handling() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

/// Run function that parses command-line arguments and executes the specified command.
///
/// This function initializes the CLI, parses the command-line arguments, and routes
/// the commands to their respective handlers.
///
/// # Errors
///
/// Returns an error if it fails to read or parse the `Scripts.toml` file.
pub fn run() -> Result<(), CargoScriptError> {
    let cli = Cli::parse();
    
    // Don't show banner for completions or dry-run commands (they interfere with output)
    if !matches!(cli.command, Commands::Completions { .. } | Commands::Run { dry_run: true, .. }) {
        let init_msg = format!("A CLI tool to run custom scripts in Rust, defined in [ Scripts.toml ] {}", emoji::objects::computer::FLOPPY_DISK.glyph);
        print_framed_message(&init_msg);
    }
    
    let scripts_path = &cli.scripts_path;

    match &cli.command {
        Commands::Run { script, env, dry_run } => {
            let scripts_content = fs::read_to_string(scripts_path)
                .map_err(|e| CargoScriptError::ScriptFileNotFound {
                    path: scripts_path.clone(),
                    source: e,
                })?;
            
            let scripts: Scripts = toml::from_str(&scripts_content)
                .map_err(|e| {
                    let message = e.message().to_string();
                    let line = e.span().map(|s| s.start);
                    CargoScriptError::InvalidToml {
                        path: scripts_path.clone(),
                        message,
                        line,
                    }
                })?;
            
            run_script(&scripts, script, env.clone(), *dry_run)?;
        }
        Commands::Init => {
            init_script_file();
        }
        Commands::Show => {
            let scripts_content = fs::read_to_string(scripts_path)
                .map_err(|e| CargoScriptError::ScriptFileNotFound {
                    path: scripts_path.clone(),
                    source: e,
                })?;
            
            let scripts: Scripts = toml::from_str(&scripts_content)
                .map_err(|e| {
                    let message = e.message().to_string();
                    let line = e.span().map(|s| s.start);
                    CargoScriptError::InvalidToml {
                        path: scripts_path.clone(),
                        message,
                        line,
                    }
                })?;
            
            show_scripts(&scripts);
        }
        Commands::Completions { shell } => {
            let mut app = Cli::command();
            generate_completions(shell.clone(), &mut app);
        }
        Commands::Validate => {
            let scripts_content = fs::read_to_string(scripts_path)
                .map_err(|e| CargoScriptError::ScriptFileNotFound {
                    path: scripts_path.clone(),
                    source: e,
                })?;
            
            let scripts: Scripts = toml::from_str(&scripts_content)
                .map_err(|e| {
                    let message = e.message().to_string();
                    let line = e.span().map(|s| s.start);
                    CargoScriptError::InvalidToml {
                        path: scripts_path.clone(),
                        message,
                        line,
                    }
                })?;
            
            let validation_result = validate_scripts(&scripts);
            print_validation_results(&validation_result);
            
            if !validation_result.is_valid() {
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

/// Prints a framed message with a dashed line frame.
///
/// This function prints a framed message to the console, making it more visually
/// appealing and easier to read.
///
/// # Arguments
///
/// * `message` - A string slice that holds the message to be framed.
///
fn print_framed_message(message: &str) {
    let framed_message = format!("| {} |", message);
    let frame = "-".repeat(framed_message.len()-2);
    println!("\n{}\n{}\n{}\n", frame.yellow(), framed_message.yellow(), frame.yellow());
}