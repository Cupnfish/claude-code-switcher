use anyhow::{Result, anyhow};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::Configurable;
use crate::snapshots::SnapshotScope;
use crate::templates::TemplateType;

/// Main Claude Code settings structure
#[derive(Debug, Clone, Serialize, PartialEq, Default)]
pub struct ClaudeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attribution: Option<Attribution>,
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
    #[serde(rename = "effortLevel", skip_serializing_if = "Option::is_none")]
    pub effort_level: Option<String>,
}

impl<'de> Deserialize<'de> for ClaudeSettings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        struct ClaudeSettingsRaw {
            #[serde(default, deserialize_with = "deserialize_env_opt")]
            env: Option<HashMap<String, String>>,
            #[serde(default)]
            model: Option<String>,
            #[serde(default)]
            output_style: Option<String>,
            #[serde(default)]
            attribution: Option<Attribution>,
            #[serde(default)]
            permissions: Option<Permissions>,
            #[serde(default)]
            hooks: Option<Hooks>,
            #[serde(default)]
            api_key_helper: Option<String>,
            #[serde(default)]
            cleanup_period_days: Option<u32>,
            #[serde(default)]
            disable_all_hooks: Option<bool>,
            #[serde(default)]
            force_login_method: Option<String>,
            #[serde(default)]
            force_login_org_uuid: Option<String>,
            #[serde(default)]
            enable_all_project_mcp_servers: Option<bool>,
            #[serde(default)]
            enabled_mcpjson_servers: Option<Vec<String>>,
            #[serde(default)]
            disabled_mcpjson_servers: Option<Vec<String>>,
            #[serde(default)]
            aws_auth_refresh: Option<String>,
            #[serde(default)]
            aws_credential_export: Option<String>,
            #[serde(default)]
            status_line: Option<StatusLine>,
            #[serde(default)]
            subagent_model: Option<String>,
            #[serde(default, rename = "effortLevel")]
            effort_level: Option<String>,
        }

        let raw = ClaudeSettingsRaw::deserialize(deserializer)?;
        Ok(ClaudeSettings {
            env: raw.env,
            model: raw.model,
            output_style: raw.output_style,
            attribution: raw.attribution,
            permissions: raw.permissions,
            hooks: raw.hooks,
            api_key_helper: raw.api_key_helper,
            cleanup_period_days: raw.cleanup_period_days,
            disable_all_hooks: raw.disable_all_hooks,
            force_login_method: raw.force_login_method,
            force_login_org_uuid: raw.force_login_org_uuid,
            enable_all_project_mcp_servers: raw.enable_all_project_mcp_servers,
            enabled_mcpjson_servers: raw.enabled_mcpjson_servers,
            disabled_mcpjson_servers: raw.disabled_mcpjson_servers,
            aws_auth_refresh: raw.aws_auth_refresh,
            aws_credential_export: raw.aws_credential_export,
            status_line: raw.status_line,
            subagent_model: raw.subagent_model,
            effort_level: raw.effort_level,
        })
    }
}

