//! Credential selector using the unified selector framework

use crate::{
    CredentialManager,
    credentials::{CredentialStore, SavedCredential, mask_api_key},
    templates::get_template_instance,
};
use crate::{
    selectors::{
        base::SelectableItem,
        confirmation::ConfirmationService,
        error::{SelectorError, SelectorResult},
    },
    templates,
};
use std::collections::HashSet;
use std::io::{self, Write};
use uuid::Uuid;

/// Action for credential management
#[derive(Debug, Clone)]
pub enum CredentialManagementAction {
    Delete(usize),
    Rename(usize),
    Back,
    Exit,
}

/// Credential selector using the unified framework
pub struct CredentialSelector {
    credentials: Vec<SavedCredential>,
}

impl CredentialSelector {
    /// Create a new credential selector with all credentials
    pub fn new_all() -> SelectorResult<Self> {
        let store = CredentialStore::new().map_err(|e| {
            SelectorError::Storage(format!("Failed to create credential store: {}", e))
        })?;
        let credentials = store
            .load_credentials()
            .map_err(|e| SelectorError::Storage(format!("Failed to load credentials: {}", e)))?;

        Ok(Self { credentials })
    }

    /// Create a credential selector filtered by template type
    pub fn new_for_template(template_type: &templates::TemplateType) -> SelectorResult<Self> {
        let store = CredentialStore::new().map_err(|e| {
            SelectorError::Storage(format!("Failed to create credential store: {}", e))
        })?;
        let all_credentials = store
            .load_credentials()
            .map_err(|e| SelectorError::Storage(format!("Failed to load credentials: {}", e)))?;

        let credentials = all_credentials
            .into_iter()
            .filter(|cred| cred.template_type() == template_type)
            .collect();

        Ok(Self { credentials })
    }

    /// Run interactive credential management
    pub fn run_management(&mut self) -> SelectorResult<()> {
        if self.credentials.is_empty() {
            println!("No credentials found.");
            return Ok(());
        }

        loop {
            match self.select_credential_action()? {
                Some(CredentialManagementAction::Delete(index)) => {
                    if self.delete_credential(index)? && index < self.credentials.len() {
                        self.credentials.remove(index);
                    }
                }
                Some(CredentialManagementAction::Rename(index)) => {
                    if let Some(should_continue) = self.rename_credential(index)?
                        && !should_continue
                    {
                        continue;
                    }
                }
                Some(CredentialManagementAction::Back) => continue,
                Some(CredentialManagementAction::Exit) => break,
                None => break,
            }

            if self.credentials.is_empty() {
                println!("No more credentials found.");
                break;
            }
        }

        Ok(())
    }

    /// Simple API key selection (for template application)
    pub fn select_api_key(
        template_type: templates::TemplateType,
    ) -> SelectorResult<Option<String>> {
        let selector = Self::new_for_template(&template_type)?;

        if selector.credentials.is_empty() {
            // No saved credentials, prompt for new API key directly
            return Self::prompt_new_api_key(&template_type);
        }

        let mut credentials = selector.credentials;
        let title = format!("Select {} API key:", template_type);

        loop {
            if credentials.is_empty() {
                // All credentials deleted, prompt for new API key
                return Self::prompt_new_api_key(&template_type);
            }

            let credential_items: Vec<CredentialListItem> = credentials
                .iter()
                .enumerate()
                .map(|(index, cred)| CredentialListItem {
                    index,
                    credential: cred.clone(),
                    is_filter: false,
                })
                .collect();

            // Picker mode: Enter directly selects, but d/n/r management shortcuts are available
            let config = crate::selectors::base::SelectorConfig {
                allow_create: true,
                allow_management: true,
                ..crate::selectors::base::SelectorConfig::default()
            };

            let mut sel =
                crate::selectors::base::Selector::new(&title, credential_items).with_config(config);

            match sel.prompt()? {
                crate::selectors::base::SelectionResult::Selected(item)
                | crate::selectors::base::SelectionResult::ViewDetails(item) => {
                    return Ok(Some(item.credential.api_key().to_string()));
                }
                crate::selectors::base::SelectionResult::Create => {
                    return Self::prompt_new_api_key(&template_type);
                }
                crate::selectors::base::SelectionResult::Delete(item) => {
                    // Perform deletion with confirmation
                    let credential = &item.credential;
                    let confirmed =
                        ConfirmationService::confirm_deletion(credential.name(), "credential")?;
                    if confirmed {
                        let store = CredentialStore::new().map_err(|e| {
                            SelectorError::Storage(format!(
                                "Failed to create credential store: {}",
                                e
                            ))
                        })?;
                        store.delete_credential(credential.id()).map_err(|e| {
                            SelectorError::OperationFailed(format!(
                                "Failed to delete credential: {}",
                                e
                            ))
                        })?;
                        credentials.remove(item.index);
                    }
                    // Loop back to re-show the list
                    continue;
                }
                crate::selectors::base::SelectionResult::Back
                | crate::selectors::base::SelectionResult::Exit => {
                    return Ok(None);
                }
                // Other actions (Rename, Refresh, etc.) — just re-show the list
                _ => continue,
            }
        }
    }

