//! BeeAPI AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// BeeAPI AI provider template
#[derive(Debug, Clone)]
pub struct BeeApiTemplate;

impl Template for BeeApiTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::BeeApi
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["BEEAPI_API_KEY", "BEEAPI_AUTH_TOKEN", "BEEAPI_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "BeeAPI"
    }

    fn description(&self) -> &'static str {
        "BeeAPI - Anthropic-compatible endpoint"
    }

    fn api_host(&self) -> Option<&'static str> {
        Some("beeapi.ai")
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("claude-opus-4-8[1m]".to_string());
            settings.effort_level = Some("max".to_string());

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
                "https://beeapi.ai/anthropic".to_string(),
            );
            env.insert("ANTHROPIC_MODEL".to_string(), "claude-opus-4-8[1m]".to_string());
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                "claude-opus-4-8[1m]".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "claude-opus-4-8[1m]".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                "claude-opus-4-8[1m]".to_string(),
            );
            env.insert("CLAUDE_CODE_EFFORT_LEVEL".to_string(), "max".to_string());
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create BeeAPI template settings (legacy compatibility function)
pub fn create_beeapi_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = BeeApiTemplate;
    template.create_settings(api_key, scope)
}
