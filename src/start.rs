use crate::commands::{Scripts, Commands};
use std::fs;
use clap::Parser;
use colored::*;
use emoji::symbols;

#[derive(Parser, Debug)]
#[command(name = "cargo-script")]
#[command(about = format!("A CLI tool to run custom scripts in Rust, defined in Scripts.toml {:?}", emoji::objects::computer::FLOPPY_DISK.glyph))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}


pub fn run() {
    let cli = Cli::parse();
    let scripts: Scripts = toml::from_str(&fs::read_to_string("Scripts.toml").expect("Fail to load Scripts.toml"))
    .expect("Fail to parse Scripts.toml");

    match &cli.command {
        Commands::Script { run } => {
            if let Some(script_cmd) = scripts.scripts.get(run) {
                println!("\n");
                let msg: String = format!("{} {}: {}", symbols::other_symbol::CHECK_MARK.glyph, "Running script".green(), run);
                println!("{}\n", msg);
                crate::commands::run_script(script_cmd);
            } else {
                let msg = format!("{} {}: {}", symbols::other_symbol::CROSS_MARK.glyph, "Script not found".red(), run);
                println!("{}", msg);
            }
        }
        Commands::Init => {
            crate::commands::init_script_file();
        }
    }
}