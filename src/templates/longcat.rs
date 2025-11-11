//! Longcat AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Longcat AI provider template
#[derive(Debug, Clone)]
pub struct LongcatTemplate;

impl Template for LongcatTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Longcat
    }

    fn env_var_name(&self) -> &'static str {
        "LONGCAT_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "Longcat"
    }

    fn description(&self) -> &'static str {
        "Longcat Flash Chat API - Fast and efficient conversational AI"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("LongCat-Flash-Chat".to_string());

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

        if matches!(scope, SnapshotScope::Env | SnapshotScope::All) {
            let mut env = HashMap::new();
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.longcat.chat/anthropic".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "LongCat-Flash-Chat".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "LongCat-Flash-Chat".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "LongCat-Flash-Chat".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_MAX_OUTPUT_TOKENS".to_string(),
                "8192".to_string(),
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

/// Create Longcat template settings (legacy compatibility function)
pub fn create_longcat_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = LongcatTemplate;
    template.create_settings(api_key, scope)
}
