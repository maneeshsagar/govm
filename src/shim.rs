use anyhow::Result;
use std::env;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use crate::constants::GO_BINARIES;

/// Ensure shims exist - only creates them if missing or outdated
pub fn ensure_shims(shims_dir: &Path) -> Result<()> {
    let govm_path = env::current_exe()?;
    let govm_path_str = govm_path.display().to_string();

    for binary in GO_BINARIES {
        let shim_path = shims_dir.join(binary);

        // Check if shim exists and contains correct govm path
        let needs_update = if shim_path.exists() {
            match fs::read_to_string(&shim_path) {
                Ok(content) => !content.contains(&govm_path_str),
                Err(_) => true,
            }
        } else {
            true
        };

        if needs_update {
            create_shim(binary, &govm_path, shims_dir)?;
        }
    }

    Ok(())
}

/// Create a single shim script
pub fn create_shim(binary: &str, govm_path: &Path, shims_dir: &Path) -> Result<()> {
    let shim_path = shims_dir.join(binary);

    let shim_content = format!(
        r#"#!/bin/sh
# Shim created by govm - DO NOT EDIT
# This shim intercepts calls to '{binary}' and delegates to the appropriate Go version

exec "{govm}" exec "{binary}" "$@"
"#,
        govm = govm_path.display(),
        binary = binary
    );

    fs::write(&shim_path, &shim_content)?;

    // Make executable
    let mut perms = fs::metadata(&shim_path)?.permissions();
    perms.set_mode(0o755);
    fs::set_permissions(&shim_path, perms)?;

    Ok(())
}

/// Force recreate all shims (used by rehash command)
pub fn create_all_shims(shims_dir: &Path) -> Result<()> {
    let govm_path = env::current_exe()?;

    for binary in GO_BINARIES {
        create_shim(binary, &govm_path, shims_dir)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_create_shim_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();
        let govm_path = PathBuf::from("/usr/local/bin/govm");

        create_shim("go", &govm_path, &shims_dir).unwrap();

        let shim_path = shims_dir.join("go");
        assert!(shim_path.exists(), "Shim file should exist");
    }

    #[test]
    fn test_create_shim_content() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();
        let govm_path = PathBuf::from("/usr/local/bin/govm");

        create_shim("go", &govm_path, &shims_dir).unwrap();

        let shim_path = shims_dir.join("go");
        let content = fs::read_to_string(&shim_path).unwrap();

        assert!(content.starts_with("#!/bin/sh"), "Shim should start with shebang");
        assert!(content.contains("/usr/local/bin/govm"), "Shim should contain govm path");
        assert!(content.contains("exec"), "Shim should contain exec command");
        assert!(content.contains("\"go\""), "Shim should contain binary name");
    }

    #[test]
    fn test_create_shim_executable() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();
        let govm_path = PathBuf::from("/usr/local/bin/govm");

        create_shim("go", &govm_path, &shims_dir).unwrap();

        let shim_path = shims_dir.join("go");
        let metadata = fs::metadata(&shim_path).unwrap();
        let permissions = metadata.permissions();

        // Check if executable (mode & 0o111 != 0)
        assert!(
            permissions.mode() & 0o111 != 0,
            "Shim should be executable"
        );
    }

    #[test]
    fn test_create_all_shims() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();

        create_all_shims(&shims_dir).unwrap();

        // Check that both go and gofmt shims exist
        assert!(shims_dir.join("go").exists(), "go shim should exist");
        assert!(shims_dir.join("gofmt").exists(), "gofmt shim should exist");
    }

    #[test]
    fn test_create_shim_for_gofmt() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();
        let govm_path = PathBuf::from("/usr/local/bin/govm");

        create_shim("gofmt", &govm_path, &shims_dir).unwrap();

        let shim_path = shims_dir.join("gofmt");
        let content = fs::read_to_string(&shim_path).unwrap();

        assert!(content.contains("\"gofmt\""), "Shim should contain gofmt");
    }

    #[test]
    fn test_ensure_shims_creates_missing() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();

        ensure_shims(&shims_dir).unwrap();

        // Both shims should be created
        assert!(shims_dir.join("go").exists());
        assert!(shims_dir.join("gofmt").exists());
    }

    #[test]
    fn test_ensure_shims_updates_outdated() {
        let temp_dir = TempDir::new().unwrap();
        let shims_dir = temp_dir.path().to_path_buf();

        // Create an outdated shim with wrong path
        let shim_path = shims_dir.join("go");
        fs::write(&shim_path, "#!/bin/sh\nexec /wrong/path/govm exec go \"$@\"\n").unwrap();

        ensure_shims(&shims_dir).unwrap();

        // Shim should be updated with correct path
        let content = fs::read_to_string(&shim_path).unwrap();
        assert!(
            !content.contains("/wrong/path/govm"),
            "Outdated shim should be replaced"
        );
    }
}
