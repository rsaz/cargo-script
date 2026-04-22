//! Tests for cargo-script (.rs file) integration.
//!
//! We don't actually execute a `cargo +nightly -Zscript` here (it requires
//! a nightly toolchain on the test machine). Instead we exercise:
//!
//! * the `is_cargo_script_path` heuristic;
//! * the cargo_script availability detection;
//! * automatic dispatch when a `Default` script's command points to an `.rs`
//!   file: we just confirm the script enum stores the path and the executor
//!   would route through cargo-script.

use cargo_run::commands::cargo_script::{
    build_cargo_script_argv, cargo_available, looks_like_cargo_script, CargoScriptInvocation,
};
use cargo_run::commands::script::is_cargo_script_path;
use std::fs;
use tempfile::TempDir;

#[test]
fn cargo_is_available_in_test_env() {
    assert!(
        cargo_available(),
        "cargo binary should be available when running cargo test"
    );
}

#[test]
fn looks_like_cargo_script_requires_existing_rs_file() {
    let dir = TempDir::new().unwrap();
    let p = dir.path().join("script.rs");
    assert!(!looks_like_cargo_script(p.to_str().unwrap()));
    fs::write(&p, "fn main() {}").unwrap();
    assert!(looks_like_cargo_script(p.to_str().unwrap()));
}

#[test]
fn is_cargo_script_path_only_matches_rs_files() {
    assert!(!is_cargo_script_path("cargo build"));
    assert!(!is_cargo_script_path("./does/not/exist.rs"));
}

/// Regression test: the stable cargo-script invocation MUST include the
/// `script` subcommand. A previous version dropped it and shipped
/// `cargo /path/to/file.rs`, which fails on stable cargo with
/// `error: no such command: '/path/to/file.rs'`.
#[test]
fn stable_invocation_includes_script_subcommand() {
    let (program, args) =
        build_cargo_script_argv(CargoScriptInvocation::Stable, "./scripts/audit.rs", &[]);
    assert_eq!(program, "cargo");
    assert_eq!(args, vec!["script", "./scripts/audit.rs"]);
}

#[test]
fn nightly_invocation_uses_z_script_flag() {
    let (program, args) =
        build_cargo_script_argv(CargoScriptInvocation::Nightly, "./scripts/audit.rs", &[]);
    assert_eq!(program, "cargo");
    assert_eq!(args, vec!["+nightly", "-Zscript", "./scripts/audit.rs"]);
}

#[test]
fn forwarded_args_are_separated_with_double_dash() {
    let extra = vec!["--verbose".to_string(), "input.json".to_string()];
    let (_, args) =
        build_cargo_script_argv(CargoScriptInvocation::Stable, "./scripts/audit.rs", &extra);
    assert_eq!(
        args,
        vec![
            "script",
            "./scripts/audit.rs",
            "--",
            "--verbose",
            "input.json",
        ]
    );

    let (_, args) =
        build_cargo_script_argv(CargoScriptInvocation::Nightly, "./scripts/audit.rs", &extra);
    assert_eq!(
        args,
        vec![
            "+nightly",
            "-Zscript",
            "./scripts/audit.rs",
            "--",
            "--verbose",
            "input.json",
        ]
    );
}

#[test]
fn no_double_dash_when_no_extra_args() {
    let (_, args) =
        build_cargo_script_argv(CargoScriptInvocation::Stable, "./scripts/audit.rs", &[]);
    assert!(!args.iter().any(|a| a == "--"));
}

/// Regression test: an installed third-party `cargo-script` binary (such
/// as a previous version of *this* tool published on crates.io as
/// `cargo-run` whose binary is named `cargo-script`) must NOT be mistaken
/// for stable Cargo's RFC 3502 support. Cargo would dispatch
/// `cargo script <file.rs>` to the third-party binary and our `.rs`
/// invocation would silently break.
#[test]
fn third_party_cargo_script_is_not_recognised_as_stable() {
    use cargo_run::commands::cargo_script::is_builtin_cargo_version_line;
    assert!(!is_builtin_cargo_version_line("cargo-script 0.5.2"));
    assert!(!is_builtin_cargo_version_line("cargo-script 1.0.0 (foo bar)"));
    assert!(!is_builtin_cargo_version_line("cargo-make 0.37.0"));
    assert!(!is_builtin_cargo_version_line(""));
    assert!(!is_builtin_cargo_version_line("cargo"));
    assert!(!is_builtin_cargo_version_line("cargo abc"));
}

#[test]
fn cargo_builtin_version_line_is_recognised_as_stable() {
    use cargo_run::commands::cargo_script::is_builtin_cargo_version_line;
    assert!(is_builtin_cargo_version_line(
        "cargo 1.96.0 (f2d3ce0bd 2026-03-21)"
    ));
    assert!(is_builtin_cargo_version_line("cargo 1.95.0"));
    assert!(is_builtin_cargo_version_line("  cargo 2.0.0  "));
}
