use std::{collections::HashMap, fs, process::Command};
use clap::{Parser, Subcommand, ArgAction};
use serde::Deserialize;
use colored::*;
use emoji::symbols;


#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml")]
    Script {
        #[arg(short, long, value_name = "SCRIPT_NAME", action = ArgAction::Set)]
        run: String,
    }
}


#[derive(Parser, Debug)]
#[command(name = "cargo-script")]
#[command(about = format!("A CLI tool to run custom scripts in Rust, defined in Scripts.toml {:?}", emoji::objects::computer::FLOPPY_DISK.glyph))]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Deserialize)]
struct Scripts {
    scripts: HashMap<String, String>
}


fn main() {
    
    let cli = Cli::parse();
    let scripts: Scripts = toml::from_str(&fs::read_to_string("Scripts.toml").expect("Fail to load Scripts.toml"))
    .expect("Fail to parse Scripts.toml");

    match &cli.command {
        Commands::Script { run } => {
            if let Some(script_cmd) = scripts.scripts.get(run) {
                println!("\n");
                let msg: String = format!("{} {}: {}", symbols::other_symbol::CHECK_MARK.glyph, "Running script".green(), run);
                println!("{}\n", msg);
                run_script(script_cmd);
            } else {
                let msg = format!("{} {}: {}", symbols::other_symbol::CROSS_MARK.glyph, "Script not found".red(), run);
                println!("{}", msg);
            }
        }
    }
        
}

fn run_script(script: &str) {
    match detect_os() {
        "windows_cmd" => {
            Command::new("cmd")
                .args(&["/C", script])
                .status()
                .expect("Failed to execute script using [CMD]");
        },
        "windows_powershell" => {
            Command::new("powershell")
                .args(&["-Command", script])
                .status()
                .expect("Failed to execute script using [Powershell]");
        },
        "macos" | "linux" => {
            Command::new("sh")
                .args(&["-c", script])
                .status()
                .expect("Failed to execute script using [sh]");
        }
        _ => {
            println!("Unsupported OS");
        }
    }
}

fn detect_os() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows_cmd"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        "unknown"
    }
}