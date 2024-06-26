use std::{collections::HashMap, fs, io, process::Command};
use clap::{Subcommand, ArgAction};
use serde::Deserialize;
use emoji::symbols;

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


#[derive(Deserialize)]
pub struct Scripts {
    pub scripts: HashMap<String, String>
}

pub fn run_script(script: &str) {
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
[scripts]
script1 = "echo Running script 1"
script2 = "echo Running script 2"
"#;
    fs::write(file_path, default_content).expect("Failed to write Scripts.toml");
    println!("{} Scripts.toml has been created.", symbols::other_symbol::CHECK_MARK.glyph);
}