# Cargo-script (.rs file) example

Two ways to invoke a Rust single-file package from `Scripts.toml`:

```bash
cd examples/cargo-scripts
cargo script greet
cargo script report                 # uses default title
cargo script report "My Title"      # positional
cargo script report title="Hi"      # named
```

Requires either a stable cargo with `cargo script` (RFC 3502, lands ~2026)
or a nightly toolchain (`rustup toolchain install nightly`). cargo-run
detects which is available automatically.
