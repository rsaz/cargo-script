//! This module contains the main logic for the cargo-script CLI tool.
//!
//! It parses the command-line arguments and executes the appropriate commands.
use crate::commands::{init_script_file, run_script, show_scripts, Commands, Scripts};
use std::fs;
use clap::Parser;
use colored::*;

/// Command-line arguments structure for the cargo-script CLI tool.
#[derive(Parser, Debug)]
#[command(name = "cargo-script")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
    /// Optional path to the Scripts.toml file.
    #[arg(long, default_value = "Scripts.toml", global = true)]
    scripts_path: String,
}

/// Main function that parses command-line arguments and executes the specified command.
pub fn run() {
    let init_msg = format!("A CLI tool to run custom scripts in Rust, defined in [ Scripts.toml ] {}", emoji::objects::computer::FLOPPY_DISK.glyph);
    print_framed_message(&init_msg);

    let cli = Cli::parse();
    
    let scripts_path = &cli.scripts_path;

    match &cli.command {
        Commands::Run { script, env } => {
            let scripts: Scripts = toml::from_str(&fs::read_to_string(scripts_path).expect("Fail to load Scripts.toml"))
                .expect("Fail to parse Scripts.toml");
            run_script(&scripts, script, env.clone());
        }
        Commands::Init => {
            init_script_file();
        }
        Commands::Show => {
            let scripts: Scripts = toml::from_str(&fs::read_to_string(scripts_path).expect("Fail to load Scripts.toml"))
                .expect("Fail to parse Scripts.toml");
            show_scripts(&scripts);
        }
    }
}

/// Prints a framed message with a dashed line frame.
fn print_framed_message(message: &str) {
    let framed_message = format!("| {} |", message);
    let frame = "-".repeat(framed_message.len()-2);
    println!("\n{}\n{}\n{}\n", frame.yellow(), framed_message.yellow(), frame.yellow());
}