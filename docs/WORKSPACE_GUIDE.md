# Workspace Guide

`cargo-run` (binaries: `cargo-script`, `cgs`) is **workspace-aware** out of the
box. Run a script once in every member crate, in parallel or sequentially,
without writing ad-hoc shell loops.

## TL;DR

```toml
# Scripts.toml at the workspace root
[scripts]
test-all      = { command = "cargo test", workspace = "all",      info = "Test every member" }
test-parallel = { command = "cargo test", workspace = "parallel", info = "Test members in parallel" }
fmt-check     = "cargo fmt --all -- --check"  # workspace-wide root command
```

```bash
cargo script test-all       # sequential
cargo script test-parallel  # uses tokio under the hood
cargo script test-all --workspace=parallel  # CLI override
cargo script test-all --no-workspace        # force single-process run
```

## Concepts

### Workspace modes

| Mode      | Behaviour                                                  |
| --------- | ---------------------------------------------------------- |
| `root`    | (default) run once at the workspace root / cwd             |
| `all`     | run once per member, sequentially, fail-fast               |
| `parallel`| run once per member concurrently (requires `parallel` feat) |

Modes are declared in `Scripts.toml`:

```toml
[scripts.lint]
command = "cargo clippy --all-targets -- -D warnings"
workspace = "all"
```

…or chosen on the CLI:

```bash
cargo script lint --workspace=parallel
```

### Member discovery

Members are auto-discovered from the **workspace root's `Cargo.toml`**:

```toml
# Cargo.toml
[workspace]
members = ["crates/*"]
exclude = ["crates/legacy"]
```

You can override discovery from `Scripts.toml`:

```toml
[workspace]
members = ["crates/api", "crates/cli"]
exclude = ["crates/wip"]
```

Glob patterns are limited to a trailing `/*`, matching Cargo's own most common
form. For more exotic layouts, list members explicitly.

### Inspecting members

```bash
cargo script workspace list
cargo script workspace list --json
cargo script workspace run test --parallel
```

## Worked example

Project layout:

```
my-workspace/
├── Cargo.toml
├── Scripts.toml
├── crates/
│   ├── server/
│   │   └── Cargo.toml
│   ├── client/
│   │   └── Cargo.toml
│   └── shared/
│       └── Cargo.toml
```

`Scripts.toml`:

```toml
[global_env]
RUST_BACKTRACE = "1"

[scripts]
ci = { include = ["fmt-check", "lint", "test-parallel"], info = "Full CI" }

fmt-check     = "cargo fmt --all -- --check"
lint          = "cargo clippy --workspace --all-targets -- -D warnings"
test-parallel = { command = "cargo test", workspace = "parallel" }

# Per-member release builds, sequentially (less RAM than `parallel`).
release-all   = { command = "cargo build --release", workspace = "all" }
```

Run it:

```bash
cargo script ci
cargo script release-all
cargo script test-parallel --json | jq '.includes[].duration_ms'
```

## Composability

`workspace = "..."` composes naturally with **hooks**:

```toml
[scripts.deploy-all]
command    = "kubectl rollout restart deploy/{{member_name}}"
workspace  = "all"
pre        = ["fmt-check", "lint"]
on_failure = ["notify-slack"]
```

…and with **parameters** (`{{name}}` substitution):

```toml
[scripts.bench]
command   = "cargo bench --bench {{suite}}"
workspace = "parallel"
args      = ["suite"]
defaults  = { suite = "default" }
```

```bash
cargo script bench codegen
```

## Caveats

* **Parallel mode** requires the `parallel` Cargo feature (default-on). If it
  is disabled, cargo-run gracefully falls back to sequential execution and
  emits a warning.
* Each member run executes inside the member's directory (`cargo-run`
  temporarily switches `cwd`). Avoid relying on the calling shell's cwd inside
  workspace commands; use `${CARGO_MANIFEST_DIR}` if you need the
  per-member path.
