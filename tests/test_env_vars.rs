use assert_cmd::Command;
use std::fs;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up the Scripts.toml file with the specified content.
#[allow(dead_code)]
fn setup_scripts_toml(content: &str) {
    fs::write("Scripts.toml", content).unwrap();
}

/// Tests the `test01_env` script defined in `Scripts.toml`.
/// This script sets the `EXAMPLE_VAR` to `change_value` and keeps the global `RUST_LOG` value.
#[test]
fn test01_env() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "test01_env", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("change_value"));
}

/// Tests the `test02_env` script defined in `Scripts.toml`.
/// This script overrides the global `RUST_LOG` value.
#[test]
fn test02_env() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "test02_env", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("warn"));
}

/// Tests the `test03_env` script defined in `Scripts.toml`.
/// This script sets `EXAMPLE_VAR` to `change_value_again` and overrides the global `RUST_LOG` value.
#[test]
fn test03_env() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "test03_env", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("change_value_again"))
        .stdout(predicates::str::contains("info"));
}
