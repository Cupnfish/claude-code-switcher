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

    /// Get all supported environment variable names for this provider
    fn env_var_names(&self) -> Vec<&'static str>;

    /// Create Claude settings for this template
    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings;

    /// Get display name for the template
    fn display_name(&self) -> &'static str;

    /// Get description for the template
    fn description(&self) -> &'static str;

    /// Get API key acquisition URL (if available)
    fn api_key_url(&self) -> Option<&'static str> {
        None
    }

    /// Check if this template requires additional configuration (like endpoint ID)
    fn requires_additional_config(&self) -> bool {
        false
    }

    /// Get additional configuration if needed
    fn get_additional_config(&self) -> Result<HashMap<String, String>> {
        Ok(HashMap::new())
    }

    /// Check if this template has sub-variants (like Pro/Air versions)
    fn has_variants(&self) -> bool {
        false
    }

    /// Get available variants if this template supports them
    fn get_variants() -> Result<Vec<Self>>
    where
        Self: Sized,
    {
        Ok(Vec::new())
    }

    /// Create a template instance interactively (for templates with variants)
    fn create_interactively() -> Result<Self>
    where
        Self: Sized,
    {
        Err(anyhow!(
            "This template does not support interactive creation"
        ))
    }
}

/// Type of AI provider template
#[derive(Debug, Clone, Serialize, PartialEq)]
pub enum TemplateType {
    DeepSeek,
    Zai,
    KatCoder,
    Kimi, // Unified Moonshot services (K2, K2 Thinking, Kimi For Coding)
    Longcat,
    Fishtrip,
    MiniMax,
    SeedCode,
    Zenmux,
    Duojie,
}

impl<'de> Deserialize<'de> for TemplateType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        // Handle backward compatibility for old variant names
        match s.as_str() {
            "KatCoderPro" | "KatCoderAir" => Ok(TemplateType::KatCoder),
            "DeepSeek" => Ok(TemplateType::DeepSeek),
            "Zai" => Ok(TemplateType::Zai),
            "K2" | "K2Thinking" => Ok(TemplateType::Kimi), // Backward compatibility
            "KatCoder" => Ok(TemplateType::KatCoder),
            "Kimi" => Ok(TemplateType::Kimi),
            "Longcat" => Ok(TemplateType::Longcat),
            "Fishtrip" => Ok(TemplateType::Fishtrip),
            "MiniMax" => Ok(TemplateType::MiniMax),
            "SeedCode" => Ok(TemplateType::SeedCode),
            "Zenmux" => Ok(TemplateType::Zenmux),
            "Duojie" => Ok(TemplateType::Duojie),
            _ => Err(serde::de::Error::custom(format!(
                "unknown template type: {}",
                s
            ))),
        }
    }
}

impl std::str::FromStr for TemplateType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "deepseek" | "ds" => Ok(TemplateType::DeepSeek),
            "glm" | "zhipu" | "zai" | "zai-china" | "zai-ch" | "zai-international" | "zai-int" => {
                Ok(TemplateType::Zai)
            }
            // K2 and K2 Thinking are now part of unified Kimi template
            "k2" | "moonshot" | "k2-thinking" | "k2thinking" | "kimi" | "kimi-for-coding" => {
                Ok(TemplateType::Kimi)
            }
            "kat-coder" | "katcoder" | "kat" => Ok(TemplateType::KatCoder), // Unified KatCoder
            "kat-coder-pro" | "katcoder-pro" | "katpro" => Ok(TemplateType::KatCoder), // Points to KatCoder with variant selection
            "kat-coder-air" | "katcoder-air" | "katair" => Ok(TemplateType::KatCoder), // Points to KatCoder with variant selection
            "longcat" => Ok(TemplateType::Longcat),
            "fishtrip" | "fish" => Ok(TemplateType::Fishtrip),
            "minimax"
            | "minimax-anthropic"
            | "minimax-china"
            | "minimax-ch"
            | "minimax-international"
            | "minimax-int"
            | "minimax-io" => Ok(TemplateType::MiniMax),
            "seed-code" | "seedcode" | "seed_code" => Ok(TemplateType::SeedCode),
            "zenmux" => Ok(TemplateType::Zenmux),
            "duojie" | "dj" => Ok(TemplateType::Duojie),
            _ => Err(anyhow!(
                "Unknown template: {}. Available templates: deepseek, glm, k2, k2-thinking, kat-coder, kimi, longcat, fishtrip, fish, minimax, seed-code, zenmux, duojie",
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
            TemplateType::KatCoder => write!(f, "kat-coder"),
            TemplateType::Kimi => write!(f, "kimi"), // Unified Moonshot services
            TemplateType::Longcat => write!(f, "longcat"),
            TemplateType::Fishtrip => write!(f, "fishtrip"),
            TemplateType::MiniMax => write!(f, "minimax"),
            TemplateType::SeedCode => write!(f, "seed-code"),
            TemplateType::Zenmux => write!(f, "zenmux"),
            TemplateType::Duojie => write!(f, "duojie"),
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
        TemplateType::KatCoder,
        TemplateType::Kimi, // Unified Moonshot services
        TemplateType::Longcat,
        TemplateType::Fishtrip,
        TemplateType::MiniMax,
        TemplateType::SeedCode,
        TemplateType::Zenmux,
        TemplateType::Duojie,
    ]
}

