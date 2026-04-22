# CI/CD Guide

cargo-run ships with a built-in **template system** that scaffolds both your
`Scripts.toml` and the matching CI configuration in one shot. It also makes a
great runner inside CI because:

* Every script is a single, predictable entry point (`cargo script ci`).
* The same commands run locally and in CI — no drift.
* Structured `--json` output is consumable from any pipeline tool.

## Quick start

```bash
# List what's available
cargo script init --list-templates

# Scaffold a GitHub Actions setup
cargo script init --template github-actions
```

This drops two files at the repo root:

* `Scripts.toml` — sensible defaults: `fmt-check`, `lint`, `test`, `build`, `ci`.
* `.github/workflows/ci.yml` — runs `cargo script ci` on every push and PR.

Pass `--force` to overwrite existing files.

## Available templates

| Template          | Files generated                                      | Best for                       |
| ----------------- | ---------------------------------------------------- | ------------------------------ |
| `github-actions`  | `Scripts.toml`, `.github/workflows/ci.yml`           | GitHub-hosted projects         |
| `gitlab-ci`       | `Scripts.toml`, `.gitlab-ci.yml`                     | GitLab-hosted projects         |
| `rust-project`    | `Scripts.toml`                                       | Single-crate projects          |
| `workspace`       | `Scripts.toml`                                       | Multi-crate workspaces         |

## Why a single `cargo script ci`?

A standard CI pipeline often looks like:

```yaml
- run: cargo fmt --check
- run: cargo clippy -- -D warnings
- run: cargo test
- run: cargo build --release
```

That's four steps to keep in sync between CI and developer machines, four
places to add a new check, and four chances for drift.

With cargo-run:

```toml
[scripts]
ci = { include = ["fmt-check", "lint", "test", "build"] }
fmt-check = "cargo fmt --all -- --check"
lint      = "cargo clippy --all-targets --all-features -- -D warnings"
test      = "cargo test --all-features"
build     = "cargo build --release"
```

```yaml
- run: cargo install cargo-run
- run: cargo script ci
```

One source of truth. New check? Add it to `include`.

## Machine-readable output

For pipelines that want to ingest results:

```bash
cargo script ci --json > result.json
jq -e '.success == true' result.json
```

The JSON document contains the script tree, per-script duration in
milliseconds, exit codes and any error message — enough to publish a rich
status page without parsing terminal output.

## CI matrix example (workspace)

```yaml
# .github/workflows/ci.yml
jobs:
  ci:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
        toolchain: [stable, nightly]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.toolchain }}
          components: rustfmt, clippy
      - run: cargo install cargo-run
      - run: cargo script ci --workspace=parallel
```

This runs the full CI pipeline across every workspace member, in parallel,
on three OSes and two toolchains — and the script definitions live in
`Scripts.toml`, not in YAML.

## Local pre-push checks

Mirror your CI exactly:

```toml
[scripts]
push-check = { include = ["fmt", "lint", "test"], info = "Run before pushing" }
```

```bash
cargo script push-check
```
