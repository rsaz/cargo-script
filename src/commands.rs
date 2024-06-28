//! This module defines the commands and their execution logic for the cargo-script CLI tool.
//!
//! It includes functionalities to run scripts, initialize the Scripts.toml file, and handle script execution.
use std::{collections::HashMap, env, fs, io, process::Command, sync::{Arc, Mutex}, time::{Duration, Instant}};
use clap::{Subcommand, ArgAction};
use serde::Deserialize;
use emoji::symbols;
use colored::*;

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

/// Enum representing a script, which can be either a default command or a detailed script with additional metadata.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    Default(String),
    Detailed {
        interpreter: Option<String>,
        command: Option<String>,
        info: Option<String>,
        env: Option<HashMap<String, String>>,
        include: Option<Vec<String>>,
    }
}

/// Struct representing the collection of scripts defined in Scripts.toml.
#[derive(Deserialize)]
pub struct Scripts {
    pub global_env: Option<HashMap<String, String>>,
    pub scripts: HashMap<String, Script>
}

/// Show all script names and descriptions in a table format.
/// 
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
pub fn show_scripts(scripts: &Scripts) {   
    let mut max_script_name_len = "Script".len();
    let mut max_description_len = "Description".len();

    for (name, script) in &scripts.scripts {
        max_script_name_len = max_script_name_len.max(name.len() + 2);
        let description = match script {
            Script::Default(_) => "",
            Script::Detailed { info, .. } => info.as_deref().unwrap_or(""),
        };
        max_description_len = max_description_len.max(description.len() + 2);
    }
   
    println!("{:<width1$} {:<width2$}", "Script".yellow(), "Description".yellow(), width1 = max_script_name_len, width2 = max_description_len);
    println!("{:<width1$} {:<width2$}", "-".repeat(max_script_name_len).yellow(), "-".repeat(max_description_len).yellow(), width1 = max_script_name_len, width2 = max_description_len);

    for (name, script) in &scripts.scripts {
        let description = match script {
            Script::Default(_) => "".to_string(),
            Script::Detailed { info, .. } => info.clone().unwrap_or_else(|| "".to_string()),
        };
        println!("{:<width1$} {:<width2$}", name.green(), description, width1 = max_script_name_len, width2 = max_description_len);
    }
}

/// Run a script by name, executing any included scripts in sequence.
/// 
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
/// * `script_name` - The name of the script to run.
/// * `env_overrides` - A vector of command line environment variable overrides.
pub fn run_script(scripts: &Scripts, script_name: &str, env_overrides: Vec<String>) {
    let script_durations = Arc::new(Mutex::new(HashMap::new()));

    fn run_script_with_level(
        scripts: &Scripts,
        script_name: &str,
        env_overrides: Vec<String>,
        level: usize,
        script_durations: Arc<Mutex<HashMap<String, Duration>>>,
    ) {
        let mut env_vars = scripts.global_env.clone().unwrap_or_default();
        let indent = "  ".repeat(level);

        let script_start_time = Instant::now();

        if let Some(script) = scripts.scripts.get(script_name) {
            match script {
                Script::Default(cmd) => {
                    let msg = format!(
                        "{}{}  {}: [ {} ]",
                        indent,
                        symbols::other_symbol::CHECK_MARK.glyph,
                        "Running script".green(),
                        script_name
                    );
                    println!("{}\n", msg);
                    apply_env_vars(&env_vars, &env_overrides);
                    execute_command(None, cmd);
                }
                Script::Detailed {
                    interpreter,
                    command,
                    info,
                    env,
                    include,
                } => {
                    let description = format!(
                        "{}  {}: {}",
                        emoji::objects::book_paper::BOOKMARK_TABS.glyph,
                        "Description".green(),
                        info.as_deref().unwrap_or("No description provided")
                    );

                    if let Some(include_scripts) = include {
                        let msg = format!(
                            "{}{}  {}: [ {} ]  {}",
                            indent,
                            symbols::other_symbol::CHECK_MARK.glyph,
                            "Running include script".green(),
                            script_name,
                            description
                        );
                        println!("{}\n", msg);
                        for include_script in include_scripts {
                            run_script_with_level(
                                scripts,
                                include_script,
                                env_overrides.clone(),
                                level + 1,
                                script_durations.clone(),
                            );
                        }
                    }

                    if let Some(cmd) = command {
                        let msg = format!(
                            "{}{}  {}: [ {} ]  {}",
                            indent,
                            symbols::other_symbol::CHECK_MARK.glyph,
                            "Running script".green(),
                            script_name,
                            description
                        );
                        println!("{}\n", msg);

                        if let Some(script_env) = env {
                            env_vars.extend(script_env.clone());
                        }
                        apply_env_vars(&env_vars, &env_overrides);
                        execute_command(interpreter.as_deref(), cmd);
                    }
                }
            }

            let script_duration = script_start_time.elapsed();
            if level > 0 || scripts.scripts.get(script_name).map_or(false, |s| matches!(s, Script::Default(_) | Script::Detailed { command: Some(_), .. })) {
                script_durations
                    .lock()
                    .unwrap()
                    .insert(script_name.to_string(), script_duration);
            }
        } else {
            println!(
                "{}{} {}: [ {} ]",
                indent,
                symbols::other_symbol::CROSS_MARK.glyph,
                "Script not found".red(),
                script_name
            );
        }
    }

    run_script_with_level(scripts, script_name, env_overrides, 0, script_durations.clone());

    let durations = script_durations.lock().unwrap();
    let total_duration: Duration = durations.values().cloned().sum();
    
    println!("\n");
    println!("{}", "Scripts Performance".bold().yellow());
    println!("{}", "-".repeat(80).yellow());
    for (script, duration) in durations.iter() {
        println!("✔️  Script: {:<25}  🕒 Running time: {:.2?}", script.green(), duration);
    }
    if !durations.is_empty() {
        println!("\n🕒 Total running time: {:.2?}", total_duration);
    }
}

/// Apply environment variables from global, script-specific, and command line overrides.
/// 
/// # Arguments
///
/// * `env_vars` - A reference to the global environment variables.
/// * `env_overrides` - A vector of command line environment variable overrides.
fn apply_env_vars(env_vars: &HashMap<String, String>, env_overrides: &[String]) {
    let mut final_env = env_vars.clone();

    for override_str in env_overrides {
        if let Some((key, value)) = override_str.split_once('=') {
            final_env.insert(key.to_string(), value.to_string());
        }
    }

    for (key, value) in &final_env {
        env::set_var(key, value);
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
        println!("{}  [ {} ] already exists. Do you want to replace it? ({}/{})", symbols::warning::WARNING.glyph, file_path.yellow(), "y".green(), "n".red());
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        if input.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return;
        }
    }
    let default_content = r#"
[global_env]

[scripts]
dev = "cargo run"
build = { command = "cargo build", env = { RUST_LOG = "info" } }
release = "cargo build --release"
test = { command = "cargo test", env = { RUST_LOG = "warn" } }
doc = "cargo doc --no-deps --open"
"#;
    fs::write(file_path, default_content).expect("Failed to write Scripts.toml");
    println!("{}  [ {} ] has been created.", symbols::other_symbol::CHECK_MARK.glyph, "Scripts.toml".green());
}
