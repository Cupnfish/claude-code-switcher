//! KatCoder (WanQing) AI provider template implementation

use crate::{
    credentials::CredentialStore,
    settings::{ClaudeSettings, Permissions},
    simple_selector::get_endpoint_id_interactively,
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use std::collections::HashMap;

/// KatCoder AI provider variants
#[derive(Debug, Clone)]
pub enum KatCoderVariant {
    Pro,
    Air,
}

impl KatCoderVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            KatCoderVariant::Pro => "KatCoder Pro (WanQing)",
            KatCoderVariant::Air => "KatCoder Air (WanQing)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KatCoderVariant::Pro => {
                "WanQing KAT-Coder Pro V1 - Professional coding AI with advanced capabilities"
            }
            KatCoderVariant::Air => {
                "WanQing KAT-Coder Air V1 - Lightweight coding AI with fast response"
            }
        }
    }

    pub fn model_name(&self) -> &'static str {
        match self {
            KatCoderVariant::Pro => "KAT-Coder-Pro-V1",
            KatCoderVariant::Air => "KAT-Coder-Air-V1",
        }
    }
}

/// KatCoder AI provider template
#[derive(Debug, Clone)]
pub struct KatCoderTemplate {
    variant: KatCoderVariant,
}

impl KatCoderTemplate {
    pub fn new(variant: KatCoderVariant) -> Self {
        Self { variant }
    }

    pub fn pro() -> Self {
        Self::new(KatCoderVariant::Pro)
    }

    pub fn air() -> Self {
        Self::new(KatCoderVariant::Air)
    }
}

impl Template for KatCoderTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::KatCoder
    }

    fn env_var_name(&self) -> &'static str {
        "KAT_CODER_API_KEY"
    }

    fn display_name(&self) -> &'static str {
        self.variant.display_name()
    }

    fn description(&self) -> &'static str {
        self.variant.description()
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://console.volcengine.com/ark/region:ark+cn-beijing/apikey")
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(vec![Self::pro(), Self::air()])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "KatCoder requires interactive mode to select variant. Use 'kat-coder-pro' or 'kat-coder-air' explicitly if not in interactive mode."
            ));
        }

        let variants = [
            (
                "KatCoder Pro",
                "Professional coding AI with advanced capabilities",
            ),
            ("KatCoder Air", "Lightweight coding AI with fast response"),
        ];

        let options: Vec<String> = variants.iter().map(|(name, _)| name.to_string()).collect();

        let choice = Select::new("Select KatCoder variant:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get variant selection: {}", e))?;

        let template = match choice.as_str() {
            "KatCoder Pro" => Self::pro(),
            "KatCoder Air" => Self::air(),
            _ => unreachable!(),
        };

        Ok(template)
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
            settings.model = Some(self.variant.model_name().to_string());

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
            env.insert("ANTHROPIC_BASE_URL".to_string(), base_url);
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                self.variant.model_name().to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                self.variant.model_name().to_string(),
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

    // Use interactive endpoint ID selector
    let endpoint_id = get_endpoint_id_interactively(&crate::templates::TemplateType::KatCoder)?;

    // Auto-save the endpoint ID if it's new and we have credentials
    if let Ok(credential_store) = CredentialStore::new()
        && let Ok(credentials) = credential_store
            .store
            .find_by_template_type(&crate::templates::TemplateType::KatCoder)
        && !credentials.is_empty()
    {
        // Save endpoint ID to the most recent credential
        let most_recent = credentials.iter().max_by_key(|c| c.created_at());
        if let Some(credential) = most_recent {
            if credential_store
                .has_endpoint_id(&endpoint_id, &crate::templates::TemplateType::KatCoder)
            {
                println!("  ✓ Endpoint ID already saved for KatCoder");
            } else if credential_store
                .save_endpoint_id(credential.id(), &endpoint_id)
                .is_ok()
            {
                println!("  ✓ Endpoint ID saved automatically for future use");
            }
        }
    }

    Ok(endpoint_id)
}

/// Create KatCoder template settings (legacy compatibility function)
pub fn create_kat_coder_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KatCoderTemplate::pro(); // Default to Pro for backward compatibility
    template.create_settings(api_key, scope)
}
