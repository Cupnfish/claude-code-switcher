//! Claude Code Switcher - Modular CLI tool for managing Claude Code settings
//!
//! This binary provides a modular architecture for managing Claude Code settings
//! across multiple AI providers through templates and snapshots.

use clap::Parser as _;

use crate::cli::Cli;

pub mod cli;
pub mod commands;
pub mod credentials;
pub mod selectors;
pub mod settings;
pub mod snapshots;
pub mod templates;
pub mod utils;

// Core traits for abstraction
pub trait Configurable: Sized {
    /// Merge this configuration with another, with priority given to self
    fn merge_with(self, other: Self) -> Self;

    /// Filter settings by the specified scope
    fn filter_by_scope(self, scope: &snapshots::SnapshotScope) -> Self;

    /// Mask sensitive data for display purposes
    fn mask_sensitive_data(self) -> Self;
}

pub trait Storage<T>: Send + Sync {
    /// Load data from storage
    fn load(&self) -> anyhow::Result<T>;

    /// Save data to storage
    fn save(&self, data: &T) -> anyhow::Result<()>;

    /// Get the storage path
    fn path(&self) -> std::path::PathBuf;
}

pub trait CredentialManager: Send + Sync {
    /// Save a credential
    fn save_credential(
        &self,
        name: String,
        api_key: &str,
        template_type: templates::TemplateType,
    ) -> anyhow::Result<()>;

    /// Load all stored credentials
    fn load_credentials(&self) -> anyhow::Result<Vec<credentials::SavedCredential>>;

    /// Delete a credential by ID
    fn delete_credential(&self, id: &str) -> anyhow::Result<()>;

    /// Clear all credentials
    fn clear_credentials(&self) -> anyhow::Result<()>;
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    // Run the command
    commands::run_command(&cli)?;

    Ok(())
}

// Tests moved here since we no longer have a library
#[cfg(test)]
mod tests {
    use crate::{selectors::SelectorError, templates::TemplateType};

    use super::*;

    // Basic unit tests for core functionality
    #[test]
    fn test_template_type_display() {
        assert_eq!(format!("{}", TemplateType::DeepSeek), "deepseek");
        assert_eq!(format!("{}", TemplateType::Zai), "zai");
        assert_eq!(format!("{}", TemplateType::KatCoder), "kat-coder");
        assert_eq!(format!("{}", TemplateType::Fishtrip), "fishtrip");
        assert_eq!(format!("{}", TemplateType::Duojie), "duojie");
    }

    #[test]
    fn test_selector_error_creation() {
        let cancelled_error = SelectorError::Cancelled;
        assert!(cancelled_error.is_cancellation());

        let not_found_error = SelectorError::not_found();
        assert!(!not_found_error.is_cancellation());

        let failed_error = SelectorError::failed("Something went wrong");
        assert!(!failed_error.is_cancellation());
        assert!(failed_error.to_string().contains("Something went wrong"));
    }

    #[test]
    fn test_credential_creation() {
        use credentials::CredentialData;

        let cred = CredentialData::new(
            "test-credential".to_string(),
            "sk-test123".to_string(),
            TemplateType::DeepSeek,
        );

        assert_eq!(cred.name(), "test-credential");
        assert_eq!(cred.api_key(), "sk-test123");
        assert_eq!(cred.template_type(), &TemplateType::DeepSeek);
        assert!(!cred.created_at().is_empty());
        assert!(!cred.updated_at().is_empty());
    }

    #[test]
    fn test_mask_api_key() {
        let short_key = "short";
        let long_key = "sk-thisisaverylongapikeyfortesting";

        // Test the masking function (we can add this to utils if needed)
        let masked_short = if short_key.len() <= 8 {
            "••••••••".to_string()
        } else {
            format!("{}••••••••", &short_key[..short_key.len().min(8)])
        };

        let masked_long = if long_key.len() <= 8 {
            "••••••••".to_string()
        } else {
            format!("{}••••••••", &long_key[..long_key.len().min(8)])
        };

        assert_eq!(masked_short, "••••••••");
        assert_eq!(masked_long, "sk-thisi••••••••");
    }

    #[test]
    fn test_snapshot_scope_display() {
        assert_eq!(format!("{}", snapshots::SnapshotScope::Env), "env");
        assert_eq!(format!("{}", snapshots::SnapshotScope::Common), "common");
        assert_eq!(format!("{}", snapshots::SnapshotScope::All), "all");
    }

    #[test]
    fn test_template_type_parsing() {
        assert_eq!(
            "deepseek".parse::<TemplateType>().unwrap(),
            TemplateType::DeepSeek
        );
        assert_eq!("zai".parse::<TemplateType>().unwrap(), TemplateType::Zai);
        assert_eq!(
            "kat-coder".parse::<TemplateType>().unwrap(),
            TemplateType::KatCoder
        );
        assert_eq!("kimi".parse::<TemplateType>().unwrap(), TemplateType::Kimi);
        assert_eq!(
            "fishtrip".parse::<TemplateType>().unwrap(),
            TemplateType::Fishtrip
        );
        assert_eq!(
            "fish".parse::<TemplateType>().unwrap(),
            TemplateType::Fishtrip
        );
        assert_eq!(
            "duojie".parse::<TemplateType>().unwrap(),
            TemplateType::Duojie
        );
        assert_eq!("dj".parse::<TemplateType>().unwrap(), TemplateType::Duojie);
    }
}
