//! Kimi/Moonshot AI provider templates implementation
//!
//! This module provides unified support for Moonshot's various services:
//! - K2: General-purpose conversational AI
//! - K2 Thinking: High-speed reasoning with extended context
//! - Kimi: Specialized coding AI

use crate::{
    settings::{ClaudeSettings, Permissions},
    snapshots::SnapshotScope,
    templates::Template,
};
use anyhow::{Result, anyhow};
use atty;
use inquire::Select;
use std::collections::HashMap;

/// Kimi/Moonshot service variants
#[derive(Debug, Clone)]
pub enum KimiVariant {
    K2,
    K2Thinking,
    KimiForCoding,
}

impl KimiVariant {
    pub fn display_name(&self) -> &'static str {
        match self {
            KimiVariant::K2 => "K2 (Moonshot)",
            KimiVariant::K2Thinking => "K2 Thinking (Moonshot)",
            KimiVariant::KimiForCoding => "Kimi For Coding",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            KimiVariant::K2 => "Moonshot K2 API - Advanced conversational AI with large context",
            KimiVariant::K2Thinking => {
                "Moonshot K2 Thinking API - High-speed reasoning with 256K context"
            }
            KimiVariant::KimiForCoding => "Kimi For Coding API - Specialized for coding tasks",
        }
    }

    pub fn model_name(&self) -> &'static str {
        match self {
            KimiVariant::K2 => "kimi-k2-0905-preview",
            KimiVariant::K2Thinking => "kimi-k2-thinking",
            KimiVariant::KimiForCoding => "kimi-for-coding",
        }
    }

    pub fn api_base(&self) -> &'static str {
        match self {
            KimiVariant::K2 => "https://api.moonshot.cn/v1",
            KimiVariant::K2Thinking => "https://api.moonshot.cn/anthropic",
            KimiVariant::KimiForCoding => "https://api.kimi.com/coding/",
        }
    }

    pub fn api_key_url(&self) -> &'static str {
        match self {
            KimiVariant::K2 | KimiVariant::K2Thinking => {
                "https://platform.moonshot.cn/console/api-keys"
            }
            KimiVariant::KimiForCoding => "https://kimi.moonshot.cn/user-center/apikeys",
        }
    }

    pub fn env_var_names(&self) -> Vec<&'static str> {
        match self {
            KimiVariant::K2 | KimiVariant::K2Thinking => {
                vec![
                    "MOONSHOT_API_KEY",
                    "MOONSHOT_TOKEN",
                    "K2_API_KEY",
                    "MOONSHOT",
                ]
            }
            KimiVariant::KimiForCoding => {
                vec![
                    "KIMI_API_KEY",
                    "KIMI_TOKEN",
                    "KIMI_FOR_CODING_API_KEY",
                    "KIMI",
                ]
            }
        }
    }
}

/// Kimi/Moonshot AI provider template
#[derive(Debug, Clone)]
pub struct KimiTemplate {
    variant: KimiVariant,
}

impl KimiTemplate {
    pub fn new(variant: KimiVariant) -> Self {
        Self { variant }
    }

    pub fn k2() -> Self {
        Self::new(KimiVariant::K2)
    }

    pub fn k2_thinking() -> Self {
        Self::new(KimiVariant::K2Thinking)
    }

    pub fn kimi_for_coding() -> Self {
        Self::new(KimiVariant::KimiForCoding)
    }
}

impl Template for KimiTemplate {
    fn template_type(&self) -> crate::templates::TemplateType {
        crate::templates::TemplateType::Kimi
    }

    fn env_var_names(&self) -> Vec<&'static str> {
        self.variant.env_var_names()
    }

    fn display_name(&self) -> &'static str {
        self.variant.display_name()
    }

    fn description(&self) -> &'static str {
        self.variant.description()
    }

    fn api_key_url(&self) -> Option<&'static str> {
        Some(self.variant.api_key_url())
    }

    fn has_variants(&self) -> bool {
        true
    }

    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(vec![
            Self::k2(),
            Self::k2_thinking(),
            Self::kimi_for_coding(),
        ])
    }

    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        if !atty::is(atty::Stream::Stdin) {
            return Err(anyhow!(
                "Kimi/Moonshot requires interactive mode to select service. Use 'k2', 'k2-thinking', or 'kimi' explicitly if not in interactive mode."
            ));
        }

        let variants = [
            ("Kimi For Coding", "Kimi - Specialized for coding tasks"),
            ("K2", "Moonshot K2 - Advanced conversational AI"),
            ("K2 Thinking", "Moonshot K2 Thinking - High-speed reasoning"),
        ];

        let options: Vec<String> = variants.iter().map(|(name, _)| name.to_string()).collect();

        let choice = Select::new("Select Moonshot service:", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get service selection: {}", e))?;

        let template = match choice.as_str() {
            "K2" => Self::k2(),
            "K2 Thinking" => Self::k2_thinking(),
            "Kimi For Coding" => Self::kimi_for_coding(),
            _ => unreachable!(),
        };

        Ok(template)
    }

    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
        let mut settings = ClaudeSettings::new();

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

        if matches!(
            scope,
            SnapshotScope::Env | SnapshotScope::Common | SnapshotScope::All
        ) {
            let mut env = HashMap::new();

            // Different authentication for different services
            match self.variant {
                KimiVariant::K2 => {
                    env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
                    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
                }
                KimiVariant::K2Thinking | KimiVariant::KimiForCoding => {
                    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
                }
            }

            env.insert(
                "ANTHROPIC_BASE_URL".to_string(),
                self.variant.api_base().to_string(),
            );
            env.insert(
                "ANTHROPIC_MODEL".to_string(),
                self.variant.model_name().to_string(),
            );
            env.insert(
                "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
                self.variant.model_name().to_string(),
            );

            // Additional models for K2
            if matches!(self.variant, KimiVariant::K2) {
                env.insert(
                    "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
                    self.variant.model_name().to_string(),
                );
            }

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

/// Legacy compatibility functions
pub fn create_k2_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KimiTemplate::k2();
    template.create_settings(api_key, scope)
}

pub fn create_k2_thinking_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KimiTemplate::k2_thinking();
    template.create_settings(api_key, scope)
}

pub fn create_kimi_template(api_key: &str, scope: &SnapshotScope) -> ClaudeSettings {
    let template = KimiTemplate::kimi_for_coding();
    template.create_settings(api_key, scope)
}
