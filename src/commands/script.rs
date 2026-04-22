//! This module provides the functionality to run scripts defined in `Scripts.toml`.
//!
//! In addition to the original execution engine, this module now supports:
//!
//! * Workspace-aware execution (run a script in every workspace member, in
//!   parallel or sequentially).
//! * Cargo script (single-file Rust packages, RFC 3502) integration: any
//!   command pointing at a `.rs` file is executed via `cargo +nightly -Zscript`
//!   (or stable `cargo` when available).
//! * Pre/post/on_success/on_failure hooks.
//! * Parameterized commands using `{{arg}}` template substitution and
//!   key=value CLI arguments.
//! * Optional structured JSON output.

use std::{
    collections::HashMap,
    env,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use serde::Deserialize;
use emoji::symbols;
use colored::*;
use dialoguer::FuzzySelect;

/// Workspace execution mode for a single script.
#[derive(Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum WorkspaceMode {
    /// Execute the script once in every workspace member sequentially.
    All,
    /// Execute the script in every workspace member in parallel.
    Parallel,
    /// Execute the script only at the workspace root (default behaviour).
    Root,
}

/// Top-level workspace configuration declared inside `Scripts.toml`.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct WorkspaceConfig {
    /// Optional explicit list of members. When omitted, members are discovered
    /// from `Cargo.toml`.
    pub members: Option<Vec<String>>,
    /// Members to exclude from workspace runs.
    pub exclude: Option<Vec<String>>,
}

/// Enum representing a script. Backward compatible with all v0.5 layouts.
#[derive(Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Script {
    /// Simple form: `name = "command"`.
    Default(String),
    /// Inline / table form. All fields are optional to remain backward
    /// compatible. Scripts that previously parsed as `Inline` keep parsing
    /// the same way; new fields are additive.
    Inline {
        command: Option<String>,
        requires: Option<Vec<String>>,
        toolchain: Option<String>,
        info: Option<String>,
        env: Option<HashMap<String, String>>,
        include: Option<Vec<String>>,
        interpreter: Option<String>,
        // ---- NEW FIELDS (all opt-in / backward compatible) -----------------
        /// Explicit cargo-script (single-file `.rs`) path.
        script_file: Option<String>,
        /// Names of CLI arguments accepted by this script (used with
        /// `{{name}}` substitution in `command`).
        args: Option<Vec<String>>,
        /// Default values for `args`.
        defaults: Option<HashMap<String, String>>,
        /// Workspace execution mode.
        workspace: Option<WorkspaceMode>,
        /// Hooks: scripts run before the main command.
        pre: Option<Vec<String>>,
        /// Hooks: scripts run after the main command (always).
        post: Option<Vec<String>>,
        /// Hooks: scripts run only when the main command succeeds.
        on_success: Option<Vec<String>>,
        /// Hooks: scripts run only when the main command fails.
        on_failure: Option<Vec<String>>,
    },
    /// CI-like layout, retained for backward compatibility.
    CILike {
        script: String,
        command: Option<String>,
        requires: Option<Vec<String>>,
        toolchain: Option<String>,
        info: Option<String>,
        env: Option<HashMap<String, String>>,
        include: Option<Vec<String>>,
        interpreter: Option<String>,
        // NEW FIELDS - mirror Inline
        script_file: Option<String>,
        args: Option<Vec<String>>,
        defaults: Option<HashMap<String, String>>,
        workspace: Option<WorkspaceMode>,
        pre: Option<Vec<String>>,
        post: Option<Vec<String>>,
        on_success: Option<Vec<String>>,
        on_failure: Option<Vec<String>>,
    }
}

/// Struct representing the collection of scripts defined in `Scripts.toml`.
#[derive(Deserialize, Debug, Clone, Default)]
pub struct Scripts {
    pub global_env: Option<HashMap<String, String>>,
    #[serde(default)]
    pub scripts: HashMap<String, Script>,
    /// Optional workspace configuration.
    pub workspace: Option<WorkspaceConfig>,
}

use crate::error::{CargoScriptError, create_tool_not_found_error, create_toolchain_not_found_error};

// ---------------------------------------------------------------------------
// Public options & result types
// ---------------------------------------------------------------------------

