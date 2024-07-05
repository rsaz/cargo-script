//! Main entry point for the cgs shortname CLI tool.
//!
//! This module initializes and runs the CLI using the `cargo-script` library.
use cargo_run::start;

/// Main function that runs the CLI tool.
///
/// This function serves as the entry point for the application, initializing
/// and running the command-line interface (CLI) by calling `start::run()`.
fn main() {
    start::run();
}
