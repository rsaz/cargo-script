use assert_cmd::Command;
use std::fs;
use std::process::Command as ProcessCommand;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up the test scripts by creating a directory and a test script file,
/// and making the script executable.
fn setup_test_scripts() {
    let script_content = r#"
#!/usr/bin/env bash
echo "Test script executed"
    "#;
    fs::create_dir_all(".scripts").unwrap();
    fs::write(".scripts/test_script.sh", script_content).unwrap();
    ProcessCommand::new("chmod")
        .args(&["+x", ".scripts/test_script.sh"])
        .status()
        .expect("Failed to make test script executable");
}

/// Sets up the Scripts.toml file with the specified content.
#[allow(dead_code)]
fn setup_scripts_toml(content: &str) {
    fs::write("Scripts.toml", content).unwrap();
}

/// Tests the `i_am_shell` script defined in `Scripts.toml`.
/// This script should output "Test script executed".
#[test]
fn test_i_am_shell() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "i_am_shell", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("Test script executed"));
}

/// Tests the `i_am_shell_obj` script defined in `Scripts.toml`.
/// This script uses the bash interpreter and includes an info message.
/// The output should contain both the info message and "Test script executed".
#[test]
fn test_i_am_shell_obj() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "i_am_shell_obj", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("Detect shell script"))
        .stdout(predicates::str::contains("Test script executed"));
}

/// Tests the `build` script defined in `Scripts.toml`.
/// This script should output "build".
#[test]
fn test_build() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "build", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("build"));
}

/// Tests the `release` script defined in `Scripts.toml`.
/// This script includes the `i_am_shell` and `build` scripts.
/// The output should contain both "Test script executed" and "build".
#[test]
fn test_release() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "release", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("Test script executed"))
        .stdout(predicates::str::contains("build"));
}

/// Tests the `release_info` script defined in `Scripts.toml`.
/// This script includes an info message, the `i_am_shell_obj`, and `build` scripts.
/// The output should contain the info message, "Test script executed", and "build".
#[test]
fn test_release_info() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "release_info", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("Release info"))
        .stdout(predicates::str::contains("Test script executed"))
        .stdout(predicates::str::contains("build"));
}
