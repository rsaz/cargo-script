//! This module provides the functionality to run scripts defined in `Scripts.toml`.

use std::{collections::HashMap, env, process::{Command, Stdio}, sync::{Arc, Mutex}, time::{Duration, Instant}};
use serde::Deserialize;
use emoji::symbols;
use colored::*;
use dialoguer::FuzzySelect;

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

use crate::error::{CargoScriptError, create_tool_not_found_error, create_toolchain_not_found_error};

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
/// * `dry_run` - If true, only show what would be executed without actually running it.
/// * `quiet` - If true, suppress all output except errors.
/// * `verbose` - If true, show detailed output.
/// * `show_metrics` - If true, show performance metrics after execution.
///
/// # Errors
///
/// Returns an error if the script is not found or if execution fails.
pub fn run_script(scripts: &Scripts, script_name: &str, env_overrides: Vec<String>, dry_run: bool, quiet: bool, verbose: bool, show_metrics: bool) -> Result<(), CargoScriptError> {
    if dry_run {
        if !quiet {
            println!("{}", "DRY-RUN MODE: Preview of what would be executed".bold().yellow());
            println!("{}\n", "=".repeat(80).yellow());
        }
        dry_run_script(scripts, script_name, env_overrides, 0, quiet, verbose)?;
        if !quiet {
            println!("\n{}", "No commands were actually executed.".italic().green());
        }
        return Ok(());
    }

    let script_durations = Arc::new(Mutex::new(HashMap::new()));

    fn run_script_with_level(
        scripts: &Scripts,
        script_name: &str,
        env_overrides: Vec<String>,
        level: usize,
        script_durations: Arc<Mutex<HashMap<String, Duration>>>,
        quiet: bool,
        verbose: bool,
    ) -> Result<(), CargoScriptError> {
        let mut env_vars = scripts.global_env.clone().unwrap_or_default();
        let indent = "  ".repeat(level);

        let script_start_time = Instant::now();

        if let Some(script) = scripts.scripts.get(script_name) {
            match script {
                Script::Default(cmd) => {
                    if !quiet {
                        let msg = format!(
                            "{}{}  {}: [ {} ]",
                            indent,
                            symbols::other_symbol::CHECK_MARK.glyph,
                            "Running script".green(),
                            script_name
                        );
                        println!("{}\n", msg);
                    }
                    let final_env = get_final_env(&env_vars, &env_overrides);
                    apply_env_vars(&env_vars, &env_overrides);
                    execute_command(script_name, None, cmd, None, &final_env)?;
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
                        return Err(e);
                    }

                    // Always show description unless quiet (not just in verbose mode)
                    let description = info.as_deref().map(|desc| {
                        format!(
                            "{}  {}: {}",
                            emoji::objects::book_paper::BOOKMARK_TABS.glyph,
                            "Description".green(),
                            desc
                        )
                    });

                    if let Some(include_scripts) = include {
                        if !quiet {
                            let desc_str = description.as_deref().unwrap_or("");
                            let msg = format!(
                                "{}{}  {}: [ {} ]{}",
                                indent,
                                symbols::other_symbol::CHECK_MARK.glyph,
                                "Running include script".green(),
                                script_name,
                                if desc_str.is_empty() { String::new() } else { format!("  {}", desc_str) }
                            );
                            println!("{}\n", msg);
                        }
                        for include_script in include_scripts {
                            run_script_with_level(
                                scripts,
                                include_script,
                                env_overrides.clone(),
                                level + 1,
                                script_durations.clone(),
                                quiet,
                                verbose,
                            )?;
                        }
                    }

                    if let Some(cmd) = command {
                        if !quiet {
                            let desc_str = description.as_deref().unwrap_or("");
                            let msg = format!(
                                "{}{}  {}: [ {} ]{}",
                                indent,
                                symbols::other_symbol::CHECK_MARK.glyph,
                                "Running script".green(),
                                script_name,
                                if desc_str.is_empty() { String::new() } else { format!("  {}", desc_str) }
                            );
                            println!("{}\n", msg);
                        }

                        if let Some(script_env) = env {
                            env_vars.extend(script_env.clone());
                        }
                        let final_env = get_final_env(&env_vars, &env_overrides);
                        apply_env_vars(&env_vars, &env_overrides);
                        execute_command(script_name, interpreter.as_deref(), cmd, toolchain.as_deref(), &final_env)?;
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
            Ok(())
        } else {
            let available_scripts: Vec<String> = scripts.scripts.keys().cloned().collect();
            return Err(CargoScriptError::ScriptNotFound {
                script_name: script_name.to_string(),
                available_scripts,
            });
        }
    }

    run_script_with_level(scripts, script_name, env_overrides, 0, script_durations.clone(), quiet, verbose)?;

    // Show performance metrics only if enabled and not in quiet mode
    if show_metrics && !quiet {
        let durations = script_durations.lock().unwrap();
        if !durations.is_empty() {
            let total_duration: Duration = durations.values().cloned().sum();
            
            println!("\n");
            println!("{}", "Scripts Performance".bold().yellow());
            println!("{}", "-".repeat(80).yellow());
            for (script, duration) in durations.iter() {
                println!("✔️  Script: {:<25}  🕒 Running time: {:.2?}", script.green(), duration);
            }
            println!("\n🕒 Total running time: {:.2?}", total_duration);
        }
    }
    
    Ok(())
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
/// * `script_name` - The name of the script being executed (for error messages).
/// * `interpreter` - An optional string representing the interpreter to use.
/// * `command` - The command to execute.
/// * `toolchain` - An optional string representing the toolchain to use.
/// * `env_vars` - A reference to the environment variables to set for the command.
///
/// # Errors
///
/// Returns an error if it fails to execute the command.
fn execute_command(script_name: &str, interpreter: Option<&str>, command: &str, toolchain: Option<&str>, env_vars: &HashMap<String, String>) -> Result<(), CargoScriptError> {
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
            .map_err(|e| CargoScriptError::ExecutionError {
                script: script_name.to_string(),
                command: command.to_string(),
                source: e,
            })?
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
                    .map_err(|e| CargoScriptError::ExecutionError {
                        script: "unknown".to_string(),
                        command: command.to_string(),
                        source: e,
                    })?
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
                    .map_err(|e| CargoScriptError::ExecutionError {
                        script: "unknown".to_string(),
                        command: command.to_string(),
                        source: e,
                    })?
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
                    .map_err(|e| CargoScriptError::ExecutionError {
                        script: "unknown".to_string(),
                        command: command.to_string(),
                        source: e,
                    })?
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
                    .map_err(|e| CargoScriptError::ExecutionError {
                        script: "unknown".to_string(),
                        command: command.to_string(),
                        source: e,
                    })?
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
                    .map_err(|e| CargoScriptError::ExecutionError {
                        script: "unknown".to_string(),
                        command: command.to_string(),
                        source: e,
                    })?
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
                        .map_err(|e| CargoScriptError::ExecutionError {
                            script: "unknown".to_string(),
                            command: command.to_string(),
                            source: e,
                        })?
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
                        .map_err(|e| CargoScriptError::ExecutionError {
                            script: "unknown".to_string(),
                            command: command.to_string(),
                            source: e,
                        })?
                }
            }
        }
    };

    let exit_status = cmd.wait().map_err(|e| CargoScriptError::ExecutionError {
        script: script_name.to_string(),
        command: command.to_string(),
        source: e,
    })?;
    
    // Check if command failed and might be a Windows self-replacement issue
    if !exit_status.success() {
        let is_self_replace_attempt = cfg!(target_os = "windows")
            && (command.contains("cargo install --path .") 
                || command.contains("cargo install --path")
                || (command.contains("cargo install") && command.contains("--path")));
        
        if is_self_replace_attempt {
            return Err(CargoScriptError::WindowsSelfReplacementError {
                script: script_name.to_string(),
                command: command.to_string(),
            });
        }
    }
    
    Ok(())
}

