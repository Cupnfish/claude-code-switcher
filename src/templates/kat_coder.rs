//! KatCoder (WanQing) AI provider template implementation

use crate::{
    settings::{
        ClaudeSettings, EndpointConfig, HTTPConfig, ModelConfig, Permissions, ProviderConfig,
    },
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::{Confirm, Text};
use std::collections::HashMap;

/// KatCoder Pro AI provider template
#[derive(Debug, Clone)]
pub struct KatCoderProTemplate;

impl Template for KatCoderProTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::KatCoderPro
    }

    fn env_var_name(&self) -> &'static str {
        "KAT_CODER_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "KatCoder Pro (WanQing)"
    }

    fn description(&self) -> &'static str {
        "WanQing KAT-Coder Pro V1 - Professional coding AI with advanced capabilities"
    }

    fn requires_additional_config(&self) -> bool {
        true
    }

    fn get_additional_config(&self) -> Result<HashMap<String, String>> {
        let endpoint_id = get_kat_coder_endpoint_id()?;
        let mut config = HashMap::new();
        config.insert("endpoint_id".to_string(), endpoint_id);
        Ok(config)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        // Get endpoint ID for KatCoder
        let endpoint_id = get_kat_coder_endpoint_id().unwrap_or_else(|_| "default".to_string());
        let base_url = format!(
            "https://wanqing.streamlakeapi.com/api/gateway/v1/endpoints/{}/claude-code-proxy",
            endpoint_id
        );

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "wanqing".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "KAT-Coder-Pro-V1".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "wanqing".to_string(),
                api_base: base_url.clone(),
                api_key: None,
                endpoint_id: Some(endpoint_id.clone()),
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
            env.insert("ANTHROPIC_BASE_URL".to_string(), base_url);
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "KAT-Coder-Pro-V1".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "KAT-Coder-Pro-V1".to_string(),
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

/// KatCoder Air AI provider template
#[derive(Debug, Clone)]
pub struct KatCoderAirTemplate;

impl Template for KatCoderAirTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::KatCoderAir
    }

    fn env_var_name(&self) -> &'static str {
        "KAT_CODER_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        "KatCoder Air (WanQing)"
    }

    fn description(&self) -> &'static str {
        "WanQing KAT-Coder Air V1 - Lightweight coding AI with fast response"
    }

    fn requires_additional_config(&self) -> bool {
        true
    }

    fn get_additional_config(&self) -> Result<HashMap<String, String>> {
        let endpoint_id = get_kat_coder_endpoint_id()?;
        let mut config = HashMap::new();
        config.insert("endpoint_id".to_string(), endpoint_id);
        Ok(config)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        // Get endpoint ID for KatCoder
        let endpoint_id = get_kat_coder_endpoint_id().unwrap_or_else(|_| "default".to_string());
        let base_url = format!(
            "https://wanqing.streamlakeapi.com/api/gateway/v1/endpoints/{}/claude-code-proxy",
            endpoint_id
        );

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.provider = Some(ProviderConfig {
                id: "wanqing".to_string(),
                metadata: None,
            });

            settings.model = Some(ModelConfig {
                name: "KAT-Coder-Air-V1".to_string(),
                metadata: None,
            });

            settings.endpoint = Some(EndpointConfig {
                id: "wanqing".to_string(),
                api_base: base_url.clone(),
                api_key: None,
                endpoint_id: Some(endpoint_id.clone()),
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
            env.insert("ANTHROPIC_BASE_URL".to_string(), base_url);
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                "KAT-Coder-Air-V1".to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "KAT-Coder-Air-V1".to_string(),
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

/// Get KatCoder endpoint ID from environment or prompt user
fn get_kat_coder_endpoint_id() -> Result<String> {
    // Try to get from environment first
    let env_var = "WANQING_ENDPOINT_ID";

    if let Ok(id) = std::env::var(env_var) {
        println!(
            "  ✓ Using endpoint ID from environment variable {}",
            env_var
        );
        return Ok(id);
    }

    // If not found and we're in non-interactive mode, error
    if !atty::is(atty::Stream::Stdin) {
        return Err(anyhow!(
            "Endpoint ID required for kat-coder template. Set {} environment variable or use interactive mode.",
            env_var
        ));
    }

    // Prompt user for endpoint ID
    let prompt = "Enter WanQing endpoint ID (format: ep-xxx-xxx):";
    let endpoint_id = Text::new(prompt)
        .prompt()
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;

    if endpoint_id.trim().is_empty() {
        return Err(anyhow!("Endpoint ID cannot be empty"));
    }

    // Ask if user wants to save to environment
    let save_env = Confirm::new(&format!(
        "Save {} to environment variable for future use?",
        env_var
    ))
    .with_default(false)
    .prompt()
    .unwrap_or(false);

    if save_env {
        println!("  💡 To save permanently, add this to your shell profile:");
        println!("     export {}=\"***\"", env_var);
    }

    Ok(endpoint_id)
}

/// Create KatCoder Pro template settings (legacy compatibility function)
pub fn create_kat_coder_pro_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KatCoderProTemplate;
    template.create_settings(api_key, scope)
}

/// Create KatCoder Air template settings (legacy compatibility function)
pub fn create_kat_coder_air_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KatCoderAirTemplate;
    template.create_settings(api_key, scope)
}
