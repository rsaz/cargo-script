use assert_cmd::cargo::cargo_bin_cmd;

mod constants;
use constants::SCRIPT_TOML;


/// Tests the `test_requires` script defined in `Scripts.toml`.
/// This script requires a specific version of a tool (e.g., rustup).
#[test]
fn test_requires() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    let output = cmd.args(&["run", "test_requires", "--scripts-path", SCRIPT_TOML])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Requirement check failed: Required version for rustup is < 1.24.3"));
}

/// Tests the `cilike_script` defined in `Scripts.toml`.
/// This script uses the CILike format.
#[test]
fn test_cilike_script() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "cilike_script", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("CILike Test"));
}

/// Tests the `inline_script` defined in `Scripts.toml`.
/// This script uses the Inline format and checks for `requires` and `toolchain`.
#[test]
fn test_inline_script() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    let output = cmd.args(&["run", "inline_script", "--scripts-path", SCRIPT_TOML])
        .output()
        .expect("Failed to execute command");

    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(stderr.contains("Requirement check failed: Required version for rustup is < 1.24.3"));
}