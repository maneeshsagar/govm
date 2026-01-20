//! govm - Go Version Manager
//!
//! A shim-based Go version manager written in Rust.
//! Inspired by rbenv, pyenv, and nvm.

mod cli;
mod constants;
mod download;
mod govm;
mod shim;
mod types;
mod version;

use anyhow::Result;
use clap::Parser;
use colored::*;

use cli::{Cli, Commands};
use govm::GoVM;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let govm = GoVM::new()?;

    match cli.command {
        Commands::Install { go_version } => {
            govm.install_version(&go_version).await?;
        }
        Commands::Use { go_version, local } => {
            govm.use_version(&go_version, local).await?;
        }
        Commands::Global { go_version } => match go_version {
            Some(v) => govm.set_global_version(&v)?,
            None => match govm.get_global_version()? {
                Some(v) => println!("{}", v),
                None => println!("{} No global version set", "→".blue()),
            },
        },
        Commands::Local { go_version } => {
            govm.set_local_version(&go_version)?;
        }
        Commands::Version => {
            govm.show_version()?;
        }
        Commands::Versions => {
            govm.list_versions()?;
        }
        Commands::ListRemote { all, limit } => {
            govm.list_remote_versions(all, limit).await?;
        }
        Commands::Uninstall { go_version } => {
            govm.uninstall_version(&go_version)?;
        }
        Commands::Which { command } => {
            govm.which_command(&command)?;
        }
        Commands::Exec { command, args } => {
            govm.exec_command(&command, &args)?;
        }
        Commands::Rehash => {
            govm.rehash()?;
        }
        Commands::Prune { keep } => {
            govm.prune_versions(keep)?;
        }
    }

    Ok(())
}
