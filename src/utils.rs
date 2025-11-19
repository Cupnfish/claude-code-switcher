use anyhow::{Result, anyhow};
use console::style;
use std::path::{Path, PathBuf};

use crate::settings::ClaudeSettings;

/// Get the path to the settings file
pub fn get_settings_path(settings_path: Option<PathBuf>) -> PathBuf {
    settings_path.unwrap_or_else(|| {
        // Use current directory by default for project-specific settings
        PathBuf::from(".claude").join("settings.json")
    })
}

/// Get the path to the environment-specific settings file
pub fn get_env_var_path() -> PathBuf {
    PathBuf::from(".claude").join("settings.json")
}

/// Get the snapshots directory
pub fn get_snapshots_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home_dir.join(".claude").join("snapshots")
}

/// Get the credentials directory
pub fn get_credentials_dir() -> PathBuf {
    let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
    home_dir.join(".claude").join("credentials")
}

/// Confirm an action with the user using enhanced selector
pub fn confirm_action(message: &str, default: bool) -> Result<bool> {
    crate::selectors::confirmation::ConfirmationService::confirm_enhanced(message, default)
        .map_err(|e| anyhow::anyhow!("Confirmation failed: {}", e))
}

/// Create a backup of current settings
pub fn backup_settings(settings_path: &Path) -> Result<Option<PathBuf>> {
    if !settings_path.exists() {
        return Ok(None);
    }

    let backup_path = settings_path.with_extension("json.backup");
    std::fs::copy(settings_path, &backup_path)
        .map_err(|e| anyhow!("Failed to create backup: {}", e))?;
    Ok(Some(backup_path))
}

/// Restore settings from backup
pub fn restore_from_backup(settings_path: &Path) -> Result<()> {
    let backup_path = settings_path.with_extension("json.backup");

    if !backup_path.exists() {
        return Err(anyhow!("Backup file not found: {}", backup_path.display()));
    }

    std::fs::copy(&backup_path, settings_path)
        .map_err(|e| anyhow!("Failed to restore from backup: {}", e))?;

    std::fs::remove_file(&backup_path)
        .map_err(|e| anyhow!("Failed to remove backup file: {}", e))?;

    Ok(())
}

/// Get the current working directory's claude settings path
pub fn get_local_settings_path() -> PathBuf {
    PathBuf::from(".claude").join("settings.json")
}

/// Check if we should use local or global settings
pub fn should_use_local_settings() -> bool {
    let local_path = get_local_settings_path();
    if local_path.exists() {
        return true;
    }

    // Check for .claude directory in current working directory
    let local_claude_dir = PathBuf::from(".claude");
    local_claude_dir.exists()
}

/// Format bytes to human readable format
pub fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    if bytes == 0 {
        return "0 B".to_string();
    }

    let mut size = bytes as f64;
    let mut unit_index = 0;

    while size >= 1024.0 && unit_index < UNITS.len() - 1 {
        size /= 1024.0;
        unit_index += 1;
    }

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.1} {}", size, UNITS[unit_index])
    }
}

/// Format duration to human readable format
pub fn format_duration(seconds: i64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m {}s", seconds / 60, seconds % 60)
    } else {
        format!(
            "{}h {}m {}s",
            seconds / 3600,
            (seconds % 3600) / 60,
            seconds % 60
        )
    }
}

/// Truncate text to a maximum length
pub fn truncate_text(text: &str, max_length: usize) -> String {
    if text.len() <= max_length {
        text.to_string()
    } else {
        format!("{}...", &text[..max_length.saturating_sub(3)])
    }
}

/// Get a colored status indicator
pub fn status_indicator(success: bool, message: &str) -> String {
    if success {
        format!("{} {}", style("✓").green().bold(), message)
    } else {
        format!("{} {}", style("✗").red().bold(), message)
    }
}

/// Format a list of items for display
pub fn format_list(items: &[&str], separator: &str) -> String {
    items.join(separator)
}

/// Get the file size of a path
pub fn get_file_size(path: &Path) -> Result<u64> {
    if path.exists() {
        let metadata =
            std::fs::metadata(path).map_err(|e| anyhow!("Failed to get file metadata: {}", e))?;
        Ok(metadata.len())
    } else {
        Ok(0)
    }
}

/// Ensure a directory exists
pub fn ensure_dir_exists(dir: &Path) -> Result<()> {
    if !dir.exists() {
        std::fs::create_dir_all(dir)
            .map_err(|e| anyhow!("Failed to create directory {}: {}", dir.display(), e))?;
    }
    Ok(())
}

/// Check if a string is a valid UUID
pub fn is_valid_uuid(uuid_str: &str) -> bool {
    uuid::Uuid::parse_str(uuid_str).is_ok()
}

/// Get timestamp for display
pub fn get_timestamp() -> String {
    let now = chrono::Utc::now();
    now.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

/// Format settings summary for display
pub fn format_settings_summary(settings: &ClaudeSettings) -> String {
    let mut summary = String::new();

    if let Some(ref model) = settings.model {
        summary.push_str(&format!("Model: {}\n", model));
    }

    if let Some(ref permissions) = settings.permissions {
        if let Some(ref allowed) = permissions.allow
            && allowed.contains(&"network".to_string())
        {
            summary.push_str("Network Access: Allowed\n");
        }
        if let Some(ref denied) = permissions.deny
            && denied.contains(&"network".to_string())
        {
            summary.push_str("Network Access: Denied\n");
        }
        if let Some(ref allowed) = permissions.allow
            && allowed.contains(&"filesystem".to_string())
        {
            summary.push_str("Filesystem Access: Allowed\n");
        }
        if let Some(ref denied) = permissions.deny
            && denied.contains(&"filesystem".to_string())
        {
            summary.push_str("Filesystem Access: Denied\n");
        }
        if let Some(ref denied) = permissions.deny
            && denied.contains(&"command".to_string())
        {
            summary.push_str("Command Execution: Denied\n");
        }
    }

    summary.trim_end().to_string()
}
