use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::Configurable;
use crate::snapshots::SnapshotScope;
use crate::templates::TemplateType;

/// Main Claude Code settings structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct ClaudeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_co_authored_by: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_all_hooks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_org_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all_project_mcp_servers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_mcpjson_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_mcpjson_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_auth_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_credential_export: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_line: Option<StatusLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_model: Option<String>,
}

/// Snapshot structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scope: SnapshotScope,
    pub settings: ClaudeSettings,
    pub description: Option<String>,
    #[serde(skip)]
    pub show_api_key: bool,
}

/// Snapshot storage
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SnapshotStore {
    pub snapshots: Vec<Snapshot>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn find_snapshot(&self, name: &str) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.name == name)
    }

    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn delete_snapshot(&mut self, name: &str) -> Result<()> {
        let index = self
            .snapshots
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| anyhow!("Snapshot '{}' not found", name))?;
        self.snapshots.remove(index);
        Ok(())
    }
}

/// Permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_bypass_permissions_mode: Option<String>,
}

/// Hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_command: Option<Vec<String>>,
}

/// Status line configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusLine {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

impl ClaudeSettings {
    /// Create empty settings
    pub fn new() -> Self {
        Self::default()
    }

    /// Read settings from file
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .map_err(|e| anyhow!("Failed to read settings file {}: {}", path.display(), e))?;

        if content.trim().is_empty() {
            return Ok(Self::new());
        }

        serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse settings file {}: {}", path.display(), e))
    }

    /// Write settings to file
    pub fn to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        let parent = path.parent().ok_or_else(|| {
            anyhow!(
                "Settings file path {} has no parent directory",
                path.display()
            )
        })?;

        fs::create_dir_all(parent).map_err(|e| {
            anyhow!(
                "Failed to create settings directory {}: {}",
                parent.display(),
                e
            )
        })?;

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize settings: {}", e))?;

        fs::write(path, content)
            .map_err(|e| anyhow!("Failed to write settings file {}: {}", path.display(), e))
    }

    /// Capture environment variables relevant to Claude Code
    pub fn capture_environment() -> HashMap<String, String> {
        let mut env = HashMap::new();

        // Claude Code specific environment variables
        if let Ok(value) = std::env::var("CLAUDE_CODE_API_KEY") {
            env.insert("CLAUDE_CODE_API_KEY".to_string(), value);
        }
        if let Ok(value) = std::env::var("ANTHROPIC_API_KEY") {
            env.insert("ANTHROPIC_API_KEY".to_string(), value);
        }

        env
    }

    /// Capture environment variables for a specific template type
    pub fn capture_template_environment(template_type: &TemplateType) -> HashMap<String, String> {
        let mut env = HashMap::new();
        
        // Use the template's env_var_names method to get all supported env vars
        let env_var_names = crate::templates::get_env_var_names(template_type);
        
        for env_var_name in env_var_names {
            if let Ok(value) = std::env::var(env_var_name) {
                env.insert(env_var_name.to_string(), value);
            }
        }

        env
    }

    /// Mask API keys in settings for display
    pub fn mask_api_keys(&self) -> Self {
        let mut masked = self.clone();
        if let Some(ref mut env) = masked.env {
            let keys_to_mask: Vec<String> = env
                .keys()
                .filter(|key| {
                    key.contains("API_KEY") || key.contains("AUTH_TOKEN") || key.contains("TOKEN")
                })
                .cloned()
                .collect();

            for key in keys_to_mask {
                if let Some(value) = env.get(&key) {
                    env.insert(key, mask_api_key(value));
                }
            }
        }
        masked
    }

    /// Get API key from settings or environment
    pub fn get_api_key(&self) -> Option<String> {
        // First try from settings
        if let Some(ref env) = self.env {
            if let Some(key) = env.get("ANTHROPIC_API_KEY") {
                return Some(key.clone());
            }
            if let Some(key) = env.get("ANTHROPIC_AUTH_TOKEN") {
                return Some(key.clone());
            }
        }

        // Then try environment variables
        if let Ok(key) = std::env::var("CLAUDE_CODE_API_KEY") {
            return Some(key);
        }
        if let Ok(key) = std::env::var("ANTHROPIC_API_KEY") {
            return Some(key);
        }

        None
    }
}

