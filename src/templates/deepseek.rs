//! DeepSeek AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// DeepSeek AI provider template
#[derive(Debug, Clone)]
pub struct DeepSeekTemplate;

impl Template for DeepSeekTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::DeepSeek
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["DEEPSEEK_API_KEY", "DEEPSEEK_API_TOKEN", "DEEPSEEK_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "DeepSeek"
    }

    fn description(&self) -> &'static str {
        "DeepSeek V4 API - Thinking mode enabled with V4 Pro/Flash models"
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://platform.deepseek.com/api_keys")
    }

    fn api_host(&self) -> Option<&'static str> {
        Some("api.deepseek.com")
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("deepseek-v4-pro".to_string());
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
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.deepseek.com/anthropic".to_string(),
            );
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
            env.insert("ENABLE_THINKING".to_string(), "true".to_string());
            env.insert("ANTHROPIC_MODEL".to_string(), "deepseek-v4-pro".to_string());
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                "deepseek-v4-flash".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "deepseek-v4-flash".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                "deepseek-v4-pro".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create DeepSeek template settings (legacy compatibility function)
pub fn create_deepseek_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = DeepSeekTemplate;
    template.create_settings(api_key, scope)
}
