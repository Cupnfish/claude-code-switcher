//! Unified selector framework
//!
//! Provides a consistent interface for interactive selection across different
//! types of items (credentials, snapshots, templates, etc.).

pub mod base;
pub mod confirmation;
pub mod error;
pub mod navigation;

// Concrete selector implementations
pub mod credential;
pub mod snapshot;
pub mod template;

// Re-export commonly used types
pub use base::{SelectableItem, SelectionResult, Selector};
pub use confirmation::ConfirmationService;
pub use error::{SelectorError, SelectorResult};
pub use navigation::{NavigationManager, NavigationResult};
