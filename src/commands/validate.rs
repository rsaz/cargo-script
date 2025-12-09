//! This module provides validation functionality for Scripts.toml files.
//!
//! It validates syntax, script references, and tool requirements.

use crate::commands::script::{Scripts, Script};
use std::collections::HashSet;
use std::process::Command;
use colored::*;

/// Validation result containing all issues found.
#[derive(Debug, Default)]
pub struct ValidationResult {
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// Validation error with context.
#[derive(Debug)]
pub struct ValidationError {
    pub script: Option<String>,
    pub message: String,
}

/// Validation warning with context.
#[derive(Debug)]
pub struct ValidationWarning {
    pub script: Option<String>,
    pub message: String,
}

impl ValidationResult {
    /// Check if validation passed (no errors).
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    /// Add an error to the validation result.
    pub fn add_error(&mut self, script: Option<String>, message: String) {
        self.errors.push(ValidationError { script, message });
    }

    /// Add a warning to the validation result.
    pub fn add_warning(&mut self, script: Option<String>, message: String) {
        self.warnings.push(ValidationWarning { script, message });
    }
}

/// Validate the Scripts.toml file.
///
/// This function performs comprehensive validation including:
/// - Script reference validation (includes)
/// - Tool requirement checking
/// - Toolchain validation
///
/// # Arguments
///
/// * `scripts` - The parsed Scripts collection to validate
///
/// # Returns
///
/// A ValidationResult containing all errors and warnings found.
pub fn validate_scripts(scripts: &Scripts) -> ValidationResult {
    let mut result = ValidationResult::default();
    let script_names: HashSet<&String> = scripts.scripts.keys().collect();

    // Validate each script
    for (script_name, script) in &scripts.scripts {
        validate_script(script_name, script, &script_names, &mut result);
    }

    result
}

/// Validate a single script.
fn validate_script(
    script_name: &str,
    script: &Script,
    available_scripts: &HashSet<&String>,
    result: &mut ValidationResult,
) {
    match script {
        Script::Default(_) => {
            // Simple scripts don't need validation beyond syntax
        }
        Script::Inline {
            include,
            requires,
            toolchain,
            ..
        } | Script::CILike {
            include,
            requires,
            toolchain,
            ..
        } => {
            // Validate includes
            if let Some(includes) = include {
                for include_name in includes {
                    if !available_scripts.contains(include_name) {
                        result.add_error(
                            Some(script_name.to_string()),
                            format!("Script '{}' references non-existent script '{}'", script_name, include_name),
                        );
                    }
                }
            }

            // Validate tool requirements
            if let Some(reqs) = requires {
                for req in reqs {
                    validate_requirement(script_name, req, result);
                }
            }

            // Validate toolchain
            if let Some(tc) = toolchain {
                validate_toolchain(script_name, tc, result);
            }
        }
    }
}

/// Validate a tool requirement.
fn validate_requirement(script_name: &str, requirement: &str, result: &mut ValidationResult) {
    if let Some((tool, version_req)) = requirement.split_once(' ') {
        // Check if tool exists
        let output = Command::new(tool).arg("--version").output();
        match output {
            Ok(output_result) => {
                let output_str = String::from_utf8_lossy(&output_result.stdout);
                let version_line = output_str.lines().next().unwrap_or("");
                
                // For simple version checks (contains), use substring matching
                // For complex version checks (>=, <=, etc.), we'll do basic validation
                if version_req.starts_with(">=") || version_req.starts_with("<=") || 
                   version_req.starts_with(">") || version_req.starts_with("<") {
                    // Complex version comparison - for now, just check if tool exists
                    // Full semantic version comparison would require a version parsing library
                    result.add_warning(
                        Some(script_name.to_string()),
                        format!(
                            "Tool '{}' found (version: {}), but complex version requirement '{}' validation is limited",
                            tool,
                            version_line,
                            version_req
                        ),
                    );
                } else if !version_line.contains(version_req) {
                    // Simple version check (contains)
                    result.add_error(
                        Some(script_name.to_string()),
                        format!(
                            "Tool '{}' version requirement '{}' not met. Found: {}",
                            tool,
                            version_req,
                            version_line
                        ),
                    );
                }
            }
            Err(_) => {
                result.add_error(
                    Some(script_name.to_string()),
                    format!("Required tool '{}' is not installed or not in PATH", tool),
                );
            }
        }
    } else {
        // Just check if tool exists
        let output = Command::new(requirement).output();
        if output.is_err() {
            result.add_error(
                Some(script_name.to_string()),
                format!("Required tool '{}' is not installed or not in PATH", requirement),
            );
        }
    }
}

/// Validate a toolchain requirement.
fn validate_toolchain(script_name: &str, toolchain: &str, result: &mut ValidationResult) {
    // Check if it's a Python toolchain (python:X.Y format)
    if toolchain.starts_with("python:") {
        let python_version = toolchain.strip_prefix("python:").unwrap_or("");
        // Check if Python is installed
        let output = Command::new("python").arg("--version").output()
            .or_else(|_| Command::new("python3").arg("--version").output());
        
        match output {
            Ok(output_result) => {
                let output_str = String::from_utf8_lossy(&output_result.stdout);
                if !output_str.contains(python_version) {
                    result.add_warning(
                        Some(script_name.to_string()),
                        format!(
                            "Python toolchain '{}' requirement: Python found ({}), but version '{}' not verified",
                            toolchain,
                            output_str.trim(),
                            python_version
                        ),
                    );
                }
            }
            Err(_) => {
                result.add_error(
                    Some(script_name.to_string()),
                    format!("Python toolchain '{}' required but Python is not installed or not in PATH", toolchain),
                );
            }
        }
    } else {
        // Rust toolchain validation via rustup
        let output = Command::new("rustup")
            .arg("toolchain")
            .arg("list")
            .output();

        match output {
            Ok(output_result) => {
                let output_str = String::from_utf8_lossy(&output_result.stdout);
                if !output_str.contains(toolchain) {
                    result.add_error(
                        Some(script_name.to_string()),
                        format!("Required Rust toolchain '{}' is not installed", toolchain),
                    );
                }
            }
            Err(_) => {
                result.add_error(
                    Some(script_name.to_string()),
                    "rustup is not installed or not in PATH".to_string(),
                );
            }
        }
    }
}

/// Print validation results in a user-friendly format.
pub fn print_validation_results(result: &ValidationResult) {
    if result.is_valid() && result.warnings.is_empty() {
        println!("{}", "✓ All validations passed!".green().bold());
        return;
    }

    if !result.errors.is_empty() {
        println!("\n{}", "❌ Validation Errors:".red().bold());
        for (idx, error) in result.errors.iter().enumerate() {
            if let Some(script) = &error.script {
                println!(
                    "  {}. Script '{}': {}",
                    idx + 1,
                    script.bold().yellow(),
                    error.message.red()
                );
            } else {
                println!("  {}. {}", idx + 1, error.message.red());
            }
        }
    }

    if !result.warnings.is_empty() {
        println!("\n{}", "⚠️  Validation Warnings:".yellow().bold());
        for (idx, warning) in result.warnings.iter().enumerate() {
            if let Some(script) = &warning.script {
                println!(
                    "  {}. Script '{}': {}",
                    idx + 1,
                    script.bold().yellow(),
                    warning.message.yellow()
                );
            } else {
                println!("  {}. {}", idx + 1, warning.message.yellow());
            }
        }
    }

    println!();
    if result.is_valid() {
        println!("{}", "✓ Validation completed with warnings".green().bold());
    } else {
        println!(
            "{}",
            format!("✗ Found {} error(s)", result.errors.len()).red().bold()
        );
    }
}

