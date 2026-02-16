//! AnyRouter AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use std::collections::HashMap;

/// AnyRouter provider regions
#[derive(Debug, Clone)]
pub enum AnyRouterRegion {
    China,    // Fast but may be unstable
    Fallback, // Stable main endpoint
}

impl AnyRouterRegion {
    pub fn display_name(&self) -> &'static str {
        match self {
            AnyRouterRegion::China => "AnyRouter China (Fast)",
            AnyRouterRegion::Fallback => "AnyRouter Fallback (Stable)",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            AnyRouterRegion::China => "Fast endpoint in China - May be unstable, use as default",
            AnyRouterRegion::Fallback => {
                "Stable main endpoint - Use as fallback when China endpoint is unavailable"
            }
        }
    }

    pub fn base_url(&self) -> &'static str {
        match self {
            AnyRouterRegion::China => "https://a-ocnfniawgw.cn-shanghai.fcapp.run",
            AnyRouterRegion::Fallback => "https://anyrouter.top",
        }
    }

    pub fn model_name(&self) -> &'static str {
        "claude-opus-4-6"
    }

    pub fn small_fast_model(&self) -> &'static str {
        "claude-opus-4-6"
    }
}

/// AnyRouter AI provider template
#[derive(Debug, Clone)]
pub struct AnyRouterTemplate {
    region: AnyRouterRegion,
}

impl AnyRouterTemplate {
    pub fn new(region: AnyRouterRegion) -> Self {
        Self { region }
    }

    pub fn china() -> Self {
        Self::new(AnyRouterRegion::China)
    }

    pub fn fallback() -> Self {
        Self::new(AnyRouterRegion::Fallback)
    }
}

impl Template for AnyRouterTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::AnyRouter
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["ANTHROPIC_AUTH_TOKEN", "ANYROUTER_API_KEY"]
    }

    fn display_name(&self) -> &'static str {
        self.region.display_name()
    }

    fn description(&self) -> &'static str {
        self.region.description()
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(vec![Self::china(), Self::fallback()])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "AnyRouter requires interactive mode to select region. Use 'anyrouter-china' or 'anyrouter-fallback' explicitly if not in interactive mode."
            ));
        }

        let regions = [
            (
                "AnyRouter China (Fast)",
                "Fast endpoint in China - May be unstable",
            ),
            (
                "AnyRouter Fallback (Stable)",
                "Stable main endpoint - Use when China endpoint is unavailable",
            ),
        ];

        let options: Vec<String> = regions.iter().map(|(name, _)| name.to_string()).collect();

        let choice = Select::new("Select AnyRouter region:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get region selection: {}", e))?;

        let template = match choice.as_str() {
            "AnyRouter China (Fast)" => Self::china(),
            "AnyRouter Fallback (Stable)" => Self::fallback(),
            _ => unreachable!(),
        };

        Ok(template)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some(self.region.model_name().to_string());

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
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                self.region.base_url().to_string(),
            );
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                self.region.model_name().to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                self.region.small_fast_model().to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create AnyRouter template settings (legacy compatibility function)
pub fn create_anyrouter_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = AnyRouterTemplate::china(); // Default to China for fast access
    template.create_settings(api_key, scope)
}
