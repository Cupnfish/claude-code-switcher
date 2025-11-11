//! K2 (Moonshot) AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// K2 AI provider template
#[derive(Debug, Clone)]
pub struct K2Template;

impl Template for K2Template {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::K2
    }

    fn env_var_name(&self) -> &'static str {
        "MOONSHOT_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "K2 (Moonshot)"
    }

    fn description(&self) -> &'static str {
        "Moonshot K2 API - Advanced conversational AI with large context"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "moonshot".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "kimi-k2-0905-preview".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "moonshot".to_string(),
                api_base: "https://api.moonshot.cn/v1".to_string(),
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
                "https://api.moonshot.cn/v1".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "kimi-k2-0905-preview".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "kimi-k2-0905-preview".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "kimi-k2-0905-preview".to_string(),
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

/// K2 Thinking AI provider template
#[derive(Debug, Clone)]
pub struct K2ThinkingTemplate;

impl Template for K2ThinkingTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::K2Thinking
    }

    fn env_var_name(&self) -> &'static str {
        "MOONSHOT_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "K2 Thinking (Moonshot)"
    }

    fn description(&self) -> &'static str {
        "Moonshot K2 Thinking API - High-speed reasoning with 256K context"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "moonshot".to_string(),
                metadata: Some(HashMap::from([(
                    "thinking".to_string(),
                    "true".to_string(),
                )])),
            });

            settings.model = Some(ModelConfig {
                name: "kimi-k2-thinking".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "moonshot".to_string(),
                api_base: "https://api.moonshot.cn/anthropic".to_string(),
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
                "https://api.moonshot.cn/anthropic".to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "kimi-k2-thinking".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "kimi-k2-thinking".to_string(),
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

/// Create K2 template settings (legacy compatibility function)
pub fn create_k2_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = K2Template;
    template.create_settings(api_key, scope)
}

/// Create K2 Thinking template settings (legacy compatibility function)
pub fn create_k2_thinking_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = K2ThinkingTemplate;
    template.create_settings(api_key, scope)
}
