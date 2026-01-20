use anyhow::Result;
use flate2::read::GzDecoder;
use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use tar::Archive;

use crate::constants::GO_VERSION_LIST;
use crate::types::GoVersion;

/// Fetch list of available Go versions from the official API
pub async fn fetch_remote_versions() -> Result<Vec<GoVersion>> {
    let client = reqwest::Client::new();
    let response = client
        .get(GO_VERSION_LIST)
        .header("User-Agent", "govm/0.1.0")
        .send()
        .await?
        .json::<Vec<GoVersion>>()
        .await?;
    Ok(response)
}

/// Download a file with progress bar
pub async fn download_file(url: &str, path: &PathBuf, total_size: u64) -> Result<()> {
    let client = reqwest::Client::new();
    let response = client
        .get(url)
        .header("User-Agent", "govm/0.1.0")
        .send()
        .await?;

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")?
            .progress_chars("█▓▒░"),
    );

    let mut file = File::create(path)?;
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }

    pb.finish_with_message("Download complete");
    Ok(())
}

/// Extract a .tar.gz archive to a destination directory
pub fn extract_archive(archive_path: &PathBuf, dest_dir: &PathBuf, temp_dir: &PathBuf) -> Result<()> {
    let tar_gz = File::open(archive_path)?;
    let tar = GzDecoder::new(tar_gz);
    let mut archive = Archive::new(tar);

    // Create a temporary directory for extraction
    fs::create_dir_all(temp_dir)?;

    archive.unpack(temp_dir)?;

    // Move the 'go' directory to the version directory
    let extracted_go = temp_dir.join("go");
    if extracted_go.exists() {
        fs::rename(&extracted_go, dest_dir)?;
    }

    // Clean up temp directory
    let _ = fs::remove_dir_all(temp_dir);

    Ok(())
}

/// Get the current platform (os, arch)
pub fn get_platform() -> (&'static str, &'static str) {
    let os = if cfg!(target_os = "macos") {
        "darwin"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "windows") {
        "windows"
    } else {
        "unknown"
    };

    let arch = if cfg!(target_arch = "x86_64") {
        "amd64"
    } else if cfg!(target_arch = "aarch64") {
        "arm64"
    } else if cfg!(target_arch = "x86") {
        "386"
    } else {
        "unknown"
    };

    (os, arch)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_platform_returns_valid_os() {
        let (os, _) = get_platform();
        assert!(
            ["darwin", "linux", "windows", "unknown"].contains(&os),
            "OS should be one of darwin, linux, windows, or unknown"
        );
    }

    #[test]
    fn test_get_platform_returns_valid_arch() {
        let (_, arch) = get_platform();
        assert!(
            ["amd64", "arm64", "386", "unknown"].contains(&arch),
            "Arch should be one of amd64, arm64, 386, or unknown"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn test_get_platform_macos() {
        let (os, _) = get_platform();
        assert_eq!(os, "darwin");
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn test_get_platform_linux() {
        let (os, _) = get_platform();
        assert_eq!(os, "linux");
    }

    #[cfg(target_arch = "x86_64")]
    #[test]
    fn test_get_platform_amd64() {
        let (_, arch) = get_platform();
        assert_eq!(arch, "amd64");
    }

    #[cfg(target_arch = "aarch64")]
    #[test]
    fn test_get_platform_arm64() {
        let (_, arch) = get_platform();
        assert_eq!(arch, "arm64");
    }
}
