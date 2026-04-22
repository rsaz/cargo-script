---
title: "Workspace orchestration in Rust without writing a Justfile per crate"
description: "How cargo-run 0.6 turns a Rust workspace into a single, declarative task graph — sequentially or in parallel."
tags: ["rust", "cargo", "workspace", "tooling"]
canonical_url: "https://github.com/lpotthast/cargo-run"
published: false
---

# Workspace orchestration in Rust, without the boilerplate

Cargo workspaces are great until you need to *do* something across all
members. You start with:

```bash
for d in crates/*; do (cd "$d" && cargo test) || exit 1; done
```

A week later you've got a `Justfile` with `_test-crate-a`, `_test-crate-b`,
`test-all`, and a comment that says "remember to add new crates here". A
month later you wrote an `xtask` to do it properly, and now half your
contributors don't know which command to run.

`cargo-run` 0.6 fixes this with a single declaration.

## One Scripts.toml, every member

```toml
# Scripts.toml at the workspace root

[workspace]
mode = "all"     # default execution mode for scripts that opt in

[scripts]
test  = { command = "cargo test",                          workspace = "parallel" }
lint  = { command = "cargo clippy --all-targets -- -Dwarnings", workspace = "all" }
fmt   = { command = "cargo fmt --all -- --check",           workspace = "root"     }
build = { command = "cargo build --release",                workspace = "parallel" }
```

That's it. No per-crate config. No `xtask`. No `for` loops.

```bash
cargo script test     # runs `cargo test` inside every workspace member, in parallel
cargo script lint     # runs sequentially across members
cargo script fmt      # runs once, at the workspace root
```

## Three modes, explicit semantics

| Mode        | Where it runs                                  | Use for                          |
| ----------- | ---------------------------------------------- | -------------------------------- |
| `root`      | Once, at the workspace root                    | `cargo fmt`, top-level commands  |
| `all`       | Once per member, sequentially                  | `cargo audit`, deterministic CI  |
| `parallel`  | Once per member, concurrently (Tokio)          | `cargo test`, `cargo build`      |

You can override per-invocation:

```bash
cargo script test --workspace=all       # force sequential for stable logs
cargo script test --no-workspace        # run only at the root
cargo script test --workspace=parallel  # explicit parallel
```

## Discovering members

`cargo-run` discovers members in two ways, with a clear precedence:

1. **`Scripts.toml` opt-in list** (recommended for production):
    ```toml
    [workspace]
    mode    = "all"
    members = ["crates/api", "crates/db", "crates/cli"]
    ```
2. **`Cargo.toml` `[workspace] members`** — auto-detected when omitted.

Use `cargo script workspace list` to confirm what `cargo-run` sees:

```text
Workspace root: /home/me/myproj
Default mode:   parallel
Members:
  • crates/api
  • crates/cli
  • crates/db
```

## Parallel execution that doesn't lie

The naive "run them all at once" approach destroys log readability and
hides failures. `cargo-run` 0.6:

- **Buffers output per member** so logs don't interleave mid-line.
- **Streams a header per member** when a job starts/finishes.
- **Aggregates exit codes** — any failure fails the whole script with a
  per-member summary.
- **Exits non-zero** the moment a member fails when `fail_fast = true`.

In practice, parallel mode for a four-crate workspace cuts our local
`test` time from 38s → 11s.

## Composing with hooks and parameters

Workspace scripts compose with the rest of `Scripts.toml`:

```toml
[scripts.ci]
include = ["fmt", "lint", "test"]
info    = "Full CI pipeline"

[scripts.release]
command    = "cargo publish -p {{crate}}"
args       = ["crate"]
pre        = ["ci"]
on_success = ["tag-release"]
```

```bash
cargo script ci --json     # full pipeline, machine-readable
cargo script release my-crate
```

## Try it

```bash
cargo install cargo-run --features parallel
cargo script init --template workspace
```

The `workspace` template scaffolds a two-crate workspace with a complete
`Scripts.toml` so you can see the pattern end-to-end.

Full reference: [Workspace Guide](../WORKSPACE_GUIDE.md).

— The `cargo-run` maintainers
