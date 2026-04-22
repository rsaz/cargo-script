# cargo-run

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/cargo-run?label=latest)](https://crates.io/crates/cargo-run)
[![Documentation](https://docs.rs/cargo-run/badge.svg)](https://docs.rs/cargo-run)
![Version](https://img.shields.io/badge/rustc-1.79+-ab6000.svg)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/cargo-run.svg)
[![Dependency Status](https://deps.rs/crate/cargo-run/latest/status.svg)](https://deps.rs/crate/cargo-run/latest)
<br />
[![Download](https://img.shields.io/crates/d/cargo-run.svg)](https://crates.io/crates/cargo-run)

<!-- prettier-ignore-end -->

> **A powerful, fast, and developer-friendly CLI tool for managing project scripts in Rust.**  
> Think `npm scripts`, `make`, or `just` — but built specifically for the Rust ecosystem, **workspace-aware**, and ready for the upcoming `cargo-script` (RFC 3502) era.

## What's new in 0.6

- **Workspace orchestration** — `workspace = "all" | "parallel"` runs a script across every member crate. CLI override: `--workspace=parallel` ([guide](docs/WORKSPACE_GUIDE.md))
- **Cargo-script (RFC 3502) integration** — invoke `.rs` single-file packages directly from `Scripts.toml`. Stable + nightly auto-detected. ([guide](docs/CARGO_SCRIPT_INTEGRATION.md))
- **CI/CD templates** — `cargo script init --template github-actions` scaffolds `Scripts.toml` + the matching workflow. ([guide](docs/CI_CD_GUIDE.md))
- **Lifecycle hooks** — `pre`, `post`, `on_success`, `on_failure` per script.
- **Parallel execution** — `--parallel` and `workspace = "parallel"` (Tokio).
- **Watch mode** — `cargo script test --watch` re-runs on file changes.
- **Parameters** — `args`, `defaults`, `{{name}}` substitution.
- **JSON output** — `--json` for pipeline-friendly results.

See [`docs/ADVANCED_FEATURES.md`](docs/ADVANCED_FEATURES.md) for the full reference and [`examples/`](examples/) for runnable demos.

## Why `cargo-run`?

**Stop writing one-off shell scripts.** `cargo-run` provides a unified, type-safe way to manage all your project automation:

- ✅ **Workspace-aware** — Run scripts across every member, sequentially or in parallel
- ✅ **Cargo-script ready** — First-class RFC 3502 (`.rs`) script support
- ✅ **Zero runtime dependencies** — Single binary, fast startup
- ✅ **Cross-platform** — Works on Windows, macOS, and Linux
- ✅ **Modern CLI UX** — Simplified syntax, interactive selection, shell completions
- ✅ **Powerful features** — Script chaining, hooks, parallel/watch, parameters, env vars, toolchains
- ✅ **CI/CD ready** — Templates, validation, JSON output
- ✅ **Rust-native** — Built with Rust, for Rust projects

### Quick Comparison

| Feature                     | `cargo-run` | `make` | `just` | `cargo-make` | `npm scripts` |
| --------------------------- | ----------- | ------ | ------ | ------------ | ------------- |
| Zero runtime dependencies   | ✅          | ✅     | ✅     | ✅           | ❌ (Node.js)  |
| Cross-platform              | ✅          | ⚠️     | ✅     | ✅           | ✅            |
| Workspace orchestration     | ✅          | ❌     | ❌     | ✅           | ❌            |
| **Parallel execution**      | ✅          | ⚠️     | ❌     | ✅           | ⚠️            |
| **Watch mode (built-in)**   | ✅          | ❌     | ❌     | ❌           | ❌            |
| **Cargo-script (RFC 3502)** | ✅          | ❌     | ❌     | ❌           | ❌            |
| **CI/CD templates**         | ✅          | ❌     | ❌     | ⚠️           | ❌            |
| Lifecycle hooks             | ✅          | ❌     | ❌     | ✅           | ⚠️            |
| Parameters / substitution   | ✅          | ⚠️     | ✅     | ✅           | ⚠️            |
| JSON output                 | ✅          | ❌     | ⚠️     | ❌           | ❌            |
| Shell completions           | ✅          | ⚠️     | ✅     | ✅           | ✅            |
| Dry-run mode                | ✅          | ❌     | ✅     | ✅           | ❌            |
| Validation                  | ✅          | ❌     | ⚠️     | ⚠️           | ❌            |
| Toolchain support           | ✅          | ❌     | ❌     | ⚠️           | ❌            |

## 📦 Installation

```sh
cargo install cargo-run
```

After installation, you'll have multiple ways to invoke the tool:

- `cargo script` — **Recommended**: Use as a Cargo subcommand (e.g., `cargo script run build`)
- `cargo-script` — Direct binary invocation
- `cgs` — Short alias (used in examples below for brevity)

**Note:** When installed via `cargo install`, the `cargo-script` binary is automatically available in your PATH, enabling `cargo script` subcommand usage.

## ⚡ Quick Start

1. **Initialize** a `Scripts.toml` file:

    ```sh
    # Using Cargo subcommand (recommended)
    cargo script init

    # Or using direct binary
    cgs init
    ```

2. **Run** a script:

    ```sh
    # Direct script execution
    cargo script build

    # Explicit form
    cargo script run build

    # Or using direct binary
    cgs build
    ```

3. **Discover** scripts interactively:

    ```sh
    # Interactive fuzzy selection
    cargo script --interactive
    cargo script -i

    # Show all scripts
    cargo script show

    # Filter scripts
    cargo script show --filter test
    ```

4. **Preview** what would run (dry-run):

    ```sh
    cargo script build --dry-run
    ```

5. **Validate** your configuration:

    ```sh
    cargo script validate
    ```

That's it! You're ready to go. 🎉

> **💡 Tip:** Using `cargo script` integrates seamlessly with Cargo's ecosystem and provides a familiar interface for Rust developers.

## 📚 Features

### Core Features

- **Script Execution** — Run scripts defined in `Scripts.toml`
- **Script Chaining** — Compose complex workflows with `include`
- **Environment Variables** — Global, script-specific, and command-line overrides
- **Multiple Interpreters** — bash, zsh, PowerShell, cmd, or custom
- **Toolchain Support** — Rust toolchains via rustup, Python versions
- **Requirements Checking** — Validate tool versions before execution

### v0.6 Highlights

- **Workspace Mode** — `workspace = "all" \| "parallel"` per script ([guide](docs/WORKSPACE_GUIDE.md))
- **Cargo Script Integration** — invoke `.rs` files (RFC 3502) ([guide](docs/CARGO_SCRIPT_INTEGRATION.md))
- **CI/CD Templates** — `init --template github-actions \| gitlab-ci \| rust-project \| workspace` ([guide](docs/CI_CD_GUIDE.md))
- **Lifecycle Hooks** — `pre`, `post`, `on_success`, `on_failure`
- **Parallel Execution** — `--parallel <SCRIPT>` and workspace-parallel
- **Watch Mode** — `--watch`, `--watch-path`, `--watch-exclude`
- **Parameters** — `args`, `defaults`, `{{name}}` substitution
- **JSON Output** — `--json` for machine-readable execution results
- **Workspace Subcommand** — `cargo script workspace list / run`

### Developer Experience

- **Simplified Syntax** — Run scripts directly: `cargo script build`
- **Interactive Selection** — Fuzzy-find scripts with `--interactive` flag
- **Script Filtering** — Filter scripts by name or description
- **Shell Completions** — Tab completion for bash, zsh, fish, and PowerShell
- **Dry-Run Mode** — Preview execution without side effects
- **Verbosity Control** — `--quiet` and `--verbose` flags for output control
- **Optional Metrics** — `--no-metrics` to suppress performance output
- **Helpful Errors** — Actionable error messages with quick-fix suggestions
- **Validation** — Catch configuration errors early
- **Performance Metrics** — Track script execution times (optional)

### v0.6 examples

```toml
# Scripts.toml — workspace + hooks + parameters + cargo-script
[global_env]
RUST_BACKTRACE = "1"

[scripts]
ci         = { include = ["fmt-check", "lint", "test-parallel"], info = "Full CI" }
fmt-check  = "cargo fmt --all -- --check"
lint       = "cargo clippy --workspace --all-targets -- -D warnings"

# Run cargo test in every workspace member, in parallel.
test-parallel = { command = "cargo test", workspace = "parallel" }

# Parameterised deploy with hooks.
[scripts.deploy]
command    = "kubectl apply -f manifests/{{env}}.yaml"
args       = ["env"]
defaults   = { env = "staging" }
pre        = ["fmt-check", "lint"]
on_failure = ["rollback"]

# A cargo-script (RFC 3502) Rust file
[scripts.audit]
script_file = "./scripts/audit.rs"
info        = "Run a Rust audit script"
```

```bash
cargo script ci                              # full pipeline
cargo script ci --json                       # machine-readable
cargo script test --watch                    # re-run on file changes
cargo script deploy production               # parameter substitution
cargo script test --parallel build --parallel doc   # multi-script parallel
cargo script init --template github-actions  # scaffold CI
cargo script workspace list                  # show discovered members
```

## 📖 Usage Guide

### Initialize `Scripts.toml`

Create a new `Scripts.toml` file with sensible defaults:

```sh
# Using Cargo subcommand (recommended)
cargo script init

# Or using direct binary
cgs init
```

This creates a `Scripts.toml` file with:

```toml
[global_env]

[scripts]
dev = "cargo run"
build = { command = "cargo build", env = { RUST_LOG = "info" } }
release = "cargo build --release"
test = { command = "cargo test", env = { RUST_LOG = "warn" } }
doc = "cargo doc --no-deps --open"
```

### Run Scripts

```sh
# Direct script execution
cargo script build
cargo script test

# Explicit form
cargo script run build

# With flags
cargo script build --env RUST_LOG=debug
cargo script test --dry-run
cargo script build --no-metrics

# Interactive selection
cargo script --interactive
cargo script -i

# Quiet mode (minimal output)
cargo script build --quiet

# Verbose mode (detailed output)
cargo script build --verbose
```

### Script Configuration

#### Simple Script

```toml
[scripts]
build = "cargo build"
```

#### Script with Metadata

```toml
[scripts]
build = {
    command = "cargo build",
    info = "Build the project in release mode",
    env = { RUST_LOG = "info" }
}
```

#### Script with Interpreter

```toml
[scripts]
deploy = {
    interpreter = "bash",
    command = "./scripts/deploy.sh",
    info = "Deploy to production"
}
```

#### Script Chaining (Includes)

```toml
[scripts]
prepublish_clean = "cargo clean"
prepublish_doc = "cargo doc --no-deps"
prepublish_dry = "cargo publish --dry-run"
prepublish_check = "cargo package --list"

prepublish = {
    include = ["prepublish_clean", "prepublish_doc", "prepublish_dry", "prepublish_check"],
    info = "Run all prepublish checks"
}
```

#### Script with Requirements

```toml
[scripts]
deploy = {
    command = "./deploy.sh",
    requires = ["docker >= 19.03", "kubectl >= 1.18"],
    toolchain = "stable",
    info = "Deploy application"
}
```

#### CI/CD-like Format

```toml
[scripts.build]
script = "build"
command = "cargo build"
info = "Build the project"

[scripts.test]
script = "test"
command = "cargo test"
requires = ["rustup >= 1.70"]
toolchain = "stable"
```

### Environment Variables

#### Global Environment Variables

```toml
[global_env]
RUST_BACKTRACE = "1"
RUST_LOG = "info"
```

#### Script-Specific Environment Variables

```toml
[scripts]
test = {
    command = "cargo test",
    env = { RUST_LOG = "debug" }
}
```

#### Command-Line Overrides

```sh
# Using Cargo subcommand
cargo script run test --env RUST_LOG=trace

# Or using direct binary
cgs run test --env RUST_LOG=trace
```

**Precedence Order:**

1. Command-line overrides (`--env`)
2. Script-specific (`env` in script)
3. Global (`[global_env]`)

### Show All Scripts

```sh
# Show all scripts
cargo script show

# Filter scripts by name or description
cargo script show --filter test
cargo script show -f build

# Default behavior - show scripts when no command provided
cargo script
```

Output:

```
Script   Description
-------- --------------------------------------
build    Build the project
test     Run tests
release  Build release version
```

With filter:

```sh
$ cargo script show --filter test

Found 2 script(s) matching 'test':

Script   Description
-------- --------------------------------------
test     Run tests
test-all Run all test suites
```

### Dry-Run Mode

Preview what would be executed without actually running it:

```sh
# Simplified syntax
cargo script prepublish --dry-run

# Explicit form
cargo script run prepublish --dry-run
```

### Interactive Script Selection

Use fuzzy selection to find and run scripts interactively:

```sh
# Interactive mode
cargo script --interactive
cargo script -i

# Or via run command
cargo script run --interactive
```

This opens an interactive fuzzy finder where you can:

- Type to search scripts
- See script descriptions
- Select and run scripts easily

**Output:**

```
DRY-RUN MODE: Preview of what would be executed
================================================================================

📋  Would run script: [ prepublish ]
    Description: Run all prepublish checks
    Would run include scripts:
      📋  Would run script: [ prepublish_clean ]
          Command: cargo clean

      📋  Would run script: [ prepublish_doc ]
          Command: cargo doc --no-deps

      📋  Would run script: [ prepublish_dry ]
          Command: cargo publish --dry-run

      📋  Would run script: [ prepublish_check ]
          Command: cargo package --list

No commands were actually executed.
```

### Shell Completions

Enable tab completion for a better developer experience:

**Bash:**

```sh
# Using Cargo subcommand (recommended)
cargo script completions bash > ~/.bash_completion.d/cargo-script
# Or system-wide:
cargo script completions bash | sudo tee /etc/bash_completion.d/cargo-script

# Or using direct binary
cgs completions bash > ~/.bash_completion.d/cgs
```

**Zsh:**

```sh
mkdir -p ~/.zsh/completions
# Using Cargo subcommand (recommended)
cargo script completions zsh > ~/.zsh/completions/_cargo-script
# Or using direct binary
cgs completions zsh > ~/.zsh/completions/_cgs
# Add to ~/.zshrc:
fpath=(~/.zsh/completions $fpath)
autoload -U compinit && compinit
```

**Fish:**

```sh
# Using Cargo subcommand (recommended)
cargo script completions fish > ~/.config/fish/completions/cargo-script.fish
# Or using direct binary
cgs completions fish > ~/.config/fish/completions/cgs.fish
```

**PowerShell:**

```powershell
# Using Cargo subcommand (recommended)
cargo script completions power-shell > $PROFILE
# Or using direct binary
cgs completions power-shell > completions.ps1
. .\completions.ps1
```

After installation, restart your shell and enjoy tab completion! 🎉

### Validation

Catch configuration errors before they cause problems:

```sh
# Using Cargo subcommand (recommended)
cargo script validate

# Or using direct binary
cgs validate
```

**What it checks:**

- ✅ TOML syntax validity
- ✅ Script references in `include` arrays
- ✅ Tool requirements (checks if tools are installed)
- ✅ Toolchain requirements (checks if Rust/Python toolchains are installed)

**Example output:**

```
✓ All validations passed!
```

**With errors:**

```
❌ Validation Errors:
  1. Script 'release': Script 'release' references non-existent script 'build'
  2. Script 'deploy': Required tool 'docker' is not installed or not in PATH

✗ Found 2 error(s)
```

**CI/CD Integration:**

```yaml
# .github/workflows/ci.yml
- name: Validate Scripts.toml
  run: cargo script validate
```

### Error Messages

`cargo-run` provides helpful, actionable error messages:

**Script Not Found:**

```bash
$ cargo script buid
❌ Script not found

Error:
  Script 'buid' not found in Scripts.toml

Did you mean:
  • build

Quick fix:
  Run 'cargo script show' to see all available scripts
  Or use 'cargo script init' to initialize Scripts.toml if it doesn't exist
```

**Invalid TOML:**

```bash
$ cargo script test
❌ Invalid TOML syntax

Error:
  File: Scripts.toml
  Message: invalid table header
  Line 10: See error details above

Quick fix:
  Check your Scripts.toml syntax. Common issues:
  - Missing quotes around strings
  - Trailing commas in arrays
  - Invalid table syntax
  Validate your file with: cargo script validate
```

**Missing Tool:**

```bash
$ cargo script run deploy
❌ Required tool not found

Error:
  Tool 'docker' is not installed or not in PATH

Suggestion:
  Install docker and ensure it's available in your PATH
```

## Use Cases

### Development Workflow

```toml
[scripts]
dev = "cargo run"
test = "cargo test"
test-watch = { command = "cargo watch -x test", requires = ["cargo-watch"] }
lint = "cargo clippy -- -D warnings"
fmt = "cargo fmt --check"
check = { include = ["fmt", "lint", "test"], info = "Run all checks" }
```

### CI/CD Pipeline

```toml
[scripts]
ci = {
    include = ["check", "test", "build"],
    info = "Run CI pipeline"
}

[scripts.check]
command = "cargo clippy -- -D warnings"

[scripts.test]
command = "cargo test --all-features"

[scripts.build]
command = "cargo build --release"
```

### Multi-Language Projects

```toml
[scripts]
build-rust = "cargo build"
build-python = {
    command = "python setup.py build",
    requires = ["python >= 3.8"],
    toolchain = "python:3.8"
}
build-all = { include = ["build-rust", "build-python"] }
```

### Deployment Scripts

```toml
[scripts]
deploy-staging = {
    command = "./scripts/deploy.sh staging",
    requires = ["docker >= 19.03", "kubectl >= 1.18"],
    env = { ENV = "staging" }
}

deploy-production = {
    command = "./scripts/deploy.sh production",
    requires = ["docker >= 19.03", "kubectl >= 1.18"],
    env = { ENV = "production" }
}
```

## 🔧 Advanced Configuration

### Custom Scripts Path

Use a different `Scripts.toml` file:

```sh
# Using Cargo subcommand
cargo script run build --scripts-path ./config/scripts.toml

# Or using direct binary
cgs run build --scripts-path ./config/scripts.toml
```

### Performance Metrics

Script execution times are automatically tracked and displayed (can be disabled):

```sh
# Show metrics (default)
cargo script build

# Hide metrics
cargo script build --no-metrics
```

Output:

```
Scripts Performance
--------------------------------------------------------------------------------
✔️  Script: prepublish_clean        🕒 Running time: 1.23s
✔️  Script: prepublish_doc          🕒 Running time: 3.45s
✔️  Script: prepublish_dry          🕒 Running time: 2.10s

🕒 Total running time: 6.78s
```

### Verbosity Control

Control output verbosity with `--quiet` and `--verbose` flags:

```sh
# Quiet mode - minimal output
cargo script build --quiet

# Verbose mode - detailed output
cargo script build --verbose

# Normal mode (default)
cargo script build
```

## 📚 Documentation

- [Workspace Guide](docs/WORKSPACE_GUIDE.md) — multi-crate orchestration
- [Cargo-Script Integration](docs/CARGO_SCRIPT_INTEGRATION.md) — `.rs` script files (RFC 3502)
- [CI/CD Guide](docs/CI_CD_GUIDE.md) — templates + JSON output
- [Advanced Features](docs/ADVANCED_FEATURES.md) — hooks, parallel, watch, parameters
- [Migration Guides](docs/MIGRATION_GUIDES.md) — moving from `just` / `make` / `cargo-make` / `npm scripts`
- [Examples](examples/) — runnable demo projects

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the [MIT License](LICENSE).

## 🙏 Acknowledgments

- Inspired by `npm scripts`, `make`, `just`, and `cargo-make`
- Designed to complement [Rust RFC 3502 (`cargo-script`)](https://rust-lang.github.io/rfcs/3502-cargo-script.html) — single-file Rust scripts
- Built with [clap](https://github.com/clap-rs/clap) for excellent CLI experience
- Uses [colored](https://github.com/mackwic/colored) for beautiful terminal output
- Parallel execution powered by [tokio](https://tokio.rs); watch mode by [notify](https://github.com/notify-rs/notify)

---

**Made with ❤️ for the Rust community**
