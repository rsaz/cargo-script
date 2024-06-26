//! Main entry point for the cgs shortname CLI tool.
//! 
//! This module initializes and runs the CLI using the `cargo-script` library.
use cargo_run::start;

/// Main function that runs the CLI tool.
fn main() {
    start::run();
}