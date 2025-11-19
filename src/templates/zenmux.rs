//! Zenmux AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Zenmux AI provider template
#[derive(Debug, Clone)]
pub struct ZenmuxTemplate;

impl ZenmuxTemplate {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ZenmuxTemplate {
    fn default() -> Self {
        Self::new()
    }
}

impl Template for ZenmuxTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Zenmux
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["ZENMUX_API_KEY", "ZENMUX_AUTH_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "Zenmux"
    }

    fn description(&self) -> &'static str {
        "Zenmux AI - Anthropic-compatible API with multiple model support including Claude and Gemini"
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://zenmux.ai/settings/keys")
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("google/gemini-3-pro-preview-free".to_string());

            // Use the new permissions format
            settings.permissions = Some(Permissions {
                allow: Some(vec![
                    "Bash".to_string(),
                    "Read".to_string(),
                    "Write".to_string(),
                    "Edit".to_string(),
                    "MultiEdit".to_string(),
                    "Glob".to_string(),
                    "Grep".to_string(),
                    "WebFetch".to_string(),
                ]),
                ask: None,
                deny: Some(vec!["WebSearch".to_string()]),
                additional_directories: None,
                default_mode: None,
                disable_bypass_permissions_mode: None,
            });
        }

        if matches!(
            scope,
            SnapshotScope::Env | SnapshotScope::Common | SnapshotScope::All
        ) {
            let mut env = HashMap::new();
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://zenmux.ai/api/anthropic".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "google/gemini-3-pro-preview-free".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "google/gemini-3-pro-preview-free".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create Zenmux template settings (legacy compatibility function)
pub fn create_zenmux_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = ZenmuxTemplate;
    template.create_settings(api_key, scope)
}
