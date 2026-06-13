//! Interactive selection helpers (inquire-based).
//!
//! The old custom crossterm full-screen Selector was removed in favor of a
//! single, consistent prompt style via `inquire`.

pub mod confirmation;
pub mod credential;
pub mod error;
pub mod snapshot;
pub mod template;

// Re-export commonly used types
pub use confirmation::ConfirmationService;
pub use error::{SelectorError, SelectorResult};