/// Custom deserializer for optional env field that accepts mixed types
fn deserialize_env_opt<'de, D>(deserializer: D) -> Result<Option<HashMap<String, String>>, D::Error>
where
    D: Deserializer<'de>,
{
    let raw_opt: Option<Value> = Option::deserialize(deserializer)?;

    match raw_opt {
        Some(Value::Object(map)) => {
            let mut env_map = HashMap::new();
            for (key, value) in map.into_iter() {
                let string_value = match value {
                    Value::String(s) => s,
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    Value::Null => String::new(),
                    _ => serde_json::to_string(&value).unwrap_or_default(),
                };
                env_map.insert(key, string_value);
            }
            Ok(Some(env_map))
        }
        Some(_) => Err(serde::de::Error::custom("env field must be an object")),
        None => Ok(None),
    }
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

/// Attribution configuration for git commits and pull requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct Attribution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub commit: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pr: Option<String>,
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

    /// Merge `template` settings into `existing`, bounded by `scope`.
    ///
    /// The template wins for the fields its scope owns; `existing` fills the
    /// rest. `env` is a **key-level union** (template keys override, existing
    /// keys the template doesn't set are preserved), so switching providers
    /// never silently drops the user's unrelated env vars.
    ///
    /// Because a template apply is a *provider switch*, the small set of
    /// provider/model routing keys (endpoints, credentials, and every model
    /// selector such as `ANTHROPIC_MODEL` / `CLAUDE_CODE_SUBAGENT_MODEL`) is
    /// **reset** before the union: a routing key the new template leaves unset
    /// is dropped rather than inherited from the previous provider. Without
    /// this, switching away from a template that sets
    /// `CLAUDE_CODE_SUBAGENT_MODEL` (e.g. Qianwen) to one that doesn't
    /// (e.g. GLM) would leave the stale subagent model pointing at the old
    /// provider.
    ///
    /// This is deliberately separate from `Configurable::merge_with`, whose
    /// scalar-field priority is inconsistent with its env priority
    /// (`settings.rs` `merge_with`).
    pub fn merge_by_scope(existing: Self, template: Self, scope: &SnapshotScope) -> Self {
        let mut merged = existing;

        // A template apply is a provider switch: drop stale provider/model
        // routing keys the new template doesn't own, so leftovers from the
        // previous provider (e.g. CLAUDE_CODE_SUBAGENT_MODEL) don't bleed
        // through. Keys outside this set (the user's custom env) are still
        // preserved by the union below.
        strip_provider_model_env(&mut merged.env);
        // env is always a key-level union (template overrides per key)
        merged.env = merge_hashmaps(template.env.clone(), merged.env);

        // In-scope scalar/struct fields: template value replaces when set.
        match scope {
            SnapshotScope::Env => {
                // env handled above; nothing else owned
            }
            SnapshotScope::Common => {
                merged.model = template.model.or(merged.model);
                merged.output_style = template.output_style.or(merged.output_style);
                merged.attribution = template.attribution.or(merged.attribution);
                merged.permissions = template.permissions.or(merged.permissions);
                merged.hooks = template.hooks.or(merged.hooks);
                merged.status_line = template.status_line.or(merged.status_line);
                merged.subagent_model = template.subagent_model.or(merged.subagent_model);
                merged.effort_level = template.effort_level.or(merged.effort_level);
            }
            SnapshotScope::All => {
                // template owns every field
                merged.model = template.model.or(merged.model);
                merged.output_style = template.output_style.or(merged.output_style);
                merged.attribution = template.attribution.or(merged.attribution);
                merged.permissions = template.permissions.or(merged.permissions);
                merged.hooks = template.hooks.or(merged.hooks);
                merged.status_line = template.status_line.or(merged.status_line);
                merged.subagent_model = template.subagent_model.or(merged.subagent_model);
                merged.effort_level = template.effort_level.or(merged.effort_level);
                merged.api_key_helper = template.api_key_helper.or(merged.api_key_helper);
                merged.cleanup_period_days =
                    template.cleanup_period_days.or(merged.cleanup_period_days);
                merged.disable_all_hooks = template.disable_all_hooks.or(merged.disable_all_hooks);
                merged.force_login_method =
                    template.force_login_method.or(merged.force_login_method);
                merged.force_login_org_uuid = template
                    .force_login_org_uuid
                    .or(merged.force_login_org_uuid);
                merged.enable_all_project_mcp_servers = template
                    .enable_all_project_mcp_servers
                    .or(merged.enable_all_project_mcp_servers);
                merged.enabled_mcpjson_servers = template
                    .enabled_mcpjson_servers
                    .or(merged.enabled_mcpjson_servers);
                merged.disabled_mcpjson_servers = template
                    .disabled_mcpjson_servers
                    .or(merged.disabled_mcpjson_servers);
                merged.aws_auth_refresh = template.aws_auth_refresh.or(merged.aws_auth_refresh);
                merged.aws_credential_export = template
                    .aws_credential_export
                    .or(merged.aws_credential_export);
            }
        }

        merged
    }

    /// Apply a per-tier model override on top of the template's settings.
    ///
    /// Only tiers set to `Some` are rewritten; tiers left `None` keep the
    /// template's value. Must be called on the template-output settings before
    /// [`ClaudeSettings::merge_by_scope`] so the override wins the env union.
    pub fn apply_model_override(&mut self, model_override: &ModelOverride) {
        for tier in ModelTier::ALL {
            if let Some(value) = model_override.get(tier) {
                self.env
                    .get_or_insert_with(HashMap::new)
                    .insert(tier.env_key().to_string(), value.to_string());
                if tier == ModelTier::Primary {
                    self.model = Some(value.to_string());
                }
            }
        }
    }
}

/// A Claude Code model tier that can be overridden independently at apply time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ModelTier {
    Primary,
    Opus,
    Sonnet,
    Haiku,
}

impl ModelTier {
    /// All tiers in display order.
    pub const ALL: [ModelTier; 4] = [Self::Primary, Self::Opus, Self::Sonnet, Self::Haiku];

