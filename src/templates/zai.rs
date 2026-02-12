//! ZAI (GLM/Zhipu) AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use std::collections::HashMap;

/// ZAI (GLM/Zhipu) AI provider regions
#[derive(Debug, Clone)]
pub enum ZaiRegion {
    China,
    International,
}

impl ZaiRegion {
    pub fn display_name(&self) -> &'static str {
        match self {
            ZaiRegion::China => "ZAI China (智谱AI)",
            ZaiRegion::International => "ZAI International",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            ZaiRegion::China => {
                "Zhipu AI GLM-5 in China - Coding aligned with Claude Opus 4.5, with thinking capabilities"
            }
            ZaiRegion::International => {
                "Zhipu AI GLM-5 International - Global access with optimized routing"
            }
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            ZaiRegion::China => "https://open.bigmodel.cn/api/anthropic",
            ZaiRegion::International => "https://api.z.ai/api/anthropic",
        }
    }

    pub fn model_name(&self) -> &'static str {
        match self {
            ZaiRegion::China => "glm-5",
            ZaiRegion::International => "glm-5",
        }
    }

    pub fn small_fast_model(&self) -> &'static str {
        match self {
            ZaiRegion::China => "glm-5",
            ZaiRegion::International => "glm-5",
        }
    }

    pub fn api_key_url(&self) -> &'static str {
        match self {
            ZaiRegion::China => "https://open.bigmodel.cn/usercenter/apikeys",
            ZaiRegion::International => "https://console.z.ai/apikeys",
        }
    }
}

/// ZAI (GLM/Zhipu) AI provider template
#[derive(Debug, Clone)]
pub struct ZaiTemplate {
    region: ZaiRegion,
}

impl ZaiTemplate {
    pub fn new(region: ZaiRegion) -> Self {
        Self { region }
    }

    pub fn china() -> Self {
        Self::new(ZaiRegion::China)
    }

    pub fn international() -> Self {
        Self::new(ZaiRegion::International)
    }
}

impl Template for ZaiTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Zai
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec![
            "Z_AI_API_KEY",
            "ZAI_API_KEY",
            "GLM_API_KEY",
            "ZHIPU_API_KEY",
        ]
    }

    fn display_name(&self) -> &'static str {
        self.region.display_name()
    }

    fn description(&self) -> &'static str {
        self.region.description()
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some(self.region.api_key_url())
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(vec![Self::china(), Self::international()])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "ZAI requires interactive mode to select region. Use 'zai-china' or 'zai-international' explicitly if not in interactive mode."
            ));
        }

        let regions = [
            (
                "ZAI China (智谱AI)",
                "Fast response with thinking capabilities, optimized for China users",
            ),
            (
                "ZAI International",
                "Global access with optimized routing for international users",
            ),
        ];

        let options: Vec<String> = regions.iter().map(|(name, _)| name.to_string()).collect();

        let choice = Select::new("Select ZAI region:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get region selection: {}", e))?;

        let template = match choice.as_str() {
            "ZAI China (智谱AI)" => Self::china(),
            "ZAI International" => Self::international(),
            _ => unreachable!(),
        };

        Ok(template)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some(self.region.model_name().to_string());

            // Use the new permissions format from the provided version
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
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                self.region.base_url().to_string(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                self.region.model_name().to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                self.region.small_fast_model().to_string(),
            );
            env.insert("ENABLE_THINKING".to_string(), "true".to_string());
            env.insert("REASONING_EFFORT".to_string(), "ultrathink".to_string());
            env.insert("MAX_THINKING_TOKENS".to_string(), "32000".to_string());
            env.insert("ENABLE_STREAMING".to_string(), "true".to_string());
            env.insert("MAX_OUTPUT_TOKENS".to_string(), "128000".to_string());
            env.insert("MAX_MCP_OUTPUT_TOKENS".to_string(), "64000".to_string());
            env.insert("AUTH_HEADER_MODE".to_string(), "x-api-key".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create ZAI template settings (legacy compatibility function)
pub fn create_zai_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = ZaiTemplate::china(); // Default to China for backward compatibility
    template.create_settings(api_key, scope)
}
