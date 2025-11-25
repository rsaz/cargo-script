//! This module provides the functionality to run scripts defined in `Scripts.toml`.

use std::{collections::HashMap, env, process::{Command, Stdio}, sync::{Arc, Mutex}, time::{Duration, Instant}};
use serde::Deserialize;
use emoji::symbols;
use colored::*;

/// Enum representing a script, which can be either a default command or a detailed script with additional metadata.
#[derive(Deserialize, Debug)]
#[serde(untagged)]
pub enum Script {
    Default(String),
    Inline {
        command: Option<String>,
        requires: Option<Vec<String>>,
        toolchain: Option<String>,
        info: Option<String>,
        env: Option<HashMap<String, String>>,
        include: Option<Vec<String>>,
        interpreter: Option<String>,
    },
    CILike {
        script: String,
        command: Option<String>,
        requires: Option<Vec<String>>,
        toolchain: Option<String>,
        info: Option<String>,
        env: Option<HashMap<String, String>>,
        include: Option<Vec<String>>,
        interpreter: Option<String>,
    }
}

/// Struct representing the collection of scripts defined in Scripts.toml.
#[derive(Deserialize)]
pub struct Scripts {
    pub global_env: Option<HashMap<String, String>>,
    pub scripts: HashMap<String, Script>
}

/// Run a script by name, executing any included scripts in sequence.
///
/// This function runs a script and any scripts it includes, measuring the execution time
/// for each script and printing performance metrics.
///
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
/// * `script_name` - The name of the script to run.
/// * `env_overrides` - A vector of command line environment variable overrides.
///
/// # Panics
///
/// This function will panic if it fails to execute the script commands.
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
                    let final_env = get_final_env(&env_vars, &env_overrides);
                    apply_env_vars(&env_vars, &env_overrides);
                    execute_command(None, cmd, None, &final_env);
                }
                Script::Inline {
                    command,
                    info,
                    env,
                    include,
                    interpreter,
                    requires,
                    toolchain,
                    ..
                } | Script::CILike {
                    command,
                    info,
                    env,
                    include,
                    interpreter,
                    requires,
                    toolchain,
                    ..
                } => {
                    if let Err(e) = check_requirements(requires.as_deref().unwrap_or(&[]), toolchain.as_ref()) {
                        eprintln!("{} {}: {}", symbols::other_symbol::CROSS_MARK.glyph, "Requirement check failed".red(), e);
                        return;
                    }

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
                        let final_env = get_final_env(&env_vars, &env_overrides);
                        apply_env_vars(&env_vars, &env_overrides);
                        execute_command(interpreter.as_deref(), cmd, toolchain.as_deref(), &final_env);
                    }
                }
            }

            let script_duration = script_start_time.elapsed();
            if level > 0 || scripts.scripts.get(script_name).map_or(false, |s| matches!(s, Script::Default(_) | Script::Inline { command: Some(_), .. } | Script::CILike { command: Some(_), .. })) {
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
    if !durations.is_empty() {
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
}


/// Get the final environment variables map from global, script-specific, and command line overrides.
///
/// This function computes the final environment variables, giving precedence
/// to command line overrides over script-specific variables, and script-specific variables over global variables.
///
/// # Arguments
///
/// * `env_vars` - A reference to the global environment variables.
/// * `env_overrides` - A vector of command line environment variable overrides.
///
/// # Returns
///
/// A HashMap containing the final environment variables.
fn get_final_env(env_vars: &HashMap<String, String>, env_overrides: &[String]) -> HashMap<String, String> {
    let mut final_env = env_vars.clone();

    for override_str in env_overrides {
        if let Some((key, value)) = override_str.split_once('=') {
            final_env.insert(key.to_string(), value.to_string());
        }
    }

    final_env
}

/// Apply environment variables from global, script-specific, and command line overrides.
///
/// This function sets the environment variables for the script execution, giving precedence
/// to command line overrides over script-specific variables, and script-specific variables over global variables.
///
/// # Arguments
///
/// * `env_vars` - A reference to the global environment variables.
/// * `env_overrides` - A vector of command line environment variable overrides.
fn apply_env_vars(env_vars: &HashMap<String, String>, env_overrides: &[String]) {
    let final_env = get_final_env(env_vars, env_overrides);

    for (key, value) in &final_env {
        // SAFETY: Setting environment variables for child processes is safe.
        // We're in a single-threaded context when setting these variables,
        // and they're only used for the child process spawned immediately after.
        unsafe {
            env::set_var(key, value);
        }
    }
}

/// Execute a command using the specified interpreter, or the default shell if none is specified.
///
/// This function runs the command with the appropriate interpreter, depending on the operating system
/// and the specified interpreter.
///
/// # Arguments
///
/// * `interpreter` - An optional string representing the interpreter to use.
/// * `command` - The command to execute.
/// * `toolchain` - An optional string representing the toolchain to use.
/// * `env_vars` - A reference to the environment variables to set for the command.
///
/// # Panics
///
/// This function will panic if it fails to execute the command.
fn execute_command(interpreter: Option<&str>, command: &str, toolchain: Option<&str>, env_vars: &HashMap<String, String>) {
    let mut cmd = if let Some(tc) = toolchain {
        let mut command_with_toolchain = format!("cargo +{} ", tc);
        command_with_toolchain.push_str(command);
        let mut cmd = Command::new("sh");
        cmd.arg("-c")
            .arg(command_with_toolchain)
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit());
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        cmd.spawn()
            .expect("Failed to execute command")
    } else {
        match interpreter {
            Some("bash") => {
                let mut cmd = Command::new("bash");
                cmd.arg("-c")
                    .arg(command)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
                cmd.spawn()
                    .expect("Failed to execute script using bash")
            },
            Some("zsh") => {
                let mut cmd = Command::new("zsh");
                cmd.arg("-c")
                    .arg(command)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
                cmd.spawn()
                    .expect("Failed to execute script using zsh")
            },
            Some("powershell") => {
                let mut cmd = Command::new("powershell");
                cmd.args(&["-NoProfile", "-Command", command])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
                cmd.spawn()
                    .expect("Failed to execute script using PowerShell")
            },
            Some("cmd") => {
                let mut cmd = Command::new("cmd");
                cmd.args(&["/C", command])
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
                cmd.spawn()
                    .expect("Failed to execute script using cmd")
            },
            Some(other) => {
                let mut cmd = Command::new(other);
                cmd.arg("-c")
                    .arg(command)
                    .stdout(Stdio::inherit())
                    .stderr(Stdio::inherit());
                for (key, value) in env_vars {
                    cmd.env(key, value);
                }
                cmd.spawn()
                    .expect(&format!("Failed to execute script using {}", other))
            },
            None => {
                if cfg!(target_os = "windows") {
                    let mut cmd = Command::new("cmd");
                    cmd.args(&["/C", command])
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());
                    for (key, value) in env_vars {
                        cmd.env(key, value);
                    }
                    cmd.spawn()
                        .expect("Failed to execute script using cmd")
                } else {
                    let mut cmd = Command::new("sh");
                    cmd.arg("-c")
                        .arg(command)
                        .stdout(Stdio::inherit())
                        .stderr(Stdio::inherit());
                    for (key, value) in env_vars {
                        cmd.env(key, value);
                    }
                    cmd.spawn()
                        .expect("Failed to execute script using sh")
                }
            }
        }
    };

    cmd.wait().expect("Command wasn't running");
}

