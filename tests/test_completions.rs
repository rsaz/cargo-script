use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;

/// Tests that bash completion generation works
#[test]
fn test_completions_bash() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "bash"])
        .assert()
        .success()
        .stdout(contains("_cargo-script"));
}

/// Tests that zsh completion generation works
#[test]
fn test_completions_zsh() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "zsh"])
        .assert()
        .success()
        .stdout(contains("_cargo-script"));
}

/// Tests that fish completion generation works
#[test]
fn test_completions_fish() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "fish"])
        .assert()
        .success()
        .stdout(contains("cargo-script"));
}

/// Tests that PowerShell completion generation works
#[test]
#[cfg(target_os = "windows")]
fn test_completions_powershell() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "power-shell"])
        .assert()
        .success()
        .stdout(contains("cargo-script"));
}

/// Tests that completions command shows help
#[test]
fn test_completions_help() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "--help"])
        .assert()
        .success()
        .stdout(contains("Generate shell completion scripts"));
}

/// Tests that invalid shell shows error
#[test]
fn test_completions_invalid_shell() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["completions", "invalid"])
        .assert()
        .failure();
}

