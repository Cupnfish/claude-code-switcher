//! Claude Code Switcher - Modular CLI tool for managing Claude Code settings
//!
//! This is a complete refactoring of the original monolithic main.rs file
//! into a modular architecture with trait abstractions for better maintainability.

use clap::Parser;
use claude_code_switcher as ccs;

fn main() -> anyhow::Result<()> {
    let cli = ccs::Cli::parse();

    // Run the command
    ccs::run_command(&cli)?;

    Ok(())
}
