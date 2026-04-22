//! Tests for the JSON output format.

use std::collections::HashMap;
use cargo_run::commands::script::{
    run_script_with_options, ExecutionResult, RunOptions, Script, Scripts,
};
use cargo_run::output::json::to_string;

#[test]
fn execution_result_serialises_to_json() {
    let mut r = ExecutionResult::new("hello");
    r.command = Some("echo hi".into());
    r.duration_ms = 12;
    r.exit_code = Some(0);
    let s = to_string(&r);
    assert!(s.contains("\"script\""));
    assert!(s.contains("\"command\""));
    assert!(s.contains("\"hello\""));
}

#[test]
fn run_returns_structured_result() {
    let echo = if cfg!(target_os = "windows") { "echo hi" } else { "true" };
    let mut scripts: HashMap<String, Script> = HashMap::new();
    scripts.insert("noop".into(), Script::Default(echo.into()));
    let scripts = Scripts {
        global_env: None,
        scripts,
        workspace: None,
    };
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        ..RunOptions::default()
    };
    let res = run_script_with_options(&scripts, "noop", &opts).unwrap();
    assert_eq!(res.script, "noop");
    assert!(res.success);
}
