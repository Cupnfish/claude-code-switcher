//! Enhanced confirmation service with consistent UI/UX

use crate::selectors::{
    error::{SelectorError, SelectorResult},
    navigation::NavigationManager,
};
use console::style;

/// Service for handling confirmation dialogs
pub struct ConfirmationService;

impl ConfirmationService {
    /// Standard confirmation with Yes/No options
    pub fn confirm(message: &str) -> SelectorResult<bool> {
        NavigationManager::confirm(message, false)
    }

    /// Confirmation with a default value
    pub fn confirm_with_default(message: &str, default: bool) -> SelectorResult<bool> {
        NavigationManager::confirm(message, default)
    }

    /// Enhanced confirmation with styled options and quit option
    pub fn confirm_enhanced(message: &str, default: bool) -> SelectorResult<bool> {
        // If not in interactive mode, return default
        if !atty::is(atty::Stream::Stdin) {
            return Ok(default);
        }

        // Create enhanced options with keyboard shortcuts
        let options = [
            format!("✓ Yes {}", style("(Y)").green()),
            format!("✗ No {}", style("(N)").red()),
            format!("⚠ Quit {}", style("(Q)").yellow()),
        ];

        match NavigationManager::select_option(
            message,
            &options.iter().map(|s| s.as_str()).collect::<Vec<_>>(),
            Some("Press Y for Yes, N for No, or Q to Quit"),
        ) {
            Ok(choice) => {
                match choice.as_str() {
                    choice if choice.contains("Yes") => Ok(true),
                    choice if choice.contains("No") => Ok(false),
                    choice if choice.contains("Quit") => {
                        println!("{}", style("🚫 Operation cancelled by user.").yellow());
                        std::process::exit(0);
                    }
                    _ => Ok(default), // Should not happen, but provide fallback
                }
            }
            Err(SelectorError::Cancelled) => Ok(default),
            Err(e) => Err(e),
        }
    }

    /// Confirm deletion of an item
    pub fn confirm_deletion(item_name: &str, item_type: &str) -> SelectorResult<bool> {
        let message = format!(
            "Are you sure you want to delete '{}' {}? This action cannot be undone",
            item_name, item_type
        );
        Self::confirm_enhanced(&message, false)
    }

    /// Confirm overwrite of an existing item
    pub fn confirm_overwrite(item_name: &str, item_type: &str) -> SelectorResult<bool> {
        let message = format!("{} '{}' already exists. Overwrite?", item_type, item_name);
        Self::confirm_enhanced(&message, false)
    }

    /// Confirm an action with a custom message
    pub fn confirm_action(action_description: &str) -> SelectorResult<bool> {
        Self::confirm_enhanced(action_description, false)
    }

    /// Get confirmation for a list of actions
    pub fn confirm_actions(actions: &[&str]) -> SelectorResult<bool> {
        let message = format!(
            "This will perform the following actions:\n{}\n\nContinue?",
            actions
                .iter()
                .enumerate()
                .map(|(i, action)| format!("  {}. {}", i + 1, action))
                .collect::<Vec<_>>()
                .join("\n")
        );
        Self::confirm_enhanced(&message, false)
    }

    /// Quick yes/no confirmation without styling
    pub fn quick_confirm(message: &str) -> SelectorResult<bool> {
        Self::confirm(message)
    }
}
