//! Persistent user preferences for Claude Code Switcher.
//!
//! Remembers per-template choices (variant, last-used key) and global defaults
//! (scope, effort, co-author) so the common `apply` path can be zero-prompt:
//! `ccs apply zai` for a returning user applies instantly with no prompts.

use anyhow::{Result, anyhow};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

use crate::snapshots::SnapshotScope;
use crate::templates::TemplateType;

/// Current prefs data-format version.
pub const PREFS_VERSION: &str = "v1";

/// Reference to a previously-selected API key source.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum KeyRef {
    /// A saved credential id (under `~/.claude/credentials/`).
    Credential(String),
    /// An environment variable name.
    EnvVar(String),
}

/// Remembered choices for a single template type.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TemplatePref {
    /// Canonical input alias, e.g. `"zai-china"`, `"k2"`. Reconstructable via
    /// [`crate::templates::get_template_instance_with_input`].
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variant: Option<String>,

    /// Last-used key source for this template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_key: Option<KeyRef>,

    /// Last-used scope for this template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_scope: Option<SnapshotScope>,

    /// Last-used effort level for this template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_effort: Option<String>,

    /// Last-used co-author setting for this template (`true` = enabled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_co_author: Option<bool>,

    /// Timestamp of the last apply for this template.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_used_at: Option<String>,
}

/// Persistent preferences store, serialized to `~/.claude/ccs-prefs.json`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prefs {
    #[serde(default = "default_version")]
    pub version: String,

    /// Global default scope for `apply` (defaults to `Common`).
    #[serde(default)]
    pub default_scope: SnapshotScope,

    /// Global default effort level, e.g. `"max"`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_effort: Option<String>,

    /// Global default co-author setting (`false` == disabled).
    #[serde(default)]
    pub default_co_author: bool,

    /// Per-template remembered choices, keyed by `TemplateType` display string.
    #[serde(default)]
    pub templates: HashMap<String, TemplatePref>,
}

fn default_version() -> String {
    PREFS_VERSION.to_string()
}

impl Default for Prefs {
    fn default() -> Self {
        Self {
            version: PREFS_VERSION.to_string(),
            default_scope: SnapshotScope::Common,
            default_effort: None,
            default_co_author: false,
            templates: HashMap::new(),
        }
    }
}

impl Prefs {
    /// Path to the prefs file: `~/.claude/ccs-prefs.json`.
    pub fn path() -> PathBuf {
        let home = dirs::home_dir().unwrap_or_else(|| PathBuf::from("."));
        home.join(".claude").join("ccs-prefs.json")
    }

    /// Whether a prefs file exists on disk (used to trigger first-run onboarding).
    pub fn exists() -> bool {
        Self::path().exists()
    }

    /// Load prefs from disk, or return defaults if the file is missing or unreadable.
    pub fn load_or_default() -> Self {
        Self::load().unwrap_or_default()
    }

