use anyhow::Result;
use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Normalize version string by removing prefixes like 'v' or 'go'
pub fn normalize(version: &str) -> String {
    version
        .trim_start_matches('v')
        .trim_start_matches("go")
        .to_string()
}

/// Parse version into components for comparison
pub fn parse(v: &str) -> (u32, u32, u32, String) {
    let re = Regex::new(r"^(\d+)\.(\d+)(?:\.(\d+))?(.*)$").unwrap();
    if let Some(caps) = re.captures(v) {
        let major: u32 = caps.get(1).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let minor: u32 = caps.get(2).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let patch: u32 = caps.get(3).map_or(0, |m| m.as_str().parse().unwrap_or(0));
        let suffix = caps.get(4).map_or("", |m| m.as_str()).to_string();
        (major, minor, patch, suffix)
    } else {
        (0, 0, 0, v.to_string())
    }
}

/// Resolve the Go version to use based on priority:
/// 1. GOVM_VERSION environment variable
/// 2. .go-version file in current or parent directories
/// 3. Global version file (~/.govm/version)
pub fn resolve(global_version_file: &PathBuf) -> Result<Option<String>> {
    // 1. Check environment variable
    if let Ok(version) = env::var("GOVM_VERSION") {
        let version = normalize(&version);
        if !version.is_empty() {
            return Ok(Some(version));
        }
    }

    // 2. Check .go-version file in current and parent directories
    if let Some(version) = find_local_version()? {
        return Ok(Some(version));
    }

    // 3. Check global version
    if let Some(version) = get_global_version(global_version_file)? {
        return Ok(Some(version));
    }

    Ok(None)
}

/// Search for .go-version file starting from current directory and walking up
pub fn find_local_version() -> Result<Option<String>> {
    let mut current = env::current_dir()?;

    loop {
        let version_file = current.join(".go-version");
        if version_file.exists() {
            let content = fs::read_to_string(&version_file)?;
            let version = normalize(content.trim());
            if !version.is_empty() {
                return Ok(Some(version));
            }
        }

        if !current.pop() {
            break;
        }
    }

    Ok(None)
}

/// Get the global version from ~/.govm/version
pub fn get_global_version(global_version_file: &PathBuf) -> Result<Option<String>> {
    if global_version_file.exists() {
        let content = fs::read_to_string(global_version_file)?;
        let version = normalize(content.trim());
        if !version.is_empty() {
            return Ok(Some(version));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_normalize_plain_version() {
        assert_eq!(normalize("1.21.0"), "1.21.0");
        assert_eq!(normalize("1.22.5"), "1.22.5");
    }

    #[test]
    fn test_normalize_with_v_prefix() {
        assert_eq!(normalize("v1.21.0"), "1.21.0");
        assert_eq!(normalize("v1.22.5"), "1.22.5");
    }

    #[test]
    fn test_normalize_with_go_prefix() {
        assert_eq!(normalize("go1.21.0"), "1.21.0");
        assert_eq!(normalize("go1.22.5"), "1.22.5");
    }

    #[test]
    fn test_normalize_empty_string() {
        assert_eq!(normalize(""), "");
    }

    #[test]
    fn test_parse_full_version() {
        assert_eq!(parse("1.21.0"), (1, 21, 0, String::new()));
        assert_eq!(parse("1.22.5"), (1, 22, 5, String::new()));
        assert_eq!(parse("2.0.0"), (2, 0, 0, String::new()));
    }

    #[test]
    fn test_parse_version_without_patch() {
        assert_eq!(parse("1.21"), (1, 21, 0, String::new()));
        assert_eq!(parse("1.22"), (1, 22, 0, String::new()));
    }

    #[test]
    fn test_parse_version_with_suffix() {
        assert_eq!(parse("1.21.0rc1"), (1, 21, 0, "rc1".to_string()));
        assert_eq!(parse("1.22.0beta1"), (1, 22, 0, "beta1".to_string()));
        assert_eq!(parse("1.23rc2"), (1, 23, 0, "rc2".to_string()));
    }

    #[test]
    fn test_parse_invalid_version() {
        assert_eq!(parse("invalid"), (0, 0, 0, "invalid".to_string()));
        assert_eq!(parse(""), (0, 0, 0, String::new()));
    }

    #[test]
    fn test_version_comparison() {
        // Higher versions should sort first
        assert!(parse("1.22.0") > parse("1.21.0"));
        assert!(parse("1.21.5") > parse("1.21.0"));
        assert!(parse("2.0.0") > parse("1.99.99"));
    }

    #[test]
    fn test_get_global_version_exists() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("version");
        fs::write(&version_file, "1.22.0\n").unwrap();

        let result = get_global_version(&version_file).unwrap();
        assert_eq!(result, Some("1.22.0".to_string()));
    }

    #[test]
    fn test_get_global_version_not_exists() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("version");

        let result = get_global_version(&version_file).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_global_version_empty_file() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("version");
        fs::write(&version_file, "").unwrap();

        let result = get_global_version(&version_file).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn test_get_global_version_with_whitespace() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("version");
        fs::write(&version_file, "  1.22.0  \n").unwrap();

        let result = get_global_version(&version_file).unwrap();
        assert_eq!(result, Some("1.22.0".to_string()));
    }

    #[test]
    fn test_get_global_version_normalizes() {
        let temp_dir = TempDir::new().unwrap();
        let version_file = temp_dir.path().join("version");
        fs::write(&version_file, "go1.22.0\n").unwrap();

        let result = get_global_version(&version_file).unwrap();
        assert_eq!(result, Some("1.22.0".to_string()));
    }
}
