use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up a temporary Scripts.toml file with the specified content.
fn setup_scripts_toml(content: &str, test_name: &str) -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = format!("Scripts_test_cargo_{}_{}.toml", test_name, timestamp);
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

/// Tests that cargo script run works when invoked as cargo subcommand
#[test]
fn test_cargo_script_run() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = "echo 'Hello from cargo script'"
"#,
        "run",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "run", "test_script", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("Hello from cargo script"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script init works
#[test]
fn test_cargo_script_init() {
    use std::path::Path;
    use std::time::{SystemTime, UNIX_EPOCH};
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let test_file = format!("Scripts_test_init_{}.toml", timestamp);
    
    // Make sure file doesn't exist
    if Path::new(&test_file).exists() {
        fs::remove_file(&test_file).ok();
    }

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "init", "--scripts-path", &test_file])
        .assert()
        .success();
    
    // Verify file was created
    assert!(Path::new(&test_file).exists(), "Scripts.toml should be created");
    
    // Verify content
    let content = fs::read_to_string(&test_file).unwrap();
    assert!(content.contains("[scripts]"));
    assert!(content.contains("dev"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script show works
#[test]
fn test_cargo_script_show() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = { command = "cargo build", info = "Build the project" }
test = { command = "cargo test", info = "Run tests" }
"#,
        "show",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "show", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("build"))
        .stdout(contains("test"))
        .stdout(contains("Build the project"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script validate works
#[test]
fn test_cargo_script_validate() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
test = "cargo test"
"#,
        "validate",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "validate", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("All validations passed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script completions works
#[test]
fn test_cargo_script_completions() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "completions", "bash"])
        .assert()
        .success()
        .stdout(contains("complete"));
}

/// Tests backward compatibility - direct invocation still works
#[test]
fn test_backward_compatibility_direct() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = "echo 'Direct invocation works'"
"#,
        "backward",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "test_script", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("Direct invocation works"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests backward compatibility - cgs alias still works
#[test]
fn test_backward_compatibility_cgs() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = "echo 'CGS alias works'"
"#,
        "cgs",
    );

    let mut cmd = cargo_bin_cmd!("cgs");
    cmd.args(&["run", "test_script", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("CGS alias works"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script run with --dry-run works
#[test]
fn test_cargo_script_dry_run() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = "echo 'This should not execute'"
"#,
        "dry_run",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "run", "test_script", "--dry-run", "--scripts-path", &test_file])
        .assert()
        .success()
        .stdout(contains("DRY-RUN MODE"))
        .stdout(contains("Would run script"))
        .stdout(contains("No commands were actually executed"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that cargo script run with --env works
#[test]
fn test_cargo_script_env_override() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
test_script = { command = "echo $TEST_VAR", env = { TEST_VAR = "default" } }
"#,
        "env",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["script", "run", "test_script", "--env", "TEST_VAR=overridden", "--scripts-path", &test_file])
        .assert()
        .success();
    
    cleanup_scripts_toml(&test_file);
}

