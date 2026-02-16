//! Fishtrip AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Fishtrip AI provider template
#[derive(Debug, Clone)]
pub struct FishtripTemplate;

impl Template for FishtripTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Fishtrip
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["FISHTRIP_API_KEY", "FISHTRIP_AUTH_TOKEN", "FISHTRIP_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "Fishtrip"
    }

    fn description(&self) -> &'static str {
        "Fishtrip API - Anthropic-compatible endpoint"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("claude-opus-4-6".to_string());

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
                "https://api.fishtrip.net".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "claude-opus-4-6".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "claude-opus-4-6".to_string(),
            );
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

/// Create Fishtrip template settings (legacy compatibility function)
pub fn create_fishtrip_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = FishtripTemplate;
    template.create_settings(api_key, scope)
}
