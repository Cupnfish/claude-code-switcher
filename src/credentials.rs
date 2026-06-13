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
use std::collections::HashSet;
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

use crate::prefs::KeyRef;
use crate::templates::TemplateType;
use crate::CredentialManager;

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
    /// Last usage timestamp in UTC (None if never used)
    pub last_used_at: Option<String>,
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
            last_used_at: None,
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
            last_used_at: None,
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

    /// Get last usage timestamp
    pub fn last_used_at(&self) -> Option<&str> {
        self.last_used_at.as_deref()
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

    /// Get a specific metadata value
    pub fn get_metadata(&self, key: &str) -> Option<String> {
        self.metadata.as_ref()?.get(key).cloned()
    }

    /// Set a specific metadata value
    pub fn set_metadata_value(&mut self, key: String, value: String) {
        if let Some(ref mut metadata) = self.metadata {
            metadata.insert(key, value);
        } else {
            let mut new_metadata = std::collections::HashMap::new();
            new_metadata.insert(key, value);
            self.metadata = Some(new_metadata);
        }
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

    /// Generate a smart credential name with auto-incrementing numbers
    pub fn generate_smart_name(
        &self,
        template_type: &TemplateType,
        base_name: Option<&str>,
    ) -> Result<String> {
        let binding = template_type.to_string();
        let base = base_name.unwrap_or(&binding);

        // Get existing credentials for this template type
        let existing_credentials = self
            .store
            .find_by_template_type(template_type)
            .unwrap_or_default();

        // Extract existing names and find the highest number
        let mut max_number = 0;
        let mut has_base_name = false;

        for credential in &existing_credentials {
            let name = credential.name();
            if name.starts_with(base) {
                has_base_name = true;
                // Extract number if it exists (e.g., "deepseek-2" -> 2)
                if let Some(number_part) = name.strip_prefix(&format!("{}-", base))
                    && let Ok(number) = number_part.parse::<u32>()
                {
                    max_number = max_number.max(number);
                }
            }
        }

        // Generate name with auto-incrementing number if base already exists
        if has_base_name {
            Ok(format!("{}-{}", base, max_number + 1))
        } else {
            Ok(base.to_string())
        }
    }

    /// Create and save a new credential with smart naming
    pub fn create_credential_smart(
        &self,
        api_key: &str,
        template_type: TemplateType,
        custom_name: Option<&str>,
    ) -> Result<SavedCredential> {
        let name = if let Some(custom_name) = custom_name {
            custom_name.to_string()
        } else {
            self.generate_smart_name(&template_type, None)?
        };

        self.create_credential(name, api_key, template_type)
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

    /// Check if API key already exists for this template type
    pub fn has_api_key(&self, api_key: &str, template_type: &TemplateType) -> bool {
        if let Ok(credentials) = self.store.find_by_template_type(template_type) {
            for credential in credentials {
                if credential.api_key() == api_key {
                    return true;
                }
            }
        }
        false
    }

    /// Get saved endpoint IDs for a template type (from credential metadata)
    pub fn get_endpoint_ids(&self, template_type: &TemplateType) -> Vec<(String, String)> {
        let mut endpoint_ids = Vec::new();
        if let Ok(credentials) = self.store.find_by_template_type(template_type) {
            for credential in credentials {
                if let Some(endpoint_id) = credential.get_metadata("endpoint_id") {
                    let name = format!("{} - {}", credential.name(), endpoint_id);
                    endpoint_ids.push((name, endpoint_id));
                }
            }
        }
        endpoint_ids
    }

    /// Save endpoint ID to credential metadata
    pub fn save_endpoint_id(&self, credential_id: &str, endpoint_id: &str) -> Result<()> {
        let mut credential = self.store.load(credential_id)?;
        credential.set_metadata_value("endpoint_id".to_string(), endpoint_id.to_string());
        self.store.save(&credential)?;
        Ok(())
    }

    /// Check if endpoint ID exists
    pub fn has_endpoint_id(&self, endpoint_id: &str, template_type: &TemplateType) -> bool {
        if let Ok(credentials) = self.store.find_by_template_type(template_type) {
            for credential in credentials {
                if let Some(saved_endpoint) = credential.get_metadata("endpoint_id")
                    && saved_endpoint == endpoint_id
                {
                    return true;
                }
            }
        }
        false
    }

    /// Update credential name
    pub fn update_name(&self, credential_id: &str, new_name: String) -> Result<()> {
        let mut credential = self.store.load(credential_id)?;
        credential.name = new_name;
        credential.update_timestamp();
        self.store.save(&credential)?;
        Ok(())
    }

    /// Update last_used_at timestamp for a credential
    pub fn touch_last_used(&self, credential_id: &str) -> Result<()> {
        let mut credential = self.store.load(credential_id)?;
        credential.last_used_at = Some(Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string());
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
            self.store.delete(credential.id())?;
        }
        Ok(())
    }
}

// ── API key acquisition for `apply` ──────────────────────────────────────────

/// A resolved API key plus a reference to its source (so the caller can
/// remember it across runs).
#[derive(Debug, Clone)]
pub struct ApiKeyChoice {
    /// The API key to use.
    pub key: String,
    /// Where the key came from, for remembering in prefs (or `None` for a
    /// one-off `--api-key` / unsaved entry).
    pub source: Option<KeyRef>,
}

/// A selectable API key source (env var or saved credential).
#[derive(Debug, Clone)]
pub enum ApiKeySource {
    /// API key read from an environment variable.
    EnvVar { env_var_name: String, api_key: String },
    /// API key from a saved credential.
    Saved { credential: SavedCredential },
}

impl ApiKeySource {
    pub fn api_key(&self) -> &str {
        match self {
            ApiKeySource::EnvVar { api_key, .. } => api_key,
            ApiKeySource::Saved { credential } => credential.api_key(),
        }
    }

    pub fn to_key_ref(&self) -> KeyRef {
        match self {
            ApiKeySource::EnvVar { env_var_name, .. } => KeyRef::EnvVar(env_var_name.clone()),
            ApiKeySource::Saved { credential } => KeyRef::Credential(credential.id().to_string()),
        }
    }

    pub fn display(&self) -> String {
        match self {
            ApiKeySource::EnvVar {
                env_var_name,
                api_key,
            } => format!("🌐 {} = {}", env_var_name, mask_api_key(api_key)),
            ApiKeySource::Saved { credential } => format!(
                "🔑 {} ({}) - {}",
                credential.name(),
                credential.template_type(),
                mask_api_key(credential.api_key())
            ),
        }
    }
}

/// Collect unified API key sources (env vars + saved credentials) for a
/// template, sorted by last usage and de-duplicated. Env var keys win.
pub fn collect_api_key_sources(template_type: &TemplateType) -> Result<Vec<ApiKeySource>> {
    let mut sources: Vec<ApiKeySource> = Vec::new();

    // 1. environment variables
    let env_var_names = crate::templates::get_env_var_names(template_type);
    for env_var_name in &env_var_names {
        if let Some(api_key) = std::env::var(env_var_name)
            .ok()
            .filter(|key| !key.trim().is_empty())
        {
            sources.push(ApiKeySource::EnvVar {
                env_var_name: env_var_name.to_string(),
                api_key,
            });
        }
    }

    // 2. saved credentials for this template type
    if let Ok(store) = CredentialStore::new()
        && let Ok(all) = store.load_credentials()
    {
        for credential in all.into_iter().filter(|c| c.template_type() == template_type) {
            sources.push(ApiKeySource::Saved { credential });
        }
    }

    // 3. sort: used creds (last_used desc) → env vars → unused creds (created desc)
    sources.sort_by(|a, b| {
        let priority = |s: &ApiKeySource| match s {
            ApiKeySource::Saved { credential } => match credential.last_used_at() {
                Some(ts) => (2u8, ts.to_string()),
                None => (0u8, credential.created_at().to_string()),
            },
            ApiKeySource::EnvVar { .. } => (1u8, String::new()),
        };
        priority(b).cmp(&priority(a))
    });

    // 4. dedup: env var keys win; also dedup saved credentials by api_key
    let env_keys: HashSet<String> = sources
        .iter()
        .filter_map(|s| match s {
            ApiKeySource::EnvVar { api_key, .. } => Some(api_key.clone()),
            _ => None,
        })
        .collect();

    let mut seen = env_keys;
    let mut deduped = Vec::new();
    for source in sources {
        match &source {
            ApiKeySource::EnvVar { .. } => deduped.push(source),
            ApiKeySource::Saved { credential } => {
                if seen.insert(credential.api_key().to_string()) {
                    deduped.push(source);
                }
            }
        }
    }

    Ok(deduped)
}

/// Find a source matching a remembered [`KeyRef`].
fn find_source_by_ref<'a>(
    sources: &'a [ApiKeySource],
    key_ref: &KeyRef,
) -> Option<&'a ApiKeySource> {
    sources.iter().find(|s| match (s, key_ref) {
        (ApiKeySource::EnvVar { env_var_name, .. }, KeyRef::EnvVar(name)) => {
            env_var_name == name
        }
        (ApiKeySource::Saved { credential }, KeyRef::Credential(id)) => credential.id() == id,
        _ => false,
    })
}

