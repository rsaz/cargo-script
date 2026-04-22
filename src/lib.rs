//! The cargo-run library (binary names: `cargo-script`, `cgs`).
//!
//! This crate exposes the building blocks used by the binaries — script
//! parsing, execution, workspace orchestration, cargo-script integration,
//! template rendering, hooks, parallel/watch execution and JSON output —
//! so that downstream tooling can embed cargo-run programmatically.

pub mod commands;
pub mod start;
pub mod error;
pub mod output;
