# cargo-run

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/cargo-run?label=latest)](https://crates.io/crates/cargo-run)
[![Documentation](https://docs.rs/cargo-run/badge.svg)](https://docs.rs/cargo-run)
![Version](https://img.shields.io/badge/rustc-1.79+-ab6000.svg)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/cargo-run.svg)
[![Dependency Status](https://deps.rs/crate/cargo-run/0.1.0/status.svg)](https://deps.rs/crate/cargo-run/0.1.0)
<br />
[![Download](https://img.shields.io/crates/d/cargo-run.svg)](https://crates.io/crates/cargo-run)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

A CLI tool to run custom scripts in Rust, defined in `Scripts.toml`.

## Features

-   Run scripts defined in `Scripts.toml`.
-   Specify interpreters for scripts (e.g., bash, zsh, PowerShell).
-   Initialize a `Scripts.toml` file with default content.
-   Chain multiple scripts together using the `include` feature.
-   Set global environment variables and script-specific environment variables with precedence handling.
-   Show detailed information about scripts.

## Installation

To install `cargo-run`, use the following command:

```sh
cargo install cargo-run
```

## Usage

When `cargo-run` crate is installed it provides a binary `cargo-script` or `cgs` to run custom scripts. Commands can start with `cargo-script` or `cgs`.

The examples below use `cgs` as the command prefix for simplicity.

### Initialize `Scripts.toml`

The `init` command initializes a `Scripts.toml` file in the root of your project directory with default content. This file is used to define and manage your custom scripts.

To initialize a `Scripts.toml` file, use the following command:

```sh
cgs init
```

Default `Scripts.toml` content:

```toml
[global_env]

[scripts]
dev = "cargo run"
build = { command = "cargo build", env = { RUST_LOG = "info" } }
release = "cargo build --release"
test = { command = "cargo test", env = { RUST_LOG = "warn" } }
doc = "cargo doc --no-deps --open"
```

### Run a Script

To run a script, use the following command:

```sh
cgs run <script_name>
```

## Understanding `Scripts.toml`

The `Scripts.toml` file is used to define scripts. The file is located in the root of the project directory. The following is an example of a `Scripts.toml` file:

### Simple Script

A simple script that runs a command directly.

```toml
[scripts]
build = "echo 'build'"
```

### Script with Interpreter

You can specify an interpreter for the script.

```toml
[scripts]
config = { interpreter = "bash", command = "echo 'test'", info = "Script to test" }
```

### Chain of Scripts

You can chain multiple scripts together using the include feature.

```toml
[scripts]
release = { include = ["i_am_shell", "build"] }
```

### Detailed Script

A detailed script can include interpreter, command, info, and other scripts to run.

```toml
[scripts]
i_am_shell_obj = { interpreter = "bash", command = "./.scripts/i_am_shell.sh", info = "Detect shell script" }
```

### Add info to a script

You can add info to a script to provide more details about the script.

```toml
[scripts]
build = { command = "cargo build", info = "Build the project" }
```

### Global Environment Variables

You can define global environment variables that will be available to all scripts. Script-specific environment variables can override these global variables.

```toml
[global_env]
RUST_BACKTRACE = "1"
EXAMPLE_VAR = "example_value"
```

### Script-Specific Environment Variables

You can define script-specific environment variables that will override global environment variables.

```toml
[scripts]
example01 = { command = "echo $EXAMPLE_VAR", env = { EXAMPLE_VAR = "change_value" } }
example02 = { command = "echo ${RUST_LOG:-unset} ${COMMON_VAR:-unset}", env = { RUST_LOG = "warn" } }
example03 = { command = "echo ${EXAMPLE_VAR:-unset} ${RUST_LOG:-unset} ${COMMON_VAR:-unset}", env = { EXAMPLE_VAR = "change_value_again", RUST_LOG = "info" } }
```

### Environment Variables Precedence

The precedence order for environment variables is as follows:

1. Command-line overrides: Environment variables passed through the command line when running a script.
2. Script-specific environment variables: Variables defined in the env section of a script.
3. Global environment variables: Variables defined in the [global_env] section.

This order ensures that command-line overrides have the highest precedence, followed by script-specific variables, and finally global variables.

### Running a Script with Environment Variables

To run a script and override environment variables from the command line, use the following format:

```sh
cgs run <script_name> --env <ENV_VAR1>=<value1>
```

### Show command

To show all the scripts and their details, use the following command:

```sh
cgs show
```

<!-- cargo-rdme end -->

## Explanation

-   **Features**: Summarizes the main features of the tool.
-   **Installation**: Provides the command to install the tool.
-   **Usage**: Explains how to run scripts using `cargo-script` or `cgs`.
-   **Initializing `Scripts.toml`**: Explains the purpose of the `init` command and provides the command to initialize the file.
-   **Default `Scripts.toml` Content**: Shows the default content created by the `init` command.
-   **Understanding `Scripts.toml`**: Details different configurations possible in the `Scripts.toml` file, including simple scripts, scripts with interpreters, chained scripts, and detailed scripts.
-   **Example `Scripts.toml` File**: Provides a complete example of a `Scripts.toml` file.
-   **Example Usage**: Shows how to run scripts and initialize the `Scripts.toml` file.
-   **Global Environment Variables**: Explains how to define global environment variables.
-   **Script-Specific Environment Variables**: Explains how to define script-specific environment variables.
-   **Environment Variables Precedence**: Explains the order of precedence for environment variables.
-   **Show Command**: Explains how to show all the scripts and their details.
