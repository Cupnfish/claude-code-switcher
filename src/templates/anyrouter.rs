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
        "claude-fable-5[1m]"
    }

    pub fn api_host(&self) -> &'static str {
        match self {
            AnyRouterRegion::China => "a-ocnfniawgw.cn-shanghai.fcapp.run",
            AnyRouterRegion::Fallback => "anyrouter.top",
        }
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
        vec![
            "ANYROUTER_AUTH_TOKEN",
            "ANYROUTER_API_KEY",
            "ANYROUTER_API_TOKEN",
        ]
    }

    fn display_name(&self) -> &'static str {
        self.region.display_name()
    }

    fn description(&self) -> &'static str {
        self.region.description()
    }

    fn api_host(&self) -> Option<&'static str> {
        Some(self.region.api_host())
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
        let model = self.region.model_name();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some(model.to_string());
            settings.effort_level = Some("max".to_string());

            // Disable co-authored-by in git commits.
            settings.attribution = Some(crate::settings::Attribution {
                commit: Some(String::new()),
                pr: Some(String::new()),
            });

            settings.permissions = Some(Permissions {
                allow: Some(crate::templates::default_allow_list()),
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
            env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
            env.insert("ANTHROPIC_MODEL".to_string(), model.to_string());
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                model.to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                model.to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                model.to_string(),
            );
            env.insert(
                "ANTHROPIC_BETAS".to_string(),
                "context-1m-2025-08-07".to_string(),
            );
            env.insert("ENABLE_THINKING".to_string(), "true".to_string());
            env.insert("ENABLE_STREAMING".to_string(), "true".to_string());
            env.insert("ENABLE_TOOL_SEARCH".to_string(), "true".to_string());
            env.insert("REASONING_EFFORT".to_string(), "max".to_string());
            env.insert("CLAUDE_CODE_EFFORT_LEVEL".to_string(), "max".to_string());
            env.insert("CLAUDE_CODE_MAX_RETRIES".to_string(), "15".to_string());
            env.insert("MAX_OUTPUT_TOKENS".to_string(), "128000".to_string());
            env.insert(
                "CLAUDE_CODE_MAX_OUTPUT_TOKENS".to_string(),
                "128000".to_string(),
            );
            env.insert("MAX_MCP_OUTPUT_TOKENS".to_string(), "64000".to_string());
            env.insert(
                "CLAUDE_CODE_AUTO_COMPACT_WINDOW".to_string(),
                "768000".to_string(),
            );
            env.insert("AUTH_HEADER_MODE".to_string(), "x-api-key".to_string());
            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS".to_string(),
                "1".to_string(),
            );
            // Windows-only tooling: PowerShell tool + Git Bash path for the Bash tool.
            #[cfg(target_os = "windows")]
            {
                env.insert(
                    "CLAUDE_CODE_USE_POWERSHELL_TOOL".to_string(),
                    "1".to_string(),
                );
                env.insert(
                    "CLAUDE_CODE_GIT_BASH_PATH".to_string(),
                    "C:\\Program Files\\Git\\bin\\bash.exe".to_string(),
                );
            }
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn china_env_matches_full_configuration() {
        let settings =
            AnyRouterTemplate::china().create_settings("sk-test-key", &SnapshotScope::Common);
        assert_eq!(settings.model.as_deref(), Some("claude-fable-5[1m]"));
        assert_eq!(settings.effort_level.as_deref(), Some("max"));
        // attribution disables co-authored-by
        let attr = settings.attribution.expect("attribution set");
        assert_eq!(attr.commit.as_deref(), Some(""));
        assert_eq!(attr.pr.as_deref(), Some(""));

        let env = settings.env.expect("env set");
        // token must come from the api_key parameter, never hardcoded
        assert_eq!(
            env.get("ANTHROPIC_AUTH_TOKEN").map(String::as_str),
            Some("sk-test-key"),
        );
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://a-ocnfniawgw.cn-shanghai.fcapp.run"),
        );
        for key in [
            "ANTHROPIC_MODEL",
            "ANTHROPIC_DEFAULT_HAIKU_MODEL",
            "ANTHROPIC_DEFAULT_SONNET_MODEL",
            "ANTHROPIC_DEFAULT_OPUS_MODEL",
        ] {
            assert_eq!(
                env.get(key).map(String::as_str),
                Some("claude-fable-5[1m]"),
                "{key}"
            );
        }
        assert_eq!(
            env.get("ANTHROPIC_BETAS").map(String::as_str),
            Some("context-1m-2025-08-07"),
        );
        assert_eq!(
            env.get("API_TIMEOUT_MS").map(String::as_str),
            Some("3000000")
        );
        assert_eq!(env.get("ENABLE_THINKING").map(String::as_str), Some("true"));
        assert_eq!(
            env.get("ENABLE_STREAMING").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            env.get("ENABLE_TOOL_SEARCH").map(String::as_str),
            Some("true")
        );
        assert_eq!(env.get("REASONING_EFFORT").map(String::as_str), Some("max"));
        assert_eq!(
            env.get("CLAUDE_CODE_EFFORT_LEVEL").map(String::as_str),
            Some("max"),
        );
        assert_eq!(
            env.get("CLAUDE_CODE_MAX_RETRIES").map(String::as_str),
            Some("15")
        );
        assert_eq!(
            env.get("MAX_OUTPUT_TOKENS").map(String::as_str),
            Some("128000")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_MAX_OUTPUT_TOKENS").map(String::as_str),
            Some("128000"),
        );
        assert_eq!(
            env.get("MAX_MCP_OUTPUT_TOKENS").map(String::as_str),
            Some("64000")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_AUTO_COMPACT_WINDOW")
                .map(String::as_str),
            Some("768000"),
        );
        assert_eq!(
            env.get("AUTH_HEADER_MODE").map(String::as_str),
            Some("x-api-key")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC")
                .map(String::as_str),
            Some("1"),
        );
        assert_eq!(
            env.get("CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS")
                .map(String::as_str),
            Some("1"),
        );

        // Windows-only tooling env vars are emitted only on Windows.
        #[cfg(target_os = "windows")]
        {
            assert_eq!(
                env.get("CLAUDE_CODE_USE_POWERSHELL_TOOL")
                    .map(String::as_str),
                Some("1"),
            );
            assert_eq!(
                env.get("CLAUDE_CODE_GIT_BASH_PATH").map(String::as_str),
                Some("C:\\Program Files\\Git\\bin\\bash.exe"),
            );
        }
        #[cfg(not(target_os = "windows"))]
        {
            assert!(env.get("CLAUDE_CODE_USE_POWERSHELL_TOOL").is_none());
            assert!(env.get("CLAUDE_CODE_GIT_BASH_PATH").is_none());
        }

        // permissions: WebSearch denied; PowerShell allowed only on Windows.
        let perms = settings.permissions.expect("permissions set");
        assert_eq!(perms.deny, Some(vec!["WebSearch".to_string()]));
        let allow = perms.allow.expect("allow list");
        #[cfg(target_os = "windows")]
        assert!(allow.contains(&"PowerShell".to_string()));
        #[cfg(not(target_os = "windows"))]
        assert!(!allow.contains(&"PowerShell".to_string()));
    }

    #[test]
    fn fallback_region_uses_main_endpoint() {
        let settings =
            AnyRouterTemplate::fallback().create_settings("sk-test-key", &SnapshotScope::Env);
        let env = settings.env.expect("env set");
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://anyrouter.top"),
        );
        // same full env surface regardless of region
        assert_eq!(
            env.get("ENABLE_TOOL_SEARCH").map(String::as_str),
            Some("true")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_AUTO_COMPACT_WINDOW")
                .map(String::as_str),
            Some("768000"),
        );
    }

    #[test]
    fn env_scope_omits_model_and_permissions() {
        let settings =
            AnyRouterTemplate::china().create_settings("sk-test-key", &SnapshotScope::Env);
        assert!(settings.model.is_none());
        assert!(settings.permissions.is_none());
        assert!(settings.env.is_some());
    }
}
