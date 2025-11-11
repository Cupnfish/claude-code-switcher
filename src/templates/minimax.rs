//! MiniMax AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// MiniMax AI provider template
#[derive(Debug, Clone)]
pub struct MiniMaxTemplate;

impl Template for MiniMaxTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::MiniMax
    }

    fn env_var_name(&self) -> &'static str {
        "MINIMAX_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "MiniMax"
    }

    fn description(&self) -> &'static str {
        "MiniMax API (recommended) - High-performance AI with Anthropic compatibility"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "minimax".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "MiniMax-M2".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "minimax".to_string(),
                api_base: "https://api.minimax.io/anthropic".to_string(),
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
            env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.minimax.io/anthropic".to_string(),
            );
            env.insert("ANTHROPIC_MODEL".to_string(), "MiniMax-M2".to_string());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "MiniMax-M2".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "MiniMax-M2".to_string(),
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

/// Create MiniMax template settings (legacy compatibility function)
pub fn create_minimax_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = MiniMaxTemplate;
    template.create_settings(api_key, scope)
}