/// Prompt the user to pick an API key source (or enter a new one).
fn prompt_api_key_choice(
    template_type: &TemplateType,
    sources: &[ApiKeySource],
) -> Result<Option<ApiKeyChoice>> {
    let mut options: Vec<String> = sources.iter().map(|s| s.display()).collect();
    options.push("➕ Enter a new API key...".to_string());

    let title = format!("Select {} API key:", template_type);
    let selection = match Select::new(&title, options.clone())
        .with_help_message("↑/↓ navigate, type to filter, Enter select, Esc cancel")
        .prompt()
    {
        Ok(s) => s,
        Err(inquire::InquireError::OperationCanceled)
        | Err(inquire::InquireError::OperationInterrupted) => return Ok(None),
        Err(e) => return Err(anyhow!("Selection failed: {}", e)),
    };

    if selection == "➕ Enter a new API key..." {
        return prompt_new_api_key_choice(template_type);
    }

    let index = options
        .iter()
        .position(|o| o == &selection)
        .ok_or_else(|| anyhow!("Selected source not found"))?;
    let source = &sources[index];

    if let ApiKeySource::Saved { credential } = source
        && let Ok(store) = CredentialStore::new()
    {
        let _ = store.touch_last_used(credential.id());
    }

    Ok(Some(ApiKeyChoice {
        key: source.api_key().to_string(),
        source: Some(source.to_key_ref()),
    }))
}

