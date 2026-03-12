//! Claude CLI management module
//!
//! This module handles finding and modifying the Claude CLI installation
//! to support custom API hosts.

use anyhow::{Context, Result, anyhow};
use regex::Regex;
use std::env;
use std::fs;
use std::path::PathBuf;

/// Default API host if API_HOST environment variable is not set
const DEFAULT_API_HOST: &str = "anyrouter.top";

/// Error type for Claude CLI operations
#[derive(Debug)]
pub enum ClaudeCliError {
    /// Claude CLI not found in PATH
    NotFound,
    /// Failed to read CLI file
    ReadError(std::io::Error),
    /// Failed to write CLI file
    WriteError(std::io::Error),
    /// Failed to resolve CLI path
    PathResolutionError(String),
}

impl std::fmt::Display for ClaudeCliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ClaudeCliError::NotFound => {
                write!(f, "Claude CLI not found in PATH. Host patching skipped.")
            }
            ClaudeCliError::ReadError(e) => {
                write!(f, "Failed to read Claude CLI file: {}", e)
            }
            ClaudeCliError::WriteError(e) => {
                write!(f, "Failed to write Claude CLI file: {}", e)
            }
            ClaudeCliError::PathResolutionError(msg) => {
                write!(f, "Failed to resolve Claude CLI path: {}", msg)
            }
        }
    }
}

impl std::error::Error for ClaudeCliError {}

/// Find the Claude CLI executable path
///
/// This function searches for the `claude` command in PATH and returns
/// its full path. On Windows, it also checks for `claude.cmd` and `claude.exe`.
pub fn find_claude_cli() -> Result<PathBuf> {
    let claude_exe = if cfg!(windows) {
        // On Windows, try different extensions in order of preference
        ["claude.cmd", "claude.exe", "claude.bat", "claude.ps1"]
            .iter()
            .find_map(|name| which_in_path(name))
    } else {
        // On Unix-like systems, just look for "claude"
        which_in_path("claude")
    };

    claude_exe.ok_or_else(|| anyhow!(ClaudeCliError::NotFound))
}

/// Find an executable in PATH
fn which_in_path(name: &str) -> Option<PathBuf> {
    // First, check if it's an absolute path or relative path that exists
    if let Ok(current_exe) = env::current_exe() {
        if current_exe.file_name()?.to_str()? == name {
            return Some(current_exe);
        }
    }

    // Check PATH environment variable
    let path_var = env::var("PATH").ok()?;
    let path_sep = if cfg!(windows) { ";" } else { ":" };

    for path_dir in path_var.split(path_sep) {
        let full_path = PathBuf::from(path_dir).join(name);
        if full_path.exists() {
            return Some(full_path);
        }
    }

    None
}

/// Resolve the actual Claude CLI JavaScript file path
///
/// This function handles different installation methods:
/// 1. pnpm installation - extracts the path from the shell wrapper script
/// 2. npm installation - finds the cli.js directly
/// 3. Direct installation - returns the path as-is
pub fn resolve_cli_path(claude_exe: &PathBuf) -> Result<PathBuf> {
    // Read the wrapper script content
    let content = fs::read_to_string(claude_exe)
        .with_context(|| format!("Failed to read claude executable at {:?}", claude_exe))?;

    // Look for pnpm-style installation
    // Pattern: grep for "node_modules/@anthropic-ai/claude-code/cli.js"
    // and extract the basedir or direct path
    let cli_js_path = extract_cli_js_path(&content, claude_exe);

    if let Some(path) = cli_js_path {
        return Ok(path);
    }

    // If we couldn't find a cli.js path, try common installation locations
    let common_paths = find_common_cli_paths(claude_exe);
    for path in common_paths {
        if path.exists() {
            return Ok(path);
        }
    }

    // As a last resort, return the original path
    Ok(claude_exe.clone())
}

