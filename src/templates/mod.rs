//! Template module for AI provider configurations
//!
//! This module provides a modular approach to managing different AI provider templates.
//! Each template is implemented as a separate module with the Template trait.

use crate::{settings::ClaudeSettings, snapshots::SnapshotScope};
use anyhow::{Result, anyhow};

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Selectable auto-compaction threshold for providers that run a 1M context
/// model and should compact before the full window is exhausted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AutoCompactWindow {
    K256,
    K512,
    K768,
    K896,
}

pub const AUTO_COMPACT_WINDOWS: [AutoCompactWindow; 4] = [
    AutoCompactWindow::K896,
    AutoCompactWindow::K768,
    AutoCompactWindow::K512,
    AutoCompactWindow::K256,
];

impl AutoCompactWindow {
    pub fn as_str(self) -> &'static str {
        match self {
            AutoCompactWindow::K256 => "256k",
            AutoCompactWindow::K512 => "512k",
            AutoCompactWindow::K768 => "768k",
            AutoCompactWindow::K896 => "896k",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            AutoCompactWindow::K256 => "256K",
            AutoCompactWindow::K512 => "512K",
            AutoCompactWindow::K768 => "768K",
            AutoCompactWindow::K896 => "896K",
        }
    }

    pub fn env_value(self) -> &'static str {
        match self {
            AutoCompactWindow::K256 => "256000",
            AutoCompactWindow::K512 => "512000",
            AutoCompactWindow::K768 => "768000",
            AutoCompactWindow::K896 => "896000",
        }
    }
}

impl std::fmt::Display for AutoCompactWindow {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl std::str::FromStr for AutoCompactWindow {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.trim().to_ascii_lowercase().as_str() {
            "896k" | "896-k" | "896000" => Ok(AutoCompactWindow::K896),
            "768k" | "768-k" | "768000" => Ok(AutoCompactWindow::K768),
            "512k" | "512-k" | "512000" => Ok(AutoCompactWindow::K512),
            "256k" | "256-k" | "256000" => Ok(AutoCompactWindow::K256),
            _ => Err(anyhow!(
                "Invalid auto-compact window '{}'. Use one of: 896k, 768k, 512k, 256k",
                s
            )),
        }
    }
}

/// Trait that all AI provider templates must implement
pub trait Template {
    /// Get the template type identifier
    fn template_type(&self) -> TemplateType;

    /// Get all supported environment variable names for this provider
    fn env_var_names(&self) -> Vec<&'static str>;

    /// Create Claude settings for this template
    fn create_settings(&self, api_key: &str, scope: &SnapshotScope) -> ClaudeSettings;

    /// Auto-compaction thresholds supported by this template. Empty means the
    /// provider has no editable auto-compact option.
    fn supported_auto_compact_windows(&self) -> &'static [AutoCompactWindow] {
        &[]
    }

    /// Default auto-compaction threshold, when the provider supports it.
    fn default_auto_compact_window(&self) -> Option<AutoCompactWindow> {
        self.supported_auto_compact_windows().first().copied()
    }

    /// Create Claude settings using a selected auto-compaction threshold.
    /// Templates that do not override this keep their existing behavior and
    /// reject unsupported explicit choices.
    fn create_settings_with_auto_compact(
        &self,
        api_key: &str,
        scope: &SnapshotScope,
        auto_compact_window: Option<AutoCompactWindow>,
    ) -> Result<ClaudeSettings> {
        if let Some(auto_compact_window) = auto_compact_window
            && !self
                .supported_auto_compact_windows()
                .contains(&auto_compact_window)
        {
            return Err(anyhow!(
                "{} does not support auto-compact window '{}'",
                self.display_name(),
                auto_compact_window
            ));
        }
        Ok(self.create_settings(api_key, scope))
    }

    /// Get display name for the template
    fn display_name(&self) -> &'static str;

    /// Get description for the template
    fn description(&self) -> &'static str;

    /// Get API key acquisition URL (if available)
    fn api_key_url(&self) -> Option<&'static str> {
        None
    }

    /// Get the API host for this template (for patching Claude CLI)
    /// Returns the host portion of the API URL (e.g., "api.deepseek.com")
    fn api_host(&self) -> Option<&'static str> {
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

