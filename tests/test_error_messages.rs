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
    let test_file = format!("Scripts_test_error_{}_{}.toml", test_name, timestamp);
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

/// Tests that script not found error shows suggestions
#[test]
fn test_script_not_found_with_suggestions() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
test = "cargo test"
release = "cargo build --release"
"#,
        "not_found",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "buil", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stderr(contains("Script not found"))
        .stderr(contains("Did you mean"))
        .stderr(contains("build"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that script not found error shows available scripts when no close match
#[test]
fn test_script_not_found_shows_available() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
test = "cargo test"
"#,
        "available",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "xyz", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stderr(contains("Script not found"))
        .stderr(contains("Available scripts"))
        .stderr(contains("build"))
        .stderr(contains("test"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that invalid TOML shows error message
#[test]
fn test_invalid_toml_error() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
invalid = { command = "test", }
"#,
        "invalid_toml",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "invalid", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stderr(contains("Invalid TOML"))
        .stderr(contains("Quick fix"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that missing script file shows helpful error
#[test]
fn test_missing_script_file() {
    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "test", "--scripts-path", "nonexistent_file.toml"])
        .assert()
        .failure()
        .stderr(contains("Script file not found"))
        .stderr(contains("Quick fix"));
}

/// Tests that script not found suggests similar names (fuzzy matching)
#[test]
fn test_fuzzy_matching_suggestions() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
prepublish = { include = ["clean", "build"] }
prepublish_clean = "cargo clean"
prepublish_doc = "cargo doc"
"#,
        "fuzzy",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "prepublis", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stderr(contains("Did you mean"))
        .stderr(contains("prepublish"));
    
    cleanup_scripts_toml(&test_file);
}

/// Tests that error message includes suggestion to use 'show' command
#[test]
fn test_error_suggests_show_command() {
    let test_file = setup_scripts_toml(
        r#"
[scripts]
build = "cargo build"
"#,
        "show_suggestion",
    );

    let mut cmd = cargo_bin_cmd!("cargo-script");
    cmd.args(&["run", "unknown", "--scripts-path", &test_file])
        .assert()
        .failure()
        .stderr(contains("show"))
        .stderr(contains("available scripts"));
    
    cleanup_scripts_toml(&test_file);
}

