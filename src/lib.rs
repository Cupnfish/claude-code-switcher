//! Claude Code Switcher - Modular CLI tool for managing Claude Code settings
//!
//! This library provides a modular architecture for managing Claude Code settings
//! across multiple AI providers through templates and snapshots.

pub mod cli;
pub mod commands;
pub mod confirm_selector;
pub mod credentials;
pub mod settings;
pub mod simple_selector;
pub mod snapshots;
pub mod templates;
pub mod utils;

// Re-export key types for convenience
pub use cli::{Cli, Commands, CredentialCommands};
pub use commands::run_command;
pub use credentials::{
    CredentialStore, SavedCredential, SavedCredentialStore, get_api_key_interactively,
};
pub use settings::{
    ClaudeSettings, Hooks, Permissions, StatusLine, format_settings_for_display, merge_settings,
};
pub use snapshots::{Snapshot, SnapshotScope, SnapshotStore};
pub use templates::{TemplateType, get_all_templates, get_template, get_template_type};
pub use utils::{get_credentials_dir, get_snapshots_dir};

// Core traits for abstraction
pub trait Configurable: Sized {
    /// Merge this configuration with another, with priority given to self
    fn merge_with(self, other: Self) -> Self;

    /// Filter settings by the specified scope
    fn filter_by_scope(self, scope: &SnapshotScope) -> Self;

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
        template_type: TemplateType,
    ) -> anyhow::Result<()>;

    /// Load all stored credentials
    fn load_credentials(&self) -> anyhow::Result<Vec<SavedCredential>>;

    /// Delete a credential by ID
    fn delete_credential(&self, id: &str) -> anyhow::Result<()>;

    /// Clear all credentials
    fn clear_credentials(&self) -> anyhow::Result<()>;
}
