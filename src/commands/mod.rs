//! This module defines the commands and their execution logic for the cargo-script CLI tool.
//!
//! It includes functionalities to run scripts, initialize the Scripts.toml file, and handle script execution.
use clap::{Subcommand, ArgAction};

/// Enum representing the different commands supported by the CLI tool.
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml")]
    Run {
        #[arg(value_name = "SCRIPT_NAME", action = ArgAction::Set)]
        script: String,
        #[arg(short, long, value_name = "KEY=VALUE", action = ArgAction::Append)]
        env: Vec<String>,
    },
    #[command(about = "Initialize a Scripts.toml file in the current directory")]
    Init,
    #[command(about = "Show all script names and descriptions defined in Scripts.toml")]
    Show,
}

pub mod init;
pub mod script;
pub mod show;