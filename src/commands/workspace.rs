//! Workspace-aware execution.
//!
//! This module discovers Cargo workspace members and dispatches a script
//! across them, sequentially or in parallel.

use std::collections::HashMap;
use std::path::{Path, PathBuf};

use colored::*;
use serde::Deserialize;

use crate::commands::script::{
    run_script_with_options, ExecutionResult, RunOptions, Scripts, WorkspaceMode,
};
use crate::error::CargoScriptError;

// ---------------------------------------------------------------------------
// Cargo.toml workspace parsing
// ---------------------------------------------------------------------------

#[derive(Deserialize, Debug, Default)]
struct CargoManifest {
    workspace: Option<CargoWorkspace>,
}

#[derive(Deserialize, Debug, Default)]
struct CargoWorkspace {
    #[serde(default)]
    members: Vec<String>,
    #[serde(default)]
    exclude: Vec<String>,
}

/// Locate the workspace root by walking up from `start_dir` looking for a
/// `Cargo.toml` that contains a `[workspace]` table.
pub fn detect_workspace_root(start_dir: &Path) -> Result<PathBuf, CargoScriptError> {
    let mut cur = start_dir
        .canonicalize()
        .unwrap_or_else(|_| start_dir.to_path_buf());
    loop {
        let manifest = cur.join("Cargo.toml");
        if manifest.exists() {
            if let Ok(text) = std::fs::read_to_string(&manifest) {
                if let Ok(parsed) = toml::from_str::<CargoManifest>(&text) {
                    if parsed.workspace.is_some() {
                        return Ok(cur);
                    }
                }
            }
        }
        if !cur.pop() {
            break;
        }
    }
    Err(CargoScriptError::WorkspaceNotFound {
        path: start_dir.display().to_string(),
    })
}

/// Resolve member directories, honoring `Scripts.toml`'s `[workspace]`
/// section first, then falling back to `Cargo.toml`'s `[workspace.members]`.
pub fn discover_members(scripts: &Scripts) -> Result<Vec<PathBuf>, CargoScriptError> {
    let cwd = std::env::current_dir().map_err(|e| CargoScriptError::WorkspaceNotFound {
        path: format!("(cwd unavailable: {})", e),
    })?;
    let root = detect_workspace_root(&cwd)?;

    // Prefer explicit overrides from Scripts.toml when provided.
    let (members_globs, excludes) = if let Some(ws) = &scripts.workspace {
        (
            ws.members.clone().unwrap_or_default(),
            ws.exclude.clone().unwrap_or_default(),
        )
    } else {
        let manifest = std::fs::read_to_string(root.join("Cargo.toml"))
            .map_err(|e| CargoScriptError::WorkspaceNotFound {
                path: format!("{}: {}", root.display(), e),
            })?;
        let parsed: CargoManifest = toml::from_str(&manifest).map_err(|e| {
            CargoScriptError::InvalidToml {
                path: root.join("Cargo.toml").display().to_string(),
                message: e.message().to_string(),
                line: None,
            }
        })?;
        let ws = parsed.workspace.unwrap_or_default();
        (ws.members, ws.exclude)
    };

    let mut resolved = Vec::new();
    for pattern in &members_globs {
        for path in expand_member_pattern(&root, pattern) {
            if !excludes.iter().any(|ex| path.ends_with(ex)) {
                resolved.push(path);
            }
        }
    }

    if resolved.is_empty() {
        // No explicit members: treat the root itself as the only member.
        resolved.push(root);
    }

    resolved.sort();
    resolved.dedup();
    Ok(resolved)
}

/// Best-effort glob expansion limited to the simple `*` at the end of a
/// segment (the most common cargo workspace pattern). Avoids pulling in a
/// `glob` dependency for such a narrow use case.
fn expand_member_pattern(root: &Path, pattern: &str) -> Vec<PathBuf> {
    if let Some(prefix) = pattern.strip_suffix("/*") {
        let dir = root.join(prefix);
        if let Ok(entries) = std::fs::read_dir(&dir) {
            return entries
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.is_dir() && p.join("Cargo.toml").exists())
                .collect();
        }
        Vec::new()
    } else {
        let p = root.join(pattern);
        if p.join("Cargo.toml").exists() {
            vec![p]
        } else {
            Vec::new()
        }
    }
}