/// Get the environment variable name for a template type
/// Get all supported environment variable names for a template type
pub fn get_env_var_names(template_type: &TemplateType) -> Vec<&'static str> {
    let template_instance = get_template_instance(template_type);
    template_instance.env_var_names()
}

/// Get a template instance by type and original input string
pub fn get_template_instance_with_input(
    template_type: &TemplateType,
    input: &str,
) -> Box<dyn Template> {
    match template_type {
        TemplateType::DeepSeek => Box::new(deepseek::DeepSeekTemplate),
        TemplateType::Zai => {
            // Check if specific region was requested
            match input.to_lowercase().as_str() {
                "zai-china" | "zai-ch" => Box::new(zai::ZaiTemplate::china()),
                "zai-international" | "zai-int" => Box::new(zai::ZaiTemplate::international()),
                _ => Box::new(zai::ZaiTemplate::china()), // Default to China for general "zai"
            }
        }
        TemplateType::KatCoder => {
            // Check if specific variant was requested
            match input.to_lowercase().as_str() {
                "kat-coder-pro" | "katcoder-pro" | "katpro" => {
                    Box::new(kat_coder::KatCoderTemplate::pro())
                }
                "kat-coder-air" | "katcoder-air" | "katair" => {
                    Box::new(kat_coder::KatCoderTemplate::air())
                }
                _ => Box::new(kat_coder::KatCoderTemplate::pro()), // Default to Pro for general "kat-coder"
            }
        }
        TemplateType::Kimi => {
            // Check if specific Moonshot service was requested
            match input.to_lowercase().as_str() {
                "k2" | "moonshot" => Box::new(kimi::KimiTemplate::k2()),
                "k2-thinking" | "k2thinking" => Box::new(kimi::KimiTemplate::k2_thinking()),
                "kimi" | "kimi-for-coding" => Box::new(kimi::KimiTemplate::kimi_for_coding()),
                _ => Box::new(kimi::KimiTemplate::k2()), // Default to K2 for general "kimi"
            }
        }
        TemplateType::Longcat => Box::new(longcat::LongcatTemplate),
        TemplateType::Fishtrip => Box::new(fishtrip::FishtripTemplate),
        TemplateType::MiniMax => {
            // Check if specific region was requested
            match input.to_lowercase().as_str() {
                "minimax-international" | "minimax-int" | "minimax-io" => {
                    Box::new(minimax::MiniMaxTemplate::international())
                }
                _ => Box::new(minimax::MiniMaxTemplate::china()), // Default to China
            }
        }
        TemplateType::SeedCode => Box::new(seed_code::SeedCodeTemplate),
        TemplateType::Zenmux => Box::new(zenmux::ZenmuxTemplate),
        TemplateType::Duojie => Box::new(duojie::DuojieTemplate),
    }
}

/// Get a template instance by type (for backward compatibility)
pub fn get_template_instance(template_type: &TemplateType) -> Box<dyn Template> {
    get_template_instance_with_input(template_type, "")
}

/// Legacy compatibility function - creates a settings function for backwards compatibility
pub fn get_template(template_type: &TemplateType) -> fn(&str, &SnapshotScope) -> ClaudeSettings {
    match template_type {
        TemplateType::DeepSeek => create_deepseek_template,
        TemplateType::Zai => create_zai_template,
        TemplateType::KatCoder => create_kat_coder_template,
        TemplateType::Kimi => create_k2_template, // Default to K2 for backward compatibility
        TemplateType::Longcat => create_longcat_template,
        TemplateType::Fishtrip => create_fishtrip_template,
        TemplateType::MiniMax => create_minimax_template,
        TemplateType::SeedCode => create_seed_code_template,
        TemplateType::Zenmux => create_zenmux_template,
        TemplateType::Duojie => create_duojie_template,
    }
}

// Import all template modules
pub mod deepseek;
pub mod duojie;
pub mod fishtrip;
pub mod kat_coder;
pub mod kimi; // Unified module for all Moonshot services
pub mod longcat;
pub mod minimax;
pub mod seed_code;
pub mod zai;
pub mod zenmux;

// Re-export for backward compatibility
pub use deepseek::*;
pub use duojie::*;
pub use fishtrip::*;
pub use kat_coder::*;
pub use kimi::*; // Includes legacy k2 functions
pub use longcat::*;
pub use minimax::*;
pub use seed_code::*;
pub use zai::*;
pub use zenmux::*;
