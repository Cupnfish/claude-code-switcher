use anyhow::{Result, anyhow};
use console::style;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::{Configurable, SnapshotScope, TemplateType};

/// Main Claude Code settings structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ClaudeSettings {
    /// Provider configuration
    pub provider: Option<ProviderConfig>,

    /// Model settings
    pub model: Option<ModelConfig>,

    /// API endpoint configuration
    pub endpoint: Option<EndpointConfig>,

    /// HTTP client settings
    pub http: Option<HTTPConfig>,

    /// Permissions configuration
    pub permissions: Option<Permissions>,

    /// Hook configurations
    pub hooks: Option<Hooks>,

    /// Status line configuration
    pub status_line: Option<StatusLine>,

    /// Environment variables captured at snapshot time
    pub environment: Option<HashMap<String, String>>,
}

/// Provider configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProviderConfig {
    pub id: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Model configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ModelConfig {
    pub name: String,
    pub metadata: Option<HashMap<String, String>>,
}

/// Endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EndpointConfig {
    pub id: String,
    pub api_base: String,
    pub api_key: Option<String>,
    pub endpoint_id: Option<String>,
    pub metadata: Option<HashMap<String, String>>,
}

/// HTTP client configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HTTPConfig {
    pub timeout_ms: Option<u64>,
    pub max_retries: Option<u32>,
    pub retry_backoff_factor: Option<f64>,
}

/// Permissions configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Permissions {
    pub allow_network_access: Option<bool>,
    pub allow_filesystem_access: Option<bool>,
    pub allow_command_execution: Option<bool>,
}

/// Hooks configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Hooks {
    pub on_start: Option<Vec<String>>,
    pub on_save: Option<Vec<String>>,
    pub on_send_message: Option<Vec<String>>,
    pub on_receive_message: Option<Vec<String>>,
}

/// Status line configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatusLine {
    pub enabled: Option<bool>,
    pub format: Option<String>,
    pub style: Option<String>,
}

impl ClaudeSettings {
    /// Create empty settings
    pub fn new() -> Self {
        Self {
            provider: None,
            model: None,
            endpoint: None,
            http: None,
            permissions: None,
            hooks: None,
            status_line: None,
            environment: None,
        }
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

        match template_type {
            TemplateType::DeepSeek => {
                if let Ok(value) = std::env::var("DEEPSEEK_API_KEY") {
                    env.insert("DEEPSEEK_API_KEY".to_string(), value);
                }
            }
            TemplateType::Zai => {
                if let Ok(value) = std::env::var("Z_AI_API_KEY") {
                    env.insert("Z_AI_API_KEY".to_string(), value);
                }
            }
            TemplateType::K2 | TemplateType::K2Thinking => {
                if let Ok(value) = std::env::var("MOONSHOT_API_KEY") {
                    env.insert("MOONSHOT_API_KEY".to_string(), value);
                }
            }
            TemplateType::KatCoder | TemplateType::KatCoderPro | TemplateType::KatCoderAir => {
                if let Ok(value) = std::env::var("KAT_CODER_API_KEY") {
                    env.insert("KAT_CODER_API_KEY".to_string(), value);
                }
            }
            TemplateType::Kimi => {
                if let Ok(value) = std::env::var("KIMI_API_KEY") {
                    env.insert("KIMI_API_KEY".to_string(), value);
                }
            }
            TemplateType::Longcat => {
                if let Ok(value) = std::env::var("LONGCAT_API_KEY") {
                    env.insert("LONGCAT_API_KEY".to_string(), value);
                }
            }
            TemplateType::MiniMax => {
                if let Ok(value) = std::env::var("MINIMAX_API_KEY") {
                    env.insert("MINIMAX_API_KEY".to_string(), value);
                }
            }
        }

        env
    }

    /// Mask API keys in settings for display
    pub fn mask_api_keys(&self) -> Self {
        let mut masked = self.clone();

        if let Some(ref mut endpoint) = masked.endpoint {
            if endpoint.api_key.is_some() {
                endpoint.api_key = Some("••••••••".to_string());
            }
        }

        masked
    }

