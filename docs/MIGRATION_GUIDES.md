# Migration Guides

Coming from `make`, `just`, `cargo-make`, `npm scripts`, or your own bespoke
shell scripts? Here's how the equivalent setup looks in cargo-run.

> All examples assume cargo-run is installed (`cargo install cargo-run`) and
> use `cargo script ...` for invocation. The shorter alias `cgs ...` is
> always available.

---

## From `just`

`justfile`:

```just
default: test
test:
    cargo test
build:
    cargo build --release
ci: test build
    cargo fmt --check
    cargo clippy -- -D warnings
```

`Scripts.toml`:

```toml
[scripts]
default   = { include = ["test"], info = "Default task" }
test      = "cargo test"
build     = "cargo build --release"
fmt-check = "cargo fmt --all -- --check"
lint      = "cargo clippy -- -D warnings"
ci        = { include = ["test", "build", "fmt-check", "lint"], info = "Full CI" }
```

| Feature                        | `just`            | `cargo-run`                                  |
| ------------------------------ | ----------------- | -------------------------------------------- |
| Multi-line recipes             | yes               | yes (any shell command works)                |
| Recipe dependencies            | `recipe: dep`     | `include = ["dep"]`                          |
| Argument passing               | positional        | positional + named (`key=value`) + defaults  |
| Workspace orchestration        | manual loops      | `workspace = "all" \| "parallel"`            |
| File-watch                     | external tool     | `--watch` (built-in)                         |
| Hooks                          | manual            | `pre`/`post`/`on_success`/`on_failure`       |
| CI templates                   | none              | `init --template ...`                        |

---

## From `make`

`Makefile`:

```make
.PHONY: ci test build
test:
	cargo test
build:
	cargo build --release
ci: test build
	cargo fmt --check
	cargo clippy -- -D warnings
```

`Scripts.toml`:

```toml
[scripts]
test  = "cargo test"
build = "cargo build --release"
ci    = { include = ["test", "build"], info = "Full CI", post = ["fmt-check", "lint"] }
fmt-check = "cargo fmt --all -- --check"
lint      = "cargo clippy -- -D warnings"
```

`make` upsides we don't try to replicate: declarative file dependencies,
incremental builds. Use cargo for those — cargo-run delegates them happily.

cargo-run upsides:

* Cross-platform without `if-eq` / shell tricks.
* No tab-vs-spaces foot-guns.
* Built-in dry run, JSON output, parallel execution.
* `cargo install cargo-run` works on Windows out of the box.

---

## From `cargo-make`

`cargo-make` is the closest competitor. The migration is largely about
shorter syntax and Rust-native tooling.

`Makefile.toml`:

```toml
[tasks.test]
command = "cargo"
args    = ["test"]

[tasks.lint]
command = "cargo"
args    = ["clippy", "--", "-D", "warnings"]

[tasks.ci]
dependencies = ["test", "lint"]
```

`Scripts.toml`:

```toml
[scripts]
test = "cargo test"
lint = "cargo clippy -- -D warnings"
ci   = { include = ["test", "lint"] }
```

| Feature                          | `cargo-make`        | `cargo-run`                       |
| -------------------------------- | ------------------- | --------------------------------- |
| TOML configuration               | yes                 | yes (single file)                 |
| Cross-platform                   | yes                 | yes                               |
| Workspace support                | yes (heavy)         | yes (lean, opt-in per script)     |
| Cargo-script (RFC 3502)          | no                  | **yes**                           |
| Watch mode                       | external            | **built-in**                      |
| Built-in CI templates            | partial             | **yes**                           |
| Startup time                     | ~100 ms             | <100 ms target                    |
| Default config size              | large               | tiny                              |

---

## From `npm scripts`

`package.json`:

```json
{
  "scripts": {
    "test": "cargo test",
    "build": "cargo build --release",
    "ci": "npm run test && npm run build"
  }
}
```

`Scripts.toml`:

```toml
[scripts]
test  = "cargo test"
build = "cargo build --release"
ci    = { include = ["test", "build"] }
```

You get everything you used `npm run` for, plus typed errors, JSON output,
hooks, and workspace orchestration — without dragging in node.

---

## From bespoke shell scripts

If you have a `scripts/` folder full of `.sh` files:

1. **Keep them.** cargo-run can call any shell from any script:

   ```toml
   [scripts.deploy]
   command     = "./scripts/deploy.sh"
   interpreter = "bash"
   ```

2. **Or, port them to Rust** and let cargo-run treat them as cargo scripts:

   ```toml
   [scripts.deploy]
   script_file = "./scripts/deploy.rs"
   ```

   See [`CARGO_SCRIPT_INTEGRATION.md`](CARGO_SCRIPT_INTEGRATION.md).

---

## Migration checklist

- [ ] Install: `cargo install cargo-run`
- [ ] Initialize: `cargo script init` (or pick a template)
- [ ] Translate top-level tasks (test, build, lint, fmt) into `[scripts]`
- [ ] Group with `include = [...]` instead of shell concatenation
- [ ] Add `pre`/`post`/`on_success`/`on_failure` for lifecycle behaviour
- [ ] If multi-crate: switch to `workspace = "all"` or `"parallel"`
- [ ] Replace `cargo watch -x ...` with `cargo script <task> --watch`
- [ ] Add `--json` to your CI for richer reporting
- [ ] Validate: `cargo script validate`
- [ ] Delete the old runner once parity is achieved