/// Extract cli.js path from wrapper script content
fn extract_cli_js_path(content: &str, wrapper_path: &PathBuf) -> Option<PathBuf> {
    // Look for patterns like:
    // - "$basedir/node_modules/@anthropic-ai/claude-code/cli.js"
    // - "node_modules/@anthropic-ai/claude-code/cli.js"
    // - "/some/path/node_modules/@anthropic-ai/claude-code/cli.js"

    let lines: Vec<&str> = content.lines().collect();

    // Find the line containing "node_modules/@anthropic-ai/claude-code/cli.js"
    for line in &lines {
        if line.contains("node_modules/@anthropic-ai/claude-code/cli.js") {
            // Try to extract the path
            if let Some(path) = extract_path_from_line(line, wrapper_path) {
                return Some(path);
            }
        }
    }

    None
}

/// Extract a file path from a wrapper script line
fn extract_path_from_line(line: &str, wrapper_path: &PathBuf) -> Option<PathBuf> {
    // Handle different wrapper script formats:

    // Format 1: "$basedir/node_modules/..."
    if line.contains("$basedir") {
        // Extract the relative path after $basedir
        if let Ok(re) = Regex::new(r#"\$basedir/(node_modules/@anthropic-ai/claude-code/cli\.js)"#)
        {
            if let Some(caps) = re.captures(line) {
                let relative_path = caps.get(1)?.as_str();
                // Get the directory containing the wrapper script
                let wrapper_dir = wrapper_path.parent()?;
                return Some(wrapper_dir.join(relative_path));
            }
        }
    }

    // Format 2: "$(dirname "$0")/node_modules/..." (common in shell scripts)
    if line.contains("dirname") || line.contains("$0") {
        let wrapper_dir = wrapper_path.parent()?;
        return Some(
            wrapper_dir
                .join("node_modules/@anthropic-ai/claude-code/cli.js")
                .clone(),
        );
    }

    // Format 3: Direct path (absolute or with %APPDATA% on Windows)
    if line.contains("cli.js") {
        // Try to extract a quoted path
        if let Ok(re) =
            Regex::new(r#"["']([^"']*node_modules/@anthropic-ai/claude-code/cli\.js)["']"#)
        {
            if let Some(caps) = re.captures(line) {
                let path_str = caps.get(1)?.as_str();
                // Handle Windows environment variables like %APPDATA%
                let expanded = expand_env_vars(path_str);
                return Some(PathBuf::from(expanded));
            }
        }
    }

    // Format 4: Windows cmd format with %~dp0
    if line.contains("%~dp0") {
        let wrapper_dir = wrapper_path.parent()?;
        let relative = line
            .replace("%~dp0", "")
            .replace("\\", "/")
            .trim()
            .trim_matches('"')
            .to_string();
        return Some(wrapper_dir.join(relative));
    }

    None
}

/// Expand environment variables in a path string
fn expand_env_vars(s: &str) -> String {
    let mut result = s.to_string();

    // Handle Windows-style %VAR% syntax
    if let Ok(re) = Regex::new(r"%([^%]+)%") {
        while let Some(caps) = re.captures(&result) {
            let var_name = &caps[1];
            if let Ok(value) = env::var(var_name) {
                result = result.replace(&caps[0], &value);
            } else {
                break;
            }
        }
    }

    // Handle Unix-style $VAR and ${VAR} syntax
    if let Ok(re) = Regex::new(r"\$\{?([A-Za-z_][A-Za-z0-9_]*)\}?") {
        while let Some(caps) = re.captures(&result) {
            let var_name = &caps[1];
            if let Ok(value) = env::var(var_name) {
                result = result.replace(&caps[0], &value);
            } else {
                break;
            }
        }
    }

    result
}

/// Find common CLI installation paths based on the wrapper script location
fn find_common_cli_paths(wrapper_path: &PathBuf) -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(wrapper_dir) = wrapper_path.parent() {
        // pnpm-style installation
        paths.push(
            wrapper_dir
                .join("node_modules/@anthropic-ai/claude-code/cli.js")
                .clone(),
        );

        // Try going up one directory level
        if let Some(parent_dir) = wrapper_dir.parent() {
            paths.push(
                parent_dir
                    .join("node_modules/@anthropic-ai/claude-code/cli.js")
                    .clone(),
            );
        }
    }

    // Global npm installation paths
    if let Ok(npm_prefix) = env::var("npm_prefix") {
        paths.push(
            PathBuf::from(npm_prefix)
                .join("node_modules/@anthropic-ai/claude-code/cli.js")
                .clone(),
        );
    }

    // Common global installation locations
    let common_global_paths = vec![
        "/usr/local/lib/node_modules/@anthropic-ai/claude-code/cli.js",
        "/usr/lib/node_modules/@anthropic-ai/claude-code/cli.js",
        "/opt/homebrew/lib/node_modules/@anthropic-ai/claude-code/cli.js",
    ];

    for path in common_global_paths {
        paths.push(PathBuf::from(path));
    }

    // Windows AppData paths
    if cfg!(windows) {
        if let Ok(appdata) = env::var("APPDATA") {
            paths.push(
                PathBuf::from(appdata)
                    .join("npm/node_modules/@anthropic-ai/claude-code/cli.js")
                    .clone(),
            );
        }
        if let Ok(localappdata) = env::var("LOCALAPPDATA") {
            paths.push(
                PathBuf::from(localappdata)
                    .join("npm/node_modules/@anthropic-ai/claude-code/cli.js")
                    .clone(),
            );
        }
    }

    paths
}

/// Patch the Claude CLI to use a custom API host
///
/// This function modifies the cli.js file to replace "api.anthropic.com"
/// with the specified host (from API_HOST environment variable or default).
///
/// # Arguments
/// * `dry_run` - If true, only check what would be changed without modifying
///
/// # Returns
/// * `Ok(true)` if changes were made (or would be made in dry_run mode)
/// * `Ok(false)` if no changes were needed
/// * `Err` if an error occurred
pub fn patch_claude_cli_host(dry_run: bool) -> Result<bool> {
    let host = env::var("API_HOST").unwrap_or_else(|_| DEFAULT_API_HOST.to_string());

    // Find and resolve the CLI path
    let claude_exe = find_claude_cli()?;
    let cli_path = resolve_cli_path(&claude_exe)?;

    // Read the file content
    let content = fs::read_to_string(&cli_path)
        .with_context(|| format!("Failed to read CLI file at {:?}", cli_path))?;

    // Check if the file contains the target pattern
    let original_host = "api.anthropic.com";

    if !content.contains(original_host) {
        // File doesn't contain the expected pattern
        // This might mean it's already patched or a different version
        return Ok(false);
    }

    if dry_run {
        return Ok(true);
    }

    // Replace the host
    let new_content = content.replace(original_host, &host);

    // Write back to file
    fs::write(&cli_path, new_content)
        .with_context(|| format!("Failed to write CLI file at {:?}", cli_path))?;

    Ok(true)
}

/// Patch the Claude CLI with a specific host (not from environment variable)
///
/// This is useful when you want to explicitly set a host rather than
/// reading from the API_HOST environment variable.
///
/// Returns:
/// - Ok(true) if patching was successful
/// - Ok(false) if no patching was needed (already patched)
/// - Err if an error occurred (with friendly error message)
pub fn patch_claude_cli_with_host(host: &str, dry_run: bool) -> Result<bool> {
    // Find and resolve the CLI path
    let claude_exe = match find_claude_cli() {
        Ok(path) => path,
        Err(_) => {
            // Return a friendly error that won't break the workflow
            return Err(anyhow!(ClaudeCliError::NotFound));
        }
    };

    let cli_path = match resolve_cli_path(&claude_exe) {
        Ok(path) => path,
        Err(_) => {
            // Return a friendly error
            return Err(anyhow!(
                "Could not resolve Claude CLI path at {:?}. Host patching skipped.",
                claude_exe
            ));
        }
    };

    // Read the file content
    let content = match fs::read_to_string(&cli_path) {
        Ok(content) => content,
        Err(_) => {
            return Err(anyhow!(
                "Could not read Claude CLI file at {:?}. Host patching skipped.",
                cli_path
            ));
        }
    };

    // Pattern to match: we need to find the current host and replace it
    // This is more flexible - it can replace any previously patched host
    let original_host = "api.anthropic.com";

    // Check what's currently in the file
    let needs_patching =
        content.contains(original_host) || content.contains(&format!("\"{}", host));

    if !needs_patching {
        return Ok(false);
    }

    // If the file still has the original host, replace it
    let new_content = if content.contains(original_host) {
        content.replace(original_host, host)
    } else {
        // Already patched, no need to change
        return Ok(false);
    };

    if dry_run {
        return Ok(true);
    }

    // Write back to file
    match fs::write(&cli_path, new_content) {
        Ok(_) => Ok(true),
        Err(_) => Err(anyhow!(
            "Could not write to Claude CLI file at {:?}. Host patching skipped.",
            cli_path
        )),
    }
}

/// Check if the Claude CLI needs to be patched
///
/// Returns the path to the CLI file and whether it needs patching
pub fn check_cli_needs_patching() -> Result<(PathBuf, bool)> {
    let claude_exe = find_claude_cli()?;
    let cli_path = resolve_cli_path(&claude_exe)?;

    let content = fs::read_to_string(&cli_path)
        .with_context(|| format!("Failed to read CLI file at {:?}", cli_path))?;

    let needs_patching = content.contains("api.anthropic.com");

    Ok((cli_path, needs_patching))
}

/// Get the current API host from the CLI file (if patched) or None (if not patched)
pub fn get_current_cli_host() -> Result<Option<String>> {
    let claude_exe = find_claude_cli()?;
    let cli_path = resolve_cli_path(&claude_exe)?;

    let content = fs::read_to_string(&cli_path)
        .with_context(|| format!("Failed to read CLI file at {:?}", cli_path))?;

    // Look for the host pattern in the file
    // The pattern typically looks like: "https://some-host.com" or just "some-host.com"
    if let Ok(re) = Regex::new(r#"https?://([^"'\s]+)"#) {
        // Find all host references
        for cap in re.captures_iter(&content) {
            let host = &cap[1];
            // Skip if it's the original host
            if host != "api.anthropic.com" && host.contains(".") {
                return Ok(Some(host.to_string()));
            }
        }
    }

    // If the file still contains the original host, it's not patched
    if content.contains("api.anthropic.com") {
        return Ok(None);
    }

    // Couldn't determine the current host
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_claude_cli_error_display() {
        let error = ClaudeCliError::NotFound;
        assert!(error.to_string().contains("not found"));
        assert!(error.to_string().contains("skipped"));

        let error = ClaudeCliError::PathResolutionError("test error".to_string());
        assert!(error.to_string().contains("test error"));
    }

    #[test]
    fn test_extract_path_from_line_basedir() {
        let line = r#"node "$basedir/node_modules/@anthropic-ai/claude-code/cli.js" "$@""#;
        let wrapper_path = PathBuf::from("/usr/local/bin/claude");
        let result = extract_path_from_line(line, &wrapper_path);

        assert!(result.is_some());
        let path = result.unwrap();
        assert!(
            path.to_str()
                .unwrap()
                .contains("node_modules/@anthropic-ai/claude-code/cli.js")
        );
    }

    #[test]
    fn test_extract_path_from_line_absolute() {
        let line = r#"node "/some/path/node_modules/@anthropic-ai/claude-code/cli.js" "$@""#;
        let wrapper_path = PathBuf::from("/usr/local/bin/claude");
        let result = extract_path_from_line(line, &wrapper_path);

        assert!(result.is_some());
        let path = result.unwrap();
        assert_eq!(
            path,
            PathBuf::from("/some/path/node_modules/@anthropic-ai/claude-code/cli.js")
        );
    }

    #[test]
    fn test_which_in_path_with_nonexistent_command() {
        let result = which_in_path("nonexistent_command_12345");
        assert!(result.is_none());
    }

    #[test]
    fn test_find_common_cli_paths() {
        let wrapper_path = PathBuf::from("/usr/local/bin/claude");
        let paths = find_common_cli_paths(&wrapper_path);

        // Should include at least some common paths
        assert!(!paths.is_empty());

        // Check that paths contain the expected cli.js
        for path in &paths {
            assert!(
                path.to_str()
                    .unwrap()
                    .contains("node_modules/@anthropic-ai/claude-code/cli.js")
            );
        }
    }
}
