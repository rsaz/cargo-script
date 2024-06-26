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

#[derive(Deserialize)]
pub struct Scripts {
    pub scripts: HashMap<String, Script>
}

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
i_am_shell = "./.scripts/i_am_shell.sh"
i_am_shell_obj = { interpreter = "bash", command = "./.scripts/i_am_shell.sh", info = "Detect shell script" }
build = "echo 'build'"
release = { include = ["i_am_shell", "build"] }
"#;
    fs::write(file_path, default_content).expect("Failed to write Scripts.toml");
    println!("{} Scripts.toml has been created.", symbols::other_symbol::CHECK_MARK.glyph);
}
