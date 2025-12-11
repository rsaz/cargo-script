//! This module contains the main logic for the cargo-script CLI tool.
//!
//! It parses the command-line arguments and executes the appropriate commands.
use crate::commands::{init::init_script_file, script::run_script, Commands, script::Scripts, show::show_scripts, completions::generate_completions, validate::{validate_scripts, print_validation_results}};
use crate::error::CargoScriptError;
use std::fs;
use clap::{Parser, CommandFactory, ArgAction};
use colored::*;

/// Command-line arguments structure for the cargo-script CLI tool.
#[derive(Parser, Debug)]
#[command(
    name = "cargo-script",
    about = "A powerful CLI tool for managing project scripts in Rust",
    long_about = "Think npm scripts, make, or just — but built specifically for the Rust ecosystem with modern CLI best practices.",
    after_help = "EXAMPLES:\n  cargo script build                              Run the 'build' script\n  cargo script run test                          Explicitly run the 'test' script\n  cargo script test --env RUST_LOG=debug        Run with environment variable\n  cargo script test --dry-run                    Preview what would run\n  cargo script test --no-metrics                 Run without performance metrics\n  cargo script --interactive                     Interactive script selection\n  cargo script show                              List all available scripts\n  cargo script show --filter test                Filter scripts by name/description\n  cargo script init                              Initialize Scripts.toml\n  cargo script validate                         Validate Scripts.toml\n\nFor more information, visit: https://github.com/rsaz/cargo-script",
    version,
    subcommand_required = false,
    arg_required_else_help = false,
)]
pub struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
    /// Script name to run (when not using 'run' subcommand)
    #[arg(value_name = "SCRIPT_NAME")]
    script_name: Option<String>,
    /// Optional path to the Scripts.toml file.
    #[arg(long, default_value = "Scripts.toml", global = true)]
    scripts_path: String,
    /// Suppress all output except errors
    #[arg(short, long, global = true)]
    quiet: bool,
    /// Show detailed output
    #[arg(short = 'v', long, global = true)]
    verbose: bool,
    /// Environment variables to set (only used when script_name is provided)
    #[arg(short, long, value_name = "KEY=VALUE", action = ArgAction::Append, global = true)]
    env: Vec<String>,
    /// Preview what would be executed without actually running it (only used when script_name is provided)
    #[arg(long, global = true)]
    dry_run: bool,
    /// Don't show performance metrics after execution (only used when script_name is provided)
    #[arg(long, global = true)]
    no_metrics: bool,
    /// Interactive script selection (only used when script_name is provided)
    #[arg(short, long, global = true)]
    interactive: bool,
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
    // Handle Cargo subcommand invocation
    // When invoked as `cargo script`, Cargo passes "script" as the first argument
    // We need to remove it before parsing
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() > 1 && args[1] == "script" {
        // Remove "script" argument when invoked as `cargo script`
        // Also need to include the binary name for clap
        let mut cargo_args = vec![args[0].clone()]; // binary name
        cargo_args.extend(args.into_iter().skip(2)); // skip "cargo-script" and "script"
        
        // If no arguments after "script", default to show
        if cargo_args.len() == 1 {
            // Only binary name, no subcommand - default to show
            return handle_show_command("Scripts.toml", false, false, None);
        }
        
        // Check for help flag before parsing
        if cargo_args.len() == 2 && (cargo_args[1] == "--help" || cargo_args[1] == "-h") {
            let mut app = Cli::command();
            app.print_help().unwrap();
            std::process::exit(0);
        }
        
        Cli::try_parse_from(cargo_args).unwrap_or_else(|e| {
            // Let clap handle the error (will show usage)
            e.exit()
        })
    } else {
        // Normal invocation: `cargo-script` or `cgs`
        Cli::parse()
    };
    
    // Determine the actual command to execute
    // If no command but script_name is provided, treat as Run
    // If no command and no script_name, default to Show
    // Error if both command and script_name are provided
    let command = match (&cli.command, &cli.script_name) {
        (Some(_cmd), Some(_)) => {
            // Both command and script_name provided - this is an error
            eprintln!("{}", "Error: Cannot specify both a subcommand and a script name".red().bold());
            eprintln!("{}", "Use either 'cargo script <script_name>' or 'cargo script <subcommand>'".white());
            std::process::exit(1);
        }
        (Some(cmd), None) => cmd.clone(),
        (None, Some(script_name)) => {
            // Treat script_name as Run command, using global flags
            Commands::Run {
                script: Some(script_name.clone()),
                env: cli.env.clone(),
                dry_run: cli.dry_run,
                quiet: false, // Use global quiet flag instead
                verbose: false, // Use global verbose flag instead
                no_metrics: cli.no_metrics,
                interactive: cli.interactive,
            }
        }
        (None, None) => {
            // If --interactive is set, treat as Run with interactive mode
            // Otherwise default to Show
            if cli.interactive {
                Commands::Run {
                    script: None,
                    env: cli.env.clone(),
                    dry_run: cli.dry_run,
                    quiet: false,
                    verbose: false,
                    no_metrics: cli.no_metrics,
                    interactive: true,
                }
            } else {
                Commands::Show {
                    quiet: cli.quiet,
                    verbose: cli.verbose,
                    filter: None,
                }
            }
        }
    };
    
    // Conditional banner display:
    // - Never show for completions (interferes with output)
    // - Never show in quiet mode
    // - Show in verbose mode
    // - Show if Scripts.toml doesn't exist (first run)
    // - Don't show for dry-run (cleaner output)
    // - Show for Init, Show, and Validate commands (helpful context)
    let should_show_banner = !cli.quiet 
        && !matches!(&command, Commands::Completions { .. })
        && !matches!(&command, Commands::Run { dry_run: true, .. })
        && (cli.verbose 
            || !std::path::Path::new(&cli.scripts_path).exists()
            || matches!(&command, Commands::Init | Commands::Show { .. } | Commands::Validate { .. }));
    
    if should_show_banner {
        let init_msg = format!("A CLI tool to run custom scripts in Rust, defined in [ Scripts.toml ] {}", emoji::objects::computer::FLOPPY_DISK.glyph);
        print_framed_message(&init_msg);
    }
    
    let scripts_path = &cli.scripts_path;

    match &command {
        Commands::Run { script, env, dry_run, quiet, verbose, no_metrics, interactive } => {
            // Merge global and command-specific flags
            // When Run is explicitly used, its flags take precedence
            // When script_name is used (short form), global flags are used
            let final_quiet = cli.quiet || *quiet;
            let final_verbose = cli.verbose || *verbose;
            let final_dry_run = cli.dry_run || *dry_run;
            let final_no_metrics = cli.no_metrics || *no_metrics;
            let final_env = if !env.is_empty() { env.clone() } else { cli.env.clone() };
            let show_metrics = !final_no_metrics; // Show metrics by default unless --no-metrics is set
            
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
            
            // Handle interactive mode or when script is not provided
            let script_name = if *interactive || script.is_none() {
                crate::commands::script::interactive_select_script(&scripts, final_quiet)?
            } else {
                script.clone().ok_or_else(|| CargoScriptError::ScriptNotFound {
                    script_name: "".to_string(),
                    available_scripts: scripts.scripts.keys().cloned().collect(),
                })?
            };
            
            run_script(&scripts, &script_name, final_env, final_dry_run, final_quiet, final_verbose, show_metrics)?;
        }
        Commands::Init => {
            init_script_file(Some(scripts_path));
        }
        Commands::Show { quiet, verbose: _verbose, filter } => {
            // Merge global and command-specific verbosity flags
            let final_quiet = cli.quiet || *quiet;
            handle_show_command(scripts_path, final_quiet, cli.verbose, filter.as_deref())?;
        }
        Commands::Completions { shell } => {
            let mut app = Cli::command();
            generate_completions(shell.clone(), &mut app);
        }
        Commands::Validate { quiet, verbose: _verbose } => {
            // Merge global and command-specific verbosity flags
            let final_quiet = cli.quiet || *quiet;
            
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
            if !final_quiet {
                print_validation_results(&validation_result);
            }
            
            if !validation_result.is_valid() {
                std::process::exit(1);
            }
        }
    }
    
    Ok(())
}

/// Helper function to handle the Show command
fn handle_show_command(scripts_path: &str, quiet: bool, _verbose: bool, filter: Option<&str>) -> Result<(), CargoScriptError> {
    let scripts_content = fs::read_to_string(scripts_path)
        .map_err(|e| CargoScriptError::ScriptFileNotFound {
            path: scripts_path.to_string(),
            source: e,
        })?;
    
    let scripts: Scripts = toml::from_str(&scripts_content)
        .map_err(|e| {
            let message = e.message().to_string();
            let line = e.span().map(|s| s.start);
            CargoScriptError::InvalidToml {
                path: scripts_path.to_string(),
                message,
                line,
            }
        })?;
    
    if !quiet {
        show_scripts(&scripts, filter);
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