/// Prompt the user to enter a new API key, optionally saving it.
fn prompt_new_api_key_choice(template_type: &TemplateType) -> Result<Option<ApiKeyChoice>> {
    let template_instance = crate::templates::get_template_instance(template_type);
    println!();
    println!("🔑 Create new API key");
    if let Some(url) = template_instance.api_key_url() {
        println!("  💡 Get your API key from: {}", url);
    }

    let api_key = match Text::new(&format!("Enter your {} API key:", template_type))
        .with_placeholder("sk-...")
        .prompt()
    {
        Ok(s) => s.trim().to_string(),
        Err(inquire::InquireError::OperationCanceled)
        | Err(inquire::InquireError::OperationInterrupted) => return Ok(None),
        Err(e) => return Err(anyhow!("Input failed: {}", e)),
    };
    if api_key.is_empty() {
        return Err(anyhow!("API key cannot be empty"));
    }

    let source = save_credential_if_desired(template_type, &api_key)?;
    Ok(Some(ApiKeyChoice { key: api_key, source }))
}

/// Offer to save a freshly-entered key as a credential. Returns its [`KeyRef`]
/// if saved.
fn save_credential_if_desired(
    template_type: &TemplateType,
    api_key: &str,
) -> Result<Option<KeyRef>> {
    if let Ok(store) = CredentialStore::new()
        && store.has_api_key(api_key, template_type)
    {
        return Ok(None); // already stored
    }

    let should_save = match Confirm::new("Save this API key for future use?")
        .with_default(true)
        .prompt()
    {
        Ok(v) => v,
        Err(_) => return Ok(None),
    };
    if !should_save {
        return Ok(None);
    }

    let default_name = format!("{} API Key", template_type);
    let name = Text::new("Save as (alias):")
        .with_default(&default_name)
        .with_help_message(format!("Alias for {}", mask_api_key(api_key)).as_str())
        .prompt()
        .unwrap_or(default_name);
    let name = name.trim().to_string();

    if let Ok(store) = CredentialStore::new() {
        let cred = store.create_credential(name, api_key, template_type.clone())?;
        println!("✓ API key saved.");
        return Ok(Some(KeyRef::Credential(cred.id().to_string())));
    }
    Ok(None)
}

