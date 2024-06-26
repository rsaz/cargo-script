//! This module defines the commands and their execution logic for the cargo-script CLI tool.
//!
//! It includes functionalities to run scripts, initialize the Scripts.toml file, and handle script execution.
use std::{collections::HashMap, fs, io, process::Command};
use clap::{Subcommand, ArgAction};
use serde::Deserialize;
use emoji::symbols;

/// Enum representing the different commands supported by the CLI tool.
#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml")]
    Script {
        #[arg(short, long, value_name = "SCRIPT_NAME", action = ArgAction::Set)]
        run: String,
    },
    #[command(about = "Initialize a Scripts.toml file in the current directory")]
    Init,
}

/// Enum representing a script, which can be either a default command or a detailed script with additional metadata.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    Default(String),
    Detailed {
        interpreter: Option<String>,
        command: Option<String>,
        info: Option<String>,
        include: Option<Vec<String>>,
    }
}

/// Struct representing the collection of scripts defined in Scripts.toml.
#[derive(Deserialize)]
pub struct Scripts {
    pub scripts: HashMap<String, Script>
}

/// Run a script by name, executing any included scripts in sequence.
/// 
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
/// * `script_name` - The name of the script to run.
pub fn run_script(scripts: &Scripts, script_name: &str) {
    if let Some(script) = scripts.scripts.get(script_name) {
        match script {
            Script::Default(cmd) => {
                let msg: String = format!("{} {}: {}", symbols::other_symbol::CHECK_MARK.glyph, "Running script", script_name);
                println!("{}\n", msg);
                execute_command(None, cmd);
            },
            Script::Detailed { interpreter, command, info, include } => {
                if let Some(include_scripts) = include {
                    for include_script in include_scripts {
                        run_script(scripts, include_script);
                    }
                }
                if let Some(info_msg) = info {
                    println!("{}", info_msg);
                }
                if let Some(cmd) = command {
                    let msg: String = format!("{} {}: {}", symbols::other_symbol::CHECK_MARK.glyph, "Running script", script_name);
                    println!("{}\n", msg);
                    execute_command(interpreter.as_deref(), cmd);
                }
            }
        }
    } else {
        println!("{} Script not found: {}", symbols::other_symbol::CROSS_MARK.glyph, script_name);
    }
}

/// Execute a command using the specified interpreter, or the default shell if none is specified.
/// 
/// # Arguments
///
/// * `interpreter` - An optional string representing the interpreter to use.
/// * `command` - The command to execute.
fn execute_command(interpreter: Option<&str>, command: &str) {
    match interpreter {
        Some("bash") => {
            Command::new("bash")
                .arg("-c")
                .arg(command)
                .status()
                .expect("Failed to execute script using bash");
        }
        Some("zsh") => {
            Command::new("zsh")
                .arg("-c")
                .arg(command)
                .status()
                .expect("Failed to execute script using zsh");
        }
        Some("powershell") => {
            Command::new("powershell")
                .args(&["-Command", command])
                .status()
                .expect("Failed to execute script using PowerShell");
        }
        Some("cmd") => {
            Command::new("cmd")
                .args(&["/C", command])
                .status()
                .expect("Failed to execute script using cmd");
        }
        Some(other) => {
            Command::new(other)
                .arg("-c")
                .arg(command)
                .status()
                .expect(&format!("Failed to execute script using {}", other));
        }
        None => {
            if cfg!(target_os = "windows") {
                Command::new("cmd")
                    .args(&["/C", command])
                    .status()
                    .expect("Failed to execute script using cmd");
            } else {
                Command::new("sh")
                    .arg("-c")
                    .arg(command)
                    .status()
                    .expect("Failed to execute script using sh");
            }
        }
    }
}

/// Initialize a Scripts.toml file in the current directory.
/// If the file already exists, prompt the user for confirmation to replace it.
pub fn init_script_file() {
    let file_path = "Scripts.toml";
    if fs::metadata(file_path).is_ok() {
        println!("{} {} already exists. Do you want to replace it? (y/n)", symbols::warning::WARNING.glyph, file_path);
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        if input.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return;
        }
    }
    let default_content = r#"
        dev = "cargo run"
        build = "cargo build" 
        release = "cargo build --release
        test = "cargo test"
        doc = "cargo doc --no-deps --open"
    "#;
    fs::write(file_path, default_content).expect("Failed to write Scripts.toml");
    println!("{} Scripts.toml has been created.", symbols::other_symbol::CHECK_MARK.glyph);
}
