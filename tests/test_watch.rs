//! Smoke test for the watch module's surface area.
//!
//! A real watch loop blocks the thread, so we limit ourselves to verifying
//! that the module is reachable and the default config is sensible.

#![cfg(feature = "watch")]

use cargo_run::commands::watch::WatchConfig;

#[test]
fn watch_config_defaults_are_sane() {
    let cfg = WatchConfig::default();
    assert!(!cfg.watch_paths.is_empty());
    assert!(cfg.exclude.iter().any(|e| e.contains("target")));
    assert!(cfg.debounce.as_millis() > 0);
}
