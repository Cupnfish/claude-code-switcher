//! Credential management browser (inquire-based).
//!
//! Used by `ccs credentials list`. The API-key *acquisition* used by `apply`
//! lives in [`crate::credentials`] (`resolve_api_key`), not here.

use crate::credentials::{CredentialStore, SavedCredential, mask_api_key};
use crate::selectors::{
    confirmation::ConfirmationService,
    error::{SelectorError, SelectorResult},
};
use crate::templates::get_template_instance;
use crate::{CredentialManager, templates};
use inquire::InquireError;

/// Credential management browser.
pub struct CredentialSelector {
    credentials: Vec<SavedCredential>,
}

/// Inquire selection wrapper that carries its own index (so selection is
/// unambiguous even when two credentials render identically).
struct Choice {
    index: usize,
    label: String,
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

fn format_credential_line(cred: &SavedCredential) -> String {
    let env_vars = get_template_instance(cred.template_type()).env_var_names();
    let env_indicator = if env_vars.len() > 1 {
        format!(" (+{})", env_vars.len())
    } else {
        String::new()
    };
    format!(
        "{} ({}){} - {}",
        cred.name(),
        cred.template_type(),
        env_indicator,
        mask_api_key(cred.api_key())
    )
}

impl CredentialSelector {
    /// Create a browser over all saved credentials.
    pub fn new_all() -> SelectorResult<Self> {
        let store = CredentialStore::new()
            .map_err(|e| SelectorError::Storage(format!("Failed to create credential store: {}", e)))?;
        let credentials = store
            .load_credentials()
            .map_err(|e| SelectorError::Storage(format!("Failed to load credentials: {}", e)))?;
        Ok(Self { credentials })
    }

    /// Create a browser filtered to a single template type.
    pub fn new_for_template(template_type: &templates::TemplateType) -> SelectorResult<Self> {
        let mut sel = Self::new_all()?;
        sel.credentials.retain(|c| c.template_type() == template_type);
        Ok(sel)
    }

    /// Interactive credential management loop (rename / delete).
    pub fn run_management(&mut self) -> SelectorResult<()> {
        if self.credentials.is_empty() {
            println!("No credentials found.");
            return Ok(());
        }

        while let Some(index) = self.pick_credential()? {
            self.manage_credential(index)?;

            // Reload after a possible mutation so indices stay valid.
            self.credentials = Self::load_all()?;
            if self.credentials.is_empty() {
                println!("No more credentials found.");
                break;
            }
        }

        Ok(())
    }

    fn load_all() -> SelectorResult<Vec<SavedCredential>> {
        let store = CredentialStore::new()
            .map_err(|e| SelectorError::Storage(format!("Failed to create credential store: {}", e)))?;
        store
            .load_credentials()
            .map_err(|e| SelectorError::Storage(format!("Failed to load credentials: {}", e)))
    }

    /// Pick a credential from the list. Returns `None` on Esc/back.
    fn pick_credential(&self) -> SelectorResult<Option<usize>> {
        let choices: Vec<Choice> = self
            .credentials
            .iter()
            .enumerate()
            .map(|(index, cred)| Choice {
                index,
                label: format_credential_line(cred),
            })
            .collect();

        let title = format!("Select a credential to manage ({} total):", self.credentials.len());
        match inquire::Select::new(&title, choices)
            .with_help_message("↑/↓ navigate, Enter select, Esc exit")
            .prompt()
        {
            Ok(choice) => Ok(Some(choice.index)),
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                Ok(None)
            }
            Err(e) => Err(SelectorError::Failed(format!("Selection failed: {}", e))),
        }
    }

    /// Show the action menu for a single credential.
    fn manage_credential(&self, index: usize) -> SelectorResult<()> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }
        let cred = &self.credentials[index];

        println!();
        println!("🔑 {}", cred.name());
        println!("   type: {}", cred.template_type());
        println!("   key:  {}", mask_api_key(cred.api_key()));
        if let Some(last) = cred.last_used_at() {
            println!("   last used: {}", last);
        }

        let options = vec!["✏️  Rename", "🗑️  Delete", "⬅️  Back"];
        let action = match inquire::Select::new("Action:", options)
            .with_help_message("↑/↓ navigate, Enter select, Esc back")
            .prompt()
        {
            Ok(a) => a,
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                return Ok(());
            }
            Err(e) => return Err(SelectorError::Failed(format!("Selection failed: {}", e))),
        };

        match action {
            "✏️  Rename" => self.rename_credential(index)?,
            "🗑️  Delete" => self.delete_credential(index)?,
            _ => {}
        }
        Ok(())
    }

    fn delete_credential(&self, index: usize) -> SelectorResult<()> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }
        let cred = &self.credentials[index];
        if ConfirmationService::confirm_deletion(cred.name(), "credential")? {
            let store = CredentialStore::new()
                .map_err(|e| SelectorError::Storage(format!("Failed to create credential store: {}", e)))?;
            store
                .delete_credential(cred.id())
                .map_err(|e| SelectorError::OperationFailed(format!("Failed to delete credential: {}", e)))?;
            println!("✓ Credential deleted.");
        } else {
            println!("Deletion cancelled.");
        }
        Ok(())
    }

    fn rename_credential(&self, index: usize) -> SelectorResult<()> {
        if index >= self.credentials.len() {
            return Err(SelectorError::NotFound);
        }
        let cred = &self.credentials[index];
        let new_name = match inquire::Text::new(&format!("Rename '{}':", cred.name()))
            .with_help_message("Enter new name, Esc to cancel")
            .prompt()
        {
            Ok(s) => s.trim().to_string(),
            Err(InquireError::OperationCanceled) | Err(InquireError::OperationInterrupted) => {
                println!("Rename cancelled.");
                return Ok(());
            }
            Err(e) => return Err(SelectorError::Failed(format!("Input failed: {}", e))),
        };

        if new_name.is_empty() || new_name == cred.name() {
            println!("Name unchanged.");
            return Ok(());
        }

        let store = CredentialStore::new()
            .map_err(|e| SelectorError::Storage(format!("Failed to create credential store: {}", e)))?;
        store
            .update_name(cred.id(), new_name.clone())
            .map_err(|e| SelectorError::OperationFailed(format!("Failed to rename credential: {}", e)))?;
        println!("✓ Credential renamed to '{}'.", new_name);
        Ok(())
    }
}