    /// Get API key from settings or environment
    pub fn get_api_key(&self) -> Option<String> {
        // First try from settings
        if let Some(ref endpoint) = self.endpoint {
            if let Some(ref api_key) = endpoint.api_key {
                return Some(api_key.clone());
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
    fn merge_with(mut self, other: Self) -> Self {
        // Merge in priority order: self (higher priority) overrides other (lower priority)

        if other.provider.is_some() && self.provider.is_none() {
            self.provider = other.provider;
        }

        if other.model.is_some() && self.model.is_none() {
            self.model = other.model;
        }

        if other.endpoint.is_some() && self.endpoint.is_none() {
            self.endpoint = other.endpoint;
        }

        if other.http.is_some() && self.http.is_none() {
            self.http = other.http;
        }

        if other.permissions.is_some() && self.permissions.is_none() {
            self.permissions = other.permissions;
        }

        if other.hooks.is_some() && self.hooks.is_none() {
            self.hooks = other.hooks;
        }

        if other.status_line.is_some() && self.status_line.is_none() {
            self.status_line = other.status_line;
        }

        // Merge environment variables
        if let Some(other_env) = other.environment {
            let mut env = self.environment.unwrap_or_default();
            env.extend(other_env);
            self.environment = Some(env);
        }

        self
    }

    fn filter_by_scope(self, scope: &SnapshotScope) -> Self {
        match scope {
            SnapshotScope::Env => {
                // Only environment variables
                ClaudeSettings {
                    environment: self.environment,
                    ..Default::default()
                }
            }
            SnapshotScope::Common => {
                // Common settings (exclude environment)
                ClaudeSettings {
                    provider: self.provider,
                    model: self.model,
                    endpoint: self.endpoint,
                    http: self.http,
                    permissions: self.permissions,
                    hooks: self.hooks,
                    status_line: self.status_line,
                    environment: None,
                }
            }
            SnapshotScope::All => self, // Include everything
        }
    }

    fn mask_sensitive_data(self) -> Self {
        self.mask_api_keys()
    }
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self::new()
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

/// Get display formatting for settings
pub fn format_settings_for_display(settings: &ClaudeSettings, verbose: bool) -> String {
    let mut output = String::new();

    if verbose {
        output.push_str(&format!("{} Settings\n", style("Current").bold().cyan()));
        output.push_str(&format!(
            "{} {}\n",
            style("Provider:").bold(),
            settings
                .provider
                .as_ref()
                .map(|p| &p.id)
                .unwrap_or(&"None".to_string())
        ));
        output.push_str(&format!(
            "{} {}\n",
            style("Model:").bold(),
            settings
                .model
                .as_ref()
                .map(|m| &m.name)
                .unwrap_or(&"None".to_string())
        ));

        if let Some(ref endpoint) = settings.endpoint {
            output.push_str(&format!(
                "{} {} ({})\n",
                style("Endpoint:").bold(),
                endpoint.id,
                endpoint.api_base
            ));
            if let Some(ref api_key) = endpoint.api_key {
                output.push_str(&format!(
                    "{} {}\n",
                    style("API Key:").bold(),
                    if api_key.len() > 8 {
                        format!("{}••••••••", &api_key[..8])
                    } else {
                        "••••••••".to_string()
                    }
                ));
            }
        }

        if let Some(ref http) = settings.http {
            if let Some(timeout) = http.timeout_ms {
                output.push_str(&format!("{} {}ms\n", style("Timeout:").bold(), timeout));
            }
        }
    } else {
        let provider = settings
            .provider
            .as_ref()
            .map(|p| p.id.as_str())
            .unwrap_or("None");
        let model = settings
            .model
            .as_ref()
            .map(|m| m.name.as_str())
            .unwrap_or("None");

        output.push_str(&format!(
            "{}: {} | {}: {}\n",
            style("Provider").bold(),
            provider,
            style("Model").bold(),
            model
        ));
    }

    output
}
