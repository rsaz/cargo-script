use assert_cmd::Command;
use std::fs;
use std::process::Command as ProcessCommand;

// Ensure the test scripts directory exists and is executable
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

#[test]
fn test_i_am_shell() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "i_am_shell", "--scripts-path", "Scripts.toml"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Test script executed"));
}

#[test]
fn test_i_am_shell_obj() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "i_am_shell_obj", "--scripts-path", "Scripts.toml"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Detect shell script"))
        .stdout(predicates::str::contains("Test script executed"));
}

#[test]
fn test_build() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "build", "--scripts-path", "Scripts.toml"])
        .assert()
        .success()
        .stdout(predicates::str::contains("build"));
}

#[test]
fn test_release() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "release", "--scripts-path", "Scripts.toml"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Test script executed"))
        .stdout(predicates::str::contains("build"));
}

#[test]
fn test_release_info() {
    setup_test_scripts();

    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "release_info", "--scripts-path", "Scripts.toml"])
        .assert()
        .success()
        .stdout(predicates::str::contains("Release info"))
        .stdout(predicates::str::contains("Test script executed"))
        .stdout(predicates::str::contains("build"));
}
