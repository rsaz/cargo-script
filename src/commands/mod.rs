//! CLI command definitions and module registry.

use clap::{ArgAction, Subcommand, ValueEnum};

/// Workspace execution flag accepted on the CLI. Mirrors
/// [`crate::commands::script::WorkspaceMode`] but lives in this module so
/// `clap` can derive a `ValueEnum` without making the script types depend on
/// clap.
#[derive(ValueEnum, Clone, Debug, Copy)]
#[value(rename_all = "lowercase")]
pub enum WorkspaceFlag {
    All,
    Parallel,
    Root,
}

impl From<WorkspaceFlag> for crate::commands::script::WorkspaceMode {
    fn from(f: WorkspaceFlag) -> Self {
        match f {
            WorkspaceFlag::All => Self::All,
            WorkspaceFlag::Parallel => Self::Parallel,
            WorkspaceFlag::Root => Self::Root,
        }
    }
}

/// Top-level subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum Commands {
    #[command(about = "Run a script by name defined in Scripts.toml", visible_alias = "r")]
    Run {
        #[arg(value_name = "SCRIPT_NAME")]
        script: Option<String>,
        #[arg(short, long, value_name = "KEY=VALUE", action = ArgAction::Append)]
        env: Vec<String>,
        /// Preview what would be executed without actually running it
        #[arg(long)]
        dry_run: bool,
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Don't show performance metrics after execution
        #[arg(long)]
        no_metrics: bool,
        /// Interactive script selection
        #[arg(short, long)]
        interactive: bool,
        /// Run the script across workspace members (all|parallel|root)
        #[arg(long, value_enum, value_name = "MODE")]
        workspace: Option<WorkspaceFlag>,
        /// Force execution at the workspace root, ignoring any workspace
        /// mode declared in Scripts.toml.
        #[arg(long)]
        no_workspace: bool,
        /// Re-run the script when watched files change (requires the `watch`
        /// feature, enabled by default).
        #[arg(long)]
        watch: bool,
        /// Paths to watch in `--watch` mode (repeatable). Defaults to `.`.
        #[arg(long, value_name = "PATH", action = ArgAction::Append)]
        watch_path: Vec<std::path::PathBuf>,
        /// Substring patterns to ignore in `--watch` mode (repeatable).
        #[arg(long, value_name = "PATTERN", action = ArgAction::Append)]
        watch_exclude: Vec<String>,
        /// Emit a JSON document describing the execution.
        #[arg(long)]
        json: bool,
        /// Run several scripts in parallel (requires the `parallel` feature).
        /// Use this multiple times for additional scripts.
        #[arg(long, value_name = "SCRIPT", action = ArgAction::Append)]
        parallel: Vec<String>,
        /// Trailing arguments forwarded to the script (`{{name}}` substitution).
        #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
        script_args: Vec<String>,
    },
    #[command(about = "Initialize a Scripts.toml file in the current directory")]
    Init {
        /// Apply a named template (use --list-templates to see options).
        #[arg(short, long, value_name = "NAME")]
        template: Option<String>,
        /// List the available templates and exit.
        #[arg(long)]
        list_templates: bool,
        /// Overwrite existing files when applying a template.
        #[arg(long)]
        force: bool,
    },
    #[command(about = "Show all script names and descriptions defined in Scripts.toml")]
    Show {
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
        /// Filter scripts by name or description
        #[arg(short, long, value_name = "PATTERN")]
        filter: Option<String>,
    },
    #[command(about = "Generate shell completion scripts")]
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: Shell,
    },
    #[command(about = "Validate Scripts.toml syntax, script references, and tool requirements")]
    Validate {
        /// Suppress all output except errors
        #[arg(short, long)]
        quiet: bool,
        /// Show detailed output
        #[arg(short = 'v', long)]
        verbose: bool,
    },
    #[command(about = "Workspace operations: list members, run a script across them, etc.")]
    Workspace {
        #[command(subcommand)]
        cmd: WorkspaceCmd,
    },
}

/// Workspace subcommands.
#[derive(Subcommand, Debug, Clone)]
pub enum WorkspaceCmd {
    /// List discovered workspace members.
    #[command(about = "List discovered workspace members")]
    List {
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
    /// Run a script in every member.
    #[command(about = "Run a script in every workspace member")]
    Run {
        #[arg(value_name = "SCRIPT_NAME")]
        script: String,
        /// Run in parallel rather than sequentially.
        #[arg(long)]
        parallel: bool,
        /// Emit JSON.
        #[arg(long)]
        json: bool,
    },
}

/// Supported shells for completion generation.
#[derive(ValueEnum, Clone, Debug)]
#[value(rename_all = "kebab-case")]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    #[value(name = "power-shell")]
    PowerShell,
}

pub mod init;
pub mod script;
pub mod show;
pub mod completions;
pub mod validate;
pub mod workspace;
pub mod cargo_script;
pub mod templates;

#[cfg(feature = "parallel")]
pub mod parallel;

#[cfg(feature = "watch")]
pub mod watch;