/// Display what would be executed in dry-run mode without actually running anything.
///
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
/// * `script_name` - The name of the script to preview.
/// * `env_overrides` - A vector of command line environment variable overrides.
/// * `level` - The nesting level for indentation.
/// * `quiet` - If true, suppress all output except errors.
/// * `verbose` - If true, show detailed output.
fn dry_run_script(
    scripts: &Scripts,
    script_name: &str,
    env_overrides: Vec<String>,
    level: usize,
    quiet: bool,
    verbose: bool,
) -> Result<(), CargoScriptError> {
    let indent = "  ".repeat(level);
    let mut env_vars = scripts.global_env.clone().unwrap_or_default();

    if let Some(script) = scripts.scripts.get(script_name) {
        match script {
            Script::Default(cmd) => {
                if !quiet {
                    println!(
                        "{}{}  {}: [ {} ]",
                        indent,
                        "📋".yellow(),
                        "Would run script".cyan(),
                        script_name.bold()
                    );
                    println!("{}    Command: {}", indent, cmd.green());
                    let final_env = get_final_env(&env_vars, &env_overrides);
                    // In dry-run mode, always show environment variables (unless quiet)
                    if !final_env.is_empty() {
                        println!("{}    Environment variables:", indent);
                        for (key, value) in &final_env {
                            println!("{}      {} = {}", indent, key.cyan(), value.green());
                        }
                    }
                    if level == 0 {
                        println!(); // Extra spacing for top-level scripts
                    }
                }
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
                if !quiet {
                    // Check requirements (but don't fail in dry-run, just warn)
                    if verbose {
                        if let Some(reqs) = requires {
                            if !reqs.is_empty() {
                                println!(
                                    "{}{}  {}: [ {} ]",
                                    indent,
                                    "🔍".yellow(),
                                    "Would check requirements".cyan(),
                                    script_name.bold()
                                );
                                for req in reqs {
                                    println!("{}      - {}", indent, req.green());
                                }
                                println!();
                            }
                        }
                    }

                    if verbose {
                        if let Some(tc) = toolchain {
                            println!(
                                "{}{}  {}: {}",
                                indent,
                                "🔧".yellow(),
                                "Would use toolchain".cyan(),
                                tc.bold().green()
                            );
                            println!();
                        }
                    }

                    if verbose {
                        if let Some(desc) = info {
                            println!(
                                "{}{}  {}: {}",
                                indent,
                                "📝".yellow(),
                                "Description".cyan(),
                                desc.green()
                            );
                            println!();
                        }
                    }

                    if let Some(include_scripts) = include {
                        println!(
                            "{}{}  {}: [ {} ]",
                            indent,
                            "📋".yellow(),
                            "Would run include scripts".cyan(),
                            script_name.bold()
                        );
                        if verbose {
                            if let Some(desc) = info {
                                println!("{}    Description: {}", indent, desc.green());
                            }
                        }
                        println!();
                        for include_script in include_scripts {
                            dry_run_script(scripts, include_script, env_overrides.clone(), level + 1, quiet, verbose)?;
                        }
                    }

                    if let Some(cmd) = command {
                        println!(
                            "{}{}  {}: [ {} ]",
                            indent,
                            "📋".yellow(),
                            "Would run script".cyan(),
                            script_name.bold()
                        );
                        
                        // In dry-run mode, always show interpreter and toolchain (unless quiet)
                        if let Some(interp) = interpreter {
                            println!("{}    Interpreter: {}", indent, interp.green());
                        }
                        
                        if let Some(tc) = toolchain {
                            println!("{}    Toolchain: {}", indent, tc.green());
                        }
                        
                        println!("{}    Command: {}", indent, cmd.green());
                        
                        if let Some(script_env) = env {
                            env_vars.extend(script_env.clone());
                        }
                        
                        let final_env = get_final_env(&env_vars, &env_overrides);
                        // In dry-run mode, always show environment variables (unless quiet)
                        if !final_env.is_empty() {
                            println!("{}    Environment variables:", indent);
                            for (key, value) in &final_env {
                                println!("{}      {} = {}", indent, key.cyan(), value.green());
                            }
                        }
                        if level == 0 {
                            println!(); // Extra spacing for top-level scripts
                        }
                    }
                } else {
                    // Even in quiet mode, we need to process includes
                    if let Some(include_scripts) = include {
                        for include_script in include_scripts {
                            dry_run_script(scripts, include_script, env_overrides.clone(), level + 1, quiet, verbose)?;
                        }
                    }
                }
            }
        }
    } else {
        let available_scripts: Vec<String> = scripts.scripts.keys().cloned().collect();
        return Err(CargoScriptError::ScriptNotFound {
            script_name: script_name.to_string(),
            available_scripts,
        });
    }
    
    Ok(())
}

