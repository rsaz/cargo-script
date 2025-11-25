use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up the Scripts.toml file with the specified content.
#[allow(dead_code)]
fn setup_scripts_toml(content: &str) {
    fs::write("Scripts.toml", content).unwrap();
}

/// Get the script name based on the OS
fn get_script_name(base_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        base_name.to_string()
    }
    #[cfg(not(target_os = "windows"))]
    {
        format!("{}_linux", base_name)
    }
}

/// Tests the `test01_env` script defined in `Scripts.toml`.
/// This script sets the `EXAMPLE_VAR` to `change_value` and keeps the global `RUST_LOG` value.
/// Windows version uses PowerShell, Linux version uses bash.
#[test]
fn test01_env() {
    let script_name = get_script_name("test01_env");
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", &script_name, "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("change_value"));
}

/// Tests the `test02_env` script defined in `Scripts.toml`.
/// This script overrides the global `RUST_LOG` value.
/// Windows version uses PowerShell, Linux version uses bash.
#[test]
fn test02_env() {
    let script_name = get_script_name("test02_env");
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", &script_name, "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("warn"));
}

/// Tests the `test03_env` script defined in `Scripts.toml`.
/// This script sets `EXAMPLE_VAR` to `change_value_again` and overrides the global `RUST_LOG` value.
/// Windows version uses PowerShell, Linux version uses bash.
#[test]
fn test03_env() {
    let script_name = get_script_name("test03_env");
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", &script_name, "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("change_value_again"))
        .stdout(predicates::str::contains("info"));
}
