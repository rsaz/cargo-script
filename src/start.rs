//! Top-level CLI entry point for `cargo-script` / `cgs`.
//!
//! Parses arguments via clap, dispatches to the relevant command and centralises
//! error reporting.

use crate::commands::{
    completions::generate_completions,
    init::{init_from_template, init_script_file, list_templates},
    script::{run_script_with_options, RunOptions, Scripts},
    show::show_scripts,
    validate::{print_validation_results, validate_scripts},
    workspace as _workspace,
    Commands, WorkspaceCmd,
};
use crate::error::CargoScriptError;
use clap::{ArgAction, CommandFactory, Parser};
use colored::*;
use std::fs;

#[derive(Parser, Debug)]
#[command(
    name = "cargo-script",
    about = "A powerful CLI tool for managing project scripts in Rust",
    long_about = "Think npm scripts, make, or just — but built specifically for the Rust ecosystem with workspace support, cargo-script (.rs) integration, hooks, parallel execution, watch mode and CI/CD templates.",
    after_help = "EXAMPLES:\n  cargo script build                              Run the 'build' script\n  cargo script run test                          Explicitly run the 'test' script\n  cargo script test --env RUST_LOG=debug        Run with environment variable\n  cargo script test --dry-run                    Preview what would run\n  cargo script test --watch                      Re-run on file changes\n  cargo script test --json                       Emit a JSON execution report\n  cargo script ci --workspace=parallel           Run 'ci' across workspace members\n  cargo script init --template github-actions    Scaffold CI/CD config\n  cargo script workspace list                    List workspace members\n  cargo script show                              List all available scripts\n  cargo script validate                          Validate Scripts.toml\n\nFor more information, visit: https://github.com/rsaz/cargo-script",
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
    /// Preview what would be executed without actually running it
    #[arg(long, global = true)]
    dry_run: bool,
    /// Don't show performance metrics after execution
    #[arg(long, global = true)]
    no_metrics: bool,
    /// Interactive script selection
    #[arg(short, long, global = true)]
    interactive: bool,
}

