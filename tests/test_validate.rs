use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;

/// Sets up a temporary Scripts.toml file with the specified content.
fn setup_scripts_toml(content: &str, test_name: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = format!("Scripts_test_validate_{}_{}.toml", test_name, timestamp);
    fs::write(&test_file, content).unwrap();
    test_file
}

/// Clean up the temporary Scripts.toml file.
fn cleanup_scripts_toml(file: &str) {
    use std::path::Path;
    if Path::new(file).exists() {
        fs::remove_file(file).ok();
    }
}

/// Tests that valid Scripts.toml passes validation
#[test]
fn test_validate_valid_scripts() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
test = "cargo test"
"#,
        "valid",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("All validations passed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation detects missing script references in includes
#[test]
fn test_validate_missing_include() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
release = { include = ["build", "nonexistent_script"], info = "Release build" }
"#,
        "missing_include",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stdout(contains("Validation Errors"))
        .stdout(contains("non-existent script"))
        .stdout(contains("nonexistent_script"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation detects multiple missing includes
#[test]
fn test_validate_multiple_missing_includes() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script1 = { include = ["missing1", "missing2"], info = "Script with multiple missing includes" }
"#,
        "multiple_missing",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    let assert = cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure();
    
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    // Should report both missing scripts
    assert!(output.contains("missing1"));
    assert!(output.contains("missing2"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation detects missing tools
#[test]
fn test_validate_missing_tool() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script_with_tool = { command = "nonexistent_tool --version", requires = ["nonexistent_tool"], info = "Script requiring nonexistent tool" }
"#,
        "missing_tool",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stdout(contains("Validation Errors"))
        .stdout(contains("not installed"))
        .stdout(contains("nonexistent_tool"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation detects missing Rust toolchain
#[test]
fn test_validate_missing_toolchain() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script_with_toolchain = { command = "cargo build", toolchain = "nonexistent-toolchain-999", info = "Script requiring nonexistent toolchain" }
"#,
        "missing_toolchain",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stdout(contains("Validation Errors"))
        .stdout(contains("toolchain"))
        .stdout(contains("not installed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation works with nested includes
#[test]
fn test_validate_nested_includes() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
step1 = "echo step1"
step2 = "echo step2"
step3 = { include = ["step1", "step2"], info = "Combines step1 and step2" }
final = { include = ["step3"], info = "Final step" }
"#,
        "nested",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("All validations passed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation detects missing script in nested includes
#[test]
fn test_validate_nested_missing_include() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
step1 = "echo step1"
step2 = { include = ["missing"], info = "References missing script" }
final = { include = ["step2"], info = "References step2 which has missing include" }
"#,
        "nested_missing",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stdout(contains("Validation Errors"))
        .stdout(contains("missing"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation reports all issues, not just the first
#[test]
fn test_validate_reports_all_issues() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script1 = { include = ["missing1"], info = "First script with missing include" }
script2 = { include = ["missing2"], info = "Second script with missing include" }
script3 = { requires = ["nonexistent_tool"], info = "Script with missing tool" }
"#,
        "all_issues",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    let assert = cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure();
    
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    // Should report all three issues
    assert!(output.contains("missing1") || output.contains("script1"));
    assert!(output.contains("missing2") || output.contains("script2"));
    assert!(output.contains("nonexistent_tool") || output.contains("script3"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation handles CILike format scripts
#[test]
fn test_validate_cilike_format() {
    let test_file = setup_scripts_toml(
        r#"
[scripts.build]
script = "build"
command = "cargo build"
info = "Build the project"

[scripts.test]
script = "test"
command = "cargo test"
info = "Test the project"

[scripts.release]
script = "release"
include = ["build", "test"]
info = "Release build"
"#,
        "cilike",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("All validations passed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation exits with code 0 on success
#[test]
fn test_validate_exit_code_success() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
"#,
        "exit_success",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .success()
        .code(0);
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that validation exits with non-zero code on failure
#[test]
fn test_validate_exit_code_failure() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script = { include = ["nonexistent"], info = "Script with missing include" }
"#,
        "exit_failure",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["validate", "--scripts-path", &test_file])
        .assert()
        .failure()
        .code(1);
    
    cleanup_scripts_toml(&test_file);
}