    pub fn label(self) -> &'static str {
        match self {
            Self::Primary => "Primary",
            Self::Opus => "Opus",
            Self::Sonnet => "Sonnet",
            Self::Haiku => "Haiku",
        }
    }

    /// Env key this tier rewrites. `Primary` additionally rewrites the
    /// top-level `model` field (handled in [`ClaudeSettings::apply_model_override`]).
    pub fn env_key(self) -> &'static str {
        match self {
            Self::Primary => "ANTHROPIC_MODEL",
            Self::Opus => "ANTHROPIC_DEFAULT_OPUS_MODEL",
            Self::Sonnet => "ANTHROPIC_DEFAULT_SONNET_MODEL",
            Self::Haiku => "ANTHROPIC_DEFAULT_HAIKU_MODEL",
        }
    }

    pub fn as_index(self) -> usize {
        Self::ALL.iter().position(|&t| t == self).unwrap()
    }
}

/// Per-tier model override applied on top of a template's settings.
///
/// Each tier is `None` (keep the template's value) or `Some(model)`. An
/// all-`None` value means "no override". Serialized into `ccs-prefs.json`
/// under `last_model_override`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opus: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sonnet: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub haiku: Option<String>,
}

impl ModelOverride {
    /// Returns the override for `tier`, if any.
    pub fn get(&self, tier: ModelTier) -> Option<&str> {
        match tier {
            ModelTier::Primary => self.primary.as_deref(),
            ModelTier::Opus => self.opus.as_deref(),
            ModelTier::Sonnet => self.sonnet.as_deref(),
            ModelTier::Haiku => self.haiku.as_deref(),
        }
    }

    /// Sets the override for `tier` (`None` clears it).
    pub fn set(&mut self, tier: ModelTier, value: Option<String>) {
        match tier {
            ModelTier::Primary => self.primary = value,
            ModelTier::Opus => self.opus = value,
            ModelTier::Sonnet => self.sonnet = value,
            ModelTier::Haiku => self.haiku = value,
        }
    }

    /// True when no tier is overridden.
    pub fn is_empty(&self) -> bool {
        self.primary.is_none()
            && self.opus.is_none()
            && self.sonnet.is_none()
            && self.haiku.is_none()
    }

    /// Override every tier with the same model. This is what `--model X` maps
    /// to, so the flag actually takes effect (it previously only set the
    /// top-level `model` field, which Claude Code ignores in favour of the
    /// `ANTHROPIC_*` env keys).
    pub fn uniform(model: impl Into<String>) -> Self {
        let model = model.into();
        Self {
            primary: Some(model.clone()),
            opus: Some(model.clone()),
            sonnet: Some(model.clone()),
            haiku: Some(model),
        }
    }

    /// Compact human-readable summary: `None` when empty, `all:<model>` when
    /// every set tier shares one value, `Tier:value · …` otherwise. Shared by
    /// the TUI Model row and the CLI apply summary.
    pub fn summary(&self) -> Option<String> {
        let set: Vec<(&str, &str)> = ModelTier::ALL
            .iter()
            .filter_map(|&tier| self.get(tier).map(|v| (tier.label(), v)))
            .collect();
        let first_value = set.first().map(|(_, v)| *v)?;
        if set.len() == ModelTier::ALL.len() && set.iter().all(|&(_, v)| v == first_value) {
            // Fully uniform (the `--model` shape) reads best without tier noise.
            return Some(format!("all:{first_value}"));
        }
        Some(
            set.iter()
                .map(|(label, v)| format!("{label}:{v}"))
                .collect::<Vec<_>>()
                .join(" · "),
        )
    }
}

