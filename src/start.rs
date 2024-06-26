use crate::commands::{Commands, Scripts, run_script, init_script_file};
use std::fs;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(name = "cargo-script")]
#[command(about = format!("A CLI tool to run custom scripts in Rust, defined in Scripts.toml {}", emoji::objects::computer::FLOPPY_DISK.glyph))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

pub fn run() {
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Script { run } => {
            let scripts: Scripts = toml::from_str(&fs::read_to_string("Scripts.toml").expect("Fail to load Scripts.toml"))
                .expect("Fail to parse Scripts.toml");
            run_script(&scripts, run);
        }
        Commands::Init => {
            init_script_file();
        }
    }
}