    /// Prompt for a new API key on a clean page
    fn prompt_new_api_key(
        template_type: &templates::TemplateType,
    ) -> SelectorResult<Option<String>> {
        // Clear screen for a clean page transition
        print!("\x1b[2J\x1b[H");
        io::stdout().flush().ok();

        let template_instance = get_template_instance(template_type);

        println!("🔑 Create New API Key\n");

        if let Some(url) = template_instance.api_key_url() {
            println!("  💡 Get your API key from: {}\n", url);
        }

        let prompt_text = format!("Enter your {} API key:", template_type);
        let api_key = inquire::Text::new(&prompt_text)
            .with_placeholder("sk-...")
            .prompt()?;

        if !api_key.trim().is_empty() {
            Ok(Some(api_key))
        } else {
            Err(SelectorError::InvalidInput(
                "API key cannot be empty".to_string(),
            ))
        }
    }

    // ── Unified API key selector (env vars + saved credentials + create) ──

    /// Unified API key selection combining env vars, saved credentials, and create.
    /// Sorts by last used timestamp so recently-used items appear first.
    pub fn select_api_key_unified(
        template_type: templates::TemplateType,
    ) -> SelectorResult<Option<String>> {
        let title = format!("Select {} API key:", template_type);

        loop {
            let sources = Self::build_unified_api_key_list(&template_type)?;

            if sources.is_empty() {
                return Self::prompt_new_api_key(&template_type);
            }

            let items: Vec<ApiKeyListItem> = sources
                .iter()
                .enumerate()
                .map(|(index, source)| ApiKeyListItem {
                    index,
                    source: source.clone(),
                })
                .collect();

            let config = crate::selectors::base::SelectorConfig {
                allow_create: true,
                allow_management: true,
                ..crate::selectors::base::SelectorConfig::default()
            };

            let mut sel =
                crate::selectors::base::Selector::new(&title, items).with_config(config);

            match sel.prompt()? {
                crate::selectors::base::SelectionResult::Selected(item)
                | crate::selectors::base::SelectionResult::ViewDetails(item) => {
                    return match &item.source {
                        ApiKeySource::EnvVar {
                            api_key, env_var_name, ..
                        } => {
                            println!(
                                "✓ Using API key from environment variable {}",
                                env_var_name
                            );
                            Ok(Some(api_key.clone()))
                        }
                        ApiKeySource::Saved { credential } => {
                            // Update last_used_at
                            if let Ok(store) = CredentialStore::new() {
                                let _ = store.touch_last_used(credential.id());
                            }
                            Ok(Some(credential.api_key().to_string()))
                        }
                    };
                }
                crate::selectors::base::SelectionResult::Create => {
                    return Self::prompt_new_api_key(&template_type);
                }
                crate::selectors::base::SelectionResult::Delete(item) => {
                    if let ApiKeySource::Saved { credential } = &item.source {
                        let confirmed =
                            ConfirmationService::confirm_deletion(credential.name(), "credential")?;
                        if confirmed {
                            let store = CredentialStore::new().map_err(|e| {
                                SelectorError::Storage(format!(
                                    "Failed to create credential store: {}",
                                    e
                                ))
                            })?;
                            store.delete_credential(credential.id()).map_err(|e| {
                                SelectorError::OperationFailed(format!(
                                    "Failed to delete credential: {}",
                                    e
                                ))
                            })?;
                        }
                    }
                    // Env vars cannot be deleted; just loop to rebuild list
                    continue;
                }
                crate::selectors::base::SelectionResult::Rename(item) => {
                    if let ApiKeySource::Saved { credential } = &item.source {
                        let new_name = inquire::Text::new(&format!(
                            "Rename '{}':",
                            credential.name()
                        ))
                        .with_help_message("Enter new name, Esc to cancel")
                        .prompt()
                        .map_err(|e| {
                            let msg = e.to_string();
                            if msg.contains("canceled") || msg.contains("cancelled") {
                                SelectorError::Cancelled
                            } else {
                                SelectorError::Failed(format!("Input failed: {}", e))
                            }
                        })?;
                        let new_name = new_name.trim().to_string();
                        if !new_name.is_empty() && new_name != credential.name() {
                            let store = CredentialStore::new().map_err(|e| {
                                SelectorError::Storage(format!(
                                    "Failed to create credential store: {}",
                                    e
                                ))
                            })?;
                            store
                                .update_name(credential.id(), new_name)
                                .map_err(|e| {
                                    SelectorError::OperationFailed(format!(
                                        "Failed to rename credential: {}",
                                        e
                                    ))
                                })?;
                        }
                    }
                    continue;
                }
                crate::selectors::base::SelectionResult::Back
                | crate::selectors::base::SelectionResult::Exit => {
                    return Ok(None);
                }
                _ => continue,
            }
        }
    }

