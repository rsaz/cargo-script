---
title: "Migrating from just / make / cargo-make to cargo-run"
description: "A side-by-side migration guide showing how to port your existing task runner config to cargo-run 0.6 — without losing features."
tags: ["rust", "tooling", "migration", "cargo-make", "just"]
canonical_url: "https://github.com/lpotthast/cargo-run"
published: false
---

# Migrating to `cargo-run` from your existing task runner

Most Rust projects already use *something* — `just`, `make`, `cargo-make`, or
the venerable "shell scripts in `./scripts/`". This post shows how to port
each to `cargo-run` 0.6 in well under an hour, and what you gain by doing so.

For the long-form reference, see the
[Migration Guides](../MIGRATION_GUIDES.md).

## From `just`

Your `Justfile`:

```just
test:
    cargo test --workspace

lint:
    cargo clippy --workspace --all-targets -- -D warnings

ci: lint test
```

Becomes `Scripts.toml`:

```toml
[scripts]
test = { command = "cargo test", workspace = "parallel" }
lint = "cargo clippy --workspace --all-targets -- -D warnings"
ci   = { include = ["lint", "test"], info = "Run CI" }
```

What you gain:

- **Workspace mode** — no more `cargo test --workspace` only working from the
  root.
- **Parallel test execution** across members.
- **`--watch` / `--json` / `--parallel`** without writing recipes.
- **Cross-platform** — Windows users no longer need WSL or `chocolatey
  install just`.

Run with `cargo script ci` instead of `just ci`.

## From `make`

Your `Makefile`:

```make
.PHONY: test lint ci

test:
	cargo test --workspace

lint:
	cargo clippy --workspace --all-targets -- -D warnings

ci: lint test
```

Becomes the same `Scripts.toml` shown above. Plus you escape:

- **Tab vs. spaces** errors.
- **Implicit shell semantics** that differ on macOS vs. Linux vs. Git-Bash.
- **`.PHONY` declarations**.
- **No native parallelism** beyond `-j`.

Run with `cargo script ci` instead of `make ci`.

## From `cargo-make`

`cargo-make` is the closest existing analogue. Migration is mostly cosmetic:

```toml
# Makefile.toml (cargo-make)
[tasks.test]
command = "cargo"
args    = ["test"]
workspace = true

[tasks.ci]
dependencies = ["fmt", "lint", "test"]
```

→

```toml
# Scripts.toml (cargo-run)
[scripts.test]
command   = "cargo test"
workspace = "parallel"

[scripts.ci]
include = ["fmt", "lint", "test"]
```

What you gain over `cargo-make`:

- **First-class RFC 3502 (`.rs` script) integration.**
- **Built-in watch mode.**
- **CI/CD templates** (`init --template github-actions`).
- **Faster startup**, smaller binary, fewer features you don't use.
- **Simpler config** — flat `Scripts.toml`, no `[tasks.x.condition]` mini-DSL.

You keep:

- Workspace orchestration.
- Hooks.
- Cross-platform behaviour.
- Parallel execution.

Run with `cargo script ci` instead of `cargo make ci`.

## From `npm scripts`

If you've been running JS-style scripts on a Rust project ("don't ask"):

```json
{
  "scripts": {
    "test":  "cargo test",
    "lint":  "cargo clippy -- -D warnings",
    "build": "cargo build --release",
    "ci":    "npm run lint && npm run test"
  }
}
```

→

```toml
[scripts]
test  = "cargo test"
lint  = "cargo clippy -- -D warnings"
build = "cargo build --release"
ci    = { include = ["lint", "test"] }
```

What you gain:

- **No Node.js required.**
- **Real workspace support.**
- **Parallel and watch built in.**
- **Validation** before CI even starts.

## From `./scripts/*.sh`

The most common starting point. The migration is brutally simple:

```toml
[scripts]
deploy-staging    = "./scripts/deploy.sh staging"
deploy-production = "./scripts/deploy.sh production"
```

…and then progressively replace shell scripts with parameterised entries:

```toml
[scripts.deploy]
command  = "./scripts/deploy.sh {{env}}"
args     = ["env"]
defaults = { env = "staging" }
requires = ["docker >= 19.03", "kubectl >= 1.18"]
pre      = ["lint", "test"]
on_failure = ["rollback"]
```

You now have one entry point (`cargo script deploy`), automated requirement
checks, hooks, and the same UX whether you `cargo script` it locally or in CI.

## Migration checklist

1. `cargo install cargo-run --features watch,parallel`
2. `cargo script init --template rust-project`
3. Port your existing recipes one at a time. Run `cargo script <name>
   --dry-run` to preview.
4. Add `workspace = "parallel"` to anything that runs per-member.
5. Run `cargo script validate` and fix any warnings.
6. Drop `Justfile` / `Makefile` / `Makefile.toml` once CI is green.

## Coexistence

You don't have to pick one. `cargo-run` plays nicely with existing tools — a
`Justfile` recipe can call `cargo script ci`, and vice versa. Many projects
migrate gradually, starting with their CI pipeline and only then moving
local-dev recipes.

Full reference: [Migration Guides](../MIGRATION_GUIDES.md).

— The `cargo-run` maintainers
