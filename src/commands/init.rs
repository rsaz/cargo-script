//! `Scripts.toml` initialiser.
//!
//! Supports a default starter file (backward-compatible behaviour) and named
//! templates from [`crate::commands::templates`].

use std::{fs, io};
use colored::*;
use emoji::symbols;

use crate::error::CargoScriptError;

/// Initialise a `Scripts.toml` file.
///
/// * If `template` is provided, materialise the named template (creating
///   ancillary files such as `.github/workflows/ci.yml` as needed).
/// * Otherwise, write the legacy default starter (with overwrite confirmation).
pub fn init_script_file(file_path: Option<&str>) {
    let file_path = file_path.unwrap_or("Scripts.toml");
    if fs::metadata(file_path).is_ok() {
        println!(
            "{}  [ {} ] already exists. Do you want to replace it? ({}/{})",
            symbols::warning::WARNING.glyph,
            file_path.yellow(),
            "y".green(),
            "n".red()
        );
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read input");
        if input.trim().to_lowercase() != "y" {
            println!("Operation cancelled.");
            return;
        }
    }
    let default_content = r#"
[global_env]

[scripts]
dev = "cargo run"
build = { command = "cargo build", env = { RUST_LOG = "info" } }
release = "cargo build --release"
test = { command = "cargo test", env = { RUST_LOG = "warn" } }
doc = "cargo doc --no-deps --open"
"#;
    fs::write(file_path, default_content).expect("Failed to write Scripts.toml");
    println!("{}  [ {} ] has been created.", symbols::other_symbol::CHECK_MARK.glyph, "Scripts.toml".green());
}

/// Apply a named template, optionally overwriting existing files.
pub fn init_from_template(template: &str, force: bool) -> Result<(), CargoScriptError> {
    crate::commands::templates::apply(template, force)
}

/// Print all registered templates.
pub fn list_templates() {
    crate::commands::templates::print_list();
}
