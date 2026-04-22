//! Project templates: a curated set of `Scripts.toml` + ancillary file
//! starters for common scenarios (CI/CD, multi-crate workspaces, libraries…).
//!
//! Templates are kept in dedicated submodules so adding a new one is a single
//! file + one entry in [`registry`].

use std::fs;
use std::path::Path;

use colored::*;

use crate::error::CargoScriptError;

pub mod github_actions;
pub mod gitlab_ci;
pub mod rust_project;
pub mod workspace;

/// A single output file produced by a template.
#[derive(Debug, Clone)]
pub struct TemplateFile {
    /// Path relative to the project root.
    pub path: &'static str,
    /// File contents.
    pub contents: &'static str,
}

/// A template descriptor.
#[derive(Debug, Clone)]
pub struct Template {
    pub name: &'static str,
    pub description: &'static str,
    pub files: &'static [TemplateFile],
}

/// All registered templates.
pub fn registry() -> Vec<Template> {
    vec![
        github_actions::TEMPLATE,
        gitlab_ci::TEMPLATE,
        rust_project::TEMPLATE,
        workspace::TEMPLATE,
    ]
}

/// Look up a template by name.
pub fn find(name: &str) -> Option<Template> {
    registry().into_iter().find(|t| t.name == name)
}

/// Print a human-friendly listing of all templates.
pub fn print_list() {
    println!("{}", "Available templates:".cyan().bold());
    println!();
    for t in registry() {
        println!("  • {}  —  {}", t.name.green().bold(), t.description);
    }
    println!();
    println!(
        "{}\n  cargo script init --template <name>",
        "Usage:".yellow().bold()
    );
}

/// Materialise a template on disk, refusing to overwrite existing files
/// unless `force` is set.
pub fn apply(template_name: &str, force: bool) -> Result<(), CargoScriptError> {
    let template = find(template_name).ok_or_else(|| CargoScriptError::TemplateNotFound {
        name: template_name.to_string(),
        available: registry().iter().map(|t| t.name.to_string()).collect(),
    })?;

    println!(
        "{} {}",
        "✨ Applying template:".cyan().bold(),
        template.name.green().bold()
    );
    println!("   {}", template.description.italic());
    println!();

    for file in template.files {
        let p = Path::new(file.path);
        if p.exists() && !force {
            println!(
                "  {} {} (exists; pass --force to overwrite)",
                "⚠️ ".yellow(),
                file.path.yellow()
            );
            continue;
        }
        if let Some(parent) = p.parent() {
            if !parent.as_os_str().is_empty() {
                fs::create_dir_all(parent).map_err(|e| CargoScriptError::ScriptFileNotFound {
                    path: parent.display().to_string(),
                    source: e,
                })?;
            }
        }
        fs::write(p, file.contents).map_err(|e| CargoScriptError::ScriptFileNotFound {
            path: file.path.to_string(),
            source: e,
        })?;
        println!("  {} {}", "✓".green(), file.path);
    }

    println!();
    println!(
        "{} Run '{}' to see what's available.",
        "Next:".yellow().bold(),
        "cargo script show".green()
    );
    Ok(())
}
