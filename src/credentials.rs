//! Credential management module for Claude Code Switcher
//!
//! This module provides functionality to save and retrieve API keys for different AI providers.
//! Credentials are stored in plain text since they're typically managed through environment variables.
//!
//! Version management strategy:
//! - Current version: v2 (simplified from previous encryption-based approach)
//! - Future versions should increment the version number when format changes are needed

use anyhow::{Result, anyhow};
use chrono::Utc;
use inquire::{Confirm, Select, Text};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::TemplateType;

/// Current credential data format version
pub const CURRENT_CREDENTIAL_VERSION: &str = "v2";

/// Core credential data structure
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CredentialData {
    /// Data format version for compatibility
    pub version: String,
    /// Unique identifier for the credential
    pub id: String,
    /// User-friendly name for the credential
    pub name: String,
    /// API key in plain text
    pub api_key: String,
    /// Template type this credential is associated with
    pub template_type: TemplateType,
    /// Creation timestamp in UTC
    pub created_at: String,
    /// Last update timestamp in UTC
    pub updated_at: String,
    /// Optional metadata for future extensibility
    pub metadata: Option<std::collections::HashMap<String, String>>,
}

impl Default for CredentialData {
    fn default() -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        Self {
            version: CURRENT_CREDENTIAL_VERSION.to_string(),
            id: Uuid::new_v4().to_string(),
            name: String::new(),
            api_key: String::new(),
            template_type: TemplateType::KatCoder,
            created_at: now.clone(),
            updated_at: now,
            metadata: None,
        }
    }
}

impl CredentialData {
    /// Create a new credential
    pub fn new(name: String, api_key: String, template_type: TemplateType) -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        Self {
            version: CURRENT_CREDENTIAL_VERSION.to_string(),
            id: Uuid::new_v4().to_string(),
            name,
            api_key,
            template_type,
            created_at: now.clone(),
            updated_at: now,
            metadata: None,
        }
    }

    /// Update the timestamp to current time
    pub fn update_timestamp(&mut self) {
        self.updated_at = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    }

    /// Get credential ID
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Get credential name
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get API key
    pub fn api_key(&self) -> &str {
        &self.api_key
    }

    /// Get template type
    pub fn template_type(&self) -> &TemplateType {
        &self.template_type
    }

    /// Get creation timestamp
    pub fn created_at(&self) -> &str {
        &self.created_at
    }

    /// Get update timestamp
    pub fn updated_at(&self) -> &str {
        &self.updated_at
    }

    /// Get metadata
    pub fn metadata(&self) -> Option<&std::collections::HashMap<String, String>> {
        self.metadata.as_ref()
    }

    /// Update metadata
    pub fn set_metadata(&mut self, metadata: std::collections::HashMap<String, String>) {
        self.metadata = Some(metadata);
        self.update_timestamp();
    }
}

/// Result type for credential operations
pub type SavedCredential = CredentialData;

/// Storage backend for credential files
pub struct SavedCredentialStore {
    pub credentials_dir: PathBuf,
}

impl SavedCredentialStore {
    /// Create a new credential store with default directory
    pub fn new() -> Result<Self> {
        let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
        let credentials_dir = home_dir.join(".claude").join("credentials");

        let store = Self { credentials_dir };
        store.ensure_dir()?;
        Ok(store)
    }

    /// Create a new credential store with custom directory (for backward compatibility)
    pub fn new_with_dir(credentials_dir: PathBuf) -> Self {
        Self { credentials_dir }
    }

    /// Ensure the credentials directory exists
    pub fn ensure_dir(&self) -> Result<()> {
        if !self.credentials_dir.exists() {
            fs::create_dir_all(&self.credentials_dir)
                .map_err(|e| anyhow!("Failed to create credentials directory: {}", e))?;
        }
        Ok(())
    }

    /// Get the file path for a credential
    pub fn credential_path(&self, credential_id: &str) -> PathBuf {
        self.credentials_dir.join(format!("{}.json", credential_id))
    }

