//! Structured JSON output for `--json` mode.

use crate::commands::script::ExecutionResult;

/// Pretty-print an [`ExecutionResult`] as JSON on stdout.
///
/// We intentionally keep this human-readable (pretty) so it can be consumed
/// either by tools (`jq`, GitHub Actions output, …) or by humans staring at a
/// terminal.
pub fn print_execution_result(result: &ExecutionResult) {
    match serde_json::to_string_pretty(result) {
        Ok(s) => println!("{}", s),
        Err(e) => eprintln!("{{\"error\":\"failed to serialize result: {}\"}}", e),
    }
}

/// Convenience: serialize without printing (used by tests).
pub fn to_string(result: &ExecutionResult) -> String {
    serde_json::to_string_pretty(result).unwrap_or_else(|e| format!("{{\"error\":\"{}\"}}", e))
}
