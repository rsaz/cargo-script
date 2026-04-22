use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::sync::Mutex;

mod constants;
use constants::SCRIPT_TOML;

/// Sets up `.scripts/test_script.sh` so the bash-interpreter tests can execute it.
///
/// This used to remove and re-create the file on every call, then shell out to
/// `chmod +x`. That approach raced badly under `cargo test`'s parallel runner on
/// Linux: when one test was mid-`cmd.assert()` and another test re-entered this
/// helper, `fs::write` would recreate the file with the default umask (`0o644`)
/// because `OpenOptions` only applies the mode on creation. Between that write
/// and the subsequent `chmod`, the in-flight `bash` would observe a non-
/// executable file and exit with status 126 (`Permission denied`).
///
/// The new implementation is idempotent and serialised:
///   * a process-wide `Mutex` orders concurrent setups within this test binary;
///   * the file is only rewritten when its contents differ, so we never demote a
///     valid file to `0o644`;
///   * permissions are set via `fs::set_permissions` (no subprocess).
fn setup_test_scripts() {
    static SETUP_LOCK: Mutex<()> = Mutex::new(());
    let _guard = SETUP_LOCK.lock().unwrap_or_else(|p| p.into_inner());

    let script_content = "#!/usr/bin/env bash\necho \"Test script executed\"\n";
    fs::create_dir_all(".scripts").unwrap();
    let script_path = ".scripts/test_script.sh";

    let needs_write = match fs::read_to_string(script_path) {
        Ok(existing) => existing != script_content,
        Err(_) => true,
    };
    if needs_write {
        fs::write(script_path, script_content).unwrap();
    }

    #[cfg(not(target_os = "windows"))]
    {
        use std::os::unix::fs::PermissionsExt;
        let _ = fs::set_permissions(script_path, fs::Permissions::from_mode(0o755));
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