    /// Build a unified list of API key sources, sorted by last usage.
    /// Order: saved credentials with last_used_at (most recent first)
    ///      → env var keys
    ///      → saved credentials never used (newest created first)
    fn build_unified_api_key_list(
        template_type: &templates::TemplateType,
    ) -> SelectorResult<Vec<ApiKeySource>> {
        let mut sources: Vec<ApiKeySource> = Vec::new();

        // 1. Collect env var sources
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

        // 2. Collect saved credentials for this template type
        if let Ok(credential_store) = CredentialStore::new() {
            if let Ok(all_credentials) = credential_store.load_credentials() {
                for credential in all_credentials
                    .into_iter()
                    .filter(|c| c.template_type() == template_type)
                {
                    sources.push(ApiKeySource::Saved { credential });
                }
            }
        }

        // 3. Sort: used credentials (by last_used_at desc) → env vars → unused credentials (by created_at desc)
        sources.sort_by(|a, b| {
            let priority = |s: &ApiKeySource| match s {
                ApiKeySource::Saved { credential } => match credential.last_used_at() {
                    Some(ts) => (2, ts.to_string()),
                    None => (0, credential.created_at().to_string()),
                },
                ApiKeySource::EnvVar { .. } => (1, String::new()),
            };
            let pa = priority(a);
            let pb = priority(b);
            pb.cmp(&pa)
        });

        // 4. Deduplicate: env var keys take priority; also dedup within saved credentials
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
            let key = match &source {
                ApiKeySource::EnvVar { api_key, .. } => api_key.clone(),
                ApiKeySource::Saved { credential } => credential.api_key().to_string(),
            };
            if seen.insert(key) {
                deduped.push(source);
            }
        }
        sources = deduped;

