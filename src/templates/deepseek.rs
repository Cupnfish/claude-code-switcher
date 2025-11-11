//! DeepSeek AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
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

    fn env_var_name(&self) -> &'static str {
        "DEEPSEEK_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "DeepSeek"
    }

    fn description(&self) -> &'static str {
        "DeepSeek Chat API - High-performance conversational AI"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "deepseek".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "deepseek-chat".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "deepseek".to_string(),
                api_base: "https://api.deepseek.com".to_string(),
                api_key: None, // Will be set from environment
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
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                "https://api.deepseek.com/anthropic".to_string(),
            );
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
            env.insert("ANTHROPIC_MODEL".to_string(), "deepseek-chat".to_string());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "deepseek-chat".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.environment = Some(env);
        }

        settings
    }
}

/// Create DeepSeek template settings (legacy compatibility function)
pub fn create_deepseek_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = DeepSeekTemplate;
    template.create_settings(api_key, scope)
}
