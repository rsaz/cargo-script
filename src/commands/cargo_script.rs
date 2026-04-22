//! Integration with Cargo's single-file packages (RFC 3502, "cargo script").
//!
//! On stable Rust 1.93+ (when `cargo script` lands), invocation is direct:
//!     cargo /path/to/script.rs <args...>
//!
//! On older toolchains, we transparently fall back to:
//!     cargo +nightly -Zscript /path/to/script.rs <args...>
//!
//! Both forms support both the comment-frontmatter and the
//! `---cargo` ... `---` frontmatter styles.

use std::collections::HashMap;
use std::process::{Command, Stdio};
use std::sync::OnceLock;

use crate::error::CargoScriptError;

/// Cached detection result. We probe `cargo` once per process.
static SUPPORTS_STABLE: OnceLock<bool> = OnceLock::new();

/// Detect whether the host's stable cargo already understands single-file
/// packages (RFC 3502).
///
/// We probe `cargo script --version`. There are three cases:
///
/// 1. **RFC 3502 stable Cargo** — Cargo handles `script` itself and prints
///    its own version line, e.g. `cargo 1.96.0 (f2d3ce0bd 2026-03-21)`.
/// 2. **Third-party `cargo-script` on PATH** (including older versions of
///    *this* tool) — Cargo dispatches to the external binary, which prints
///    its own banner, e.g. `cargo-script 0.5.2`. This is *not* RFC 3502
///    and routing `.rs` files through it would fail.
/// 3. **No `script` subcommand at all** — exit non-zero.
///
/// We accept only case 1 by requiring the first line of stdout to look like
/// Cargo's own version line: starts with `cargo` followed by whitespace and
/// a digit (so `cargo-script 0.5.2`, `cargo-make 0.37.0`, etc. are
/// rejected).
pub fn stable_cargo_script_supported() -> bool {
    *SUPPORTS_STABLE.get_or_init(|| {
        let out = Command::new("cargo")
            .args(["script", "--version"])
            .stderr(Stdio::null())
            .output();
        match out {
            Ok(o) if o.status.success() => {
                let stdout = String::from_utf8_lossy(&o.stdout);
                stdout
                    .lines()
                    .next()
                    .map(is_builtin_cargo_version_line)
                    .unwrap_or(false)
            }
            _ => false,
        }
    })
}

/// Returns true when `line` looks like Cargo's own `--version` output
/// (e.g. `"cargo 1.96.0 (abc123 2026-04-01)"`), as opposed to a third-party
/// `cargo-<something>` subcommand banner.
pub fn is_builtin_cargo_version_line(line: &str) -> bool {
    let trimmed = line.trim();
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let head = parts.next().unwrap_or("");
    let rest = parts.next().unwrap_or("").trim_start();
    head == "cargo" && rest.chars().next().map(|c| c.is_ascii_digit()).unwrap_or(false)
}

/// Returns true when `cargo` is callable at all.
pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Returns true when `rustup` reports a `nightly` toolchain.
pub fn nightly_available() -> bool {
    let out = Command::new("rustup")
        .args(["toolchain", "list"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout).contains("nightly"),
        Err(_) => false,
    }
}

/// Which cargo invocation strategy to use for an `.rs` script file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CargoScriptInvocation {
    /// Stable cargo with RFC 3502: `cargo script <path> [-- <args>]`
    Stable,
    /// Nightly fallback: `cargo +nightly -Zscript <path> [-- <args>]`
    Nightly,
}

/// Build the argv for a `.rs` cargo-script invocation.
///
/// Pulled out of [`execute_cargo_script`] so it can be unit-tested without
/// requiring a nightly toolchain or a stable cargo with RFC 3502 enabled.
///
/// Returns `(program, args)`. `args` already includes the `--` separator
/// and forwarded `extra_args` when present.
pub fn build_cargo_script_argv(
    invocation: CargoScriptInvocation,
    path: &str,
    extra_args: &[String],
) -> (&'static str, Vec<String>) {
    let mut args: Vec<String> = match invocation {
        CargoScriptInvocation::Stable => vec!["script".into(), path.to_string()],
        CargoScriptInvocation::Nightly => {
            vec!["+nightly".into(), "-Zscript".into(), path.to_string()]
        }
    };
    if !extra_args.is_empty() {
        args.push("--".into());
        args.extend(extra_args.iter().cloned());
    }
    ("cargo", args)
}

/// Execute the cargo script located at `path`, forwarding `extra_args` and
/// the resolved environment.
///
/// `script_name` is used purely for error reporting.
pub fn execute_cargo_script(
    script_name: &str,
    path: &str,
    extra_args: &[String],
    env_vars: &HashMap<String, String>,
) -> Result<(), CargoScriptError> {
    if !cargo_available() {
        return Err(CargoScriptError::CargoScriptNotAvailable {
            suggestion: "  Install Rust toolchain from https://rustup.rs/".to_string(),
        });
    }

    let invocation = if stable_cargo_script_supported() {
        CargoScriptInvocation::Stable
    } else if nightly_available() {
        CargoScriptInvocation::Nightly
    } else {
        return Err(CargoScriptError::CargoScriptNotAvailable {
            suggestion:
                "  cargo script (RFC 3502) requires either a stable cargo with the \
                 feature stabilized, or `rustup toolchain install nightly`."
                    .to_string(),
        });
    };

    let (program, full_args) = build_cargo_script_argv(invocation, path, extra_args);
    // For error messages, keep the base invocation (without forwarded args).
    let base_args: Vec<String> = match invocation {
        CargoScriptInvocation::Stable => vec!["script".into(), path.to_string()],
        CargoScriptInvocation::Nightly => {
            vec!["+nightly".into(), "-Zscript".into(), path.to_string()]
        }
    };

    let mut cmd = Command::new(program);
    cmd.args(&full_args);
    cmd.stdout(Stdio::inherit()).stderr(Stdio::inherit());
    for (k, v) in env_vars {
        cmd.env(k, v);
    }

    let mut child = cmd
        .spawn()
        .map_err(|e| CargoScriptError::ExecutionError {
            script: script_name.to_string(),
            command: format!("{} {}", program, base_args.join(" ")),
            source: e,
        })?;

    let status = child.wait().map_err(|e| CargoScriptError::ExecutionError {
        script: script_name.to_string(),
        command: format!("{} {}", program, base_args.join(" ")),
        source: e,
    })?;

    if !status.success() {
        return Err(CargoScriptError::ExecutionError {
            script: script_name.to_string(),
            command: format!("{} {}", program, base_args.join(" ")),
            source: std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("cargo script exited with status: {}", status),
            ),
        });
    }

    Ok(())
}

/// Best-effort heuristic: does `path` look like a cargo script file?
pub fn looks_like_cargo_script(path: &str) -> bool {
    path.ends_with(".rs") && std::path::Path::new(path).exists()
}
