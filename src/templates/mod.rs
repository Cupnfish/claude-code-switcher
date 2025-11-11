//! Template module for AI provider configurations
//!
//! This module provides a modular approach to managing different AI provider templates.
//! Each template is implemented as a separate module with the Template trait.

use crate::{settings::ClaudeSettings, snapshots::SnapshotScope};
use anyhow::{Result, anyhow};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Trait that all AI provider templates must implement
pub trait Template {
    /// Get the template type identifier
    fn template_type(&self) -> TemplateType;

    /// Get the environment variable name for the API key
    fn env_var_name(&self) -> &'static str;

    /// Create Claude settings for this template
    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings;

    /// Get display name for the template
    fn display_name(&self) -> &'static str;

    /// Get description for the template
    fn description(&self) -> &'static str;

    /// Check if this template requires additional configuration (like endpoint ID)
    fn requires_additional_config(&self) -> bool {
        false
    }

    /// Get additional configuration if needed
    fn get_additional_config(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }
}

/// Type of AI provider template
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateType {
    DeepSeek,
    Zai,
    K2,
    K2Thinking,
    KatCoder,
    KatCoderPro,
    KatCoderAir,
    Kimi,
    Longcat,
    MiniMax,
}

impl std::str::FromStr for TemplateType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "deepseek" | "ds" => Ok(TemplateType::DeepSeek),
            "glm" | "zhipu" | "zai" => Ok(TemplateType::Zai),
            "k2" | "moonshot" => Ok(TemplateType::K2),
            "k2-thinking" | "k2thinking" => Ok(TemplateType::K2Thinking),
            "kat-coder" | "katcoder" | "kat" => Ok(TemplateType::KatCoderPro), // Legacy alias: points to Pro version
            "kat-coder-pro" | "katcoder-pro" | "katpro" => Ok(TemplateType::KatCoderPro),
            "kat-coder-air" | "katcoder-air" | "katair" => Ok(TemplateType::KatCoderAir),
            "kimi" | "kimi-for-coding" => Ok(TemplateType::Kimi),
            "longcat" => Ok(TemplateType::Longcat),
            "minimax" | "minimax-anthropic" => Ok(TemplateType::MiniMax),
            _ => Err(anyhow!(
                "Unknown template: {}. Available templates: deepseek, glm, k2, k2-thinking, kat-coder, kat-coder-pro, kat-coder-air, kimi, longcat, minimax",
                s
            )),
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateType::DeepSeek => write!(f, "deepseek"),
            TemplateType::Zai => write!(f, "zai"),
            TemplateType::K2 => write!(f, "k2"),
            TemplateType::K2Thinking => write!(f, "k2-thinking"),
            TemplateType::KatCoder => write!(f, "kat-coder"),
            TemplateType::KatCoderPro => write!(f, "kat-coder-pro"),
            TemplateType::KatCoderAir => write!(f, "kat-coder-air"),
            TemplateType::Kimi => write!(f, "kimi"),
            TemplateType::Longcat => write!(f, "longcat"),
            TemplateType::MiniMax => write!(f, "minimax"),
        }
    }
}

/// Get template type from string
pub fn get_template_type(template_str: &str) -> Result<TemplateType> {
    template_str.parse()
}

/// Get all available template types
pub fn get_all_templates() -> Vec<TemplateType> {
    vec![
        TemplateType::DeepSeek,
        TemplateType::Zai,
        TemplateType::K2,
        TemplateType::K2Thinking,
        TemplateType::KatCoder,
        TemplateType::KatCoderPro,
        TemplateType::KatCoderAir,
        TemplateType::Kimi,
        TemplateType::Longcat,
        TemplateType::MiniMax,
    ]
}

/// Get the environment variable name for a template type
pub fn get_env_var_name(template_type: &TemplateType) -> &'static str {
    match template_type {
        TemplateType::DeepSeek => "DEEPSEEK_API_KEY",
        TemplateType::Zai => "Z_AI_API_KEY",
        TemplateType::K2 | TemplateType::K2Thinking => "MOONSHOT_API_KEY",
        TemplateType::KatCoder | TemplateType::KatCoderPro | TemplateType::KatCoderAir => {
            "KAT_CODER_API_KEY"
        }
        TemplateType::Kimi => "KIMI_API_KEY",
        TemplateType::Longcat => "LONGCAT_API_KEY",
        TemplateType::MiniMax => "MINIMAX_API_KEY",
    }
}

/// Get a template instance by type
pub fn get_template_instance(template_type: &TemplateType) -> Box<dyn Template> {
    match template_type {
        TemplateType::DeepSeek => Box::new(deepseek::DeepSeekTemplate),
        TemplateType::Zai => Box::new(zai::ZaiTemplate),
        TemplateType::K2 => Box::new(k2::K2Template),
        TemplateType::K2Thinking => Box::new(k2::K2ThinkingTemplate),
        TemplateType::KatCoder => Box::new(kat_coder::KatCoderProTemplate), // Legacy alias
        TemplateType::KatCoderPro => Box::new(kat_coder::KatCoderProTemplate),
        TemplateType::KatCoderAir => Box::new(kat_coder::KatCoderAirTemplate),
        TemplateType::Kimi => Box::new(kimi::KimiTemplate),
        TemplateType::Longcat => Box::new(longcat::LongcatTemplate),
        TemplateType::MiniMax => Box::new(minimax::MiniMaxTemplate),
    }
}

/// Legacy compatibility function - creates a settings function for backwards compatibility
pub fn get_template(template_type: &TemplateType) -> fn(&str, &SnapshotScope) -> ClaudeSettings {
    match template_type {
        TemplateType::DeepSeek => create_deepseek_template,
        TemplateType::Zai => create_zai_template,
        TemplateType::K2 => create_k2_template,
        TemplateType::K2Thinking => create_k2_thinking_template,
        TemplateType::KatCoder => create_kat_coder_pro_template, // Legacy alias: points to Pro version
        TemplateType::KatCoderPro => create_kat_coder_pro_template,
        TemplateType::KatCoderAir => create_kat_coder_air_template,
        TemplateType::Kimi => create_kimi_template,
        TemplateType::Longcat => create_longcat_template,
        TemplateType::MiniMax => create_minimax_template,
    }
}

// Import all template modules
pub mod deepseek;
pub mod k2;
pub mod kat_coder;
pub mod kimi;
pub mod longcat;
pub mod minimax;
pub mod zai;

// Re-export for backward compatibility
pub use deepseek::*;
pub use k2::*;
pub use kat_coder::*;
pub use kimi::*;
pub use longcat::*;
pub use minimax::*;
pub use zai::*;