/// Resolve an API key for applying a template.
///
/// Decision order: explicit `api_key_param` → a remembered source that still
/// exists → a single available source → interactive prompt. Returns the key and
/// its source so the caller can remember it. `Ok(None)` means the user
/// cancelled. In `non_interactive` mode this never prompts and errors if no key
/// is available.
pub fn resolve_api_key(
    template_type: &TemplateType,
    api_key_param: Option<&str>,
    remembered: Option<&KeyRef>,
    force_prompt: bool,
    non_interactive: bool,
) -> Result<Option<ApiKeyChoice>> {
    // explicit flag always wins
    if let Some(key) = api_key_param.map(str::trim).filter(|k| !k.is_empty()) {
        return Ok(Some(ApiKeyChoice {
            key: key.to_string(),
            source: None,
        }));
    }

    let sources = collect_api_key_sources(template_type)?;

    if !force_prompt {
        // remembered source still present?
        if let Some(kr) = remembered
            && let Some(src) = find_source_by_ref(&sources, kr)
        {
            return Ok(Some(ApiKeyChoice {
                key: src.api_key().to_string(),
                source: Some(src.to_key_ref()),
            }));
        }

        // exactly one source → use silently
        if sources.len() == 1 {
            let src = &sources[0];
            if let ApiKeySource::Saved { credential } = src
                && let Ok(store) = CredentialStore::new()
            {
                let _ = store.touch_last_used(credential.id());
            }
            return Ok(Some(ApiKeyChoice {
                key: src.api_key().to_string(),
                source: Some(src.to_key_ref()),
            }));
        }
    }

    // otherwise we need a prompt
    if non_interactive {
        let env_var_names = crate::templates::get_env_var_names(template_type);
        return Err(anyhow!(
            "No API key available in non-interactive mode. Set one of: {} or use --api-key",
            env_var_names.join(", ")
        ));
    }

    if sources.is_empty() {
        prompt_new_api_key_choice(template_type)
    } else {
        prompt_api_key_choice(template_type, &sources)
    }
}

/// Mask API key for display (show first 4 and last 4 characters)
pub(crate) fn mask_api_key(api_key: &str) -> String {
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

        let loaded = store.store.load(credential.id()).unwrap();
        assert_eq!(credential.name(), loaded.name());
        assert_eq!(credential.api_key(), loaded.api_key());
    }

    #[test]
    fn test_mask_api_key() {
        assert_eq!(mask_api_key("sk-1234567890"), "sk-1•••••7890");
        assert_eq!(mask_api_key("short"), "••••••••");
    }
}
