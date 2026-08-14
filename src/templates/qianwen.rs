//! Qianwen AI Platform (千问) provider template implementation
//!
//! Two billing-plan variants sharing one `TemplateType`:
//! - [`QianwenPlan::TokenPlan`] — Token Plan (个人版/团队版共用同一端点),
//!   flagship `qwen3.8-max-preview` model with thinking always on and a
//!   983616-token context window.
//! - [`QianwenPlan::PayAsYouGo`] — pay-as-you-go via the dashscope endpoint,
//!   `qwen3.8-max-preview` model.

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use std::collections::HashMap;

/// Qianwen AI Platform billing plan variants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QianwenPlan {
    /// Token Plan (个人版/团队版共用同一端点与模型映射).
    TokenPlan,
    /// 按量计费 (pay-as-you-go via dashscope).
    PayAsYouGo,
}

impl QianwenPlan {
    /// Anthropic-compatible base URL for this plan.
    pub fn base_url(&self) -> &'static str {
        match self {
            QianwenPlan::TokenPlan => {
                "https://token-plan.cn-beijing.maas.aliyuncs.com/apps/anthropic"
            }
            QianwenPlan::PayAsYouGo => "https://dashscope.aliyuncs.com/apps/anthropic",
        }
    }

    /// Host portion of the base URL (for patching the Claude CLI).
    pub fn api_host(&self) -> &'static str {
        match self {
            QianwenPlan::TokenPlan => "token-plan.cn-beijing.maas.aliyuncs.com",
            QianwenPlan::PayAsYouGo => "dashscope.aliyuncs.com",
        }
    }

    /// Primary model id for this plan.
    ///
    /// Both plans currently target the flagship `qwen3.8-max-preview` model;
    /// the plans differ only in billing endpoint and Token-Plan specifics.
    pub fn primary_model(&self) -> &'static str {
        "qwen3.8-max-preview"
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            QianwenPlan::TokenPlan => "千问 AI 平台 - Token Plan",
            QianwenPlan::PayAsYouGo => "千问 AI 平台 - 按量计费",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            QianwenPlan::TokenPlan => {
                "Qianwen Token Plan - qwen3.8-max-preview, 983616 context, thinking always on"
            }
            QianwenPlan::PayAsYouGo => "Qianwen pay-as-you-go (dashscope) - qwen3.8-max-preview",
        }
    }
}

/// Qianwen AI Platform provider template
#[derive(Debug, Clone)]
pub struct QianwenTemplate {
    plan: QianwenPlan,
}

impl QianwenTemplate {
    /// Create a new Qianwen template with the specified plan.
    pub fn new(plan: QianwenPlan) -> Self {
        Self { plan }
    }

    /// Token Plan variant (default).
    pub fn token_plan() -> Self {
        Self::new(QianwenPlan::TokenPlan)
    }

    /// Pay-as-you-go (dashscope) variant.
    pub fn pay_as_you_go() -> Self {
        Self::new(QianwenPlan::PayAsYouGo)
    }

    /// Current plan.
    pub fn plan(&self) -> QianwenPlan {
        self.plan
    }
}

impl Default for QianwenTemplate {
    fn default() -> Self {
        Self::token_plan()
    }
}

/// Qianwen model served by the Anthropic-compatible endpoint.
pub const QIANWEN_MODEL_PRESETS: &[&str] = &["qwen3.8-max-preview"];

