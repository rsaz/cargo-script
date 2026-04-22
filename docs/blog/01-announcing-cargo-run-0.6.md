---
title: "Announcing cargo-run 0.6 — workspace-aware scripts, RFC 3502 ready"
description: "A unified task runner for Rust that finally understands workspaces, single-file Rust scripts (RFC 3502), CI/CD, hooks, parallel and watch mode."
tags: ["rust", "cargo", "tooling", "devops"]
canonical_url: "https://github.com/lpotthast/cargo-run"
published: false
---

# Announcing `cargo-run` 0.6

> TL;DR — `cargo-run` 0.6 ships **workspace orchestration**, **first-class
> support for `cargo-script` (RFC 3502)**, **CI/CD templates**, **lifecycle
> hooks**, **parallel execution**, **watch mode**, **parameter substitution**
> and **JSON output** — all behind the same `cargo script <name>` UX you
> already know.

## Why another task runner?

Every Rust project I've worked on accumulates a `scripts/` folder, a `Justfile`,
a `Makefile`, half a `cargo-make` config and a few `xtask` crates — usually
all at once. The Rust ecosystem has a `Cargo.toml` for *building*, but no
canonical place for "the things you actually run during development".

`cargo-run` started as a small CLI to keep those workflows in one
`Scripts.toml`. With **0.6**, it grows into a complete, Rust-native task
runner with the features modern projects expect.

## What's new in 0.6

### 1. Workspace orchestration

Real Rust projects are workspaces. `cargo-run` now understands them natively.

```toml
# Scripts.toml at the workspace root
[workspace]
mode = "all"            # default: run in every member, sequentially

[scripts]
test = { command = "cargo test", workspace = "parallel" }
fmt  = { command = "cargo fmt --all -- --check", workspace = "root" }
```

```bash
cargo script test                  # runs `cargo test` in every member, in parallel
cargo script workspace list        # show discovered members
cargo script workspace run test    # explicit form
cargo script test --workspace=root # one-off override
```

Read the [Workspace Guide](../WORKSPACE_GUIDE.md).

### 2. Cargo-script (RFC 3502) integration

[RFC 3502](https://rust-lang.github.io/rfcs/3502-cargo-script.html) brings
single-file Rust scripts to stable Cargo. `cargo-run` 0.6 lets you wire those
`.rs` files into the same workflow as your shell scripts:

```toml
[scripts.audit]
script_file = "./scripts/audit.rs"
info        = "Audit dependencies (Rust script, RFC 3502)"
```

`cargo-run` auto-detects whether stable `cargo script` or
`cargo +nightly -Zscript` is available and falls back gracefully if neither is.
Read the [Cargo-Script Integration guide](../CARGO_SCRIPT_INTEGRATION.md).

### 3. CI/CD templates

```bash
cargo script init --template github-actions   # Scripts.toml + .github/workflows/ci.yml
cargo script init --template gitlab-ci        # Scripts.toml + .gitlab-ci.yml
cargo script init --template workspace        # multi-crate setup
cargo script init --list-templates
```

Combined with `--json`, your CI jobs become a single line:

```yaml
- run: cargo script ci --json | tee result.json
```

Read the [CI/CD Guide](../CI_CD_GUIDE.md).

### 4. Lifecycle hooks

```toml
[scripts.deploy]
command    = "kubectl apply -f manifests/{{env}}.yaml"
args       = ["env"]
defaults   = { env = "staging" }
pre        = ["fmt-check", "lint", "test"]
on_success = ["notify-slack"]
on_failure = ["rollback"]
```

`pre`, `post`, `on_success`, `on_failure` — composable, per-script, no plugin
system required.

### 5. Parallel execution

```bash
cargo script test --parallel doc --parallel lint
```

Powered by Tokio, gated behind the `parallel` feature so the default binary
stays small.

### 6. Watch mode

```bash
cargo script test --watch
cargo script test --watch --watch-path src --watch-exclude target
```

Built on `notify` + `notify-debouncer-full`. Re-runs on file changes with sane
debouncing. Gated behind the `watch` feature.

### 7. Parameter substitution

```toml
[scripts.greet]
command  = "echo Hello {{name}} from {{env}}"
args     = ["name", "env"]
defaults = { env = "dev" }
```

```bash
cargo script greet World production    # → Hello World from production
cargo script greet World               # → Hello World from dev
```

Missing required args produce an actionable error, not a silent
`{{name}}` in your shell command.

### 8. JSON output

```bash
cargo script ci --json
```

```json
{
  "script": "ci",
  "status": "success",
  "duration_ms": 18243,
  "steps": [
    { "name": "fmt-check", "status": "success", "duration_ms": 412 },
    { "name": "lint",      "status": "success", "duration_ms": 6231 },
    { "name": "test",      "status": "success", "duration_ms": 11600 }
  ]
}
```

Pipe it to `jq`, archive it as a CI artifact, send it to Datadog — your
choice.

## How does it compare?

| Feature                       | `cargo-run` 0.6 | `make`  | `just`  | `cargo-make` | `npm scripts` |
| ----------------------------- | --------------- | ------- | ------- | ------------ | ------------- |
| Workspace orchestration       | ✅              | ❌      | ❌      | ✅           | ❌            |
| Parallel execution            | ✅              | ⚠️      | ❌      | ✅           | ⚠️            |
| Watch mode (built-in)         | ✅              | ❌      | ❌      | ❌           | ❌            |
| Cargo-script (RFC 3502)       | ✅              | ❌      | ❌      | ❌           | ❌            |
| CI/CD templates               | ✅              | ❌      | ❌      | ⚠️           | ❌            |
| Lifecycle hooks               | ✅              | ❌      | ❌      | ✅           | ⚠️            |
| Parameters / substitution     | ✅              | ⚠️      | ✅      | ✅           | ⚠️            |
| JSON output                   | ✅              | ❌      | ⚠️      | ❌           | ❌            |
| Cross-platform                | ✅              | ⚠️      | ✅      | ✅           | ✅            |
| Zero runtime dependencies     | ✅              | ✅      | ✅      | ✅           | ❌            |

## Install

```bash
cargo install cargo-run --features watch,parallel
```

Then in any Rust project:

```bash
cargo script init
cargo script <your-task>
```

## What's next

- Stabilising the `Scripts.toml` schema for a 1.0.
- Deeper RFC 3502 integration once stable Cargo ships it.
- A `cargo script doctor` to diagnose common project issues.
- Plugin system for custom step types (Docker, k8s, cloud providers).

If you have ideas, the issue tracker is open. Star the repo if you want to
follow along, and tell us what your project's `Scripts.toml` looks like — we
love seeing real-world workflows.

— The `cargo-run` maintainers
