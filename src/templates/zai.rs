//! ZAI (GLM/Zhipu) AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// ZAI (GLM/Zhipu) AI provider template
#[derive(Debug, Clone)]
pub struct ZaiTemplate;

impl Template for ZaiTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Zai
    }

    fn env_var_name(&self) -> &'static str {
        "Z_AI_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "ZAI (GLM)"
    }

    fn description(&self) -> &'static str {
        "Zhipu AI GLM-4.6 with thinking capabilities and large context window"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "zhipu".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "glm-4.6".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "zhipu".to_string(),
                api_base: "https://open.bigmodel.cn/api/anthropic".to_string(),
                api_key: None,
                endpoint_id: None,
                metadata: None,
            });

            settings.http = Some(HTTPConfig {
                timeout_ms: Some(3000000),
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
                "https://open.bigmodel.cn/api/anthropic".to_string(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
            env.insert("ANTHROPIC_MODEL".to_string(), "glm-4.6".to_string());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "glm-4.6".to_string(),
            );
            env.insert("ENABLE_THINKING".to_string(), "true".to_string());
            env.insert("REASONING_EFFORT".to_string(), "ultrathink".to_string());
            env.insert("MAX_THINKING_TOKENS".to_string(), "32000".to_string());
            env.insert("ENABLE_STREAMING".to_string(), "true".to_string());
            env.insert("MAX_OUTPUT_TOKENS".to_string(), "96000".to_string());
            env.insert("MAX_MCP_OUTPUT_TOKENS".to_string(), "64000".to_string());
            env.insert("AUTH_HEADER_MODE".to_string(), "x-api-key".to_string());
            settings.environment = Some(env);
        }

        settings
    }
}

/// Create ZAI template settings (legacy compatibility function)
pub fn create_zai_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = ZaiTemplate;
    template.create_settings(api_key, scope)
}
