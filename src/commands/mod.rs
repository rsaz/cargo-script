//! This module defines the commands and their execution logic for the cargo-script CLI tool.
//!
//! It includes functionalities to run scripts, initialize the Scripts.toml file, and handle script execution.

use clap::{Subcommand, ArgAction, ValueEnum};

/// Enum representing the different commands supported by the CLI tool.
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml")]
    Run {
        #[arg(value_name = "SCRIPT_NAME", action = ArgAction::Set)]
        script: String,
        #[arg(short, long, value_name = "KEY=VALUE", action = ArgAction::Append)]
        env: Vec<String>,
        /// Preview what would be executed without actually running it
        #[arg(long)]
        dry_run: bool,
    },
    #[command(about = "Initialize a Scripts.toml file in the current directory")]
    Init,
    #[command(about = "Show all script names and descriptions defined in Scripts.toml")]
    Show,
    #[command(about = "Generate shell completion scripts")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(about = "Validate Scripts.toml syntax, script references, and tool requirements")]
    Validate,
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