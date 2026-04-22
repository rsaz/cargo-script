//! Tests for the parallel executor.
//!
//! Only built when the `parallel` feature is enabled (default).

#![cfg(feature = "parallel")]

use std::collections::HashMap;
use cargo_run::commands::parallel::run_scripts_parallel;
use cargo_run::commands::script::{RunOptions, Script, Scripts};

fn build() -> Scripts {
    let mut scripts: HashMap<String, Script> = HashMap::new();
    let cmd = if cfg!(target_os = "windows") {
        "echo a"
    } else {
        "true"
    };
    for name in ["a", "b", "c", "d"] {
        scripts.insert(name.into(), Script::Default(cmd.into()));
    }
    Scripts { global_env: None, scripts, workspace: None }
}

#[test]
fn runs_multiple_scripts_in_parallel() {
    let scripts = build();
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        ..RunOptions::default()
    };
    let names = vec!["a".into(), "b".into(), "c".into(), "d".into()];
    let res = run_scripts_parallel(&scripts, &names, &opts).expect("parallel ok");
    assert_eq!(res.includes.len(), 4);
}

#[test]
fn parallel_aggregates_failures() {
    let mut scripts: HashMap<String, Script> = HashMap::new();
    scripts.insert("ok".into(), Script::Default(if cfg!(windows) { "echo ok" } else { "true" }.into()));
    scripts.insert("bad".into(), Script::Default(if cfg!(windows) { "exit /B 1" } else { "false" }.into()));
    let scripts = Scripts { global_env: None, scripts, workspace: None };
    let opts = RunOptions { quiet: true, show_metrics: false, ..RunOptions::default() };
    let names = vec!["ok".into(), "bad".into()];
    let res = run_scripts_parallel(&scripts, &names, &opts);
    assert!(res.is_err());
}
