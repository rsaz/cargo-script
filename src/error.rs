//! Error handling module for cargo-script CLI tool.
//!
//! This module provides custom error types and utilities for better error messages.

use colored::*;
use std::fmt;

/// Custom error type for cargo-script operations.
#[derive(Debug)]
pub enum CargoScriptError {
    /// Script file not found or cannot be read
    ScriptFileNotFound {
        path: String,
        source: std::io::Error,
    },
    /// Invalid TOML syntax in Scripts.toml
    InvalidToml {
        path: String,
        message: String,
        line: Option<usize>,
    },
    /// Script not found in Scripts.toml
    ScriptNotFound {
        script_name: String,
        available_scripts: Vec<String>,
    },
    /// Required tool is missing or wrong version
    ToolNotFound {
        tool: String,
        required_version: Option<String>,
        suggestion: String,
    },
    /// Toolchain not installed
    ToolchainNotFound {
        toolchain: String,
        suggestion: String,
    },
    /// Script execution error
    ExecutionError {
        script: String,
        command: String,
        source: std::io::Error,
    },
    /// Windows self-replacement error (trying to replace cargo-script while it's running)
    WindowsSelfReplacementError {
        script: String,
        command: String,
    },
}

impl fmt::Display for CargoScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CargoScriptError::ScriptFileNotFound { path, source } => {
                write!(
                    f,
                    "{}\n\n{}\n  {}\n  {}\n\n{}\n  {}\n  {}",
                    "❌ Script file not found".red().bold(),
                    "Error:".yellow().bold(),
                    format!("Path: {}", path).white(),
                    format!("Reason: {}", source).white(),
                    "Quick fix:".yellow().bold(),
                    format!("Run '{}' to create Scripts.toml in the current directory", "cargo script init".green()).white(),
                    format!("Or use '{}' to specify a different file path", "--scripts-path <path>".green()).white()
                )
            }
            CargoScriptError::InvalidToml { path, message, line } => {
                let line_info = if let Some(l) = line {
                    format!("\n  Line {}: {}", l, "See error details above".yellow())
                } else {
                    String::new()
                };
                write!(
                    f,
                    "{}\n\n{}\n  {}\n  {}{}\n\n{}\n  {}\n  {}\n  {}",
                    "❌ Invalid TOML syntax".red().bold(),
                    "Error:".yellow().bold(),
                    format!("File: {}", path).white(),
                    format!("Message: {}", message).white(),
                    line_info,
                    "Quick fix:".yellow().bold(),
                    "Check your Scripts.toml syntax. Common issues:".white(),
                    "  - Missing quotes around strings\n  - Trailing commas in arrays\n  - Invalid table syntax".white(),
                    format!("Validate your file with: {}", "cargo script validate".green()).white()
                )
            }
            CargoScriptError::ScriptNotFound {
                script_name,
                available_scripts,
            } => {
                let suggestions = find_similar_scripts(script_name, available_scripts);
                let suggestion_text = if !suggestions.is_empty() {
                    format!(
                        "\n\n{}\n  {}",
                        "Did you mean:".yellow().bold(),
                        suggestions
                            .iter()
                            .map(|s| format!("  • {}", s.green()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else if !available_scripts.is_empty() {
                    format!(
                        "\n\n{}\n  {}",
                        "Available scripts:".yellow().bold(),
                        available_scripts
                            .iter()
                            .take(10)
                            .map(|s| format!("  • {}", s.cyan()))
                            .collect::<Vec<_>>()
                            .join("\n")
                    )
                } else {
                    String::new()
                };

                write!(
                    f,
                    "{}\n\n{}\n  {}{}\n\n{}\n  {}\n  {}",
                    "❌ Script not found".red().bold(),
                    "Error:".yellow().bold(),
                    format!("Script '{}' not found in Scripts.toml", script_name.bold()).white(),
                    suggestion_text,
                    "Quick fix:".yellow().bold(),
                    format!("Run '{}' to see all available scripts", "cargo script show".green()).white(),
                    format!("Or use '{}' to initialize Scripts.toml if it doesn't exist", "cargo script init".green()).white()
                )
            }
            CargoScriptError::ToolNotFound {
                tool,
                required_version,
                suggestion,
            } => {
                let version_info = if let Some(v) = required_version {
                    format!(" (required: {})", v)
                } else {
                    String::new()
                };
                write!(
                    f,
                    "{}\n\n{}\n  {}{}\n\n{}\n  {}",
                    "❌ Required tool not found".red().bold(),
                    "Error:".yellow().bold(),
                    format!("Tool '{}'{} is not installed or not in PATH", tool.bold(), version_info).white(),
                    suggestion,
                    "Suggestion:".yellow().bold(),
                    format!("Install '{}' and ensure it's available in your PATH", tool).white()
                )
            }
            CargoScriptError::ToolchainNotFound {
                toolchain,
                suggestion,
            } => {
                write!(
                    f,
                    "{}\n\n{}\n  {}\n\n{}\n{}",
                    "❌ Toolchain not installed".red().bold(),
                    "Error:".yellow().bold(),
                    format!("Toolchain '{}' is not installed", toolchain.bold()).white(),
                    "Suggestion:".yellow().bold(),
                    suggestion
                )
            }
            CargoScriptError::ExecutionError {
                script,
                command,
                source,
            } => {
                // Check if this is a Windows self-replacement error
                let is_windows_self_replace = cfg!(target_os = "windows")
                    && (command.contains("cargo install --path .") || command.contains("cargo install --path"))
                    && (source.to_string().contains("Access is denied") 
                        || source.to_string().contains("os error 5")
                        || source.to_string().contains("failed to move"));

                if is_windows_self_replace {
                    write!(
                        f,
                        "{}\n\n{}\n  {}\n  {}\n\n{}\n  {}\n  {}\n  {}\n\n{}\n  {}",
                        "❌ Cannot replace cargo-script while it's running (Windows limitation)".red().bold(),
                        "Error:".yellow().bold(),
                        format!("Script: {}", script.bold()).white(),
                        format!("Command: {}", command).white(),
                        "Why:".yellow().bold(),
                        "Windows locks executable files while they're running for security and stability.".white(),
                        "When cargo-script runs 'cargo install --path .', it tries to replace itself,".white(),
                        "but Windows prevents this because cargo-script.exe is currently in use.".white(),
                        "Solution:".yellow().bold(),
                        format!("Run '{}' directly in your terminal (not via cargo script)", command.green()).white()
                    )
                } else {
                    write!(
                        f,
                        "{}\n\n{}\n  {}\n  {}\n  {}\n\n{}\n  {}",
                        "❌ Script execution failed".red().bold(),
                        "Error:".yellow().bold(),
                        format!("Script: {}", script.bold()).white(),
                        format!("Command: {}", command).white(),
                        format!("Reason: {}", source).white(),
                        "Suggestion:".yellow().bold(),
                        "Check the command syntax and ensure all required tools are installed".white()
                    )
                }
            }
            CargoScriptError::WindowsSelfReplacementError { script, command } => {
                write!(
                    f,
                    "{}\n\n{}\n  {}\n  {}\n\n{}\n  {}\n  {}\n  {}\n\n{}\n  {}",
                    "❌ Cannot replace cargo-script while it's running (Windows limitation)".red().bold(),
                    "Error:".yellow().bold(),
                    format!("Script: {}", script.bold()).white(),
                    format!("Command: {}", command).white(),
                    "Why:".yellow().bold(),
                    "Windows locks executable files while they're running for security and stability.".white(),
                    "When cargo-script runs 'cargo install --path .', it tries to replace itself,".white(),
                    "but Windows prevents this because cargo-script.exe is currently in use.".white(),
                    "Solution:".yellow().bold(),
                    format!("Run '{}' directly in your terminal (not via cargo script)", command.green()).white()
                )
            }
        }
    }
}

impl std::error::Error for CargoScriptError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            CargoScriptError::ScriptFileNotFound { source, .. } => Some(source),
            CargoScriptError::ExecutionError { source, .. } => Some(source),
            CargoScriptError::WindowsSelfReplacementError { .. } => None,
            _ => None,
        }
    }
}

