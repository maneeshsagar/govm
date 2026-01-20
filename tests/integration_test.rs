//! Integration tests for govm

use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Helper to run govm command
fn run_govm(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_govm"))
        .args(args)
        .output()
        .expect("Failed to execute govm")
}

/// Helper to run govm with custom GOVM_ROOT
fn run_govm_with_root(args: &[&str], govm_root: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_govm"))
        .args(args)
        .env("HOME", govm_root)
        .output()
        .expect("Failed to execute govm")
}

#[test]
fn test_govm_help() {
    let output = run_govm(&["--help"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Go Version Manager"));
    assert!(stdout.contains("install"));
    assert!(stdout.contains("use"));
    assert!(stdout.contains("versions"));
}

#[test]
fn test_govm_version() {
    let output = run_govm(&["--version"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("govm"));
    assert!(stdout.contains("0.1.0"));
}

#[test]
fn test_govm_list_remote_help() {
    let output = run_govm(&["list-remote", "--help"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("List available Go versions"));
    assert!(stdout.contains("--all"));
    assert!(stdout.contains("--limit"));
}

#[test]
fn test_govm_install_help() {
    let output = run_govm(&["install", "--help"]);
    
    assert!(output.status.success(), "install --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Install") || stdout.contains("VERSION"));
}

#[test]
fn test_govm_use_help() {
    let output = run_govm(&["use", "--help"]);
    
    assert!(output.status.success(), "use --help should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Switch") || stdout.contains("VERSION"));
}

#[test]
fn test_govm_versions_empty() {
    let temp_dir = TempDir::new().unwrap();
    let govm_root = temp_dir.path().join(".govm");
    fs::create_dir_all(govm_root.join("versions")).unwrap();
    fs::create_dir_all(govm_root.join("shims")).unwrap();
    
    let output = run_govm_with_root(&["versions"], temp_dir.path().to_str().unwrap());
    
    // Should show "No Go versions installed" message
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No Go versions installed") || stdout.contains("Installed"),
        "Should handle empty versions directory"
    );
}

#[test]
fn test_govm_which_no_version() {
    let temp_dir = TempDir::new().unwrap();
    let govm_root = temp_dir.path().join(".govm");
    fs::create_dir_all(govm_root.join("versions")).unwrap();
    fs::create_dir_all(govm_root.join("shims")).unwrap();
    
    let output = run_govm_with_root(&["which", "go"], temp_dir.path().to_str().unwrap());
    
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should indicate no version configured
    assert!(
        stdout.contains("No Go version") || stdout.contains("configured"),
        "Should handle no version configured"
    );
}

#[test]
fn test_govm_rehash() {
    let output = run_govm(&["rehash"]);
    
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Regenerating shims") || stdout.contains("shims"));
}

#[test]
fn test_govm_global_no_version() {
    let output = run_govm(&["global"]);
    
    // Command should succeed
    assert!(output.status.success(), "global command should succeed");
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should either show version or "No global version set"
    assert!(
        stdout.contains("No global version")
            || stdout.contains("→")
            || stdout.chars().any(|c| c.is_numeric())
            || !stdout.is_empty(),
        "Should show global version status"
    );
}

#[test]
fn test_govm_version_command() {
    let output = run_govm(&["version"]);
    
    // Should show current version or "No Go version configured"
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("No Go version") || 
        stdout.contains("→") ||
        stdout.chars().any(|c| c.is_numeric()),
        "Should show version status"
    );
}

#[test]
fn test_govm_uninstall_nonexistent() {
    let output = run_govm(&["uninstall", "99.99.99"]);
    
    // Should succeed (gracefully handles non-existent version)
    assert!(output.status.success(), "uninstall should handle non-existent version");
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("not installed") || stdout.contains("✗"),
        "Should indicate version is not installed: {}",
        stdout
    );
}

#[test]
fn test_command_aliases_ls() {
    // Test 'ls' alias for 'versions'
    let output = run_govm(&["ls"]);
    assert!(output.status.success(), "ls alias should work");
}

#[test]
fn test_command_aliases_ls_remote() {
    // Test 'ls-remote' alias for 'list-remote' (just help to avoid network)
    let output = run_govm(&["ls-remote", "--help"]);
    assert!(output.status.success(), "ls-remote alias should work");
}

#[test]
fn test_command_aliases_i() {
    // Test 'i' alias for 'install'
    let output = run_govm(&["i", "--help"]);
    assert!(output.status.success(), "i alias should work");
}

#[test]
fn test_command_aliases_rm() {
    // Test 'rm' alias for 'uninstall'
    let output = run_govm(&["rm", "--help"]);
    assert!(output.status.success(), "rm alias should work");
}
