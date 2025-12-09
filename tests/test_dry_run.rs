use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;
use std::path::Path;

/// Sets up a temporary Scripts.toml file with the specified content.
/// Returns the path to the temporary file.
fn setup_scripts_toml(content: &str, test_name: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = format!("Scripts_test_dry_run_{}_{}.toml", test_name, timestamp);
    fs::write(&test_file, content).unwrap();
    test_file
}

/// Clean up the temporary Scripts.toml file.
fn cleanup_scripts_toml(file: &str) {
    if Path::new(file).exists() {
        fs::remove_file(file).ok();
    }
}

/// Tests that dry-run mode shows execution plan without executing
#[test]
fn test_dry_run_simple_script() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = "echo 'Hello World'"
"#,
        "simple_script",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "test_script", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Would run script"))
        .stdout(contains("test_script"))
        .stdout(contains("echo 'Hello World'"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode shows script chains (includes)
#[test]
fn test_dry_run_script_chain() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
script1 = "echo 'Script 1'"
script2 = "echo 'Script 2'"
combined = { include = ["script1", "script2"] }
"#,
        "script_chain",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "combined", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Would run include scripts"))
        .stdout(contains("script1"))
        .stdout(contains("script2"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode shows environment variables
#[test]
fn test_dry_run_with_env_vars() {
    let test_file = setup_scripts_toml(
        r#"
[global_env]
GLOBAL_VAR = "global_value"

[scripts]
test_env = { command = "echo $TEST_VAR", env = { TEST_VAR = "test_value" } }
"#,
        "env_vars",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "test_env", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Environment variables"))
        .stdout(contains("TEST_VAR"))
        .stdout(contains("test_value"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode shows command-line env overrides
#[test]
fn test_dry_run_with_env_override() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_override = { command = "echo $OVERRIDE_VAR", env = { OVERRIDE_VAR = "original" } }
"#,
        "env_override",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&[
        "run",
        "test_override",
        "--dry-run",
        "--env",
        "OVERRIDE_VAR=overridden",
        "--scripts-path",
        &test_file,
    ])
    .assert()
    .success()
    .stdout(contains("DRY-RUN MODE"))
    .stdout(contains("OVERRIDE_VAR"))
    .stdout(contains("overridden"))
    .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode shows interpreter information
#[test]
fn test_dry_run_with_interpreter() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
bash_script = { interpreter = "bash", command = "echo 'bash script'" }
"#,
        "interpreter",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "bash_script", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Interpreter"))
        .stdout(contains("bash"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode shows toolchain information
#[test]
fn test_dry_run_with_toolchain() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
toolchain_script = { command = "cargo build", toolchain = "stable" }
"#,
        "toolchain",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "toolchain_script", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Toolchain"))
        .stdout(contains("stable"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode doesn't actually execute commands
#[test]
fn test_dry_run_no_execution() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
should_not_run = "echo 'This should not appear'"
"#,
        "no_execution",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    let assert = cmd.args(&["run", "should_not_run", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success();
    
    let output = String::from_utf8_lossy(&assert.get_output().stdout);
    
    // Verify dry-run mode indicators
    assert!(output.contains("DRY-RUN MODE"));
    assert!(output.contains("No commands were actually executed"));
    assert!(output.contains("Would run script"));
    
    // The command string appears in the plan (which is fine), but the actual echo output should NOT appear
    // Since we're not executing, there should be no "This should not appear" as standalone output
    // (it only appears as part of the command string in the plan)
    let lines: Vec<&str> = output.lines().collect();
    let has_standalone_output = lines.iter().any(|line| {
        line.trim() == "This should not appear" && !line.contains("Command:")
    });
    assert!(!has_standalone_output, "Command was executed when it shouldn't have been");
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that dry-run mode handles script not found gracefully
#[test]
fn test_dry_run_script_not_found() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
existing_script = "echo 'exists'"
"#,
        "not_found",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    let assert = cmd.args(&["run", "nonexistent", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .failure(); // Now returns error instead of showing in dry-run
    
    let stderr = String::from_utf8_lossy(&assert.get_output().stderr);
    assert!(stderr.contains("Script not found"));
    assert!(stderr.contains("nonexistent"));
    assert!(stderr.contains("Did you mean") || stderr.contains("Available scripts"));
    
    cleanup_scripts_toml(&test_file);
}
