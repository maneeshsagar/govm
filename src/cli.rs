use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "govm")]
#[command(author = "govm contributors")]
#[command(version = "0.1.0")]
#[command(about = "Go Version Manager (shim-based) - Install, use, and manage Go versions", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Install a specific Go version
    #[command(alias = "i")]
    Install {
        /// The Go version to install (e.g., 1.21.0, 1.22.0)
        #[arg(name = "VERSION")]
        go_version: String,
    },

    /// Switch to a specific Go version (installs if needed)
    Use {
        /// The Go version to switch to
        #[arg(name = "VERSION")]
        go_version: String,
        /// Set as local version instead of global
        #[arg(short, long)]
        local: bool,
    },

    /// Set or show the global Go version
    Global {
        /// The Go version to set as global default (omit to show current)
        #[arg(name = "VERSION")]
        go_version: Option<String>,
    },

    /// Set the local Go version (creates .go-version file)
    Local {
        /// The Go version for the current directory
        #[arg(name = "VERSION")]
        go_version: String,
    },

    /// Show the current Go version (resolved for current directory)
    Version,

    /// List installed Go versions
    #[command(alias = "ls")]
    Versions,

    /// List available Go versions for download
    #[command(alias = "ls-remote")]
    ListRemote {
        /// Show all versions including release candidates and betas
        #[arg(short, long)]
        all: bool,

        /// Maximum number of versions to show
        #[arg(short, long, default_value = "20")]
        limit: usize,
    },

    /// Uninstall a specific Go version
    #[command(alias = "rm")]
    Uninstall {
        /// The Go version to uninstall
        #[arg(name = "VERSION")]
        go_version: String,
    },

    /// Show path to the Go executable that will be used
    Which {
        /// The command to look up (default: go)
        #[arg(default_value = "go")]
        command: String,
    },

    /// Execute a command with the resolved Go version
    Exec {
        /// The command to execute
        command: String,
        /// Arguments to pass to the command
        #[arg(trailing_var_arg = true)]
        args: Vec<String>,
    },

    /// Regenerate shims for all Go binaries
    Rehash,

    /// Prune old/unused Go versions
    Prune {
        /// Keep this many latest versions
        #[arg(short, long, default_value = "3")]
        keep: usize,
    },
}