impl Template for QianwenTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Qianwen
    }

    fn model_presets(&self) -> &'static [&'static str] {
        QIANWEN_MODEL_PRESETS
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        vec![
            "DASHSCOPE_API_KEY",
            "QIANWEN_API_KEY",
            "QWEN_API_KEY",
            "ANTHROPIC_AUTH_TOKEN",
        ]
    }

    fn display_name(&self) -> &'static str {
        self.plan.display_name()
    }

    fn description(&self) -> &'static str {
        self.plan.description()
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some("https://platform.qianwenai.com/home/api-keys")
    }

    fn api_host(&self) -> Option<&'static str> {
        Some(self.plan.api_host())
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(vec![Self::token_plan(), Self::pay_as_you_go()])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "Qianwen requires interactive mode to select a plan. Use 'qwen-token-plan' or 'qwen-payg' explicitly if not in interactive mode."
            ));
        }

        let plans = [
            (
                "千问 AI 平台 - Token Plan",
                "qwen3.8-max-preview, 983616 context, thinking always on (个人版/团队版共用)",
            ),
            (
                "千问 AI 平台 - 按量计费",
                "dashscope pay-as-you-go, qwen3.8-max-preview",
            ),
        ];

        let options: Vec<String> = plans.iter().map(|(name, _)| name.to_string()).collect();

        let choice = Select::new("Select Qianwen plan:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get plan selection: {}", e))?;

        let template = match choice.as_str() {
            "千问 AI 平台 - Token Plan" => Self::token_plan(),
            "千问 AI 平台 - 按量计费" => Self::pay_as_you_go(),
            _ => unreachable!(),
        };

        Ok(template)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();
        let model_name = self.plan.primary_model();

        if matches!(scope, SnapshotScope::Common | SnapshotScope::All) {
            settings.model = Some(model_name.to_string());

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
            env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                self.plan.base_url().to_string(),
            );
            env.insert("ANTHROPIC_MODEL".to_string(), model_name.to_string());
            env.insert(
                "ANTHROPIC_DEFAULT_HAIKU_MODEL".to_string(),
                model_name.to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                model_name.to_string(),
            );
            env.insert(
                "ANTHROPIC_DEFAULT_OPUS_MODEL".to_string(),
                model_name.to_string(),
            );
            env.insert(
                "CLAUDE_CODE_SUBAGENT_MODEL".to_string(),
                model_name.to_string(),
            );
            env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());

            // Token Plan specifics: qwen3.8-max-preview thinking is always on,
            // and the documented context window is 983616 tokens.
            if let QianwenPlan::TokenPlan = self.plan {
                env.insert("ENABLE_THINKING".to_string(), "true".to_string());
                env.insert(
                    "CLAUDE_CODE_MAX_CONTEXT_TOKENS".to_string(),
                    "983616".to_string(),
                );
            }

            env.insert(
                "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
                "1".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_NONSTREAMING_FALLBACK".to_string(),
                "1".to_string(),
            );
            env.insert(
                "CLAUDE_CODE_DISABLE_EXPERIMENTAL_BETAS".to_string(),
                "1".to_string(),
            );
            settings.env = Some(env);
        }

        settings
    }
}

/// Create Qianwen template settings (legacy compatibility function).
/// Defaults to Token Plan.
pub fn create_qianwen_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = QianwenTemplate::token_plan();
    template.create_settings(api_key, scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_plan_defaults_match_docs() {
        let settings =
            QianwenTemplate::token_plan().create_settings("sk-test", &SnapshotScope::Common);

        assert_eq!(settings.model.as_deref(), Some("qwen3.8-max-preview"));
        let env = settings.env.unwrap();
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://token-plan.cn-beijing.maas.aliyuncs.com/apps/anthropic")
        );
        assert_eq!(
            env.get("ANTHROPIC_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_SONNET_MODEL")
                .map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_OPUS_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_SUBAGENT_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("CLAUDE_CODE_MAX_CONTEXT_TOKENS")
                .map(String::as_str),
            Some("983616")
        );
        assert_eq!(env.get("ENABLE_THINKING").map(String::as_str), Some("true"));
    }

    #[test]
    fn pay_as_you_go_uses_dashscope_and_qwen38_max_preview() {
        let settings =
            QianwenTemplate::pay_as_you_go().create_settings("sk-test", &SnapshotScope::Common);

        assert_eq!(settings.model.as_deref(), Some("qwen3.8-max-preview"));
        let env = settings.env.unwrap();
        assert_eq!(
            env.get("ANTHROPIC_BASE_URL").map(String::as_str),
            Some("https://dashscope.aliyuncs.com/apps/anthropic")
        );
        assert_eq!(
            env.get("ANTHROPIC_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        assert_eq!(
            env.get("ANTHROPIC_DEFAULT_HAIKU_MODEL").map(String::as_str),
            Some("qwen3.8-max-preview")
        );
        // pay-as-you-go has no Token-Plan-specific keys
        assert!(!env.contains_key("CLAUDE_CODE_MAX_CONTEXT_TOKENS"));
        assert!(!env.contains_key("ENABLE_THINKING"));
    }

    #[test]
    fn env_scope_omits_model_and_permissions() {
        let settings =
            QianwenTemplate::token_plan().create_settings("sk-test", &SnapshotScope::Env);

        assert!(settings.model.is_none());
        assert!(settings.permissions.is_none());
        assert!(settings.env.is_some());
    }

    #[test]
    fn variants_are_listed() {
        let variants = QianwenTemplate::get_variants().unwrap();
        assert_eq!(variants.len(), 2);
        assert_eq!(variants[0].plan(), QianwenPlan::TokenPlan);
        assert_eq!(variants[1].plan(), QianwenPlan::PayAsYouGo);
    }
}
