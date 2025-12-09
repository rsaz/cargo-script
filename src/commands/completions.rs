//! This module provides shell completion generation functionality.

use clap_complete::{generate, Shell as ClapShell};
use crate::commands::Shell;
use std::io;
use std::env;

/// Generate shell completion script for the specified shell.
///
/// # Arguments
///
/// * `shell` - The shell type to generate completions for
/// * `app` - The clap Command struct to generate completions from
pub fn generate_completions(shell: Shell, app: &mut clap::Command) {
    let clap_shell = match shell {
        Shell::Bash => ClapShell::Bash,
        Shell::Zsh => ClapShell::Zsh,
        Shell::Fish => ClapShell::Fish,
        Shell::PowerShell => ClapShell::PowerShell,
    };

    // Detect the binary name from the command line arguments
    // This allows completions to work for both 'cargo-script' and 'cgs'
    let binary_name = env::args()
        .next()
        .and_then(|path| {
            std::path::Path::new(&path)
                .file_name()
                .and_then(|name| name.to_str())
                .map(|s| {
                    // Strip .exe extension on Windows for cleaner completion registration
                    s.strip_suffix(".exe").unwrap_or(s).to_string()
                })
        })
        .unwrap_or_else(|| "cargo-script".to_string());

    generate(clap_shell, app, &binary_name, &mut io::stdout());
}