// ---------------------------------------------------------------------------
// Workspace dispatch
// ---------------------------------------------------------------------------

/// Run `script_name` once per workspace member, sequentially or in parallel
/// depending on `mode`.
pub fn run_in_workspace(
    scripts: &Scripts,
    script_name: &str,
    mode: WorkspaceMode,
    opts: &RunOptions,
) -> Result<ExecutionResult, CargoScriptError> {
    let members = discover_members(scripts)?;

    if !opts.quiet && !opts.json_output {
        println!(
            "{} {} member(s) detected for workspace mode {:?}:",
            "📦".cyan(),
            members.len(),
            mode
        );
        for m in &members {
            println!("  • {}", m.display().to_string().green());
        }
        println!();
    }

    let mut aggregated = ExecutionResult::new(script_name);
    aggregated.command = Some(format!("workspace::{:?}", mode));

    // Build a "leaf" RunOptions that does not recurse into workspace mode.
    let mut leaf_opts = opts.clone();
    leaf_opts.workspace_override = None;
    leaf_opts.no_workspace = true;
    leaf_opts.json_output = false; // We aggregate results ourselves.

    match mode {
        WorkspaceMode::Root => {
            // Should never happen — Root is filtered out earlier — but
            // handle defensively by running once in cwd.
            return run_script_with_options(scripts, script_name, &leaf_opts);
        }
        WorkspaceMode::All => {
            let mut failures: Vec<String> = Vec::new();
            for member in &members {
                let label = member.display().to_string();
                let res = run_member(scripts, script_name, member, &leaf_opts);
                match res {
                    Ok(child) => aggregated.includes.push(child),
                    Err(e) => {
                        eprintln!("{} {}: {}", "❌".red(), label.bold(), e);
                        failures.push(label);
                    }
                }
            }
            if !failures.is_empty() {
                aggregated.success = false;
                return Err(CargoScriptError::ParallelExecutionFailed {
                    failed_scripts: failures,
                });
            }
        }
        WorkspaceMode::Parallel => {
            #[cfg(feature = "parallel")]
            {
                aggregated = crate::commands::parallel::run_workspace_parallel(
                    scripts,
                    script_name,
                    &members,
                    &leaf_opts,
                )?;
            }
            #[cfg(not(feature = "parallel"))]
            {
                eprintln!(
                    "{} parallel feature disabled; falling back to sequential execution",
                    "⚠️".yellow()
                );
                let mut failures: Vec<String> = Vec::new();
                for member in &members {
                    match run_member(scripts, script_name, member, &leaf_opts) {
                        Ok(child) => aggregated.includes.push(child),
                        Err(_) => failures.push(member.display().to_string()),
                    }
                }
                if !failures.is_empty() {
                    aggregated.success = false;
                    return Err(CargoScriptError::ParallelExecutionFailed {
                        failed_scripts: failures,
                    });
                }
            }
        }
    }

    if opts.json_output {
        crate::output::json::print_execution_result(&aggregated);
    }

    Ok(aggregated)
}

/// Run a single script invocation inside `member_dir`. The member directory
/// is set as the current working directory only for the duration of the
/// call (saved/restored to keep concurrent workspace runs sane).
pub fn run_member(
    scripts: &Scripts,
    script_name: &str,
    member_dir: &Path,
    opts: &RunOptions,
) -> Result<ExecutionResult, CargoScriptError> {
    let prev = std::env::current_dir().ok();
    let _ = std::env::set_current_dir(member_dir);
    let result = run_script_with_options(scripts, script_name, opts);
    if let Some(p) = prev {
        let _ = std::env::set_current_dir(p);
    }
    result
}

/// Returns a small map describing the discovered workspace, useful for the
/// `--json` output of higher-level commands and for tests.
pub fn workspace_summary(scripts: &Scripts) -> Result<HashMap<String, Vec<String>>, CargoScriptError> {
    let members = discover_members(scripts)?;
    let mut out = HashMap::new();
    out.insert(
        "members".to_string(),
        members.iter().map(|p| p.display().to_string()).collect(),
    );
    Ok(out)
}
