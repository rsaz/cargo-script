# Cargo Script (RFC 3502) Integration

`cargo-run` ships first-class support for [Rust RFC 3502](https://rust-lang.github.io/rfcs/3502-cargo-script.html)
"single-file packages" — the `.rs` files with embedded Cargo manifests that
will become a stable part of `cargo` itself.

You can keep your `.rs` automation scripts beside your `Scripts.toml` and
invoke them like any other task — even mixing them with shell commands in the
same pipeline.

## Two ways to declare a cargo script

### Implicit (path heuristic)

Any command whose first whitespace-separated token is an existing `.rs` file
is automatically routed through cargo-script:

```toml
[scripts]
validate = "./scripts/validate.rs"
```

### Explicit (`script_file`)

For more configuration (env vars, info, args), use the explicit form:

```toml
[scripts.validate]
script_file = "./scripts/validate.rs"
info        = "Run pre-commit validation"
env         = { RUST_LOG = "info" }
args        = ["target"]
defaults    = { target = "all" }
```

```bash
cargo script validate           # uses default 'all'
cargo script validate api       # passes 'api' as `target`
```

## Toolchain detection

cargo-run probes the host toolchain once per process:

1. If `cargo --list` reports a `script` subcommand → use stable
   `cargo /path/to/script.rs`.
2. Otherwise, if `rustup toolchain list` includes `nightly` → use
   `cargo +nightly -Zscript /path/to/script.rs`.
3. Otherwise, fail with [`CargoScriptError::CargoScriptNotAvailable`] and a
   helpful suggestion (install nightly).

You don't need to do anything — it just works.

## Example script

`scripts/validate.rs`:

```rust
#!/usr/bin/env -S cargo +nightly -Zscript
---cargo
[dependencies]
anyhow = "1"
---

fn main() -> anyhow::Result<()> {
    println!("Validating project...");
    Ok(())
}
```

`Scripts.toml`:

```toml
[scripts]
validate = "./scripts/validate.rs"

# Composes with everything else
ci = { include = ["validate", "fmt-check", "lint", "test"], info = "Full CI" }
```

## Why this is a big deal

Before RFC 3502, automation scripts were typically:

* Bash scripts (no Rust ecosystem, platform-specific)
* Python (extra runtime, dependency management overhead)
* Embedded `cargo install` of throwaway binaries

With cargo-run + cargo-script you can:

* Stay in the Rust ecosystem end-to-end (one toolchain, one syntax)
* Pin script dependencies via cargo's lockfile semantics
* Get type-checked, compiled, fast scripts
* Mix shell commands and Rust scripts in the same pipeline

## Mixing with hooks and workspace mode

Cargo scripts work transparently with every other cargo-run feature:

```toml
[scripts.audit]
script_file = "./scripts/audit.rs"
workspace   = "all"          # run the audit in every member crate
pre         = ["fmt-check"]  # run shell hook first
on_failure  = ["./scripts/notify.rs"]
```