        Ok(sources)
    }

    /// Select credential for management
    fn select_credential_action(&mut self) -> SelectorResult<Option<CredentialManagementAction>> {
        // First select a credential
        let credential_index = self.select_credential_from_list()?;
        let index = match credential_index {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // Clear screen before showing actions
        print!("\x1b[2J\x1b[H"); // Clear screen and move cursor to top-left
        std::io::stdout().flush()?;

        // Then show actions for that credential
        self.show_credential_actions(index).map(Some)
    }

    /// Select credential from list
    fn select_credential_from_list(&mut self) -> SelectorResult<Option<usize>> {
        let items: Vec<CredentialListItem> = self
            .credentials
            .iter()
            .enumerate()
            .map(|(index, cred)| CredentialListItem {
                index,
                credential: cred.clone(),
                is_filter: false,
            })
            .collect();

        let title = format!(
            "Select a credential to manage ({} total):",
            self.credentials.len()
        );

        let mut all_items = items.clone();

        // Add filter option if there are enough items
        if all_items.len() > 5 {
            // Create a placeholder credential for the filter option
            let placeholder_credential = if self.credentials.is_empty() {
                // Create a minimal placeholder credential
                SavedCredential {
                    id: Uuid::new_v4().to_string(),
                    name: "Filter".to_string(),
                    api_key: "placeholder".to_string(),
                    template_type: crate::templates::TemplateType::DeepSeek,
                    version: "1".to_string(),
                    created_at: chrono::Utc::now()
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string(),
                    updated_at: chrono::Utc::now()
                        .format("%Y-%m-%d %H:%M:%S UTC")
                        .to_string(),
                    last_used_at: None,
                    metadata: None,
                }
            } else {
                self.credentials[0].clone()
            };

            all_items.insert(
                0,
                CredentialListItem {
                    index: 0,
                    credential: placeholder_credential,
                    is_filter: true,
                },
            );
        }

        // Create selector directly instead of using BaseSelector to handle actions ourselves
        let config = crate::selectors::base::SelectorConfig::default();
        let mut selector =
            crate::selectors::base::Selector::new(&title, all_items).with_config(config);

        match selector.prompt()? {
            crate::selectors::base::SelectionResult::Selected(item) => {
                if item.is_filter {
                    Self::filter_credentials(&self.credentials)
                } else {
                    Ok(Some(item.index))
                }
            }
            crate::selectors::base::SelectionResult::Rename(item) => {
                // Handle rename directly
                match self.rename_credential(item.index)? {
                    Some(true) | None => {
                        // Re-run selector to update the list
                        self.select_credential_from_list()
                    }
                    Some(false) => Ok(None),
                }
            }
            crate::selectors::base::SelectionResult::Delete(item) => {
                // Handle delete directly
                if self.delete_credential(item.index)? {
                    // Re-run selector to update the list
                    self.select_credential_from_list()
                } else {
                    // User cancelled deletion
                    Ok(None)
                }
            }
            crate::selectors::base::SelectionResult::ViewDetails(item) => {
                // Just return the selected index
                Ok(Some(item.index))
            }
            crate::selectors::base::SelectionResult::Back
            | crate::selectors::base::SelectionResult::Exit => Ok(None),
            _ => Ok(None),
        }
    }

    /// Filter credentials using text input
    fn filter_credentials(credentials: &[SavedCredential]) -> SelectorResult<Option<usize>> {
        use std::sync::Arc;

        let suggestions: Vec<String> = credentials.iter().map(|c| c.name().to_string()).collect();

        let suggestions = Arc::new(suggestions);

        let mut prompt = inquire::Text::new("Filter credentials:");

        prompt = prompt.with_help_message(
            "Type to filter credential names, Tab: Complete, Enter: Select, Esc: Cancel",
        );

        prompt = prompt.with_autocomplete(move |input: &str| {
            if input.is_empty() {
                return Ok(suggestions.iter().cloned().collect());
            }

            Ok(suggestions
                .iter()
                .filter(|suggestion| suggestion.to_lowercase().contains(&input.to_lowercase()))
                .cloned()
                .collect())
        });

        prompt = prompt.with_validator(|input: &str| {
            if input.trim().is_empty() {
                Ok(inquire::validator::Validation::Invalid(
                    "Please type to filter credentials".into(),
                ))
            } else {
                Ok(inquire::validator::Validation::Valid)
            }
        });

        let user_input = prompt.prompt().map_err(|e| {
            if e.to_string().contains("canceled") || e.to_string().contains("cancelled") {
                SelectorError::Cancelled
            } else {
                SelectorError::Failed(format!("Filter input failed: {}", e))
            }
        })?;

        // Find matching credential
        for (index, credential) in credentials.iter().enumerate() {
            if credential
                .name()
                .to_lowercase()
                .contains(&user_input.to_lowercase())
            {
                return Ok(Some(index));
            }
        }

        Err(SelectorError::InvalidInput(format!(
            "No credential found matching: {}",
            user_input
        )))
    }

    /// Show actions for a credential using inquire Select component
    fn show_credential_actions(&self, index: usize) -> SelectorResult<CredentialManagementAction> {
        use inquire::{InquireError, Select};

        let credential = &self.credentials[index];

        // Create credential details string
        let masked_key = if credential.api_key().len() <= 4 {
            "••••".to_string()
        } else {
            format!(
                "{}••••",
                &credential.api_key()[..credential.api_key().len().min(4)]
            )
        };

        let mut details = format!(
            "Credential: {} ({})\n\
             API Key: {}\n\
             Env: {} (primary)",
            credential.name(),
            credential.template_type(),
            masked_key,
            crate::templates::get_template_instance(credential.template_type())
                .env_var_names()
                .first()
                .unwrap_or(&"N/A")
        );

        // Add metadata if available
        if let Some(metadata) = credential.metadata()
            && !metadata.is_empty()
        {
            let first_meta = metadata
                .iter()
                .take(2)
                .map(|(k, v)| format!("{}: {}", k, v))
                .collect::<Vec<_>>()
                .join(", ");
            if !first_meta.is_empty() {
                details.push_str(&format!("\nMetadata: {}", first_meta));
            }
        }

        // Create options for the select
        let options = vec!["✏️  Rename", "🗑️  Delete", "⬅️  Back"];

        let help_message = "↑↓ to move, enter to select, esc to cancel";

        let full_details = format!(
            "Manage Credential:\n\n{}\n\nCreated: {}\nUpdated: {}",
            details,
            credential.created_at(),
            credential.updated_at()
        );

        match Select::new(&full_details, options)
            .with_help_message(help_message)
            .with_page_size(3)
            .prompt_skippable()
        {
            Ok(Some(action)) => match action {
                "✏️  Rename" => Ok(CredentialManagementAction::Rename(index)),
                "🗑️  Delete" => Ok(CredentialManagementAction::Delete(index)),
                "⬅️  Back" => Ok(CredentialManagementAction::Back),
                _ => Ok(CredentialManagementAction::Exit),
            },
            Ok(None) => Ok(CredentialManagementAction::Exit),
            Err(InquireError::OperationCanceled) => Ok(CredentialManagementAction::Exit),
            Err(e) => Err(SelectorError::failed(e.to_string())),
        }
    }

    /// Delete a credential with confirmation
    fn delete_credential(&self, index: usize) -> SelectorResult<bool> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }

        let credential = &self.credentials[index];

        let confirmation = ConfirmationService::confirm_deletion(credential.name(), "credential")?;

        if confirmation {
            let store = CredentialStore::new().map_err(|e| {
                SelectorError::Storage(format!("Failed to create credential store: {}", e))
            })?;
            store.delete_credential(credential.id()).map_err(|e| {
                SelectorError::OperationFailed(format!("Failed to delete credential: {}", e))
            })?;
            println!("✓ Credential deleted successfully!");
            Ok(true)
        } else {
            println!("Deletion cancelled.");
            Ok(false)
        }
    }

    /// Rename a credential
    fn rename_credential(&mut self, index: usize) -> SelectorResult<Option<bool>> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }

        let credential = &self.credentials[index];

        let new_name = inquire::Text::new(&format!("Rename '{}':", credential.name()))
            .with_help_message("Enter new name, Esc to cancel")
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Input failed: {}", e))
                }
            })?;

        let new_name = new_name.trim().to_string();

        if new_name.is_empty() {
            println!("Rename cancelled.");
            return Ok(Some(true));
        }

        if new_name == credential.name() {
            println!("Name unchanged.");
            return Ok(Some(true));
        }

        // Confirm the rename action
        let confirmation = ConfirmationService::confirm_action(&format!(
            "Rename '{}' to '{}'",
            credential.name(),
            new_name
        ))?;

        if confirmation {
            let store = CredentialStore::new().map_err(|e| {
                SelectorError::Storage(format!("Failed to create credential store: {}", e))
            })?;
            store
                .update_name(credential.id(), new_name.clone())
                .map_err(|e| {
                    SelectorError::OperationFailed(format!("Failed to rename credential: {}", e))
                })?;

            // Update local list
            if let Some(cred) = self.credentials.get_mut(index) {
                cred.name = new_name;
                cred.update_timestamp();
            }

            println!("✓ Credential renamed successfully!");
            Ok(Some(true))
        } else {
            println!("Rename cancelled.");
            Ok(Some(true))
        }
    }
}