impl crate::Configurable for ClaudeSettings {
    fn merge_with(self, other: Self) -> Self {
        // Merge in priority order: self (higher priority) overrides other (lower priority)
        ClaudeSettings {
            env: merge_hashmaps(self.env, other.env),
            model: other.model.or(self.model),
            output_style: other.output_style.or(self.output_style),
            attribution: merge_attribution(self.attribution, other.attribution),
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
            effort_level: other.effort_level.or(self.effort_level),
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
                attribution: self.attribution,
                permissions: self.permissions,
                hooks: self.hooks,
                status_line: self.status_line,
                subagent_model: self.subagent_model,
                effort_level: self.effort_level,
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

/// Env keys that select which provider/model Claude Code talks to.
///
/// Applying a template is a provider switch, so [`ClaudeSettings::merge_by_scope`]
/// resets these from the existing settings before unioning the template's env.
/// A key here that the new template leaves unset is dropped rather than
/// inherited — otherwise the previous provider's model selectors (most visibly
/// `CLAUDE_CODE_SUBAGENT_MODEL`) would silently survive the switch.
const PROVIDER_MODEL_ENV_KEYS: &[&str] = &[
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "ANTHROPIC_REASONING_MODEL",
    "CLAUDE_CODE_SUBAGENT_MODEL",
];

/// Remove provider/model routing keys from `env`, in place.
fn strip_provider_model_env(env: &mut Option<HashMap<String, String>>) {
    if let Some(map) = env {
        for key in PROVIDER_MODEL_ENV_KEYS {
            map.remove(*key);
        }
    }
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

/// Helper function to merge attribution settings
fn merge_attribution(base: Option<Attribution>, other: Option<Attribution>) -> Option<Attribution> {
    match (base, other) {
        (Some(base_attr), Some(other_attr)) => Some(Attribution {
            commit: other_attr.commit.or(base_attr.commit),
            pr: other_attr.pr.or(base_attr.pr),
        }),
        (Some(base_attr), None) => Some(base_attr),
        (None, Some(other_attr)) => Some(other_attr),
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json;

    #[test]
    fn test_deserialize_env_with_integer_values() {
        let json_with_int = r#"{
            "env": {
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": 1,
                "MAX_OUTPUT_TOKENS": "96000",
                "ENABLE_THINKING": true,
                "NULL_VALUE": null
            }
        }"#;

        let settings: ClaudeSettings =
            serde_json::from_str(json_with_int).expect("Should deserialize successfully");

        let env = settings.env.expect("Should have env map");
        assert_eq!(
            env.get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
            Some(&"1".to_string())
        );
        assert_eq!(env.get("MAX_OUTPUT_TOKENS"), Some(&"96000".to_string()));
        assert_eq!(env.get("ENABLE_THINKING"), Some(&"true".to_string()));
        assert_eq!(env.get("NULL_VALUE"), Some(&"".to_string()));
    }

    #[test]
    fn test_deserialize_env_string_values() {
        let json_string_only = r#"{
            "env": {
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC": "1",
                "MAX_OUTPUT_TOKENS": "96000"
            }
        }"#;

        let settings: ClaudeSettings =
            serde_json::from_str(json_string_only).expect("Should deserialize successfully");

        let env = settings.env.expect("Should have env map");
        assert_eq!(
            env.get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC"),
            Some(&"1".to_string())
        );
        assert_eq!(env.get("MAX_OUTPUT_TOKENS"), Some(&"96000".to_string()));
    }

    #[test]
    fn test_deserialize_env_no_env() {
        let json_no_env = r#"{
            "model": "test-model"
        }"#;

        let settings: ClaudeSettings =
            serde_json::from_str(json_no_env).expect("Should deserialize successfully");

        assert_eq!(settings.env, None);
    }

    #[test]
    fn test_merge_by_scope_env_unions_and_preserves_unrelated() {
        use crate::snapshots::SnapshotScope;

        let mut existing_env = HashMap::new();
        existing_env.insert("ANTHROPIC_BASE_URL".to_string(), "old".to_string());
        existing_env.insert("MY_CUSTOM_VAR".to_string(), "keep".to_string());
        let existing = ClaudeSettings {
            env: Some(existing_env),
            ..Default::default()
        };

        let mut template_env = HashMap::new();
        template_env.insert("ANTHROPIC_BASE_URL".to_string(), "new".to_string());
        template_env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), "tok".to_string());
        let template = ClaudeSettings {
            env: Some(template_env),
            ..Default::default()
        };

        let merged = ClaudeSettings::merge_by_scope(existing, template, &SnapshotScope::Common);
        let env = merged.env.expect("merged env should exist");
        // template overrides conflicting key
        assert_eq!(env.get("ANTHROPIC_BASE_URL"), Some(&"new".to_string()));
        // template adds its own keys
        assert_eq!(env.get("ANTHROPIC_AUTH_TOKEN"), Some(&"tok".to_string()));
        // existing unrelated key preserved (the data-loss fix)
        assert_eq!(env.get("MY_CUSTOM_VAR"), Some(&"keep".to_string()));
    }