/// Check if the required tools and toolchain are installed.
/// 
/// This function checks if the required tools and toolchain are installed on the system.
/// If any of the requirements are not met, an error message is returned.
/// 
/// # Arguments
/// 
/// * `requires` - A slice of strings representing the required tools.
/// * `toolchain` - An optional string representing the required toolchain.
/// 
/// # Returns
/// 
/// An empty result if all requirements are met, otherwise an error message.
/// 
/// # Errors
/// 
/// This function will return an error message if any of the requirements are not met.
fn check_requirements(requires: &[String], toolchain: Option<&String>) -> Result<(), String> {
    for req in requires {
        if let Some((tool, version)) = req.split_once(' ') {
            let output = Command::new(tool)
                .arg("--version")
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", tool, e))?;
            let output_str = String::from_utf8_lossy(&output.stdout);

            if !output_str.contains(version) {
                return Err(format!(
                    "Required version for {} is {}, but found {}",
                    tool, version, output_str
                ));
            }
        } else {
            // Just check if the tool is installed
            Command::new(req)
                .output()
                .map_err(|e| format!("Failed to execute {}: {}", req, e))?;
        }
    }

    if let Some(toolchain) = toolchain {
        let output = Command::new("rustup")
            .arg("toolchain")
            .arg("list")
            .output()
            .map_err(|e| format!("Failed to execute rustup: {}", e))?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        if !output_str.contains(toolchain) {
            return Err(format!("Required toolchain {} is not installed", toolchain));
        }
    }

    Ok(())
}