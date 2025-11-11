//! Enhanced confirmation selector with keyboard shortcuts and better UX

use anyhow::{Result, anyhow};
use console::style;
use inquire::{Confirm, Select};

/// Enhanced confirmation selector that supports y/n shortcuts, arrow keys, and q exit
pub struct ConfirmSelector {
    message: String,
    default: bool,
}

impl ConfirmSelector {
    pub fn new(message: &str, default: bool) -> Self {
        Self {
            message: message.to_string(),
            default,
        }
    }

    /// Run the enhanced confirmation selector
    pub fn run(&self) -> Result<bool> {
        // If not in interactive mode, return default
        if !atty::is(atty::Stream::Stdin) {
            return Ok(self.default);
        }

        // First try the enhanced selector
        if let Ok(result) = self.run_enhanced_selector() {
            return Ok(result);
        }

        // Fallback to standard confirm
        Confirm::new(&self.message)
            .with_default(self.default)
            .with_help_message("Press Y for Yes, N for No, or Q to Quit")
            .prompt()
            .map_err(|e| anyhow!("Failed to get confirmation: {}", e))
    }

    fn run_enhanced_selector(&self) -> Result<bool> {
        // Create enhanced options with keyboard shortcuts
        let options = vec![
            format!("✓ Yes {}", style("(Y)").green()),
            format!("✗ No {}", style("(N)").red()),
            format!("⚠ Quit {}", style("(Q)").yellow()),
        ];

        let prompt = self.message.clone();

        match Select::new(&prompt, options).prompt() {
            Ok(choice) => {
                match choice.as_str() {
                    choice if choice.contains("Yes") => Ok(true),
                    choice if choice.contains("No") => Ok(false),
                    choice if choice.contains("Quit") => {
                        println!("{}", style("🚫 Operation cancelled by user.").yellow());
                        std::process::exit(0);
                    }
                    _ => Ok(self.default), // Should not happen, but provide fallback
                }
            }
            Err(e) => Err(anyhow!("Enhanced selector failed: {}", e)),
        }
    }
}

/// Quick confirmation function with enhanced selector
pub fn confirm_with_enhanced_selector(message: &str, default: bool) -> Result<bool> {
    let selector = ConfirmSelector::new(message, default);
    selector.run()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_confirm_selector_creation() {
        let selector = ConfirmSelector::new("Test message", true);
        assert_eq!(selector.message, "Test message");
        assert_eq!(selector.default, true);
    }
}
