use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
#[cfg(not(target_os = "windows"))]
use std::process::Command as ProcessCommand;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up the test scripts by creating a directory and a test script file,
/// and making the script executable.
/// Uses a unique filename to avoid conflicts when tests run in parallel.
fn setup_test_scripts() {
    use std::path::Path;
    use std::thread;
    use std::time::Duration;
    
    let script_content = r#"
#!/usr/bin/env bash
echo "Test script executed"
    "#;
    fs::create_dir_all(".scripts").unwrap();
    
    let script_path = ".scripts/test_script.sh";
    
    // Remove existing file if it exists, with retry logic for "Text file busy" errors
    // This handles the case where another test is still executing the script
    // Error code 26 (ETXTBSY) means "Text file busy" on Linux
    if Path::new(script_path).exists() {
        let mut retries = 10;
        while retries > 0 {
            match fs::remove_file(script_path) {
                Ok(_) => break,
                Err(e) => {
                    // Check for "Text file busy" error (code 26 on Linux)
                    // This can happen when the script is being executed by another test
                    let is_busy = e.raw_os_error() == Some(26) || 
                                  e.kind() == std::io::ErrorKind::PermissionDenied ||
                                  e.to_string().contains("busy") ||
                                  e.to_string().contains("Text file busy");
                    
                    if is_busy && retries > 1 {
                        // File is busy, wait and retry
                        retries -= 1;
                        thread::sleep(Duration::from_millis(50));
                    } else {
                        // Give up after retries or if it's a different error
                        break;
                    }
                }
            }
        }
    }
    
    // Write the script file
    fs::write(script_path, script_content).unwrap();
    
    // On Windows, chmod doesn't exist and files don't need to be made executable
    #[cfg(not(target_os = "windows"))]
    {
        ProcessCommand::new("chmod")
            .args(&["+x", script_path])
            .status()
            .expect("Failed to make test script executable");
    }
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

    let mut cmd = cargo_bin_cmd!("cargo-script");
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

    let mut cmd = cargo_bin_cmd!("cargo-script");
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
    let mut cmd = cargo_bin_cmd!("cargo-script");
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

    let mut cmd = cargo_bin_cmd!("cargo-script");
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

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "release_info", "--scripts-path", SCRIPT_TOML])
        .assert()
        .success()
        .stdout(predicates::str::contains("Release info"))
        .stdout(predicates::str::contains("Test script executed"))
        .stdout(predicates::str::contains("build"));
}