/// Entry point used by `main.rs` / `cgs.rs`. Prints any error to stderr and
/// exits with a non-zero status.
pub fn run_with_error_handling() {
    if let Err(e) = run() {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

/// Pure run function: parses CLI and dispatches to the relevant subcommand.
pub fn run() -> Result<(), CargoScriptError> {
    let args: Vec<String> = std::env::args().collect();
    let cli = if args.len() > 1 && args[1] == "script" {
        // Invoked as `cargo script ...`
        let mut cargo_args = vec![args[0].clone()];
        cargo_args.extend(args.into_iter().skip(2));

        if cargo_args.len() == 1 {
            return handle_show_command("Scripts.toml", false, false, None);
        }

        if cargo_args.len() == 2 && (cargo_args[1] == "--help" || cargo_args[1] == "-h") {
            let mut app = Cli::command();
            app.print_help().unwrap();
            std::process::exit(0);
        }

        Cli::try_parse_from(cargo_args).unwrap_or_else(|e| e.exit())
    } else {
        Cli::parse()
    };

    let command = match (&cli.command, &cli.script_name) {
        (Some(_), Some(_)) => {
            eprintln!("{}", "Error: Cannot specify both a subcommand and a script name".red().bold());
            eprintln!("{}", "Use either 'cargo script <script_name>' or 'cargo script <subcommand>'".white());
            std::process::exit(1);
        }
        (Some(cmd), None) => cmd.clone(),
        (None, Some(script_name)) => Commands::Run {
            script: Some(script_name.clone()),
            env: cli.env.clone(),
            dry_run: cli.dry_run,
            quiet: false,
            verbose: false,
            no_metrics: cli.no_metrics,
            interactive: cli.interactive,
            workspace: None,
            no_workspace: false,
            watch: false,
            watch_path: vec![],
            watch_exclude: vec![],
            json: false,
            parallel: vec![],
            script_args: vec![],
        },
        (None, None) => {
            if cli.interactive {
                Commands::Run {
                    script: None,
                    env: cli.env.clone(),
                    dry_run: cli.dry_run,
                    quiet: false,
                    verbose: false,
                    no_metrics: cli.no_metrics,
                    interactive: true,
                    workspace: None,
                    no_workspace: false,
                    watch: false,
                    watch_path: vec![],
                    watch_exclude: vec![],
                    json: false,
                    parallel: vec![],
                    script_args: vec![],
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

    let should_show_banner = !cli.quiet
        && !matches!(&command, Commands::Completions { .. })
        && !matches!(&command, Commands::Run { dry_run: true, .. })
        && !matches!(&command, Commands::Run { json: true, .. })
        && (cli.verbose
            || !std::path::Path::new(&cli.scripts_path).exists()
            || matches!(
                &command,
                Commands::Init { .. } | Commands::Show { .. } | Commands::Validate { .. } | Commands::Workspace { .. }
            ));

    if should_show_banner {
        let init_msg = format!(
            "A CLI tool to run custom scripts in Rust, defined in [ Scripts.toml ] {}",
            emoji::objects::computer::FLOPPY_DISK.glyph
        );
        print_framed_message(&init_msg);
    }

    let scripts_path = &cli.scripts_path;

    match &command {
        Commands::Run {
            script,
            env,
            dry_run,
            quiet,
            verbose,
            no_metrics,
            interactive,
            workspace,
            no_workspace,
            watch,
            watch_path,
            watch_exclude,
            json,
            parallel,
            script_args,
        } => {
            let final_quiet = cli.quiet || *quiet;
            let final_verbose = cli.verbose || *verbose;
            let final_dry_run = cli.dry_run || *dry_run;
            let final_no_metrics = cli.no_metrics || *no_metrics;
            let final_env = if !env.is_empty() { env.clone() } else { cli.env.clone() };

            let scripts = load_scripts(scripts_path)?;

            // Multi-script parallel mode: --parallel A --parallel B [...] [primary]
            if !parallel.is_empty() {
                #[cfg(feature = "parallel")]
                {
                    let mut names = parallel.clone();
                    if let Some(s) = script.clone() {
                        names.insert(0, s);
                    }
                    let opts = RunOptions {
                        env_overrides: final_env,
                        dry_run: final_dry_run,
                        quiet: final_quiet,
                        verbose: final_verbose,
                        show_metrics: !final_no_metrics,
                        json_output: *json,
                        ..RunOptions::default()
                    };
                    let result = crate::commands::parallel::run_scripts_parallel(&scripts, &names, &opts)?;
                    if *json {
                        crate::output::json::print_execution_result(&result);
                    }
                    return Ok(());
                }
                #[cfg(not(feature = "parallel"))]
                {
                    eprintln!("--parallel requires the `parallel` feature");
                    std::process::exit(2);
                }
            }

            let script_name = if *interactive || script.is_none() {
                crate::commands::script::interactive_select_script(&scripts, final_quiet)?
            } else {
                script.clone().ok_or_else(|| CargoScriptError::ScriptNotFound {
                    script_name: "".to_string(),
                    available_scripts: scripts.scripts.keys().cloned().collect(),
                })?
            };

            let opts = RunOptions {
                env_overrides: final_env,
                dry_run: final_dry_run,
                quiet: final_quiet,
                verbose: final_verbose,
                show_metrics: !final_no_metrics,
                script_args: script_args.clone(),
                workspace_override: workspace.map(Into::into),
                no_workspace: *no_workspace,
                json_output: *json,
            };

            if *watch {
                #[cfg(feature = "watch")]
                {
                    let cfg = crate::commands::watch::WatchConfig {
                        watch_paths: if watch_path.is_empty() {
                            vec![std::path::PathBuf::from(".")]
                        } else {
                            watch_path.clone()
                        },
                        exclude: {
                            let mut v = vec!["target".to_string(), ".git".to_string(), "node_modules".to_string()];
                            v.extend(watch_exclude.clone());
                            v
                        },
                        ..crate::commands::watch::WatchConfig::default()
                    };
                    return crate::commands::watch::watch_and_run(&scripts, &script_name, &opts, cfg);
                }
                #[cfg(not(feature = "watch"))]
                {
                    let _ = (watch_path, watch_exclude);
                    eprintln!("--watch requires the `watch` feature");
                    std::process::exit(2);
                }
            }

            let _ = run_script_with_options(&scripts, &script_name, &opts)?;
        }
        Commands::Init { template, list_templates: list_flag, force } => {
            if *list_flag {
                list_templates();
            } else if let Some(t) = template {
                init_from_template(t, *force)?;
            } else {
                init_script_file(Some(scripts_path));
            }
        }
        Commands::Show { quiet, verbose: _verbose, filter } => {
            let final_quiet = cli.quiet || *quiet;
            handle_show_command(scripts_path, final_quiet, cli.verbose, filter.as_deref())?;
        }
        Commands::Completions { shell } => {
            let mut app = Cli::command();
            generate_completions(shell.clone(), &mut app);
        }
        Commands::Validate { quiet, verbose: _verbose } => {
            let final_quiet = cli.quiet || *quiet;
            let scripts = load_scripts(scripts_path)?;
            let validation_result = validate_scripts(&scripts);
            if !final_quiet {
                print_validation_results(&validation_result);
            }
            if !validation_result.is_valid() {
                std::process::exit(1);
            }
        }
        Commands::Workspace { cmd } => match cmd {
            WorkspaceCmd::List { json } => {
                let scripts = load_scripts(scripts_path).unwrap_or_default();
                let summary = _workspace::workspace_summary(&scripts)?;
                if *json {
                    println!("{}", serde_json::to_string_pretty(&summary).unwrap());
                } else {
                    println!("{}", "📦 Workspace members".cyan().bold());
                    if let Some(members) = summary.get("members") {
                        for m in members {
                            println!("  • {}", m.green());
                        }
                    }
                }
            }
            WorkspaceCmd::Run { script, parallel: par, json } => {
                let scripts = load_scripts(scripts_path)?;
                let mode = if *par {
                    crate::commands::script::WorkspaceMode::Parallel
                } else {
                    crate::commands::script::WorkspaceMode::All
                };
                let opts = RunOptions {
                    env_overrides: cli.env.clone(),
                    quiet: cli.quiet,
                    verbose: cli.verbose,
                    show_metrics: !cli.no_metrics,
                    workspace_override: Some(mode),
                    json_output: *json,
                    ..RunOptions::default()
                };
                let _ = run_script_with_options(&scripts, script, &opts)?;
            }
        },
    }

    Ok(())
}

fn load_scripts(scripts_path: &str) -> Result<Scripts, CargoScriptError> {
    let scripts_content = fs::read_to_string(scripts_path).map_err(|e| {
        CargoScriptError::ScriptFileNotFound {
            path: scripts_path.to_string(),
            source: e,
        }
    })?;

    toml::from_str(&scripts_content).map_err(|e| {
        let message = e.message().to_string();
        let line = e.span().map(|s| s.start);
        CargoScriptError::InvalidToml {
            path: scripts_path.to_string(),
            message,
            line,
        }
    })
}

fn handle_show_command(scripts_path: &str, quiet: bool, _verbose: bool, filter: Option<&str>) -> Result<(), CargoScriptError> {
    let scripts = load_scripts(scripts_path)?;
    if !quiet {
        show_scripts(&scripts, filter);
    }
    Ok(())
}

fn print_framed_message(message: &str) {
    let framed_message = format!("| {} |", message);
    let frame = "-".repeat(framed_message.len() - 2);
    println!("\n{}\n{}\n{}\n", frame.yellow(), framed_message.yellow(), frame.yellow());
}
