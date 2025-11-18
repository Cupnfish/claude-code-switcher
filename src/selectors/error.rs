//! Unified error handling for selector operations

use std::fmt;

/// Unified error type for all selector operations
#[derive(Debug)]
pub enum SelectorError {
    Cancelled,
    Failed(String),
    NotFound,
    InvalidInput(String),
    OperationFailed(String),
    Io(std::io::Error),
    Storage(String),
}

impl PartialEq for SelectorError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (SelectorError::Cancelled, SelectorError::Cancelled) => true,
            (SelectorError::Failed(a), SelectorError::Failed(b)) => a == b,
            (SelectorError::NotFound, SelectorError::NotFound) => true,
            (SelectorError::InvalidInput(a), SelectorError::InvalidInput(b)) => a == b,
            (SelectorError::OperationFailed(a), SelectorError::OperationFailed(b)) => a == b,
            (SelectorError::Io(a), SelectorError::Io(b)) => {
                a.kind() == b.kind() && a.raw_os_error() == b.raw_os_error()
            }
            (SelectorError::Storage(a), SelectorError::Storage(b)) => a == b,
            _ => false,
        }
    }
}

impl fmt::Display for SelectorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SelectorError::Cancelled => write!(f, "User cancelled selection"),
            SelectorError::Failed(msg) => write!(f, "Selection failed: {}", msg),
            SelectorError::NotFound => write!(f, "Item not found"),
            SelectorError::InvalidInput(msg) => write!(f, "Invalid input: {}", msg),
            SelectorError::OperationFailed(msg) => write!(f, "Operation failed: {}", msg),
            SelectorError::Io(err) => write!(f, "IO error: {}", err),
            SelectorError::Storage(msg) => write!(f, "Storage error: {}", msg),
        }
    }
}

impl std::error::Error for SelectorError {}

impl From<std::io::Error> for SelectorError {
    fn from(err: std::io::Error) -> Self {
        SelectorError::Io(err)
    }
}

impl From<inquire::InquireError> for SelectorError {
    fn from(err: inquire::InquireError) -> Self {
        if err.to_string().contains("canceled") || err.to_string().contains("cancelled") {
            SelectorError::Cancelled
        } else {
            SelectorError::Failed(err.to_string())
        }
    }
}

impl SelectorError {
    /// Check if the error represents a user cancellation
    pub fn is_cancellation(&self) -> bool {
        matches!(self, SelectorError::Cancelled)
    }

    /// Create a cancellation error
    pub fn cancelled() -> Self {
        Self::Cancelled
    }

    /// Create a not found error
    pub fn not_found() -> Self {
        Self::NotFound
    }

    /// Create a failed error with message
    pub fn failed<S: Into<String>>(message: S) -> Self {
        Self::Failed(message.into())
    }
}

/// Result type alias for selector operations
pub type SelectorResult<T> = Result<T, SelectorError>;