impl crate::Configurable for ClaudeSettings {
    fn merge_with(self, other: Self) -> Self {
        // Merge in priority order: self (higher priority) overrides other (lower priority)
        ClaudeSettings {
            env: merge_hashmaps(self.env, other.env),
            model: other.model.or(self.model),
            output_style: other.output_style.or(self.output_style),
            include_co_authored_by: other.include_co_authored_by.or(self.include_co_authored_by),
            permissions: merge_permissions(self.permissions, other.permissions),
            hooks: merge_hooks(self.hooks, other.hooks),
            api_key_helper: other.api_key_helper.or(self.api_key_helper),
            cleanup_period_days: other.cleanup_period_days.or(self.cleanup_period_days),
            disable_all_hooks: other.disable_all_hooks.or(self.disable_all_hooks),
            force_login_method: other.force_login_method.or(self.force_login_method),
            force_login_org_uuid: other.force_login_org_uuid.or(self.force_login_org_uuid),
            enable_all_project_mcp_servers: other
                .enable_all_project_mcp_servers
                .or(self.enable_all_project_mcp_servers),
            enabled_mcpjson_servers: merge_vec(
                self.enabled_mcpjson_servers,
                other.enabled_mcpjson_servers,
            ),
            disabled_mcpjson_servers: merge_vec(
                self.disabled_mcpjson_servers,
                other.disabled_mcpjson_servers,
            ),
            aws_auth_refresh: other.aws_auth_refresh.or(self.aws_auth_refresh),
            aws_credential_export: other.aws_credential_export.or(self.aws_credential_export),
            status_line: other.status_line.or(self.status_line),
            subagent_model: other.subagent_model.or(self.subagent_model),
        }
    }

    fn filter_by_scope(self, scope: &SnapshotScope) -> Self {
        match scope {
            SnapshotScope::Env => ClaudeSettings {
                env: self.env,
                ..Default::default()
            },
            SnapshotScope::All => self,
            SnapshotScope::Common => ClaudeSettings {
                env: self.env,
                model: self.model,
                output_style: self.output_style,
                include_co_authored_by: self.include_co_authored_by,
                permissions: self.permissions,
                hooks: self.hooks,
                status_line: self.status_line,
                subagent_model: self.subagent_model,
                ..Default::default()
            },
        }
    }

    fn mask_sensitive_data(self) -> Self {
        self.mask_api_keys()
    }
}

/// Merge multiple settings with priority
pub fn merge_settings(settings: Vec<ClaudeSettings>) -> ClaudeSettings {
    settings
        .into_iter()
        .fold(ClaudeSettings::new(), |acc, settings| {
            settings.merge_with(acc)
        })
}

/// Helper function to merge hashmaps
/// base_map has higher priority and overrides other_map for conflicting keys
fn merge_hashmaps<K: Clone + Eq + std::hash::Hash, V: Clone>(
    base_map: Option<HashMap<K, V>>,
    other_map: Option<HashMap<K, V>>,
) -> Option<HashMap<K, V>> {
    match (base_map, other_map) {
        (Some(base), Some(other)) => {
            let mut result = other;
            // base_map overrides other_map for conflicting keys
            for (key, value) in base {
                result.insert(key, value);
            }
            Some(result)
        }
        (Some(base_map), None) => Some(base_map),
        (None, Some(other_map)) => Some(other_map),
        (None, None) => None,
    }
}

/// Helper function to merge permissions
fn merge_permissions(
    base: Option<Permissions>,
    override_settings: Option<Permissions>,
) -> Option<Permissions> {
    match (base, override_settings) {
        (Some(base_perms), Some(override_perms)) => Some(Permissions {
            allow: merge_vec(base_perms.allow, override_perms.allow),
            ask: merge_vec(base_perms.ask, override_perms.ask),
            deny: merge_vec(base_perms.deny, override_perms.deny),
            additional_directories: merge_vec(
                base_perms.additional_directories,
                override_perms.additional_directories,
            ),
            default_mode: override_perms.default_mode.or(base_perms.default_mode),
            disable_bypass_permissions_mode: override_perms
                .disable_bypass_permissions_mode
                .or(base_perms.disable_bypass_permissions_mode),
        }),
        (Some(base_perms), None) => Some(base_perms),
        (None, Some(override_perms)) => Some(override_perms),
        (None, None) => None,
    }
}

/// Helper function to merge hooks
fn merge_hooks(base: Option<Hooks>, override_settings: Option<Hooks>) -> Option<Hooks> {
    match (base, override_settings) {
        (Some(base_hooks), Some(override_hooks)) => Some(Hooks {
            pre_command: merge_vec(base_hooks.pre_command, override_hooks.pre_command),
            post_command: merge_vec(base_hooks.post_command, override_hooks.post_command),
        }),
        (Some(base_hooks), None) => Some(base_hooks),
        (None, Some(override_hooks)) => Some(override_hooks),
        (None, None) => None,
    }
}

