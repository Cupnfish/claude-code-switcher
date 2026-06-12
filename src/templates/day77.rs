//! Day77 AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Day77 AI provider template
#[derive(Debug, Clone)]
pub struct Day77Template;

impl Template for Day77Template {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Day77
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["DAY77_API_KEY", "DAY77_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "Day77"
    }

    fn description(&self) -> &'static str {
        "Day77 API - Kimi K2.7 with 256K context"
    }

    fn api_host(&self) -> Option<&'static str> {
        Some("api.day77.icu")
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("kimi-k2.7-code".to_string());

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
                "https://api.day77.icu".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "kimi-k2.7-code".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                "kimi-k2.7-code".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "kimi-k2.7-code".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                "kimi-k2.7-code".to_string(),
            );
            env.insert(
                "ANTHROPIC_REASONING_MODEL".to_string(),
                "kimi-k2.7-code".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create Day77 template settings (legacy compatibility function)
pub fn create_day77_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = Day77Template;
    template.create_settings(api_key, scope)
}
