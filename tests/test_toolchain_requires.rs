use assert_cmd::Command;

mod constants;
use constants::SCRIPT_TOML;

/// Tests the `test_requires` script defined in `Scripts.toml`.
/// This script requires a specific version of a tool (e.g., rustup).
#[test]
fn test_requires() {
    let mut cmd = Command::cargo_bin("cargo-script").unwrap();
    cmd.args(&["run", "test_requires", "--scripts-path", SCRIPT_TOML])
        .assert()
        .stderr(predicates::str::contains("Requirement check failed"))
        .failure();
}