/// Bundle of all execution options for `run_script`. Designed so callers do
/// not have to change every signature when new options are added.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    pub env_overrides: Vec<String>,
    pub dry_run: bool,
    pub quiet: bool,
    pub verbose: bool,
    pub show_metrics: bool,
    /// CLI parameters in `key=value` form (or positional, see [`parse_script_args`]).
    pub script_args: Vec<String>,
    /// Explicit workspace mode override from CLI.
    pub workspace_override: Option<WorkspaceMode>,
    /// When true, ignore any workspace mode declared in the script.
    pub no_workspace: bool,
    /// When true, emit results as a JSON document on stdout.
    pub json_output: bool,
}

impl RunOptions {
    pub fn new() -> Self {
        Self {
            show_metrics: true,
            ..Default::default()
        }
    }
}

/// Structured result of executing a script (used for JSON output).
#[derive(Debug, Clone, serde::Serialize)]
pub struct ExecutionResult {
    pub success: bool,
    pub script: String,
    pub command: Option<String>,
    pub duration_ms: u128,
    pub exit_code: Option<i32>,
    pub error: Option<String>,
    pub includes: Vec<ExecutionResult>,
    pub hooks: Vec<ExecutionResult>,
}

impl ExecutionResult {
    pub fn new(script: &str) -> Self {
        Self {
            success: true,
            script: script.to_string(),
            command: None,
            duration_ms: 0,
            exit_code: None,
            error: None,
            includes: Vec::new(),
            hooks: Vec::new(),
        }
    }
}

// ---------------------------------------------------------------------------
// Script field accessors (avoid duplicating match arms everywhere)
// ---------------------------------------------------------------------------

/// Borrowed view over the (extended) fields of a script. Lets callers handle
/// `Inline` and `CILike` uniformly.
pub struct ScriptView<'a> {
    pub command: Option<&'a String>,
    pub requires: Option<&'a Vec<String>>,
    pub toolchain: Option<&'a String>,
    pub info: Option<&'a String>,
    pub env: Option<&'a HashMap<String, String>>,
    pub include: Option<&'a Vec<String>>,
    pub interpreter: Option<&'a String>,
    pub script_file: Option<&'a String>,
    pub args: Option<&'a Vec<String>>,
    pub defaults: Option<&'a HashMap<String, String>>,
    pub workspace: Option<WorkspaceMode>,
    pub pre: Option<&'a Vec<String>>,
    pub post: Option<&'a Vec<String>>,
    pub on_success: Option<&'a Vec<String>>,
    pub on_failure: Option<&'a Vec<String>>,
}

pub fn view(script: &Script) -> Option<ScriptView<'_>> {
    match script {
        Script::Default(_) => None,
        Script::Inline { command, requires, toolchain, info, env, include, interpreter, script_file, args, defaults, workspace, pre, post, on_success, on_failure }
        | Script::CILike { command, requires, toolchain, info, env, include, interpreter, script_file, args, defaults, workspace, pre, post, on_success, on_failure, .. } => {
            Some(ScriptView {
                command: command.as_ref(),
                requires: requires.as_ref(),
                toolchain: toolchain.as_ref(),
                info: info.as_ref(),
                env: env.as_ref(),
                include: include.as_ref(),
                interpreter: interpreter.as_ref(),
                script_file: script_file.as_ref(),
                args: args.as_ref(),
                defaults: defaults.as_ref(),
                workspace: *workspace,
                pre: pre.as_ref(),
                post: post.as_ref(),
                on_success: on_success.as_ref(),
                on_failure: on_failure.as_ref(),
            })
        }
    }
}

// ---------------------------------------------------------------------------
// Parameter substitution
// ---------------------------------------------------------------------------

/// Parse user-supplied script arguments. Supports two styles, freely mixed:
///
/// * `key=value` — explicit named argument.
/// * `value` — positional, mapped to the script's `args` definition by index.
pub fn parse_script_args(view: &ScriptView<'_>, raw: &[String]) -> HashMap<String, String> {
    let mut out: HashMap<String, String> = HashMap::new();
    if let Some(defaults) = view.defaults {
        out.extend(defaults.iter().map(|(k, v)| (k.clone(), v.clone())));
    }

    let names: Vec<String> = view
        .args
        .map(|a| a.iter().cloned().collect())
        .unwrap_or_default();
    let mut positional_idx = 0usize;

    for token in raw {
        if let Some((k, v)) = token.split_once('=') {
            out.insert(k.to_string(), v.to_string());
        } else if positional_idx < names.len() {
            out.insert(names[positional_idx].clone(), token.clone());
            positional_idx += 1;
        }
    }
    out
}

