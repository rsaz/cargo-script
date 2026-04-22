//! Tests for workspace detection and member discovery.

use std::collections::HashMap;
use std::env;
use std::fs;
use std::sync::Mutex;
use cargo_run::commands::script::{Scripts, WorkspaceConfig};
use cargo_run::commands::workspace::{detect_workspace_root, discover_members, workspace_summary};
use tempfile::TempDir;

// Tests below mutate the process-wide cwd; serialize them.
static CWD_LOCK: Mutex<()> = Mutex::new(());

fn write(p: &std::path::Path, contents: &str) {
    if let Some(parent) = p.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(p, contents).unwrap();
}

fn make_workspace() -> TempDir {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("Cargo.toml"),
        r#"[workspace]
members = ["crates/alpha", "crates/beta"]
resolver = "2"
"#,
    );
    write(
        &dir.path().join("crates/alpha/Cargo.toml"),
        r#"[package]
name = "alpha"
version = "0.1.0"
edition = "2021"
"#,
    );
    write(
        &dir.path().join("crates/beta/Cargo.toml"),
        r#"[package]
name = "beta"
version = "0.1.0"
edition = "2021"
"#,
    );
    dir
}

#[test]
fn detect_root_walks_up() {
    let dir = make_workspace();
    let nested = dir.path().join("crates/alpha");
    let root = detect_workspace_root(&nested).expect("root detected");
    assert_eq!(root, dir.path().canonicalize().unwrap_or(dir.path().to_path_buf()));
}

#[test]
fn discover_members_finds_explicit_members() {
    let _g = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = make_workspace();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    let scripts = Scripts {
        global_env: None,
        scripts: HashMap::new(),
        workspace: None,
    };
    let result = discover_members(&scripts);

    let _ = env::set_current_dir(&prev);

    let members = result.expect("members discovered");
    assert!(members.iter().any(|m| m.ends_with("alpha")));
    assert!(members.iter().any(|m| m.ends_with("beta")));
}

#[test]
fn workspace_summary_includes_members_key() {
    let _g = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = make_workspace();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    let scripts = Scripts {
        global_env: None,
        scripts: HashMap::new(),
        workspace: None,
    };
    let summary = workspace_summary(&scripts).expect("summary");

    let _ = env::set_current_dir(&prev);
    assert!(summary.contains_key("members"));
}

#[test]
fn explicit_workspace_section_overrides_cargo() {
    let _g = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = make_workspace();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    let scripts = Scripts {
        global_env: None,
        scripts: HashMap::new(),
        workspace: Some(WorkspaceConfig {
            members: Some(vec!["crates/alpha".into()]),
            exclude: None,
        }),
    };
    let result = discover_members(&scripts);

    let _ = env::set_current_dir(&prev);
    let members = result.expect("members discovered");
    assert_eq!(members.len(), 1);
    assert!(members[0].ends_with("alpha"));
}

#[test]
fn detect_root_errors_when_no_workspace() {
    let dir = TempDir::new().unwrap();
    write(
        &dir.path().join("Cargo.toml"),
        "[package]\nname = \"lonely\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    );
    let res = detect_workspace_root(dir.path());
    assert!(res.is_err());
}