/// Wrapper for credentials in selection lists
#[derive(Debug, Clone)]
struct CredentialListItem {
    index: usize,
    credential: SavedCredential,
    is_filter: bool,
}

impl SelectableItem for CredentialListItem {
    fn display_name(&self) -> String {
        if self.is_filter {
            "🔍 Filter credentials...".to_string()
        } else {
            format!(
                "{} ({})",
                self.credential.name(),
                self.credential.template_type()
            )
        }
    }

    fn format_for_list(&self) -> String {
        if self.is_filter {
            "🔍 Filter credentials...".to_string()
        } else {
            let masked_key = if self.credential.api_key().len() <= 8 {
                "••••••••".to_string()
            } else {
                format!(
                    "{}••••••••",
                    &self.credential.api_key()[..self.credential.api_key().len().min(8)]
                )
            };

            let env_vars = get_template_instance(self.credential.template_type()).env_var_names();
            let env_indicator = if env_vars.len() > 1 {
                format!(" (+{})", env_vars.len())
            } else {
                String::new()
            };

            format!(
                "{} ({}){} - {}",
                self.credential.name(),
                self.credential.template_type(),
                env_indicator,
                masked_key
            )
        }
    }

    fn id(&self) -> Option<String> {
        if self.is_filter {
            Some("filter".to_string())
        } else {
            Some(self.credential.id().to_string())
        }
    }
}

