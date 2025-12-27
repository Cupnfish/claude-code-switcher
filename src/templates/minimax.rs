//! MiniMax AI provider template implementation

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use std::collections::HashMap;

/// MiniMax API region
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MiniMaxRegion {
    /// China region (api.minimaxi.com)
    China,
    /// International region (api.minimax.io)
    International,
}

impl MiniMaxRegion {
    /// Get the base URL for this region
    fn base_url(&self) -> &'static str {
        match self {
            MiniMaxRegion::China => "https://api.minimaxi.com/anthropic",
            MiniMaxRegion::International => "https://api.minimax.io/anthropic",
        }
    }
}

/// MiniMax AI provider template
#[derive(Debug, Clone)]
pub struct MiniMaxTemplate {
    region: MiniMaxRegion,
}

impl MiniMaxTemplate {
    /// Create a new MiniMax template with the specified region
    pub fn new(region: MiniMaxRegion) -> Self {
        Self { region }
    }

    /// Create a MiniMax template for China region
    pub fn china() -> Self {
        Self::new(MiniMaxRegion::China)
    }

    /// Create a MiniMax template for International region
    pub fn international() -> Self {
        Self::new(MiniMaxRegion::International)
    }

    /// Get the current region
    pub fn region(&self) -> MiniMaxRegion {
        self.region
    }
}

impl Default for MiniMaxTemplate {
    fn default() -> Self {
        Self::china()
    }
}

impl Template for MiniMaxTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::MiniMax
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec!["MINIMAX_API_KEY", "MINIMAX_TOKEN", "MINIMAX_AUTH_TOKEN"]
    }

    fn display_name(&self) -> &'static str {
        "MiniMax"
    }

    fn description(&self) -> &'static str {
        "MiniMax M2.1 API - High-performance AI with Anthropic compatibility"
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some("MiniMax-M2.1".to_string());

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
            env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                self.region.base_url().to_string(),
            );
            env.insert("ANTHROPIC_MODEL".to_string(), "MiniMax-M2.1".to_string());
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                "MiniMax-M2.1".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                "MiniMax-M2.1".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                "MiniMax-M2.1".to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                "MiniMax-M2.1".to_string(),
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

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://platform.minimaxi.com/user-center/basic-information/interface-key")
    }
}

/// Create MiniMax template settings (legacy compatibility function)
/// Defaults to China region
pub fn create_minimax_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = MiniMaxTemplate::china();
    template.create_settings(api_key, scope)
}

/// Create MiniMax template settings for China region
pub fn create_minimax_china_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = MiniMaxTemplate::china();
    template.create_settings(api_key, scope)
}

/// Create MiniMax template settings for International region
pub fn create_minimax_international_template(
    api_key: &str,
    scope: &SnapshotScope,
) -> ClaudeSettings {
    let template = MiniMaxTemplate::international();
    template.create_settings(api_key, scope)
}
