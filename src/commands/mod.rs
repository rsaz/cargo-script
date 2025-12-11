//! This module defines the commands and their execution logic for the cargo-script CLI tool.
//!
//! It includes functionalities to run scripts, initialize the Scripts.toml file, and handle script execution.

use clap::{Subcommand, ArgAction, ValueEnum};

/// Enum representing the different commands supported by the CLI tool.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml", visible_alias = "r")]
    Run {
        #[arg(value_name = "SCRIPT_NAME")]
        script: Option<String>,
        #[arg(short, long, value_name = "KEY=VALUE", action = ArgAction::Append)]
        env: Vec<String>,
        /// Preview what would be executed without actually running it
        #[arg(long)]
        dry_run: bool,
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Don't show performance metrics after execution
        #[arg(long)]
        no_metrics: bool,
        /// Interactive script selection
        #[arg(short, long)]
        interactive: bool,
    },
    #[command(about = "Initialize a Scripts.toml file in the current directory")]
    Init,
    #[command(about = "Show all script names and descriptions defined in Scripts.toml")]
    Show {
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Filter scripts by name or description
        #[arg(short, long, value_name = "PATTERN")]
        filter: Option<String>,
    },
    #[command(about = "Generate shell completion scripts")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(about = "Validate Scripts.toml syntax, script references, and tool requirements")]
    Validate {
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
}

/// Supported shells for completion generation
#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "power-shell")]
    PowerShell,
}

pub mod init;
pub mod script;
pub mod show;
pub mod completions;
pub mod validate;