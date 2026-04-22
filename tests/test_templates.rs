//! Tests for the template registry & application.

use cargo_run::commands::templates::{apply, find, registry};
use std::env;
use std::sync::Mutex;
use tempfile::TempDir;

// Serialize tests that mutate the process-wide current directory so they
// don't race with each other (cargo runs tests in parallel by default).
static CWD_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn registry_contains_known_templates() {
    let names: Vec<&str> = registry().iter().map(|t| t.name).collect();
    for expected in &["github-actions", "gitlab-ci", "rust-project", "workspace"] {
        assert!(
            names.contains(expected),
            "missing template '{}', got {:?}",
            expected,
            names
        );
    }
}

#[test]
fn find_returns_template() {
    assert!(find("github-actions").is_some());
    assert!(find("nope").is_none());
}

#[test]
fn apply_template_creates_files() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TempDir::new().unwrap();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    let res = apply("github-actions", true);
    let scripts_exists = dir.path().join("Scripts.toml").exists();
    let workflow_exists = dir.path().join(".github/workflows/ci.yml").exists();

    let _ = env::set_current_dir(&prev);

    assert!(res.is_ok(), "apply failed: {:?}", res.err());
    assert!(scripts_exists);
    assert!(workflow_exists);
}

#[test]
fn apply_unknown_template_errors() {
    let _guard = CWD_LOCK.lock().unwrap_or_else(|e| e.into_inner());
    let dir = TempDir::new().unwrap();
    let prev = env::current_dir().unwrap();
    env::set_current_dir(dir.path()).unwrap();

    let res = apply("does-not-exist", false);
    let _ = env::set_current_dir(&prev);
    assert!(res.is_err());
}
