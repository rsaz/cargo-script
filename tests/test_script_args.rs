//! Tests for parameterised scripts (`{{var}}` substitution).

use std::collections::HashMap;
use cargo_run::commands::script::{
    parse_script_args, run_script_with_options, substitute_variables, view, RunOptions, Script,
    Scripts,
};

fn build() -> Scripts {
    let mut scripts: HashMap<String, Script> = HashMap::new();
    let cmd = if cfg!(target_os = "windows") {
        "echo {{name}} {{greet}}"
    } else {
        "printf '%s %s\\n' {{name}} {{greet}}"
    };
    scripts.insert(
        "hello".into(),
        Script::Inline {
            command: Some(cmd.into()),
            requires: None,
            toolchain: None,
            info: None,
            env: None,
            include: None,
            interpreter: None,
            script_file: None,
            args: Some(vec!["name".into(), "greet".into()]),
            defaults: Some({
                let mut m = HashMap::new();
                m.insert("greet".into(), "world".into());
                m
            }),
            workspace: None,
            pre: None,
            post: None,
            on_success: None,
            on_failure: None,
        },
    );
    Scripts { global_env: None, scripts, workspace: None }
}

#[test]
fn substitutes_template_variables() {
    let mut args = HashMap::new();
    args.insert("env".into(), "production".into());
    let s = substitute_variables("kubectl apply -f {{env}}.yaml", &args);
    assert_eq!(s, "kubectl apply -f production.yaml");
}

#[test]
fn parse_args_supports_positional_and_named() {
    let scripts = build();
    let v = view(scripts.scripts.get("hello").unwrap()).unwrap();
    let raw = vec!["alice".to_string(), "greet=hi".to_string()];
    let resolved = parse_script_args(&v, &raw);
    assert_eq!(resolved.get("name"), Some(&"alice".to_string()));
    assert_eq!(resolved.get("greet"), Some(&"hi".to_string()));
}

#[test]
fn defaults_are_applied_when_missing() {
    let scripts = build();
    let v = view(scripts.scripts.get("hello").unwrap()).unwrap();
    let raw = vec!["alice".to_string()];
    let resolved = parse_script_args(&v, &raw);
    assert_eq!(resolved.get("greet"), Some(&"world".to_string()));
}

#[test]
fn missing_required_arg_errors() {
    let scripts = build();
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        script_args: vec![],
        ..RunOptions::default()
    };
    let res = run_script_with_options(&scripts, "hello", &opts);
    assert!(res.is_err());
}

#[test]
fn run_with_args_succeeds() {
    let scripts = build();
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        script_args: vec!["alice".to_string()],
        ..RunOptions::default()
    };
    let res = run_script_with_options(&scripts, "hello", &opts);
    assert!(res.is_ok(), "run should succeed: {:?}", res.err());
}
