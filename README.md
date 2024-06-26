# cargo-script

<!-- prettier-ignore-start -->

[![crates.io](https://img.shields.io/crates/v/cargo-script?label=latest)](https://crates.io/crates/cargo-script)
[![Documentation](https://docs.rs/cargo-script/badge.svg)](https://docs.rs/cargo-script)
[![Documentation](https://docs.rs/cargo-script/badge.svg?version=0.1.0)](https://docs.rs/cargo-script/0.1.0)
![Version](https://img.shields.io/badge/rustc-1.79+-ab6000.svg)
![MIT or Apache 2.0 licensed](https://img.shields.io/crates/l/cargo-script.svg)
<br />
[![Dependency Status](https://deps.rs/crate/cargo-script/0.1.0/status.svg)](https://deps.rs/crate/cargo-script/0.1.0)
[![Download](https://img.shields.io/crates/d/cargo-script.svg)](https://crates.io/crates/cargo-script)

<!-- prettier-ignore-end -->

<!-- cargo-rdme start -->

A CLI tool to run custom scripts in Rust, defined in `Scripts.toml`.

## Features

-   Run scripts defined in `Scripts.toml`.
-   Specify interpreters for scripts (e.g., bash, zsh, PowerShell).
-   Initialize a `Scripts.toml` file with default content.
-   Chain multiple scripts together using the `include` feature.

## Installation

To install `cargo-script`, use the following command:

```sh
cargo install cargo-script
```

## Usage

To run a script, use the following command:

```sh

```

## Scripts Configuration

The `Scripts.toml` file is used to define scripts. The file is located in the root of the project directory. The following is an example of a `Scripts.toml` file:

```toml
[scripts]
i_am_shell = "./.scripts/i_am_shell.sh" // run a shell script
i_am_shell_obj = { interpreter = "bash", command = "./.scripts/i_am_shell.sh", info = "Detect shell script" } // run a shell script with interpreter, also add script info
build = "echo 'build'" // run a simple command
release = { include = ["i_am_shell", "build"] } // run a chain of scripts
```

<!-- cargo-rdme end -->