/// Substitute `{{name}}` placeholders inside `command` with values from `args`.
pub fn substitute_variables(command: &str, args: &HashMap<String, String>) -> String {
    let mut result = command.to_string();
    for (key, value) in args {
        result = result.replace(&format!("{{{{{}}}}}", key), value);
    }
    result
}

/// Verify that every required argument (declared via `args`) has a value.
pub fn check_required_args(
    script_name: &str,
    view: &ScriptView<'_>,
    resolved: &HashMap<String, String>,
) -> Result<(), CargoScriptError> {
    if let Some(arg_names) = view.args {
        for name in arg_names {
            if !resolved.contains_key(name) {
                return Err(CargoScriptError::MissingScriptArgument {
                    script_name: script_name.to_string(),
                    argument: name.clone(),
                });
            }
        }
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Top-level run_script
// ---------------------------------------------------------------------------

/// Run a script by name with the legacy 7-argument signature. Kept as a thin
/// wrapper around the new options-based API for backward compatibility with
/// any external callers.
pub fn run_script(
    scripts: &Scripts,
    script_name: &str,
    env_overrides: Vec<String>,
    dry_run: bool,
    quiet: bool,
    verbose: bool,
    show_metrics: bool,
) -> Result<(), CargoScriptError> {
    let opts = RunOptions {
        env_overrides,
        dry_run,
        quiet,
        verbose,
        show_metrics,
        ..RunOptions::default()
    };
    run_script_with_options(scripts, script_name, &opts).map(|_| ())
}

/// Run a script with full execution options. Returns the structured
/// [`ExecutionResult`] which is useful for JSON output and for tests.
pub fn run_script_with_options(
    scripts: &Scripts,
    script_name: &str,
    opts: &RunOptions,
) -> Result<ExecutionResult, CargoScriptError> {
    if opts.dry_run {
        if !opts.quiet && !opts.json_output {
            println!("{}", "DRY-RUN MODE: Preview of what would be executed".bold().yellow());
            println!("{}\n", "=".repeat(80).yellow());
        }
        dry_run_script(scripts, script_name, opts.env_overrides.clone(), 0, opts.quiet, opts.verbose)?;
        if !opts.quiet && !opts.json_output {
            println!("\n{}", "No commands were actually executed.".italic().green());
        }
        return Ok(ExecutionResult::new(script_name));
    }

    // Workspace dispatch: when the script (or CLI) requests a workspace mode
    // other than `Root`, hand off to the workspace executor.
    if !opts.no_workspace {
        let mode = opts.workspace_override.or_else(|| {
            scripts
                .scripts
                .get(script_name)
                .and_then(view)
                .and_then(|v| v.workspace)
        });
        if matches!(mode, Some(WorkspaceMode::All) | Some(WorkspaceMode::Parallel)) {
            return crate::commands::workspace::run_in_workspace(
                scripts,
                script_name,
                mode.unwrap(),
                opts,
            );
        }
    }

    let script_durations = Arc::new(Mutex::new(HashMap::new()));
    let result = run_script_with_level(
        scripts,
        script_name,
        opts.env_overrides.clone(),
        0,
        script_durations.clone(),
        opts.quiet,
        opts.verbose,
        &opts.script_args,
    )?;

    if opts.show_metrics && !opts.quiet && !opts.json_output {
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

    if opts.json_output {
        crate::output::json::print_execution_result(&result);
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Recursive script execution
// ---------------------------------------------------------------------------

/// The main recursive executor. Handles includes, hooks, parameters, cargo
/// scripts and regular shell commands.
fn run_script_with_level(
    scripts: &Scripts,
    script_name: &str,
    env_overrides: Vec<String>,
    level: usize,
    script_durations: Arc<Mutex<HashMap<String, Duration>>>,
    quiet: bool,
    verbose: bool,
    script_args: &[String],
) -> Result<ExecutionResult, CargoScriptError> {
    let mut env_vars = scripts.global_env.clone().unwrap_or_default();
    let indent = "  ".repeat(level);
    let script_start_time = Instant::now();
    let mut result = ExecutionResult::new(script_name);

    let script = scripts.scripts.get(script_name).ok_or_else(|| {
        let available_scripts: Vec<String> = scripts.scripts.keys().cloned().collect();
        CargoScriptError::ScriptNotFound {
            script_name: script_name.to_string(),
            available_scripts,
        }
    })?;

    match script {
        Script::Default(cmd) => {
            // Backward-compatible behaviour, plus auto-detection of cargo
            // script files (any command literally pointing at an existing
            // `.rs` file is executed via cargo-script).
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

            let trimmed = cmd.trim();
            if is_cargo_script_path(trimmed) {
                result.command = Some(trimmed.to_string());
                crate::commands::cargo_script::execute_cargo_script(
                    script_name,
                    trimmed,
                    &[],
                    &final_env,
                )?;
            } else {
                result.command = Some(cmd.clone());
                execute_command(script_name, None, cmd, None, &final_env)?;
            }
        }
        Script::Inline { .. } | Script::CILike { .. } => {
            let v = view(script).expect("non-default scripts have a view");

            if let Err(e) = check_requirements(
                v.requires.map(|r| r.as_slice()).unwrap_or(&[]),
                v.toolchain,
            ) {
                return Err(e);
            }

            // Resolve parameters early so they apply uniformly.
            let resolved_args = parse_script_args(&v, script_args);
            check_required_args(script_name, &v, &resolved_args)?;

            // Always show description unless quiet.
            let description = v.info.as_ref().map(|desc| {
                format!(
                    "{}  {}: {}",
                    emoji::objects::book_paper::BOOKMARK_TABS.glyph,
                    "Description".green(),
                    desc
                )
            });

            // -------- pre hooks --------
            if let Some(pre_hooks) = v.pre {
                for hook in pre_hooks {
                    let hook_res = run_script_with_level(
                        scripts,
                        hook,
                        env_overrides.clone(),
                        level + 1,
                        script_durations.clone(),
                        quiet,
                        verbose,
                        &[],
                    )
                    .map_err(|e| CargoScriptError::HookFailed {
                        hook_name: hook.clone(),
                        script_name: script_name.to_string(),
                        reason: e.to_string(),
                    })?;
                    result.hooks.push(hook_res);
                }
            }

            // -------- includes --------
            if let Some(include_scripts) = v.include {
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
                    let inc_res = run_script_with_level(
                        scripts,
                        include_script,
                        env_overrides.clone(),
                        level + 1,
                        script_durations.clone(),
                        quiet,
                        verbose,
                        &[],
                    )?;
                    result.includes.push(inc_res);
                }
            }

            // Determine env for the main command
            if let Some(script_env) = v.env {
                env_vars.extend(script_env.clone());
            }
            let final_env = get_final_env(&env_vars, &env_overrides);
            apply_env_vars(&env_vars, &env_overrides);

            // -------- main command / cargo-script --------
            let main_outcome: Result<(), CargoScriptError> = if let Some(path) = v.script_file {
                if !quiet {
                    let desc_str = description.as_deref().unwrap_or("");
                    println!(
                        "{}{}  {}: [ {} ]{}",
                        indent,
                        symbols::other_symbol::CHECK_MARK.glyph,
                        "Running cargo script".green(),
                        script_name,
                        if desc_str.is_empty() { String::new() } else { format!("  {}", desc_str) }
                    );
                    println!();
                }
                result.command = Some(path.clone());
                let extra_args: Vec<String> = script_args.to_vec();
                crate::commands::cargo_script::execute_cargo_script(
                    script_name,
                    path,
                    &extra_args,
                    &final_env,
                )
            } else if let Some(cmd) = v.command {
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

                let cmd_substituted = substitute_variables(cmd, &resolved_args);
                let trimmed = cmd_substituted.trim();
                result.command = Some(cmd_substituted.clone());

                if is_cargo_script_path(trimmed) {
                    crate::commands::cargo_script::execute_cargo_script(
                        script_name,
                        trimmed,
                        &[],
                        &final_env,
                    )
                } else {
                    execute_command(
                        script_name,
                        v.interpreter.map(|s| s.as_str()),
                        &cmd_substituted,
                        v.toolchain.map(|s| s.as_str()),
                        &final_env,
                    )
                }
            } else {
                // No command and no script_file - includes-only is fine.
                Ok(())
            };

            // -------- post / on_success / on_failure hooks --------
            // Post hooks always run (even on failure) for cleanup parity with
            // tools like make / npm scripts.
            let conditional_hooks = match &main_outcome {
                Ok(_) => v.on_success,
                Err(_) => v.on_failure,
            };

            if let Some(hooks) = conditional_hooks {
                for hook in hooks {
                    let hook_res = run_script_with_level(
                        scripts,
                        hook,
                        env_overrides.clone(),
                        level + 1,
                        script_durations.clone(),
                        quiet,
                        verbose,
                        &[],
                    );
                    if let Ok(r) = hook_res {
                        result.hooks.push(r);
                    }
                }
            }

            if let Some(post_hooks) = v.post {
                for hook in post_hooks {
                    if let Ok(r) = run_script_with_level(
                        scripts,
                        hook,
                        env_overrides.clone(),
                        level + 1,
                        script_durations.clone(),
                        quiet,
                        verbose,
                        &[],
                    ) {
                        result.hooks.push(r);
                    }
                }
            }

            main_outcome?;
        }
    }

    let script_duration = script_start_time.elapsed();
    result.duration_ms = script_duration.as_millis();
    if level > 0
        || matches!(
            scripts.scripts.get(script_name),
            Some(Script::Default(_))
                | Some(Script::Inline { command: Some(_), .. })
                | Some(Script::CILike { command: Some(_), .. })
        )
    {
        script_durations
            .lock()
            .unwrap()
            .insert(script_name.to_string(), script_duration);
    }

    Ok(result)
}

// ---------------------------------------------------------------------------
// Helpers (env, exec, dry-run, requirements, interactive)
// ---------------------------------------------------------------------------

/// Detect whether `cmd` should be treated as a cargo script invocation.
/// A command is a cargo script when it points at a file ending in `.rs` that
/// exists on disk (we deliberately keep this strict to avoid surprises).
pub fn is_cargo_script_path(cmd: &str) -> bool {
    let first = cmd.split_whitespace().next().unwrap_or("");
    first.ends_with(".rs") && std::path::Path::new(first).exists()
}

fn get_final_env(env_vars: &HashMap<String, String>, env_overrides: &[String]) -> HashMap<String, String> {
    let mut final_env = env_vars.clone();
    for override_str in env_overrides {
        if let Some((key, value)) = override_str.split_once('=') {
            final_env.insert(key.to_string(), value.to_string());
        }
    }
    final_env
}

fn apply_env_vars(env_vars: &HashMap<String, String>, env_overrides: &[String]) {
    let final_env = get_final_env(env_vars, env_overrides);
    for (key, value) in &final_env {
        // SAFETY: we are about to spawn a child; setting env vars in the
        // current process is the same approach used by the original
        // implementation. See note in the original v0.5 source.
        unsafe {
            env::set_var(key, value);
        }
    }
}

fn execute_command(
    script_name: &str,
    interpreter: Option<&str>,
    command: &str,
    toolchain: Option<&str>,
    env_vars: &HashMap<String, String>,
) -> Result<(), CargoScriptError> {
    let mut cmd = if let Some(tc) = toolchain {
        let mut command_with_toolchain = format!("cargo +{} ", tc);
        command_with_toolchain.push_str(command);
        let mut cmd = if cfg!(target_os = "windows") {
            let mut c = Command::new("cmd");
            c.args(["/C", &command_with_toolchain]);
            c
        } else {
            let mut c = Command::new("sh");
            c.arg("-c").arg(&command_with_toolchain);
            c
        };
        cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
        for (key, value) in env_vars {
            cmd.env(key, value);
        }
        cmd.spawn().map_err(|e| CargoScriptError::ExecutionError {
            script: script_name.to_string(),
            command: command.to_string(),
            source: e,
        })?
    } else {
        spawn_with_interpreter(interpreter, command, env_vars).map_err(|e| {
            CargoScriptError::ExecutionError {
                script: script_name.to_string(),
                command: command.to_string(),
                source: e,
            }
        })?
    };

    let exit_status = cmd.wait().map_err(|e| CargoScriptError::ExecutionError {
        script: script_name.to_string(),
        command: command.to_string(),
        source: e,
    })?;

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

        return Err(CargoScriptError::ExecutionError {
            script: script_name.to_string(),
            command: command.to_string(),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("exited with status: {}", exit_status),
            ),
        });
    }

    Ok(())
}

fn spawn_with_interpreter(
    interpreter: Option<&str>,
    command: &str,
    env_vars: &HashMap<String, String>,
) -> std::io::Result<std::process::Child> {
    let mut cmd = match interpreter {
        Some("bash") => {
            let mut c = Command::new("bash");
            c.arg("-c").arg(command);
            c
        }
        Some("zsh") => {
            let mut c = Command::new("zsh");
            c.arg("-c").arg(command);
            c
        }
        Some("powershell") => {
            let mut c = Command::new("powershell");
            c.args(["-NoProfile", "-Command", command]);
            c
        }
        Some("cmd") => {
            let mut c = Command::new("cmd");
            c.args(["/C", command]);
            c
        }
        Some(other) => {
            let mut c = Command::new(other);
            c.arg("-c").arg(command);
            c
        }
        None => {
            if cfg!(target_os = "windows") {
                let mut c = Command::new("cmd");
                c.args(["/C", command]);
                c
            } else {
                let mut c = Command::new("sh");
                c.arg("-c").arg(command);
                c
            }
        }
    };

    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    for (key, value) in env_vars {
        cmd.env(key, value);
    }
    cmd.spawn()
}

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

    let script = scripts.scripts.get(script_name).ok_or_else(|| {
        let available_scripts: Vec<String> = scripts.scripts.keys().cloned().collect();
        CargoScriptError::ScriptNotFound {
            script_name: script_name.to_string(),
            available_scripts,
        }
    })?;

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
                if !final_env.is_empty() {
                    println!("{}    Environment variables:", indent);
                    for (key, value) in &final_env {
                        println!("{}      {} = {}", indent, key.cyan(), value.green());
                    }
                }
                if level == 0 {
                    println!();
                }
            }
        }
        Script::Inline { .. } | Script::CILike { .. } => {
            let v = view(script).unwrap();
            if !quiet {
                if verbose {
                    if let Some(reqs) = v.requires {
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
                    if let Some(tc) = v.toolchain {
                        println!(
                            "{}{}  {}: {}",
                            indent,
                            "🔧".yellow(),
                            "Would use toolchain".cyan(),
                            tc.bold().green()
                        );
                        println!();
                    }
                    if let Some(desc) = v.info {
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

                if let Some(pre) = v.pre {
                    println!("{}{}  {}: {:?}", indent, "↩️".yellow(), "Pre hooks".cyan(), pre);
                }
                if let Some(post) = v.post {
                    println!("{}{}  {}: {:?}", indent, "↪️".yellow(), "Post hooks".cyan(), post);
                }

                if let Some(include_scripts) = v.include {
                    println!(
                        "{}{}  {}: [ {} ]",
                        indent,
                        "📋".yellow(),
                        "Would run include scripts".cyan(),
                        script_name.bold()
                    );
                    if let Some(desc) = v.info {
                        println!("{}    Description: {}", indent, desc.green());
                    }
                    println!();
                    for include_script in include_scripts {
                        dry_run_script(scripts, include_script, env_overrides.clone(), level + 1, quiet, verbose)?;
                    }
                }

                if let Some(path) = v.script_file {
                    println!(
                        "{}{}  {}: [ {} ] -> {}",
                        indent,
                        "📋".yellow(),
                        "Would run cargo script".cyan(),
                        script_name.bold(),
                        path.green(),
                    );
                }

                if let Some(cmd) = v.command {
                    println!(
                        "{}{}  {}: [ {} ]",
                        indent,
                        "📋".yellow(),
                        "Would run script".cyan(),
                        script_name.bold()
                    );

                    if let Some(interp) = v.interpreter {
                        println!("{}    Interpreter: {}", indent, interp.green());
                    }
                    if let Some(tc) = v.toolchain {
                        println!("{}    Toolchain: {}", indent, tc.green());
                    }
                    println!("{}    Command: {}", indent, cmd.green());

                    if let Some(script_env) = v.env {
                        env_vars.extend(script_env.clone());
                    }
                    let final_env = get_final_env(&env_vars, &env_overrides);
                    if !final_env.is_empty() {
                        println!("{}    Environment variables:", indent);
                        for (key, value) in &final_env {
                            println!("{}      {} = {}", indent, key.cyan(), value.green());
                        }
                    }
                    if level == 0 {
                        println!();
                    }
                }
            } else if let Some(include_scripts) = v.include {
                for include_script in include_scripts {
                    dry_run_script(scripts, include_script, env_overrides.clone(), level + 1, quiet, verbose)?;
                }
            }
        }
    }

    Ok(())
}

/// Interactive script selection using a fuzzy finder.
pub fn interactive_select_script(scripts: &Scripts, quiet: bool) -> Result<String, CargoScriptError> {
    if scripts.scripts.is_empty() {
        return Err(CargoScriptError::ScriptNotFound {
            script_name: "".to_string(),
            available_scripts: vec![],
        });
    }

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

    items.sort_by(|a, b| a.0.cmp(&b.0));

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
