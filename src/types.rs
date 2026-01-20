use serde::Deserialize;

/// Represents a Go version from the official API
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct GoVersion {
    pub version: String,
    pub stable: bool,
    pub files: Vec<GoFile>,
}

/// Represents a downloadable Go file/archive
#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct GoFile {
    pub filename: String,
    pub os: String,
    pub arch: String,
    #[allow(dead_code)]
    pub sha256: String,
    pub size: u64,
    pub kind: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_go_version_deserialize() {
        let json = r#"{
            "version": "go1.22.0",
            "stable": true,
            "files": []
        }"#;

        let version: GoVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.version, "go1.22.0");
        assert!(version.stable);
        assert!(version.files.is_empty());
    }

    #[test]
    fn test_go_file_deserialize() {
        let json = r#"{
            "filename": "go1.22.0.darwin-arm64.tar.gz",
            "os": "darwin",
            "arch": "arm64",
            "sha256": "abc123",
            "size": 12345678,
            "kind": "archive"
        }"#;

        let file: GoFile = serde_json::from_str(json).unwrap();
        assert_eq!(file.filename, "go1.22.0.darwin-arm64.tar.gz");
        assert_eq!(file.os, "darwin");
        assert_eq!(file.arch, "arm64");
        assert_eq!(file.size, 12345678);
        assert_eq!(file.kind, "archive");
    }

    #[test]
    fn test_go_version_with_files() {
        let json = r#"{
            "version": "go1.22.0",
            "stable": true,
            "files": [
                {
                    "filename": "go1.22.0.darwin-arm64.tar.gz",
                    "os": "darwin",
                    "arch": "arm64",
                    "sha256": "abc123",
                    "size": 12345678,
                    "kind": "archive"
                },
                {
                    "filename": "go1.22.0.linux-amd64.tar.gz",
                    "os": "linux",
                    "arch": "amd64",
                    "sha256": "def456",
                    "size": 23456789,
                    "kind": "archive"
                }
            ]
        }"#;

        let version: GoVersion = serde_json::from_str(json).unwrap();
        assert_eq!(version.files.len(), 2);
        assert_eq!(version.files[0].os, "darwin");
        assert_eq!(version.files[1].os, "linux");
    }

    #[test]
    fn test_go_version_unstable() {
        let json = r#"{
            "version": "go1.23rc1",
            "stable": false,
            "files": []
        }"#;

        let version: GoVersion = serde_json::from_str(json).unwrap();
        assert!(!version.stable);
    }
}