    /// Load prefs from disk (errors if the file is missing, unreadable, or malformed).
    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Err(anyhow!("prefs file not found at {}", path.display()));
        }
        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read prefs {}: {}", path.display(), e))?;
        let mut prefs: Prefs = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse prefs {}: {}", path.display(), e))?;
        if prefs.version.is_empty() {
            prefs.version = PREFS_VERSION.to_string();
        }
        Ok(prefs)
    }

    /// Save prefs to disk.
    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| anyhow!("Failed to create prefs dir {}: {}", parent.display(), e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| anyhow!("Failed to serialize prefs: {}", e))?;
        fs::write(&path, content)
            .map_err(|e| anyhow!("Failed to write prefs {}: {}", path.display(), e))?;
        Ok(())
    }

    fn key_for(template_type: &TemplateType) -> String {
        template_type.to_string()
    }

    /// Get the remembered preferences for a template type, if any.
    pub fn template_pref(&self, template_type: &TemplateType) -> Option<&TemplatePref> {
        self.templates.get(&Self::key_for(template_type))
    }

    /// Get a mutable reference to the prefs for a template type, creating an
    /// empty entry if absent.
    pub fn template_pref_mut(&mut self, template_type: &TemplateType) -> &mut TemplatePref {
        self.templates
            .entry(Self::key_for(template_type))
            .or_default()
    }

    /// Record the chosen variant alias for a template type.
    pub fn set_variant(&mut self, template_type: &TemplateType, variant: Option<String>) {
        let pref = self.template_pref_mut(template_type);
        pref.variant = variant;
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }

    /// Record the last-used key source for a template type.
    pub fn set_last_key(&mut self, template_type: &TemplateType, key: Option<KeyRef>) {
        let pref = self.template_pref_mut(template_type);
        pref.last_key = key;
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }

    /// Record the last-used scope for a template type.
    pub fn set_last_scope(&mut self, template_type: &TemplateType, scope: SnapshotScope) {
        let pref = self.template_pref_mut(template_type);
        pref.last_scope = Some(scope);
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }

    /// Record the last-used effort for a template type.
    pub fn set_last_effort(&mut self, template_type: &TemplateType, effort: Option<String>) {
        let pref = self.template_pref_mut(template_type);
        pref.last_effort = effort;
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }

    /// Record the last-used co-author setting for a template type.
    pub fn set_last_co_author(&mut self, template_type: &TemplateType, co_author: bool) {
        let pref = self.template_pref_mut(template_type);
        pref.last_co_author = Some(co_author);
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }

    /// Record everything from a completed apply in one go.
    pub fn record_apply(
        &mut self,
        template_type: &TemplateType,
        variant: Option<String>,
        key: Option<KeyRef>,
        scope: SnapshotScope,
        effort: Option<String>,
        co_author: bool,
    ) {
        let pref = self.template_pref_mut(template_type);
        pref.variant = variant;
        pref.last_key = key;
        pref.last_scope = Some(scope);
        pref.last_effort = effort;
        pref.last_co_author = Some(co_author);
        pref.last_used_at = Some(crate::utils::get_timestamp());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefs_roundtrip() {
        let mut prefs = Prefs {
            default_effort: Some("max".to_string()),
            ..Default::default()
        };
        prefs.set_variant(&TemplateType::Zai, Some("zai-china".to_string()));
        prefs.set_last_key(
            &TemplateType::Zai,
            Some(KeyRef::Credential("abc-123".to_string())),
        );

        let json = serde_json::to_string(&prefs).unwrap();
        let restored: Prefs = serde_json::from_str(&json).unwrap();

        assert_eq!(restored.default_effort.as_deref(), Some("max"));
        assert_eq!(
            restored
                .template_pref(&TemplateType::Zai)
                .unwrap()
                .variant
                .as_deref(),
            Some("zai-china")
        );
        assert_eq!(
            restored.template_pref(&TemplateType::Zai).unwrap().last_key,
            Some(KeyRef::Credential("abc-123".to_string()))
        );
    }

    #[test]
    fn test_prefs_template_absent_by_default() {
        let prefs = Prefs::default();
        assert!(prefs.template_pref(&TemplateType::DeepSeek).is_none());
    }

    #[test]
    fn test_prefs_keyref_serde_roundtrip() {
        let cases = vec![
            KeyRef::EnvVar("Z_AI_API_KEY".to_string()),
            KeyRef::Credential("cred-id".to_string()),
        ];
        for k in cases {
            let json = serde_json::to_string(&k).unwrap();
            let restored: KeyRef = serde_json::from_str(&json).unwrap();
            assert_eq!(k, restored);
        }
    }

    #[test]
    fn test_prefs_template_pref_mut_creates_entry() {
        let mut prefs = Prefs::default();
        prefs.set_variant(&TemplateType::Kimi, Some("k2".to_string()));
        assert_eq!(
            prefs.template_pref(&TemplateType::Kimi).unwrap().variant.as_deref(),
            Some("k2")
        );
    }
}
