//! Credential selector using the unified selector framework

use crate::{
    CredentialManager,
    credentials::{CredentialStore, SavedCredential},
    templates::get_template_instance,
};
use crate::{
    selectors::{
        base::{SelectableItem, Selector},
        confirmation::ConfirmationService,
        error::{SelectorError, SelectorResult},
        navigation::NavigationManager,
    },
    templates,
};
use std::io;

/// Action for credential management
#[derive(Debug, Clone)]
pub enum CredentialManagementAction {
    ViewDetails(usize),
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
                Some(CredentialManagementAction::ViewDetails(index)) => {
                    if !self.show_credential_details_with_navigation(index)? {
                        break;
                    }
                    continue;
                }
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
            // No saved credentials, prompt for new API key
            let template_instance = get_template_instance(&template_type);
            if let Some(url) = template_instance.api_key_url() {
                println!("  💡 Get your API key from: {}", url);
            }

            let prompt_text = format!("Enter your {} API key:", template_type);
            let api_key = NavigationManager::get_text_input(&prompt_text, Some("sk-..."), None)?;

            if !api_key.trim().is_empty() {
                return Ok(Some(api_key));
            }
            return Ok(None);
        }

        // Use framework for selection
        let mut base_selector = crate::selectors::base::BaseSelector::new(
            selector.credentials.clone(),
            &format!("Select {} API key:", template_type),
        )
        .with_create(true);

        match base_selector.run() {
            Ok(Some(credential)) => Ok(Some(credential.api_key().to_string())),
            Ok(None) => Ok(None), // User cancelled
            Err(_) => {
                // Handle create action
                let template_instance = get_template_instance(&template_type);
                if let Some(url) = template_instance.api_key_url() {
                    println!("  💡 Get your API key from: {}", url);
                }

                let prompt_text = format!("Enter your {} API key:", template_type);
                let api_key =
                    NavigationManager::get_text_input(&prompt_text, Some("sk-..."), None)?;

                if !api_key.trim().is_empty() {
                    Ok(Some(api_key))
                } else {
                    Err(SelectorError::InvalidInput(
                        "API key cannot be empty".to_string(),
                    ))
                }
            }
        }
    }

    /// Select credential for management
    fn select_credential_action(&mut self) -> SelectorResult<Option<CredentialManagementAction>> {
        // First select a credential
        let credential_index = self.select_credential_from_list()?;
        let index = match credential_index {
            Some(idx) => idx,
            None => return Ok(None),
        };

        // Then show actions for that credential
        self.show_credential_actions(index).map(Some)
    }

    /// Select credential from list
    fn select_credential_from_list(&self) -> SelectorResult<Option<usize>> {
        let items: Vec<CredentialListItem> = self
            .credentials
            .iter()
            .enumerate()
            .map(|(index, cred)| CredentialListItem {
                index,
                credential: cred.clone(),
            })
            .collect();

        let title = format!(
            "Select a credential to manage ({} total):",
            self.credentials.len()
        );

        match NavigationManager::select_from_list(
            &items,
            &title,
            false,
            Some("↑/↓: Navigate, →: Select, ←/Esc: Back"),
        )? {
            crate::selectors::navigation::NavigationResult::Selected(item) => Ok(Some(item.index)),
            crate::selectors::navigation::NavigationResult::Back
            | crate::selectors::navigation::NavigationResult::Exit => Ok(None),
            _ => Ok(None),
        }
    }

    /// Show actions for a credential
    fn show_credential_actions(&self, index: usize) -> SelectorResult<CredentialManagementAction> {
        let credential = &self.credentials[index];

        let actions = vec!["📋 View Details", "✏️  Rename", "🗑️  Delete", "⬅️  Back"];

        let title = format!(
            "Managing: {} ({})",
            credential.name(),
            credential.template_type()
        );

        match NavigationManager::select_option(&title, &actions, None)? {
            action if action == "📋 View Details" => {
                Ok(CredentialManagementAction::ViewDetails(index))
            }
            action if action == "✏️  Rename" => Ok(CredentialManagementAction::Rename(index)),
            action if action == "🗑️  Delete" => Ok(CredentialManagementAction::Delete(index)),
            action if action == "⬅️  Back" => Ok(CredentialManagementAction::Back),
            _ => Ok(CredentialManagementAction::Exit),
        }
    }

    /// Show credential details with navigation
    fn show_credential_details_with_navigation(&self, index: usize) -> SelectorResult<bool> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }

        self.display_credential_info(index)?;

        let actions = vec!["⬅️  Back to Credential List", "🚪 Exit Program"];

        match NavigationManager::select_option("Choose an action:", &actions, None)? {
            action if action == "⬅️  Back to Credential List" => Ok(true),
            action if action == "🚪 Exit Program" => Ok(false),
            _ => Ok(true),
        }
    }

    /// Display credential information
    fn display_credential_info(&self, index: usize) -> SelectorResult<()> {
        let credential = &self.credentials[index];

        println!("\n📋 Credential Details:");
        println!("  Name: {}", credential.name());
        println!("  Type: {}", credential.template_type());
        println!("  Created: {}", credential.created_at());
        println!("  Updated: {}", credential.updated_at());
        println!(
            "  API Key: {}••••",
            &credential.api_key()[..credential.api_key().len().min(4)]
        );

        // Show environment variables
        let template = get_template_instance(credential.template_type());
        let env_vars = template.env_var_names();
        println!("  Environment Variables:");
        for (i, env_var) in env_vars.iter().enumerate() {
            let marker = if i == 0 { " (primary)" } else { "" };
            println!("    - {}{}", env_var, marker);
        }

        // Show metadata if available
        if let Some(metadata) = credential.metadata()
            && !metadata.is_empty()
        {
            println!("  Metadata:");
            for (key, value) in metadata {
                println!("    {}: {}", key, value);
            }
        }

        Ok(())
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

        println!("Rename credential: {}", credential.name());
        println!("Press Enter without typing to cancel, or enter a new name:");

        let mut new_name = String::new();
        io::stdin()
            .read_line(&mut new_name)
            .map_err(SelectorError::Io)?;

        let new_name = new_name.trim().to_string();

        if new_name.is_empty() {
            println!("Rename cancelled.");
            return Ok(Some(true)); // Continue to main loop
        }

        if new_name == credential.name() {
            println!("ℹ️  Name unchanged.");
            return Ok(Some(true));
        }

        if new_name.trim().is_empty() {
            println!("❌ Name cannot be empty.");
            return Ok(Some(true)); // Continue to main loop
        }

        // Confirm the rename action
        let confirmation = ConfirmationService::confirm_action(&format!(
            "Rename '{}' to '{}'",
            credential.name(),
            new_name.trim()
        ))?;

        if confirmation {
            let store = CredentialStore::new().map_err(|e| {
                SelectorError::Storage(format!("Failed to create credential store: {}", e))
            })?;
            store
                .update_name(credential.id(), new_name.trim().to_string())
                .map_err(|e| {
                    SelectorError::OperationFailed(format!("Failed to rename credential: {}", e))
                })?;

            // Update local list
            if let Some(cred) = self.credentials.get_mut(index) {
                cred.name = new_name.trim().to_string();
                cred.update_timestamp();
            }

            println!("✓ Credential renamed successfully!");
            Ok(Some(true))
        } else {
            println!("Rename cancelled.");
            Ok(Some(true)) // Continue to main loop
        }
    }
}

/// Wrapper for credentials in selection lists
#[derive(Debug, Clone)]
struct CredentialListItem {
    index: usize,
    credential: SavedCredential,
}

impl SelectableItem for CredentialListItem {
    fn display_name(&self) -> String {
        format!(
            "{} ({})",
            self.credential.name(),
            self.credential.template_type()
        )
    }

    fn format_for_list(&self) -> String {
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

    fn id(&self) -> Option<String> {
        Some(self.credential.id().to_string())
    }
}

// Also implement SelectableItem directly for SavedCredential for simple selections
impl SelectableItem for SavedCredential {
    fn display_name(&self) -> String {
        format!("{} ({})", self.name(), self.template_type())
    }

    fn format_for_list(&self) -> String {
        let masked_key = if self.api_key().len() <= 8 {
            "••••••••".to_string()
        } else {
            format!("{}••••••••", &self.api_key()[..self.api_key().len().min(8)])
        };
        format!(
            "{} ({}) - {}",
            self.name(),
            self.template_type(),
            masked_key
        )
    }

    fn id(&self) -> Option<String> {
        Some(self.id().to_string())
    }
}
