use anyhow::{bail, Context, Result};
use colored::*;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::{exit, Command};

use crate::constants::{GO_BINARIES, GO_DOWNLOAD_BASE};
use crate::download::{download_file, extract_archive, fetch_remote_versions, get_platform};
use crate::shim::{create_all_shims, ensure_shims};
use crate::version::{self, find_local_version, get_global_version, normalize, parse};

/// Main GoVM manager struct
pub struct GoVM {
    pub root_dir: PathBuf,
    pub versions_dir: PathBuf,
    pub shims_dir: PathBuf,
    pub global_version_file: PathBuf,
}

impl GoVM {
    /// Create a new GoVM instance
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let root_dir = home.join(".govm");
        let versions_dir = root_dir.join("versions");
        let shims_dir = root_dir.join("shims");
        let global_version_file = root_dir.join("version");

        // Create directories if they don't exist
        fs::create_dir_all(&versions_dir)?;
        fs::create_dir_all(&shims_dir)?;

        Ok(Self {
            root_dir,
            versions_dir,
            shims_dir,
            global_version_file,
        })
    }

    /// Get list of installed Go versions
    pub fn get_installed_versions(&self) -> Result<Vec<String>> {
        let mut versions = Vec::new();
        if self.versions_dir.exists() {
            for entry in fs::read_dir(&self.versions_dir)? {
                let entry = entry?;
                if entry.file_type()?.is_dir() {
                    if let Some(name) = entry.file_name().to_str() {
                        versions.push(name.to_string());
                    }
                }
            }
        }
        // Sort by parsed version in descending order
        versions.sort_by_key(|v| std::cmp::Reverse(parse(v)));
        Ok(versions)
    }

    /// Check if a version is installed
    pub fn is_version_installed(&self, version: &str) -> bool {
        self.versions_dir.join(version).exists()
    }

    /// Get path to a binary in a specific version
    fn get_version_bin_path(&self, version: &str, binary: &str) -> PathBuf {
        self.versions_dir.join(version).join("bin").join(binary)
    }

    /// Resolve the current Go version
    pub fn resolve_version(&self) -> Result<Option<String>> {
        version::resolve(&self.global_version_file)
    }

    /// Get the global version
    pub fn get_global_version(&self) -> Result<Option<String>> {
        get_global_version(&self.global_version_file)
    }

    /// Set the global Go version
    pub fn set_global_version(&self, version: &str) -> Result<()> {
        let version = normalize(version);

        if !self.is_version_installed(&version) {
            bail!(
                "Go {} is not installed. Run 'govm install {}' first.",
                version,
                version
            );
        }

        fs::write(&self.global_version_file, format!("{}\n", version))?;
        println!(
            "{} Set global Go version to {}",
            "✓".green(),
            version.cyan()
        );
        Ok(())
    }

    /// Set the local Go version (creates .go-version file)
    pub fn set_local_version(&self, version: &str) -> Result<()> {
        let version = normalize(version);

        if !self.is_version_installed(&version) {
            bail!(
                "Go {} is not installed. Run 'govm install {}' first.",
                version,
                version
            );
        }

        let version_file = env::current_dir()?.join(".go-version");
        fs::write(&version_file, format!("{}\n", version))?;
        println!(
            "{} Set local Go version to {} ({})",
            "✓".green(),
            version.cyan(),
            version_file.display().to_string().dimmed()
        );
        Ok(())
    }

    /// Use a specific version - installs if needed, then sets as global or local
    pub async fn use_version(&self, version: &str, local: bool) -> Result<()> {
        let version = normalize(version);

        // Install if not already installed
        if !self.is_version_installed(&version) {
            println!(
                "{} Go {} is not installed, installing...",
                "→".blue(),
                version.cyan()
            );
            self.install_version(&version).await?;
        }

        // Set as local or global
        if local {
            let version_file = env::current_dir()?.join(".go-version");
            fs::write(&version_file, format!("{}\n", version))?;
            println!(
                "{} Now using Go {} {}",
                "✓".green(),
                version.cyan(),
                format!("(local: {})", version_file.display()).dimmed()
            );
        } else {
            fs::write(&self.global_version_file, format!("{}\n", version))?;
            println!(
                "{} Now using Go {} {}",
                "✓".green(),
                version.cyan(),
                "(global)".dimmed()
            );
        }

        Ok(())
    }

    /// Install a specific Go version
    pub async fn install_version(&self, version: &str) -> Result<()> {
        let version = normalize(version);

        if self.is_version_installed(&version) {
            println!(
                "{} Go {} is already installed",
                "✓".green(),
                version.cyan()
            );
            return Ok(());
        }

        println!("{} Fetching Go version information...", "→".blue());

        let versions = fetch_remote_versions().await?;
        let go_version = versions
            .iter()
            .find(|v| normalize(&v.version) == version)
            .context(format!("Version {} not found", version))?;

        let (os, arch) = get_platform();
        let file = go_version
            .files
            .iter()
            .find(|f| f.os == os && f.arch == arch && f.kind == "archive")
            .context(format!(
                "No binary available for {} {} (Go {})",
                os, arch, version
            ))?;

        let download_url = format!("{}{}", GO_DOWNLOAD_BASE, file.filename);
        let version_dir = self.versions_dir.join(&version);
        let archive_path = self.root_dir.join(&file.filename);
        let temp_dir = self.root_dir.join("temp_extract");

        println!("{} Downloading Go {}...", "↓".blue(), version.cyan());
        download_file(&download_url, &archive_path, file.size).await?;

        println!("{} Extracting archive...", "⚙".blue());
        extract_archive(&archive_path, &version_dir, &temp_dir)?;

        // Clean up archive
        fs::remove_file(&archive_path)?;

        // Create shims only if they don't exist
        ensure_shims(&self.shims_dir)?;

        println!(
            "{} Go {} installed successfully!",
            "✓".green(),
            version.cyan()
        );

        // Set as global if it's the first version
        let installed = self.get_installed_versions()?;
        if installed.len() == 1 {
            self.set_global_version(&version)?;
        }

        Ok(())
    }

    /// Uninstall a specific Go version
    pub fn uninstall_version(&self, version: &str) -> Result<()> {
        let version = normalize(version);

        if !self.is_version_installed(&version) {
            println!("{} Go {} is not installed", "✗".red(), version.cyan());
            return Ok(());
        }

        // Check if it's the global version
        let global = self.get_global_version()?;
        if global.as_ref() == Some(&version) {
            // Remove global version file
            if self.global_version_file.exists() {
                fs::remove_file(&self.global_version_file)?;
            }
            println!("{} Cleared global version", "→".blue());
        }

        let version_dir = self.versions_dir.join(&version);
        fs::remove_dir_all(&version_dir)?;

        println!(
            "{} Go {} has been uninstalled",
            "✓".green(),
            version.cyan()
        );

        Ok(())
    }

    /// Execute a command with the resolved Go version
    pub fn exec_command(&self, command: &str, args: &[String]) -> Result<()> {
        let version = self.resolve_version()?.context(
            "No Go version configured. Run 'govm global <version>' or create a .go-version file",
        )?;

        if !self.is_version_installed(&version) {
            bail!(
                "Go {} is not installed (required by current configuration). Run 'govm install {}'",
                version,
                version
            );
        }

        let binary_path = self.get_version_bin_path(&version, command);

        if !binary_path.exists() {
            bail!("Command '{}' not found in Go {}", command, version);
        }

        // Set GOROOT for the executed command
        let goroot = self.versions_dir.join(&version);

        let status = Command::new(&binary_path)
            .args(args)
            .env("GOROOT", &goroot)
            .status()
            .context(format!("Failed to execute {}", command))?;

        exit(status.code().unwrap_or(1));
    }

    /// Show which binary will be used
    pub fn which_command(&self, command: &str) -> Result<()> {
        match self.resolve_version()? {
            Some(version) => {
                if !self.is_version_installed(&version) {
                    println!(
                        "{} Go {} is configured but not installed",
                        "✗".red(),
                        version
                    );
                    return Ok(());
                }
                let path = self.get_version_bin_path(&version, command);
                if path.exists() {
                    println!("{}", path.display());
                } else {
                    println!(
                        "{} Command '{}' not found in Go {}",
                        "✗".red(),
                        command,
                        version
                    );
                }
            }
            None => {
                println!("{} No Go version configured", "✗".red());
            }
        }
        Ok(())
    }

    /// Show the current resolved version
    pub fn show_version(&self) -> Result<()> {
        match self.resolve_version()? {
            Some(version) => {
                // Show where the version is coming from
                if env::var("GOVM_VERSION").is_ok() {
                    println!(
                        "{} {} {}",
                        "→".green(),
                        version.green().bold(),
                        "(set by GOVM_VERSION)".dimmed()
                    );
                } else if let Some(local_version) = find_local_version()? {
                    if local_version == version {
                        let mut current = env::current_dir()?;
                        loop {
                            let version_file = current.join(".go-version");
                            if version_file.exists() {
                                println!(
                                    "{} {} {}",
                                    "→".green(),
                                    version.green().bold(),
                                    format!("(set by {})", version_file.display()).dimmed()
                                );
                                break;
                            }
                            if !current.pop() {
                                break;
                            }
                        }
                    }
                } else {
                    println!(
                        "{} {} {}",
                        "→".green(),
                        version.green().bold(),
                        "(set by ~/.govm/version)".dimmed()
                    );
                }

                if !self.is_version_installed(&version) {
                    println!(
                        "  {} This version is not installed. Run: govm install {}",
                        "⚠".yellow(),
                        version
                    );
                }
            }
            None => {
                println!("{} No Go version configured", "→".blue());
                println!("  Set a global version: {}", "govm global <version>".yellow());
                println!(
                    "  Or create a local .go-version file: {}",
                    "govm local <version>".yellow()
                );
            }
        }
        Ok(())
    }

    /// List installed versions
    pub fn list_versions(&self) -> Result<()> {
        let versions = self.get_installed_versions()?;
        let current = self.resolve_version()?;
        let global = self.get_global_version()?;

        if versions.is_empty() {
            println!("{} No Go versions installed", "→".blue());
            println!(
                "  Run {} to see available versions",
                "govm list-remote".yellow()
            );
            return Ok(());
        }

        println!("{}", "Installed Go versions:".bold());
        println!();

        for version in versions {
            let is_current = current.as_ref() == Some(&version);
            let is_global = global.as_ref() == Some(&version);

            let marker = if is_current {
                "→".green().to_string()
            } else {
                " ".to_string()
            };
            let version_str = if is_current {
                version.green().bold().to_string()
            } else {
                version.clone()
            };

            let mut labels = Vec::new();
            if is_current {
                labels.push("current");
            }
            if is_global {
                labels.push("global");
            }

            let label_str = if labels.is_empty() {
                String::new()
            } else {
                format!(" ({})", labels.join(", ")).dimmed().to_string()
            };

            println!("  {} {}{}", marker, version_str, label_str);
        }

        Ok(())
    }

    /// List remote available versions
    pub async fn list_remote_versions(&self, all: bool, limit: usize) -> Result<()> {
        println!("{} Fetching available Go versions...", "→".blue());

        let versions = fetch_remote_versions().await?;
        let installed = self.get_installed_versions()?;
        let current = self.resolve_version()?;

        let filtered: Vec<_> = if all {
            versions.into_iter().take(limit).collect()
        } else {
            versions
                .into_iter()
                .filter(|v| v.stable)
                .take(limit)
                .collect()
        };

        println!();
        println!("{}", "Available Go versions:".bold());
        println!();

        for v in filtered {
            let version = normalize(&v.version);
            let is_installed = installed.contains(&version);
            let is_current = current.as_ref() == Some(&version);

            let status = if is_current {
                format!("{} {}", "→".green(), version.green().bold())
            } else if is_installed {
                format!("  {} {}", "✓".dimmed(), version.dimmed())
            } else {
                format!("    {}", version)
            };

            let label = if !v.stable {
                " (unstable)".yellow().to_string()
            } else {
                String::new()
            };

            println!("{}{}", status, label);
        }

        if !all {
            println!();
            println!(
                "  Use {} to see all versions including RCs and betas",
                "govm list-remote --all".dimmed()
            );
        }

        Ok(())
    }

    /// Regenerate all shims
    pub fn rehash(&self) -> Result<()> {
        println!("{} Regenerating shims...", "→".blue());
        create_all_shims(&self.shims_dir)?;

        for binary in GO_BINARIES {
            println!("  {} {}", "✓".green(), binary);
        }

        println!(
            "{} Shims regenerated in {}",
            "✓".green(),
            self.shims_dir.display().to_string().cyan()
        );
        Ok(())
    }

    /// Prune old versions
    pub fn prune_versions(&self, keep: usize) -> Result<()> {
        let versions = self.get_installed_versions()?;
        let global = self.get_global_version()?;

        if versions.len() <= keep {
            println!(
                "{} Nothing to prune. {} versions installed, keeping {}.",
                "→".blue(),
                versions.len(),
                keep
            );
            return Ok(());
        }

        let to_remove: Vec<_> = versions
            .iter()
            .skip(keep)
            .filter(|v| global.as_ref() != Some(*v))
            .collect();

        if to_remove.is_empty() {
            println!("{} Nothing to prune.", "→".blue());
            return Ok(());
        }

        println!("{}", "The following versions will be removed:".bold());
        for v in &to_remove {
            println!("  - {}", v.red());
        }
        println!();

        print!("Continue? [y/N] ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        if input.trim().to_lowercase() == "y" {
            for v in to_remove {
                let version_dir = self.versions_dir.join(v);
                fs::remove_dir_all(&version_dir)?;
                println!("{} Removed Go {}", "✓".green(), v);
            }
        } else {
            println!("{} Prune cancelled", "→".blue());
        }

        Ok(())
    }
}