    #[test]
    fn test_merge_by_scope_drops_stale_provider_model_env() {
        use crate::snapshots::SnapshotScope;

        // Existing settings came from a template that set the subagent model
        // (e.g. Qianwen setting CLAUDE_CODE_SUBAGENT_MODEL).
        let mut existing_env = HashMap::new();
        existing_env.insert(
            "CLAUDE_CODE_SUBAGENT_MODEL".to_string(),
            "qwen3.8-max-preview".to_string(),
        );
        existing_env.insert(
            "ANTHROPIC_MODEL".to_string(),
            "qwen3.8-max-preview".to_string(),
        );
        existing_env.insert("MY_CUSTOM_VAR".to_string(), "keep".to_string());
        let existing = ClaudeSettings {
            env: Some(existing_env),
            ..Default::default()
        };

        // New template (e.g. GLM) does NOT set CLAUDE_CODE_SUBAGENT_MODEL.
        let mut template_env = HashMap::new();
        template_env.insert("ANTHROPIC_MODEL".to_string(), "glm-5.3[1m]".to_string());
        let template = ClaudeSettings {
            env: Some(template_env),
            ..Default::default()
        };

        let merged = ClaudeSettings::merge_by_scope(existing, template, &SnapshotScope::Common);
        let env = merged.env.expect("merged env should exist");
        // stale subagent model from the previous provider is dropped
        assert!(!env.contains_key("CLAUDE_CODE_SUBAGENT_MODEL"));
        // template's own routing key wins
        assert_eq!(env.get("ANTHROPIC_MODEL"), Some(&"glm-5.3[1m]".to_string()));
        // unrelated user env still preserved
        assert_eq!(env.get("MY_CUSTOM_VAR"), Some(&"keep".to_string()));
    }

    #[test]
    fn test_merge_by_scope_common_preserves_out_of_scope_fields() {
        use crate::snapshots::SnapshotScope;

        let existing = ClaudeSettings {
            model: Some("old-model".to_string()),
            enabled_mcpjson_servers: Some(vec!["my-mcp".to_string()]),
            ..Default::default()
        };
        let template = ClaudeSettings {
            model: Some("new-model".to_string()),
            ..Default::default()
        };

        let merged = ClaudeSettings::merge_by_scope(existing, template, &SnapshotScope::Common);
        assert_eq!(merged.model, Some("new-model".to_string())); // template wins (in scope)
        // out-of-scope mcp config preserved
        assert_eq!(
            merged.enabled_mcpjson_servers,
            Some(vec!["my-mcp".to_string()])
        );
    }

    #[test]
    fn test_merge_by_scope_env_scope_only_touches_env() {
        use crate::snapshots::SnapshotScope;

        let existing = ClaudeSettings {
            model: Some("keep-model".to_string()),
            permissions: Some(Permissions {
                allow: Some(vec!["Read".to_string()]),
                ask: None,
                deny: None,
                additional_directories: None,
                default_mode: None,
                disable_bypass_permissions_mode: None,
            }),
            ..Default::default()
        };
        let mut template_env = HashMap::new();
        template_env.insert("X".to_string(), "1".to_string());
        let template = ClaudeSettings {
            env: Some(template_env),
            model: Some("template-model".to_string()),
            ..Default::default()
        };

        let merged = ClaudeSettings::merge_by_scope(existing, template, &SnapshotScope::Env);
        assert_eq!(merged.env.unwrap().get("X"), Some(&"1".to_string())); // env applied
        assert_eq!(merged.model, Some("keep-model".to_string())); // model untouched
        assert!(merged.permissions.is_some()); // permissions untouched
    }

    #[test]
    fn test_merge_by_scope_template_none_keeps_existing() {
        use crate::snapshots::SnapshotScope;

        let existing = ClaudeSettings {
            model: Some("keep".to_string()),
            ..Default::default()
        };
        let template = ClaudeSettings {
            model: None,
            ..Default::default()
        };

        let merged = ClaudeSettings::merge_by_scope(existing, template, &SnapshotScope::Common);
        assert_eq!(merged.model, Some("keep".to_string()));
    }

    #[test]
    fn apply_model_override_primary_writes_model_and_env() {
        let mut settings = ClaudeSettings {
            env: Some(HashMap::new()),
            ..Default::default()
        };
        let ov = ModelOverride {
            primary: Some("claude-fable-5[1m]".to_string()),
            ..Default::default()
        };
        settings.apply_model_override(&ov);
        assert_eq!(settings.model, Some("claude-fable-5[1m]".to_string()));
        assert_eq!(
            settings.env.as_ref().unwrap().get("ANTHROPIC_MODEL"),
            Some(&"claude-fable-5[1m]".to_string())
        );
    }

