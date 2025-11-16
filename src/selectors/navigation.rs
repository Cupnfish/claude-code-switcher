//! Navigation and selection management

use crate::selectors::base::SelectableItem;
use crate::selectors::error::{SelectorError, SelectorResult};
use inquire::Select;

/// Result of a navigation operation
#[derive(Debug, Clone)]
pub enum NavigationResult<T> {
    /// An item was selected
    Selected(T),

    /// User wants to create a new item
    CreateNew,

    /// User wants to go back
    Back,

    /// User wants to exit
    Exit,
}

/// Manages navigation and selection for lists of items
pub struct NavigationManager;

impl NavigationManager {
    /// Select an item from a list with consistent navigation
    pub fn select_from_list<T: SelectableItem + Clone>(
        items: &[T],
        title: &str,
        allow_create: bool,
        help_message: Option<&str>,
    ) -> SelectorResult<NavigationResult<T>> {
        Self::select_from_list_with_create(items, title, allow_create, help_message)
    }

    /// Enhanced selection with create option support
    pub fn select_from_list_with_create<T: SelectableItem + Clone>(
        items: &[T],
        title: &str,
        allow_create: bool,
        help_message: Option<&str>,
    ) -> SelectorResult<NavigationResult<T>> {
        let mut options = Vec::new();

        // Add create option if allowed
        if allow_create {
            options.push("➕ Create New...".to_string());
        }

        // Add items to options
        for item in items {
            options.push(item.format_for_list());
        }

        // Handle empty list case
        if options.is_empty() {
            return Err(SelectorError::Failed("No options available".to_string()));
        }

        let mut select = Select::new(title, options);

        if let Some(help) = help_message {
            select = select.with_help_message(help);
        }

        let selection = select.prompt().map_err(|e| {
            if e.to_string().contains("canceled") || e.to_string().contains("cancelled") {
                SelectorError::Cancelled
            } else {
                SelectorError::Failed(format!("Selection failed: {}", e))
            }
        })?;

        // Check if create option was selected
        if allow_create && selection == "➕ Create New..." {
            return Ok(NavigationResult::CreateNew);
        }

        // Find the selected item
        for item in items {
            if selection == item.format_for_list() || selection.starts_with(&item.display_name()) {
                return Ok(NavigationResult::Selected(item.clone()));
            }
        }

        // If we can't find the item, it might be a cancellation
        Err(SelectorError::NotFound)
    }

    /// Simple binary selection (Yes/No)
    pub fn confirm(message: &str, default: bool) -> SelectorResult<bool> {
        use inquire::Confirm;

        if !atty::is(atty::Stream::Stdin) {
            return Ok(default);
        }

        Confirm::new(message)
            .with_default(default)
            .with_help_message("Y for Yes, N for No")
            .prompt()
            .map_err(|e| {
                if e.to_string().contains("canceled") || e.to_string().contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Confirmation failed: {}", e))
                }
            })
    }

    /// Select from custom options
    pub fn select_option(
        title: &str,
        options: &[&str],
        help_message: Option<&str>,
    ) -> SelectorResult<String> {
        let options: Vec<String> = options.iter().map(|s| s.to_string()).collect();

        let mut select = Select::new(title, options);

        if let Some(help) = help_message {
            select = select.with_help_message(help);
        }

        select.prompt().map_err(|e| {
            if e.to_string().contains("canceled") || e.to_string().contains("cancelled") {
                SelectorError::Cancelled
            } else {
                SelectorError::Failed(format!("Option selection failed: {}", e))
            }
        })
    }

    /// Text input with validation
    pub fn get_text_input(
        message: &str,
        placeholder: Option<&str>,
        help_message: Option<&str>,
    ) -> SelectorResult<String> {
        use inquire::Text;

        let mut prompt = Text::new(message);

        if let Some(placeholder) = placeholder {
            prompt = prompt.with_placeholder(placeholder);
        }

        if let Some(help) = help_message {
            prompt = prompt.with_help_message(help);
        }

        prompt.prompt().map_err(|e| {
            if e.to_string().contains("canceled") || e.to_string().contains("cancelled") {
                SelectorError::Cancelled
            } else {
                SelectorError::Failed(format!("Input failed: {}", e))
            }
        })
    }
}