    /// Save a credential to disk
    pub fn save(&self, credential: &CredentialData) -> Result<()> {
        self.ensure_dir()?;
        let path = self.credential_path(&credential.id);

        let content = serde_json::to_string_pretty(credential)
            .map_err(|e| anyhow!("Failed to serialize credential: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| anyhow!("Failed to write credential file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Load a credential from disk
    pub fn load(&self, credential_id: &str) -> Result<SavedCredential> {
        let path = self.credential_path(credential_id);

        if !path.exists() {
            return Err(anyhow!("Credential '{}' not found", credential_id));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read credential file {}: {}", path.display(), e))?;

        // Parse as current format
        serde_json::from_str::<CredentialData>(&content)
            .map_err(|e| anyhow!("Failed to parse credential file {}: {}", path.display(), e))
    }

    /// List all saved credentials
    pub fn list(&self) -> Result<Vec<SavedCredential>> {
        self.ensure_dir()?;

        let mut credentials = Vec::new();

        let entries = fs::read_dir(&self.credentials_dir)
            .map_err(|e| anyhow!("Failed to read credentials directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let credential_id = path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .ok_or_else(|| anyhow!("Invalid credential file name: {}", path.display()))?;

                match self.load(credential_id) {
                    Ok(credential) => credentials.push(credential),
                    Err(e) => {
                        // Log the error but continue loading other credentials
                        eprintln!(
                            "Warning: Failed to load credential '{}': {}",
                            credential_id, e
                        );
                    }
                }
            }
        }

        // Sort by creation time (newest first)
        credentials.sort_by(|a, b| b.created_at().cmp(a.created_at()));

        Ok(credentials)
    }

    /// Delete a credential
    pub fn delete(&self, credential_id: &str) -> Result<()> {
        let path = self.credential_path(credential_id);

        if !path.exists() {
            return Err(anyhow!("Credential '{}' not found", credential_id));
        }

        fs::remove_file(&path)
            .map_err(|e| anyhow!("Failed to delete credential file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Check if a credential exists
    pub fn exists(&self, credential_id: &str) -> bool {
        self.credential_path(credential_id).exists()
    }

    /// Get all credential names
    pub fn list_names(&self) -> Result<Vec<String>> {
        let credentials = self.list()?;
        Ok(credentials
            .into_iter()
            .map(|c| c.name().to_string())
            .collect())
    }

    /// Find credentials by template type
    pub fn find_by_template_type(
        &self,
        template_type: &TemplateType,
    ) -> Result<Vec<SavedCredential>> {
        let credentials = self.list()?;
        Ok(credentials
            .into_iter()
            .filter(|c| c.template_type() == template_type)
            .collect())
    }
}

/// High-level credential management
pub struct CredentialStore {
    pub store: SavedCredentialStore,
}

impl CredentialStore {
    /// Create a new credential store
    pub fn new() -> Result<Self> {
        Ok(Self {
            store: SavedCredentialStore::new()?,
        })
    }

    /// Create and save a new credential
    pub fn create_credential(
        &self,
        name: String,
        api_key: &str,
        template_type: TemplateType,
    ) -> Result<SavedCredential> {
        let credential = CredentialData::new(name, api_key.to_string(), template_type);
        self.store.save(&credential)?;
        Ok(credential)
    }

    /// Get the API key from a credential
    pub fn get_api_key(&self, credential: &SavedCredential) -> Result<String> {
        Ok(credential.api_key().to_string())
    }

    /// Update credential name
    pub fn update_name(&self, credential_id: &str, new_name: String) -> Result<()> {
        let mut credential = self.store.load(credential_id)?;
        credential.name = new_name;
        credential.update_timestamp();
        self.store.save(&credential)?;
        Ok(())
    }

    /// Update credential metadata
    pub fn update_metadata(
        &self,
        credential_id: &str,
        metadata: std::collections::HashMap<String, String>,
    ) -> Result<()> {
        let mut credential = self.store.load(credential_id)?;
        credential.set_metadata(metadata);
        self.store.save(&credential)?;
        Ok(())
    }
}

impl crate::CredentialManager for CredentialStore {
    fn save_credential(
        &self,
        name: String,
        api_key: &str,
        template_type: TemplateType,
    ) -> Result<()> {
        self.create_credential(name, api_key, template_type)?;
        Ok(())
    }

    fn load_credentials(&self) -> Result<Vec<SavedCredential>> {
        self.store.list()
    }

    fn delete_credential(&self, credential_id: &str) -> Result<()> {
        self.store.delete(credential_id)
    }

    fn clear_credentials(&self) -> Result<()> {
        let credentials = self.store.list()?;
        for credential in credentials {
            self.store.delete(&credential.id())?;
        }
        Ok(())
    }
}

/// Helper function to select a credential from a list
pub fn select_credential<'a>(
    credentials: &'a [SavedCredential],
    message: &str,
) -> Result<&'a SavedCredential> {
    let options: Vec<String> = credentials
        .iter()
        .map(|c| {
            format!(
                "{} ({} - {})",
                c.name(),
                c.template_type(),
                mask_api_key(c.api_key())
            )
        })
        .collect();

    let selected = Select::new(message, options.clone())
        .prompt()
        .map_err(|e| anyhow!("Failed to select credential: {}", e))?;

    let index = options.iter().position(|o| o == &selected).unwrap();
    Ok(&credentials[index])
}

/// Prompt user to save a credential interactively
pub fn prompt_save_credential(
    api_key: &str,
    template_type: TemplateType,
) -> Result<Option<SavedCredential>> {
    if let Ok(should_save) = Confirm::new("Would you like to save this API key for future use?")
        .with_default(true)
        .prompt()
    {
        if should_save {
            let name = Text::new("Enter a name for this credential:")
                .with_placeholder(&format!("{} API Key", template_type))
                .prompt()
                .map_err(|e| anyhow!("Failed to get credential name: {}", e))?;

            let store = CredentialStore::new()?;
            let credential = store.create_credential(name, api_key, template_type)?;

            println!("✓ Credential saved successfully!");
            return Ok(Some(credential));
        }
    }
    Ok(None)
}

/// Get API key interactively with option to save
pub fn get_api_key_interactively(template_type: TemplateType) -> Result<String> {
    // Try to use saved credentials first
    if let Ok(credential_store) = CredentialStore::new() {
        if let Ok(credentials) = credential_store.store.find_by_template_type(&template_type) {
            if !credentials.is_empty() {
                println!("Found saved credentials for {}:", template_type);

                for credential in &credentials {
                    println!(
                        "  • {}: {} ({})",
                        STYLE_CYAN.apply_to(credential.name()),
                        mask_api_key(credential.api_key()),
                        credential.created_at()
                    );
                }

                if let Ok(continue_use) = Confirm::new("Use one of these saved credentials?")
                    .with_default(true)
                    .prompt()
                {
                    if continue_use {
                        if let Ok(selected) =
                            select_credential(&credentials, "Select a credential:")
                        {
                            return credential_store.get_api_key(&selected);
                        }
                    }
                }
            }
        }
    }

    // If no saved credentials or user chooses not to use them, prompt for API key
    let prompt_text = format!("Enter your {} API key:", template_type);
    let api_key = Text::new(&prompt_text)
        .with_placeholder("sk-...")
        .prompt()
        .map_err(|e| anyhow!("Failed to get API key: {}", e))?;

    // Offer to save the credential
    if let Some(_) = prompt_save_credential(&api_key, template_type)? {
        // Credential was saved
    }

    Ok(api_key)
}

/// Mask API key for display (show first 4 and last 4 characters)
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        "••••••••".to_string()
    } else {
        format!(
            "{}{}{}",
            &api_key[..4],
            "•".repeat(api_key.len() - 8),
            &api_key[api_key.len() - 4..]
        )
    }
}

use console::Style;
const STYLE_CYAN: Style = Style::new().cyan();

#[cfg(test)]
mod tests {
    use super::*;
    fn create_test_store() -> CredentialStore {
        let temp_dir = std::env::temp_dir().join("ccs_test");
        let store = SavedCredentialStore {
            credentials_dir: temp_dir,
        };
        CredentialStore { store }
    }

    #[test]
    fn test_credential_creation() {
        let credential = CredentialData::new(
            "test".to_string(),
            "test-key".to_string(),
            TemplateType::KatCoder,
        );

        assert_eq!(credential.name(), "test");
        assert_eq!(credential.api_key(), "test-key");
        assert_eq!(credential.version, CURRENT_CREDENTIAL_VERSION);
    }

    #[test]
    fn test_credential_save_and_load() {
        let store = create_test_store();

        let credential = store
            .create_credential("test".to_string(), "test-key", TemplateType::KatCoder)
            .unwrap();

        let loaded = store.store.load(&credential.id()).unwrap();
        assert_eq!(credential.name(), loaded.name());
        assert_eq!(credential.api_key(), loaded.api_key());
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-1234567890"), "sk-1•••••7890");
        assert_eq!(mask_api_key("short"), "••••••••");
    }
}