// Also implement SelectableItem directly for SavedCredential for simple selections
impl SelectableItem for SavedCredential {
    fn display_name(&self) -> String {
        format!("{} ({})", self.name(), self.template_type())
    }

    fn format_for_list(&self) -> String {
        // Show detailed information when displayed as a single item in list-like UI
        let masked_key = if self.api_key().len() <= 8 {
            "••••••••".to_string()
        } else {
            format!("{}••••••••", &self.api_key()[..self.api_key().len().min(8)])
        };

        let template = get_template_instance(self.template_type());
        let env_vars = template.env_var_names();

        let mut details = format!(
            "Name: {}\nType: {}\nAPI Key: {}\n",
            self.name(),
            self.template_type(),
            masked_key
        );

        // Add environment variables
        details.push_str("Environment Variables:\n");
        for (i, env_var) in env_vars.iter().enumerate() {
            let marker = if i == 0 { " (primary)" } else { "" };
            details.push_str(&format!("  - {}{}\n", env_var, marker));
        }

        // Add created and updated timestamps
        details.push_str(&format!(
            "Created: {}\nUpdated: {}\n",
            self.created_at(),
            self.updated_at()
        ));

        // Add metadata if available
        if let Some(metadata) = self.metadata()
            && !metadata.is_empty()
        {
            details.push_str("Metadata:\n");
            for (key, value) in metadata {
                details.push_str(&format!("  {}: {}\n", key, value));
            }
        }

        details
    }

    fn id(&self) -> Option<String> {
        Some(self.id().to_string())
    }
}

// ── Unified API key source types ──

/// Represents a selectable API key source in the unified selector
#[derive(Debug, Clone)]
pub enum ApiKeySource {
    /// API key from an environment variable
    EnvVar {
        env_var_name: String,
        api_key: String,
    },
    /// API key from a saved credential
    Saved { credential: SavedCredential },
}

/// Display wrapper for unified API key selection
#[derive(Debug, Clone)]
pub struct ApiKeyListItem {
    #[allow(dead_code)]
    pub index: usize,
    pub source: ApiKeySource,
}

impl SelectableItem for ApiKeyListItem {
    fn display_name(&self) -> String {
        match &self.source {
            ApiKeySource::EnvVar { env_var_name, .. } => {
                format!("Env: {}", env_var_name)
            }
            ApiKeySource::Saved { credential } => {
                format!("{} ({})", credential.name(), credential.template_type())
            }
        }
    }

    fn format_for_list(&self) -> String {
        match &self.source {
            ApiKeySource::EnvVar {
                env_var_name,
                api_key,
            } => {
                let masked = mask_api_key(api_key);
                format!("🌐 {} = {}", env_var_name, masked)
            }
            ApiKeySource::Saved { credential } => {
                let masked = mask_api_key(credential.api_key());
                let env_vars = get_template_instance(credential.template_type()).env_var_names();
                let env_indicator = if env_vars.len() > 1 {
                    format!(" (+{})", env_vars.len())
                } else {
                    String::new()
                };
                format!(
                    "🔑 {} ({}){} - {}",
                    credential.name(),
                    credential.template_type(),
                    env_indicator,
                    masked
                )
            }
        }
    }

    fn id(&self) -> Option<String> {
        match &self.source {
            ApiKeySource::EnvVar { env_var_name, .. } => Some(format!("env:{}", env_var_name)),
            ApiKeySource::Saved { credential } => Some(format!("cred:{}", credential.id())),
        }
    }
}
