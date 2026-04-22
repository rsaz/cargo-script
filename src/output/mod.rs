//! Output formats for cargo-run.
//!
//! Today: structured JSON. The intent is for this module to grow into a
//! pluggable formatter system (junit, sarif, lcov, …).

pub mod json;
