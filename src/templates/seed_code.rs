//! Seed Code (Volcengine) AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Seed Code AI provider template
#[derive(Debug, Clone)]
pub struct SeedCodeTemplate;

/// Seed Code (Doubao) model served by the Volcengine endpoint.
pub const SEED_CODE_MODEL_PRESETS: &[&str] = &["doubao-seed-code-preview-latest"];

impl Template for SeedCodeTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::SeedCode
    }

    fn model_presets(&self) -> &'static [&'static str] {
        SEED_CODE_MODEL_PRESETS
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec![
            "ARK_API_KEY",
            "ARK_TOKEN",
            "SEED_CODE_API_KEY",
            "VOLCENGINE_API_KEY",
        ]
    }

    fn display_name(&self) -> &'static str {
        "Seed Code"
    }

    fn description(&self) -> &'static str {
        "Volcengine Seed Code - AI coding assistant"
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://console.volcengine.com/ark/region:ark+cn-beijing/apikey")
    }

    fn api_host(&self) -> Option<&'static str> {
        Some("ark.cn-beijing.volces.com")
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("doubao-seed-code-preview-latest".to_string());

            settings.permissions = Some(Permissions {
                allow: Some(crate::templates::default_allow_list()),
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
            env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://ark.cn-beijing.volces.com/api/coding".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "doubao-seed-code-preview-latest".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                "doubao-seed-code-preview-latest".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "doubao-seed-code-preview-latest".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                "doubao-seed-code-preview-latest".to_string(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create Seed Code template settings (legacy compatibility function)
pub fn create_seed_code_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = SeedCodeTemplate;
    template.create_settings(api_key, scope)
}