/// Helper function to merge vectors
fn merge_vec<T: Clone>(base: Option<Vec<T>>, override_settings: Option<Vec<T>>) -> Option<Vec<T>> {
    match (base, override_settings) {
        (Some(mut base_vec), Some(override_vec)) => {
            base_vec.extend(override_vec);
            Some(base_vec)
        }
        (Some(base_vec), None) => Some(base_vec),
        (None, Some(override_vec)) => Some(override_vec),
        (None, None) => None,
    }
}

/// Get display formatting for settings
pub fn format_settings_for_display(settings: &ClaudeSettings, verbose: bool) -> String {
    let mut output = String::new();

    if verbose {
        output.push_str(&format!(
            "{} Settings\n",
            console::style("Current").bold().cyan()
        ));
        output.push_str(&format!(
            "{} {}\n",
            console::style("Provider:").bold(),
            settings.model.as_deref().unwrap_or("None")
        ));
        output.push_str(&format!(
            "{} {}\n",
            console::style("Model:").bold(),
            settings.model.as_deref().unwrap_or("None")
        ));

        if let Some(ref env) = settings.env {
            output.push_str(&format!(
                "{}\n",
                console::style("Environment Variables:").bold()
            ));
            for (key, value) in env {
                let display_value = if key.contains("API_KEY")
                    || key.contains("AUTH_TOKEN")
                    || key.contains("TOKEN")
                    || key.contains("SECRET")
                    || key.contains("PASSWORD")
                    || key.contains("PRIVATE_KEY")
                {
                    mask_api_key(value)
                } else {
                    value.clone()
                };
                output.push_str(&format!("  {} = {}\n", key, display_value));
            }
        }
    } else {
        output.push_str(&format!(
            "{}: {} | {}: {}\n",
            console::style("Provider").bold(),
            "default",
            console::style("Model").bold(),
            settings.model.as_deref().unwrap_or("default")
        ));
    }

    output
}

/// Compare two settings and return a formatted string showing differences
pub fn format_settings_comparison(current: &ClaudeSettings, new: &ClaudeSettings) -> String {
    let current_provider = "default";
    let new_provider = "default";
    let current_model = current.model.as_deref().unwrap_or("default");
    let new_model = new.model.as_deref().unwrap_or("default");

    // Only show comparison if there are differences
    if current_provider == new_provider && current_model == new_model {
        "Settings are identical.".to_string()
    } else {
        let mut output = String::new();

        output.push_str(&format!(
            "{}: {} → {}\n",
            console::style("Provider").bold(),
            current_provider,
            new_provider
        ));

        output.push_str(&format!(
            "{}: {} → {}\n",
            console::style("Model").bold(),
            current_model,
            new_model
        ));

        output
    }
}

/// Mask API key for display
fn mask_api_key(api_key: &str) -> String {
    if let Some(actual_key) = api_key.strip_prefix("sk-") {
        let actual_len = actual_key.len();

        if actual_len <= 6 {
            format!("sk-{}", "*".repeat(actual_len))
        } else if actual_len <= 14 {
            format!(
                "sk-{}***{}",
                &actual_key[..2],
                &actual_key[actual_len - 3..]
            )
        } else {
            format!(
                "sk-{}{}...{} ({} chars)",
                &actual_key[..3],
                "*".repeat(std::cmp::min(actual_len - 7, 8)),
                &actual_key[actual_len - 4..],
                api_key.len()
            )
        }
    } else if api_key.len() <= 8 {
        "*".repeat(api_key.len())
    } else if api_key.len() <= 16 {
        format!("{}***{}", &api_key[..3], &api_key[api_key.len() - 3..])
    } else {
        let visible_start = &api_key[..4];
        let visible_end = &api_key[api_key.len() - 4..];
        let masked_length = api_key.len() - 8;
        format!(
            "{}{}...{} ({} chars)",
            visible_start,
            "*".repeat(std::cmp::min(masked_length, 8)),
            visible_end,
            api_key.len()
        )
    }
}

// Environment field compatibility (for backward compatibility)
impl ClaudeSettings {
    /// Get environment variables (backward compatibility)
    pub fn get_environment(&self) -> Option<&HashMap<String, String>> {
        self.env.as_ref()
    }

    /// Set environment variables (backward compatibility)
    pub fn set_environment(&mut self, env: HashMap<String, String>) {
        self.env = Some(env);
    }
}

impl ClaudeSettings {
    /// Backward compatibility property for environment
    pub fn environment(&self) -> Option<&HashMap<String, String>> {
        self.env.as_ref()
    }
}
