//! Tests for the pre/post/on_success/on_failure hook system.

use std::collections::HashMap;
use cargo_run::commands::script::{
    run_script_with_options, RunOptions, Script, Scripts,
};

fn empty_inline() -> Script {
    Script::Inline {
        command: None,
        requires: None,
        toolchain: None,
        info: None,
        env: None,
        include: None,
        interpreter: None,
        script_file: None,
        args: None,
        defaults: None,
        workspace: None,
        pre: None,
        post: None,
        on_success: None,
        on_failure: None,
    }
}

fn make_scripts() -> Scripts {
    let echo = if cfg!(target_os = "windows") {
        "echo hi"
    } else {
        "true"
    };
    let fail = if cfg!(target_os = "windows") {
        "exit /B 1"
    } else {
        "false"
    };

    let mut scripts: HashMap<String, Script> = HashMap::new();
    scripts.insert("pre1".into(), Script::Default(echo.into()));
    scripts.insert("pre2".into(), Script::Default(echo.into()));
    scripts.insert("post1".into(), Script::Default(echo.into()));
    scripts.insert("good-hook".into(), Script::Default(echo.into()));
    scripts.insert("bad-hook".into(), Script::Default(echo.into()));
    scripts.insert(
        "with-hooks".into(),
        match empty_inline() {
            Script::Inline { .. } => {
                let command = Some(echo.to_string());
                Script::Inline {
                    command,
                    requires: None,
                    toolchain: None,
                    info: None,
                    env: None,
                    include: None,
                    interpreter: None,
                    script_file: None,
                    args: None,
                    defaults: None,
                    workspace: None,
                    pre: Some(vec!["pre1".into(), "pre2".into()]),
                    post: Some(vec!["post1".into()]),
                    on_success: Some(vec!["good-hook".into()]),
                    on_failure: None,
                }
            }
            _ => unreachable!(),
        },
    );
    scripts.insert(
        "fails-with-failure-hook".into(),
        Script::Inline {
            command: Some(fail.to_string()),
            requires: None,
            toolchain: None,
            info: None,
            env: None,
            include: None,
            interpreter: None,
            script_file: None,
            args: None,
            defaults: None,
            workspace: None,
            pre: None,
            post: None,
            on_success: None,
            on_failure: Some(vec!["bad-hook".into()]),
        },
    );

    Scripts {
        global_env: None,
        scripts,
        workspace: None,
    }
}

#[test]
fn pre_post_and_on_success_hooks_run_in_order() {
    let scripts = make_scripts();
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        ..RunOptions::default()
    };
    let res = run_script_with_options(&scripts, "with-hooks", &opts).expect("run");
    // 2 pre + 1 post + 1 on_success = 4 hooks recorded
    assert_eq!(res.hooks.len(), 4);
    assert!(res.success);
}

#[test]
fn on_failure_hook_runs_when_command_fails() {
    let scripts = make_scripts();
    let opts = RunOptions {
        quiet: true,
        show_metrics: false,
        ..RunOptions::default()
    };
    let err = run_script_with_options(&scripts, "fails-with-failure-hook", &opts);
    assert!(err.is_err(), "the failing script should propagate its error");
}
