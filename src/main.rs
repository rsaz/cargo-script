//! Main entry point for the cargo-script CLI tool.
//! 
//! This module initializes and runs the CLI using the `cargo-script` library.
use cargo_script::start;

/// Main function that runs the CLI tool.
fn main() {
    start::run();
}