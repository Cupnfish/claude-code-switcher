//! Core traits and base structures for the selector framework

use crate::selectors::navigation::{NavigationManager, NavigationResult};
use anyhow::Result as AnyhowResult;

/// Trait for items that can be selected from a list
pub trait SelectableItem {
    /// Get the display name for the item
    fn display_name(&self) -> String;

    /// Format the item for display in a selection list
    fn format_for_list(&self) -> String;

    /// Get a unique identifier for the item (optional)
    fn id(&self) -> Option<String> {
        None
    }
}

/// Actions that can result from a selection
#[derive(Debug, Clone)]
pub enum SelectorAction<T> {
    /// An item was selected
    Selected(T),

    /// User wants to create a new item
    Create,

    /// User wants to go back
    Back,

    /// User wants to exit
    Exit,
}

/// Core trait for all selectors
pub trait Selector<T: SelectableItem + Clone> {
    /// Get the current list of items
    fn get_items(&self) -> Vec<T>;

    /// Get special options (like "Create new", "Back", etc.)
    fn get_special_options(&self) -> Vec<String> {
        vec![]
    }

    /// Handle selection of a specific item
    fn handle_item_selection(&self, item: T) -> AnyhowResult<SelectorAction<T>> {
        Ok(SelectorAction::Selected(item))
    }

    /// Handle selection of a special option
    fn handle_special_option(&self, option: &str) -> AnyhowResult<SelectorAction<T>> {
        match option {
            opt if opt.contains("Create") || opt.contains("New") => Ok(SelectorAction::Create),
            opt if opt.contains("Back") => Ok(SelectorAction::Back),
            opt if opt.contains("Exit") => Ok(SelectorAction::Exit),
            _ => Err(anyhow::anyhow!("Unknown special option: {}", option)),
        }
    }

    /// Get the title for the selection prompt
    fn get_title(&self) -> String;

    /// Get help message for the selection
    fn get_help_message(&self) -> Option<String> {
        Some("↑/↓: Navigate, →/Entry: Select, ←/Esc: Back".to_string())
    }

    /// Allow creation of new items
    fn allow_create(&self) -> bool {
        false
    }

    /// Run the selector and return the selected item
    fn run(&mut self) -> AnyhowResult<Option<T>> {
        let items = self.get_items();
        let title = self.get_title();
        let help = self.get_help_message();

        match NavigationManager::select_from_list(
            &items,
            &title,
            self.allow_create(),
            help.as_deref(),
        )? {
            NavigationResult::Selected(item) => match self.handle_item_selection(item)? {
                SelectorAction::Selected(item) => Ok(Some(item)),
                SelectorAction::Create => self.handle_create_action(),
                SelectorAction::Back => Ok(None),
                SelectorAction::Exit => {
                    println!("🚫 Operation cancelled by user.");
                    std::process::exit(0);
                }
            },
            NavigationResult::CreateNew => self.handle_create_action(),
            NavigationResult::Back | NavigationResult::Exit => Ok(None),
        }
    }

    /// Handle create action (to be overridden by implementations)
    fn handle_create_action(&self) -> AnyhowResult<Option<T>> {
        Err(anyhow::anyhow!("Create action not supported"))
    }
}

/// Base implementation for simple selectors
pub struct BaseSelector<T: SelectableItem> {
    items: Vec<T>,
    title: String,
    allow_create: bool,
}

impl<T: SelectableItem> BaseSelector<T> {
    pub fn new(items: Vec<T>, title: &str) -> Self {
        Self {
            items,
            title: title.to_string(),
            allow_create: false,
        }
    }

    pub fn with_create(mut self, allow_create: bool) -> Self {
        self.allow_create = allow_create;
        self
    }
}

impl<T: SelectableItem + Clone> Selector<T> for BaseSelector<T> {
    fn get_items(&self) -> Vec<T> {
        self.items.clone()
    }

    fn get_title(&self) -> String {
        self.title.clone()
    }

    fn allow_create(&self) -> bool {
        self.allow_create
    }
}