    #[test]
    fn apply_model_override_writes_each_tier() {
        let mut settings = ClaudeSettings {
            env: Some(HashMap::new()),
            ..Default::default()
        };
        let ov = ModelOverride {
            primary: Some("p".to_string()),
            opus: Some("o".to_string()),
            sonnet: Some("s".to_string()),
            haiku: Some("h".to_string()),
        };
        settings.apply_model_override(&ov);
        let env = settings.env.as_ref().unwrap();
        assert_eq!(env.get("ANTHROPIC_MODEL"), Some(&"p".to_string()));
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&"o".to_string())
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL"),
            Some(&"s".to_string())
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL"),
            Some(&"h".to_string())
        );
    }

    #[test]
    fn apply_model_override_noops_none_tiers() {
        // Template's own value must survive for tiers left at None.
        let mut env = HashMap::new();
        env.insert(
            "ANTHROPIC_MODEL".to_string(),
            "template-primary".to_string(),
        );
        env.insert(
            "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
            "template-opus".to_string(),
        );
        let mut settings = ClaudeSettings {
            env: Some(env),
            model: Some("template-primary".to_string()),
            ..Default::default()
        };
        // Override only Sonnet; everything else stays.
        let ov = ModelOverride {
            sonnet: Some("sonnet-5".to_string()),
            ..Default::default()
        };
        settings.apply_model_override(&ov);
        let env = settings.env.as_ref().unwrap();
        assert_eq!(
            env.get("ANTHROPIC_MODEL"),
            Some(&"template-primary".to_string())
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&"template-opus".to_string())
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL"),
            Some(&"sonnet-5".to_string())
        );
        assert_eq!(settings.model, Some("template-primary".to_string()));
    }

    #[test]
    fn apply_model_override_handles_env_none() {
        let mut settings = ClaudeSettings::new();
        let ov = ModelOverride {
            opus: Some("opus-5".to_string()),
            ..Default::default()
        };
        settings.apply_model_override(&ov);
        let env = settings.env.expect("env should be initialized");
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL"),
            Some(&"opus-5".to_string())
        );
    }

    #[test]
    fn apply_model_override_empty_is_noop() {
        let mut settings = ClaudeSettings {
            env: Some(HashMap::new()),
            model: Some("keep".to_string()),
            ..Default::default()
        };
        settings.apply_model_override(&ModelOverride::default());
        assert_eq!(settings.model, Some("keep".to_string()));
        assert!(settings.env.as_ref().unwrap().is_empty());
    }

    #[test]
    fn model_override_uniform_sets_all_tiers() {
        let ov = ModelOverride::uniform("claude-opus-5[1m]");
        assert_eq!(ov.get(ModelTier::Primary), Some("claude-opus-5[1m]"));
        assert_eq!(ov.get(ModelTier::Opus), Some("claude-opus-5[1m]"));
        assert_eq!(ov.get(ModelTier::Sonnet), Some("claude-opus-5[1m]"));
        assert_eq!(ov.get(ModelTier::Haiku), Some("claude-opus-5[1m]"));
        assert!(!ov.is_empty());
    }

    #[test]
    fn model_override_summary_collapses_uniform() {
        assert_eq!(ModelOverride::default().summary(), None);
        assert_eq!(
            ModelOverride::uniform("glm-5.3[1m]").summary(),
            Some("all:glm-5.3[1m]".to_string())
        );
        let ov = ModelOverride {
            opus: Some("claude-opus-5[1m]".to_string()),
            sonnet: Some("claude-sonnet-5[1m]".to_string()),
            ..Default::default()
        };
        assert_eq!(
            ov.summary(),
            Some("Opus:claude-opus-5[1m] · Sonnet:claude-sonnet-5[1m]".to_string())
        );
        // A single set tier must not collapse to "all:" — it names the tier.
        let one = ModelOverride {
            haiku: Some("claude-haiku-4-5".to_string()),
            ..Default::default()
        };
        assert_eq!(one.summary(), Some("Haiku:claude-haiku-4-5".to_string()));
    }

    #[test]
    fn model_override_serde_skips_unset_tiers() {
        let ov = ModelOverride {
            sonnet: Some("claude-sonnet-5[1m]".to_string()),
            ..Default::default()
        };
        let json = serde_json::to_string(&ov).unwrap();
        assert!(json.contains("sonnet"));
        assert!(!json.contains("primary"));
        assert!(!json.contains("opus"));
        let back: ModelOverride = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ov);
    }
}
