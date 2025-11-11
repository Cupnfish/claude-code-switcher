use clap::{Args, Parser, Subcommand};
use std::path::PathBuf;

use crate::snapshots::SnapshotScope;

/// Main CLI parser
#[derive(Parser)]
#[command(about, version, author, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

/// Available CLI commands
#[derive(Subcommand)]
pub enum Commands {
    /// List available snapshots [aliases: l, ls]
    #[command(alias = "l", alias = "ls")]
    List {
        /// Show detailed information
        #[arg(long, short, help = "Show detailed information about each snapshot")]
        verbose: bool,
    },

    /// Create a snapshot of current settings [alias: s]
    #[command(alias = "s")]
    Snap {
        /// Name for the snapshot
        name: String,

        /// What to include in the snapshot (default: common)
        #[arg(
            long,
            default_value = "common",
            help = "Scope of settings to include in snapshot"
        )]
        scope: SnapshotScope,

        /// Path to settings file (default: .claude/settings.json)
        #[arg(long, help = "Path to settings file (default: .claude/settings.json)")]
        settings_path: Option<PathBuf>,

        /// Description for the snapshot
        #[arg(long, help = "Description for the snapshot")]
        description: Option<String>,

        /// Overwrite existing snapshot with same name
        #[arg(long, help = "Overwrite existing snapshot with same name")]
        overwrite: bool,
    },

    /// Apply a snapshot or template [alias: a]
    #[command(alias = "a")]
    Apply {
        /// Snapshot name or template type (deepseek, glm, k2, k2-thinking, kat-coder-pro, kat-coder-air, kat-coder, kimi, longcat, minimax, seed-code)
        target: String,

        /// What to include in the snapshot (default: common)
        #[arg(long, default_value = "common", help = "Scope of settings to include")]
        scope: SnapshotScope,

        /// Override model setting
        #[arg(long, help = "Override model setting")]
        model: Option<String>,

        /// Path to settings file (default: .claude/settings.json)
        #[arg(long, help = "Path to settings file (default: .claude/settings.json)")]
        settings_path: Option<PathBuf>,

        /// Backup current settings before applying
        #[arg(long, help = "Create backup of current settings before applying")]
        backup: bool,

        /// Skip confirmation prompt
        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },

    /// Delete a snapshot [aliases: rm, remove, del]
    #[command(alias = "rm", alias = "remove", alias = "del")]
    Delete {
        /// Name of the snapshot to delete
        name: String,

        /// Skip confirmation prompt
        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },

    /// Manage saved credentials
    #[command(subcommand)]
    Credentials(CredentialCommands),
}

/// Credential management commands
#[derive(Subcommand)]
pub enum CredentialCommands {
    /// List saved credentials
    #[command(alias = "ls")]
    List,

    /// Delete a saved credential
    #[command(alias = "rm")]
    Delete {
        /// ID of the credential to delete
        id: String,
    },

    /// Clear all saved credentials
    Clear {
        /// Skip confirmation prompt
        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },
}

/// Arguments for snapshot creation
#[derive(Args, Clone)]
pub struct SnapArgs {
    /// Name for the snapshot
    pub name: String,

    /// What to include in the snapshot (default: common)
    #[arg(
        long,
        default_value = "common",
        help = "Scope of settings to include in snapshot"
    )]
    pub scope: SnapshotScope,

    /// Path to settings file (default: .claude/settings.json)
    #[arg(long, help = "Path to settings file (default: .claude/settings.json)")]
    pub settings_path: Option<PathBuf>,

    /// Description for the snapshot
    #[arg(long, help = "Description for the snapshot")]
    pub description: Option<String>,

    /// Overwrite existing snapshot with same name
    #[arg(long, help = "Overwrite existing snapshot with same name")]
    pub overwrite: bool,
}

/// Arguments for applying snapshots/templates
#[derive(Args, Clone)]
pub struct ApplyArgs {
    /// Snapshot name or template type
    pub target: String,

    /// What to include in the snapshot (default: common)
    #[arg(long, default_value = "common", help = "Scope of settings to include")]
    pub scope: SnapshotScope,

    /// Override model setting
    #[arg(long, help = "Override model setting")]
    pub model: Option<String>,

    /// Path to settings file (default: .claude/settings.json)
    #[arg(long, help = "Path to settings file (default: .claude/settings.json)")]
    pub settings_path: Option<PathBuf>,

    /// Backup current settings before applying
    #[arg(long, help = "Create backup of current settings before applying")]
    pub backup: bool,

    /// Skip confirmation prompt
    #[arg(long, help = "Skip confirmation prompt")]
    pub yes: bool,
}
