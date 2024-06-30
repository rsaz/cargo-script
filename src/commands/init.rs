//! This module provides the functionality to initialize a `Scripts.toml` file.

use std::{fs, io};
use colored::*;
use emoji::symbols;

/// Initialize a `Scripts.toml` file in the current directory.
///
/// If the file already exists, it prompts the user for confirmation to replace it.
/// The function creates a default `Scripts.toml` file if the user agrees.
///
/// # Panics
///
/// This function will panic if it fails to read user input or write to the `Scripts.toml` file.
pub fn init_script_file() {
    let file_path = "Scripts.toml";
    if fs::metadata(file_path).is_ok() {
        println!("{}  [ {} ] already exists. Do you want to replace it? ({}/{})", symbols::warning::WARNING.glyph, file_path.yellow(), "y".green(), "n".red());
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