pub fn settings_use_1m_model(settings: &ClaudeSettings) -> bool {
    settings
        .model
        .as_deref()
        .is_some_and(|model| model.contains("[1m]"))
        || settings.env.as_ref().is_some_and(|env| {
            env.get("ANTHROPIC_MODEL")
                .is_some_and(|model| model.contains("[1m]"))
        })
}

pub fn supports_auto_compact_option(template: &dyn Template) -> bool {
    !template.supported_auto_compact_windows().is_empty()
        && settings_use_1m_model(&template.create_settings("sk-preview", &SnapshotScope::Common))
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
    AnyRouter,
    OpenRouter,
    BeeApi,
    Day77,
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
            "AnyRouter" => Ok(TemplateType::AnyRouter),
            "OpenRouter" => Ok(TemplateType::OpenRouter),
            "BeeApi" => Ok(TemplateType::BeeApi),
            "Day77" => Ok(TemplateType::Day77),
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
            "anyrouter" | "anyr" | "ar" | "anyrouter-china" | "anyrouter-fast" | "anyr-china"
            | "anyr-fast" | "ar-china" | "ar-fast" | "anyrouter-fallback" | "anyrouter-stable"
            | "anyr-fallback" | "anyr-stable" | "ar-fallback" | "ar-stable" => {
                Ok(TemplateType::AnyRouter)
            }
            "openrouter" | "or" => Ok(TemplateType::OpenRouter),
            "beeapi" | "bee" => Ok(TemplateType::BeeApi),
            "day77" => Ok(TemplateType::Day77),
            _ => Err(anyhow!(
                "Unknown template: {}. Available templates: deepseek, glm, k2, k2-thinking, kat-coder, kimi, longcat, fishtrip, fish, minimax, seed-code, zenmux, duojie, anyrouter, openrouter, beeapi, day77",
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
            TemplateType::AnyRouter => write!(f, "anyrouter"),
            TemplateType::OpenRouter => write!(f, "openrouter"),
            TemplateType::BeeApi => write!(f, "beeapi"),
            TemplateType::Day77 => write!(f, "day77"),
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
        TemplateType::AnyRouter,
        TemplateType::OpenRouter,
        TemplateType::BeeApi,
        TemplateType::Day77,
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
        TemplateType::AnyRouter => {
            // Check if specific region was requested
            match input.to_lowercase().as_str() {
                "anyrouter-china" | "anyrouter-fast" | "anyr-china" | "anyr-fast" | "ar-china"
                | "ar-fast" => Box::new(anyrouter::AnyRouterTemplate::china()),
                "anyrouter-fallback" | "anyrouter-stable" | "anyr-fallback" | "anyr-stable"
                | "ar-fallback" | "ar-stable" => Box::new(anyrouter::AnyRouterTemplate::fallback()),
                _ => Box::new(anyrouter::AnyRouterTemplate::china()), // Default to China for fast access
            }
        }
        TemplateType::OpenRouter => Box::new(openrouter::OpenRouterTemplate::with_model(
            "anthropic/claude-3.5-sonnet",
        )),
        TemplateType::BeeApi => Box::new(beeapi::BeeApiTemplate),
        TemplateType::Day77 => Box::new(day77::Day77Template),
    }
}

/// Get a template instance by type (for backward compatibility)
pub fn get_template_instance(template_type: &TemplateType) -> Box<dyn Template> {
    get_template_instance_with_input(template_type, "")
}

/// Resolve a template instance in CLI (non-interactive) mode
/// Returns an error if the target is generic and requires variant selection
pub fn resolve_template_cli(
    template_type: &TemplateType,
    target: &str,
) -> Result<Box<dyn Template>> {
    let initial = get_template_instance_with_input(template_type, target);

    if !initial.has_variants() || !is_generic_target(target) {
        return Ok(initial);
    }

    // Generic target - suggest specific aliases
    let suggestions = match template_type {
        TemplateType::Zai => "Use 'zai-china' or 'zai-international'",
        TemplateType::KatCoder => "Use 'kat-coder-pro' or 'kat-coder-air'",
        TemplateType::Kimi => "Use 'k2', 'k2-thinking', or 'moonshot'",
        TemplateType::AnyRouter => "Use 'anyr-china' or 'anyr-fallback'",
        TemplateType::OpenRouter => "Specify a model directly or use interactive mode",
        _ => "Use a specific variant name",
    };

    Err(anyhow::anyhow!(
        "CLI mode requires a specific variant for '{}'. {}",
        target,
        suggestions
    ))
}

/// Resolve a template instance, prompting for variant selection if needed
pub fn resolve_template_interactive(
    template_type: &TemplateType,
    target: &str,
) -> Result<Box<dyn Template>> {
    let initial = get_template_instance_with_input(template_type, target);

    // If no variants or target specifies a particular variant, use as-is
    if !initial.has_variants() || !is_generic_target(target) {
        return Ok(initial);
    }

    // Generic target with variants - prompt for selection
    match template_type {
        TemplateType::KatCoder => Ok(Box::new(
            kat_coder::KatCoderTemplate::create_interactively()?
        )),
        TemplateType::Kimi => Ok(Box::new(kimi::KimiTemplate::create_interactively()?)),
        TemplateType::Zai => Ok(Box::new(zai::ZaiTemplate::create_interactively()?)),
        TemplateType::AnyRouter => Ok(Box::new(
            anyrouter::AnyRouterTemplate::create_interactively()?,
        )),
        TemplateType::OpenRouter => Ok(Box::new(
            openrouter::OpenRouterTemplate::create_with_model_selection()?,
        )),
        _ => Ok(initial),
    }
}

/// Check if target is a generic name (no specific variant specified)
pub fn is_generic_target(target: &str) -> bool {
    matches!(
        target.to_lowercase().as_str(),
        "kat-coder"
            | "katcoder"
            | "kat"
            | "kimi"
            | "minimax"
            | "zai"
            | "glm"
            | "zhipu"
            | "anyrouter"
            | "anyr"
            | "ar"
            | "openrouter"
            | "or"
    )
}

/// Concrete variant aliases for a template that offers region/variant choice.
/// Each tuple is `(canonical_input_alias, display_label)`. The alias is
/// reconstructable via [`get_template_instance_with_input`] and storable in
/// prefs. Empty for templates with no interactive variants.
pub fn variant_options(template_type: &TemplateType) -> Vec<(&'static str, &'static str)> {
    match template_type {
        TemplateType::Zai => vec![
            ("zai-china", "ZAI China (智谱AI)"),
            ("zai-international", "ZAI International"),
        ],
        TemplateType::KatCoder => vec![
            ("kat-coder-pro", "KatCoder Pro"),
            ("kat-coder-air", "KatCoder Air"),
        ],
        TemplateType::Kimi => vec![
            ("k2", "K2"),
            ("k2-thinking", "K2 Thinking"),
            ("kimi", "Kimi For Coding"),
        ],
        TemplateType::MiniMax => vec![
            ("minimax", "MiniMax China"),
            ("minimax-international", "MiniMax International"),
        ],
        // OpenRouter is model-based (handled by create_with_model_selection);
        // Day77 and the rest have no interactive variants.
        _ => vec![],
    }
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
        TemplateType::AnyRouter => create_anyrouter_template,
        TemplateType::OpenRouter => create_openrouter_template,
        TemplateType::BeeApi => create_beeapi_template,
        TemplateType::Day77 => create_day77_template,
    }
}

// Import all template modules
pub mod anyrouter;
pub mod beeapi;
pub mod day77;
pub mod deepseek;
pub mod duojie;
pub mod fishtrip;
pub mod kat_coder;
pub mod kimi; // Unified module for all Moonshot services
pub mod longcat;
pub mod minimax;
pub mod openrouter;
pub mod seed_code;
pub mod zai;
pub mod zenmux;

// Re-export for backward compatibility
pub use anyrouter::*;
pub use beeapi::*;
pub use day77::*;
pub use deepseek::*;
pub use duojie::*;
pub use fishtrip::*;
pub use kat_coder::*;
pub use kimi::*; // Includes legacy k2 functions
pub use longcat::*;
pub use minimax::*;
pub use openrouter::*;
pub use seed_code::*;
pub use zai::*;
pub use zenmux::*;
