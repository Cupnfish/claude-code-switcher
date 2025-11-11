//! Kimi AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// Kimi AI provider template
#[derive(Debug, Clone)]
pub struct KimiTemplate;

impl Template for KimiTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Kimi
    }

    fn env_var_name(&self) -> &'static str {
        "KIMI_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "Kimi"
    }

    fn description(&self) -> &'static str {
        "Kimi For Coding API - Specialized for coding tasks"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "kimi".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "kimi-for-coding".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "kimi".to_string(),
                api_base: "https://api.kimi.com/coding/".to_string(),
                api_key: None,
                endpoint_id: None,
                metadata: None,
            });

            settings.http = Some(HTTPConfig {
                timeout_ms: Some(30000),
                max_retries: Some(3),
                retry_backoff_factor: Some(2.0),
            });

            settings.permissions = Some(Permissions {
                allow_network_access: Some(true),
                allow_filesystem_access: Some(true),
                allow_command_execution: Some(false),
            });
        }

        if matches!(scope, SnapshotScope::Env | SnapshotScope::All) {
            let mut env = HashMap::new();
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.kimi.com/coding/".to_string(),
            );
            env.insert("ANTHROPIC_MODEL".to_string(), "kimi-for-coding".to_string());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "kimi-for-coding".to_string(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.environment = Some(env);
        }

        settings
    }
}

/// Create Kimi template settings (legacy compatibility function)
pub fn create_kimi_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KimiTemplate;
    template.create_settings(api_key, scope)
}
