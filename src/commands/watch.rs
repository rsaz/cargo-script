//! Watch mode: re-run a script whenever files change.
//!
//! Only compiled with the `watch` feature (default). Uses `notify` +
//! `notify-debouncer-full` so rapid bursts of file changes (e.g. a build that
//! touches many files) collapse into a single re-run.

use std::path::PathBuf;
use std::sync::mpsc;
use std::time::Duration;

use colored::*;
use notify::{RecursiveMode, Watcher};
use notify_debouncer_full::{new_debouncer, DebouncedEvent};

use crate::commands::script::{run_script_with_options, RunOptions, Scripts};
use crate::error::CargoScriptError;

/// Configuration for one `watch_and_run` invocation.
pub struct WatchConfig {
    /// Paths to watch (recursively).
    pub watch_paths: Vec<PathBuf>,
    /// Substring matches to ignore (eg. `target/`, `.git/`).
    pub exclude: Vec<String>,
    /// Debounce window (default 250 ms).
    pub debounce: Duration,
}

impl Default for WatchConfig {
    fn default() -> Self {
        Self {
            watch_paths: vec![PathBuf::from(".")],
            exclude: vec!["target".into(), ".git".into(), "node_modules".into()],
            debounce: Duration::from_millis(250),
        }
    }
}

/// Block forever (until Ctrl-C), re-running `script_name` on relevant file
/// changes.
pub fn watch_and_run(
    scripts: &Scripts,
    script_name: &str,
    opts: &RunOptions,
    cfg: WatchConfig,
) -> Result<(), CargoScriptError> {
    if !opts.quiet {
        println!(
            "{} watching {} path(s); press Ctrl-C to stop",
            "👀".cyan(),
            cfg.watch_paths.len()
        );
    }

    // Initial run.
    let _ = run_script_with_options(scripts, script_name, opts);

    let (tx, rx) = mpsc::channel::<Result<Vec<DebouncedEvent>, Vec<notify::Error>>>();
    let mut debouncer = new_debouncer(cfg.debounce, None, tx).map_err(|e| {
        CargoScriptError::WatchError {
            path: cfg.watch_paths.iter().map(|p| p.display().to_string()).collect::<Vec<_>>().join(", "),
            message: e.to_string(),
        }
    })?;

    for p in &cfg.watch_paths {
        debouncer
            .watcher()
            .watch(p, RecursiveMode::Recursive)
            .map_err(|e| CargoScriptError::WatchError {
                path: p.display().to_string(),
                message: e.to_string(),
            })?;
    }

    loop {
        match rx.recv() {
            Ok(Ok(events)) => {
                if events.iter().any(|ev| relevant_event(ev, &cfg.exclude)) {
                    if !opts.quiet {
                        println!(
                            "\n{} file change detected, re-running '{}'...",
                            "🔄".yellow(),
                            script_name.green()
                        );
                    }
                    if let Err(e) = run_script_with_options(scripts, script_name, opts) {
                        eprintln!("{}", e);
                    }
                }
            }
            Ok(Err(errors)) => {
                for e in errors {
                    eprintln!("{} watch error: {}", "⚠️".yellow(), e);
                }
            }
            Err(_) => break, // channel closed
        }
    }
    Ok(())
}

fn relevant_event(ev: &DebouncedEvent, exclude: &[String]) -> bool {
    for path in &ev.paths {
        let s = path.display().to_string();
        if exclude.iter().any(|ex| s.contains(ex)) {
            return false;
        }
    }
    true
}
