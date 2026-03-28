//! Confirmation dialogs with consistent minimalist UI

use crate::selectors::error::{SelectorError, SelectorResult};

/// Service for handling confirmation dialogs
pub struct ConfirmationService;

impl ConfirmationService {
    /// Core confirmation using inquire
    fn confirm_impl(message: &str, default: bool) -> SelectorResult<bool> {
        if !atty::is(atty::Stream::Stdin) {
            return Ok(default);
        }

        inquire::Confirm::new(message)
            .with_default(default)
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Confirmation failed: {}", e))
                }
            })
    }

    /// Standard yes/no confirmation
    pub fn confirm(message: &str, default: bool) -> SelectorResult<bool> {
        Self::confirm_impl(message, default)
    }

    /// Confirm deletion of an item
    pub fn confirm_deletion(item_name: &str, item_type: &str) -> SelectorResult<bool> {
        Self::confirm_impl(&format!("Delete '{}' {}?", item_name, item_type), false)
    }

    /// Confirm overwrite of an existing item
    pub fn confirm_overwrite(item_name: &str, item_type: &str) -> SelectorResult<bool> {
        Self::confirm_impl(
            &format!("{} '{}' already exists. Overwrite?", item_type, item_name),
            false,
        )
    }

    /// Confirm an arbitrary action
    pub fn confirm_action(action_description: &str) -> SelectorResult<bool> {
        Self::confirm_impl(action_description, false)
    }
}