/// Interactive script selection using fuzzy finder.
///
/// This function displays an interactive fuzzy selector for choosing a script to run.
///
/// # Arguments
///
/// * `scripts` - A reference to the collection of scripts.
/// * `quiet` - If true, suppress extra output.
///
/// # Returns
///
/// The selected script name, or an error if selection was cancelled or failed.
///
/// # Errors
///
/// Returns an error if no scripts are available or if the selection was cancelled.
pub fn interactive_select_script(scripts: &Scripts, quiet: bool) -> Result<String, CargoScriptError> {
    if scripts.scripts.is_empty() {
        return Err(CargoScriptError::ScriptNotFound {
            script_name: "".to_string(),
            available_scripts: vec![],
        });
    }

    // Prepare script items with descriptions for display
    let mut items: Vec<(String, String)> = scripts.scripts
        .iter()
        .map(|(name, script)| {
            let description = match script {
                Script::Default(_) => "".to_string(),
                Script::Inline { info, .. } | Script::CILike { info, .. } => {
                    info.clone().unwrap_or_else(|| "".to_string())
                }
            };
            (name.clone(), description)
        })
        .collect();
    
    // Sort by name for consistent display
    items.sort_by(|a, b| a.0.cmp(&b.0));

    // Format items for display
    let display_items: Vec<String> = items
        .iter()
        .map(|(name, desc)| {
            if desc.is_empty() {
                name.clone()
            } else {
                format!("{} - {}", name, desc)
            }
        })
        .collect();

    if !quiet {
        println!("{}", "Select a script to run:".cyan().bold());
        println!();
    }

    let selection = FuzzySelect::new()
        .with_prompt("Script")
        .items(&display_items)
        .default(0)
        .interact()
        .map_err(|e| CargoScriptError::ExecutionError {
            script: "interactive".to_string(),
            command: "fuzzy_select".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::Other, format!("Interactive selection failed: {}", e)),
        })?;

    Ok(items[selection].0.clone())
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
/// An empty result if all requirements are met, otherwise an error.
/// 
/// # Errors
/// 
/// This function will return an error if any of the requirements are not met.
fn check_requirements(requires: &[String], toolchain: Option<&String>) -> Result<(), CargoScriptError> {
    for req in requires {
        if let Some((tool, version)) = req.split_once(' ') {
            let output = Command::new(tool)
                .arg("--version")
                .output()
                .map_err(|_| create_tool_not_found_error(tool, Some(version)))?;
            let output_str = String::from_utf8_lossy(&output.stdout);

            if !output_str.contains(version) {
                return Err(create_tool_not_found_error(tool, Some(version)));
            }
        } else {
            // Just check if the tool is installed
            Command::new(req)
                .output()
                .map_err(|_| create_tool_not_found_error(req, None))?;
        }
    }

    if let Some(tc) = toolchain {
        let output = Command::new("rustup")
            .arg("toolchain")
            .arg("list")
            .output()
            .map_err(|_| create_tool_not_found_error("rustup", None))?;
        let output_str = String::from_utf8_lossy(&output.stdout);

        if !output_str.contains(tc) {
            return Err(create_toolchain_not_found_error(tc));
        }
    }

    Ok(())
}