/// Find similar script names using Levenshtein distance.
fn find_similar_scripts(query: &str, available: &[String]) -> Vec<String> {
    if available.is_empty() {
        return Vec::new();
    }

    let mut candidates: Vec<(String, usize)> = available
        .iter()
        .map(|s| {
            let distance = levenshtein_distance(query, s);
            (s.clone(), distance)
        })
        .collect();

    // Sort by distance and take the top 3 matches
    candidates.sort_by_key(|(_, d)| *d);
    candidates
        .into_iter()
        .take(3)
        .filter(|(_, d)| *d <= query.len().max(3)) // Only suggest if reasonably close
        .map(|(s, _)| s)
        .collect()
}

/// Calculate Levenshtein distance between two strings.
fn levenshtein_distance(s1: &str, s2: &str) -> usize {
    let s1_chars: Vec<char> = s1.chars().collect();
    let s2_chars: Vec<char> = s2.chars().collect();
    let s1_len = s1_chars.len();
    let s2_len = s2_chars.len();

    if s1_len == 0 {
        return s2_len;
    }
    if s2_len == 0 {
        return s1_len;
    }

    let mut matrix = vec![vec![0; s2_len + 1]; s1_len + 1];

    for i in 0..=s1_len {
        matrix[i][0] = i;
    }
    for j in 0..=s2_len {
        matrix[0][j] = j;
    }

    for i in 1..=s1_len {
        for j in 1..=s2_len {
            let cost = if s1_chars[i - 1] == s2_chars[j - 1] { 0 } else { 1 };
            matrix[i][j] = (matrix[i - 1][j] + 1)
                .min(matrix[i][j - 1] + 1)
                .min(matrix[i - 1][j - 1] + cost);
        }
    }

    matrix[s1_len][s2_len]
}

/// Helper function to create a tool not found error with installation suggestions.
pub fn create_tool_not_found_error(tool: &str, required_version: Option<&str>) -> CargoScriptError {
    let suggestion = match tool {
        "rustup" => "Install rustup: https://rustup.rs/".to_string(),
        "cargo" => "Install Rust: https://www.rust-lang.org/tools/install".to_string(),
        "python" => "Install Python: https://www.python.org/downloads/".to_string(),
        "docker" => "Install Docker: https://docs.docker.com/get-docker/".to_string(),
        "kubectl" => "Install kubectl: https://kubernetes.io/docs/tasks/tools/".to_string(),
        _ => format!("Install {} from your package manager or official website", tool),
    };

    CargoScriptError::ToolNotFound {
        tool: tool.to_string(),
        required_version: required_version.map(|s| s.to_string()),
        suggestion: format!("  {}", suggestion.cyan()),
    }
}

/// Helper function to create a toolchain not found error.
pub fn create_toolchain_not_found_error(toolchain: &str) -> CargoScriptError {
    let suggestion = if toolchain.starts_with("python:") {
        format!("Install Python {} using your system package manager", toolchain.replace("python:", ""))
    } else {
        format!("Install toolchain: rustup toolchain install {}", toolchain)
    };

    CargoScriptError::ToolchainNotFound {
        toolchain: toolchain.to_string(),
        suggestion: format!("  {}", suggestion.cyan()),
    }
}

