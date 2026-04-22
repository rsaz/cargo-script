---
title: "Wiring single-file Rust scripts (RFC 3502) into your project workflow"
description: "RFC 3502 brings cargo-script to stable. Here's how cargo-run 0.6 turns those .rs files into first-class tasks alongside your shell commands."
tags: ["rust", "rfc", "cargo-script", "tooling"]
canonical_url: "https://github.com/lpotthast/cargo-run"
published: false
---

# Wiring single-file Rust scripts into your project workflow

[RFC 3502](https://rust-lang.github.io/rfcs/3502-cargo-script.html)
("`cargo-script`") brings something Rust has wanted for years: **single-file
Rust scripts** with embedded Cargo manifests. You write a `.rs` file, run
`cargo script ./check_repo.rs`, and Cargo compiles + caches it transparently.

That's wonderful for one-off scripts. But real projects need more than
"a folder of `.rs` files you have to remember the names of". They need a
*registry* — somewhere that says *"this is the audit script, this is what it
needs, this is who runs it"*.

`cargo-run` 0.6 is that registry, and `Scripts.toml` understands `.rs`
scripts natively.

## The shape

```toml
[scripts.audit]
script_file = "./scripts/audit.rs"
info        = "Run a Rust audit script (RFC 3502)"
```

```bash
cargo script audit
```

That's it. `cargo-run` will:

1. Detect whether **stable `cargo script`** is available (Rust ≥ the version
   that stabilises RFC 3502).
2. Otherwise fall back to **`cargo +nightly -Zscript ./scripts/audit.rs`**.
3. Otherwise produce an **actionable error** telling the user how to enable
   it (rustup component or nightly toolchain).

You write the `Scripts.toml` once; users on stable, nightly, or even an
older toolchain get the right invocation automatically.

## A real example

Suppose you maintain a workspace and want a "check no crate has a
yanked dependency" script. Today that's a 60-line shell pipeline. With
RFC 3502 + `cargo-run`:

`scripts/check-yanked.rs`

```rust
#!/usr/bin/env -S cargo +nightly -Zscript
---
[dependencies]
serde_json = "1"
ureq       = "2"
---

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lock = std::fs::read_to_string("Cargo.lock")?;
    // …parse Cargo.lock, query crates.io, fail on yanked…
    Ok(())
}
```

`Scripts.toml`

```toml
[scripts.check-yanked]
script_file = "./scripts/check-yanked.rs"
info        = "Fail if any locked dependency is yanked"

[scripts.ci]
include = ["fmt", "lint", "test", "check-yanked"]
```

Now `cargo script ci` runs the Rust script alongside everything else, and
your CI doesn't need shell, jq, or curl.

## Why declare it in `Scripts.toml` instead of just `cargo script ./file.rs`?

Three reasons:

1. **Discoverability.** `cargo script show` lists every task — shell, Rust,
   workspace, hooks — in one place.
2. **Composition.** `pre`, `post`, `on_success`, `on_failure`, `include`,
   `args`, `requires`, `env` all work for `.rs` scripts the same way they
   work for shell commands.
3. **Toolchain hygiene.** `cargo-run` adds the nightly fallback only when
   needed, and surfaces "you're missing nightly" as a clean error instead of
   `error: unrecognized argument '-Zscript'`.

## What about embedded shebangs?

Fully supported. If you'd like to run the file directly as a script and also
expose it via `cargo-run`, both work. The declaration is the source of
truth for *project workflow*; the shebang is for *interactive use*.

## A look at the future

When stable Cargo ships RFC 3502, we plan to:

- Default to stable `cargo script` automatically (no `+nightly` fallback).
- Add `cargo script init --template rust-script` to scaffold a
  ready-to-edit `.rs` script with the front-matter and a Scripts.toml entry.
- Surface `script_file` in `--json` output so CI can attribute time to the
  Rust script vs. its dependencies.

Full reference: [Cargo-Script Integration](../CARGO_SCRIPT_INTEGRATION.md).

— The `cargo-run` maintainers
