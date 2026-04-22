//! Parallel script execution backed by Tokio.
//!
//! This module is only compiled when the `parallel` feature is enabled
//! (default). It provides:
//!
//! * [`run_scripts_parallel`] — run several distinct scripts at the same time.
//! * [`run_workspace_parallel`] — run a single script across every workspace
//!   member in parallel.
//!
//! Both functions aggregate their results into a single [`ExecutionResult`]
//! tree and surface a [`CargoScriptError::ParallelExecutionFailed`] when any
//! child fails.

use std::path::PathBuf;

use colored::*;
use tokio::task::JoinSet;

use crate::commands::script::{
    run_script_with_options, ExecutionResult, RunOptions, Scripts,
};
use crate::error::CargoScriptError;

/// Run a list of scripts concurrently.
pub fn run_scripts_parallel(
    scripts: &Scripts,
    script_names: &[String],
    opts: &RunOptions,
) -> Result<ExecutionResult, CargoScriptError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| CargoScriptError::ExecutionError {
            script: "parallel".to_string(),
            command: "tokio runtime".to_string(),
            source: e,
        })?;

    runtime.block_on(async {
        let mut set = JoinSet::new();

        // We clone Scripts (cheap-ish, no Arc needed for our test sizes) and
        // RunOptions into each task. Since `Scripts` is `Clone`, this is
        // straightforward.
        for name in script_names {
            let scripts = scripts.clone();
            let mut leaf = opts.clone();
            leaf.json_output = false;
            leaf.no_workspace = true;
            let n = name.clone();
            set.spawn(async move {
                (n.clone(), tokio::task::spawn_blocking(move || {
                    run_script_with_options(&scripts, &n, &leaf)
                }).await)
            });
        }

        let mut aggregated = ExecutionResult::new("parallel");
        let mut failures: Vec<String> = Vec::new();

        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((_name, Ok(Ok(child)))) => aggregated.includes.push(child),
                Ok((name, Ok(Err(e)))) => {
                    eprintln!("{} {}: {}", "❌".red(), name.bold(), e);
                    failures.push(name);
                }
                Ok((name, Err(join_err))) => {
                    eprintln!("{} {}: task panicked ({})", "❌".red(), name, join_err);
                    failures.push(name);
                }
                Err(join_err) => {
                    failures.push(format!("(join error) {}", join_err));
                }
            }
        }

        if !failures.is_empty() {
            aggregated.success = false;
            return Err(CargoScriptError::ParallelExecutionFailed {
                failed_scripts: failures,
            });
        }
        Ok(aggregated)
    })
}

/// Run a single script across every workspace member, in parallel.
pub fn run_workspace_parallel(
    scripts: &Scripts,
    script_name: &str,
    members: &[PathBuf],
    opts: &RunOptions,
) -> Result<ExecutionResult, CargoScriptError> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|e| CargoScriptError::ExecutionError {
            script: "workspace-parallel".to_string(),
            command: "tokio runtime".to_string(),
            source: e,
        })?;

    runtime.block_on(async {
        let mut set = JoinSet::new();
        for member in members {
            let scripts = scripts.clone();
            let mut leaf = opts.clone();
            leaf.json_output = false;
            leaf.no_workspace = true;
            let member = member.clone();
            let name = script_name.to_string();
            set.spawn(async move {
                let label = member.display().to_string();
                let res = tokio::task::spawn_blocking(move || {
                    crate::commands::workspace::run_member(&scripts, &name, &member, &leaf)
                })
                .await;
                (label, res)
            });
        }

        let mut aggregated = ExecutionResult::new(script_name);
        aggregated.command = Some("workspace::Parallel".to_string());
        let mut failures: Vec<String> = Vec::new();

        while let Some(joined) = set.join_next().await {
            match joined {
                Ok((_label, Ok(Ok(child)))) => aggregated.includes.push(child),
                Ok((label, Ok(Err(e)))) => {
                    eprintln!("{} {}: {}", "❌".red(), label.bold(), e);
                    failures.push(label);
                }
                Ok((label, Err(join_err))) => {
                    eprintln!("{} {}: task panicked ({})", "❌".red(), label, join_err);
                    failures.push(label);
                }
                Err(_) => {}
            }
        }

        if !failures.is_empty() {
            aggregated.success = false;
            return Err(CargoScriptError::ParallelExecutionFailed {
                failed_scripts: failures,
            });
        }
        Ok(aggregated)
    })
}
