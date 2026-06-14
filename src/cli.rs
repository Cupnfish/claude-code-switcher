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
    /// List and manage snapshots [aliases: l, ls]
    #[command(alias = "l", alias = "ls")]
    List,

    /// Apply a snapshot or template [alias: a]
    #[command(alias = "a")]
    Apply {
        /// Snapshot name or template type
        /// (deepseek, glm, k2, k2-thinking, kat-coder, kimi, longcat, fishtrip,
        /// minimax, seed-code, zenmux, duojie, anyrouter, openrouter, beeapi, day77)
        target: String,

        /// What to include (default: common). env = only env vars; common =
        /// env+model+permissions+hooks; all = everything.
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

        /// Skip the confirmation prompt (apply directly)
        #[arg(long, short = 'y', help = "Skip confirmation / apply directly")]
        yes: bool,

        /// Deprecated: non-interactive mode. Now automatic when stdin isn't a TTY.
        #[arg(long, hide = true, help = "Non-interactive mode (deprecated)")]
        cli: bool,

        /// Effort level override (max/xhigh/high/medium/low)
        #[arg(long, help = "Set effort level (overrides the default in your config)")]
        effort: Option<String>,

        /// Auto-compact threshold for supported 1M-context providers (896k/768k/512k/256k)
        #[arg(
            long = "auto-compact",
            visible_alias = "compact",
            alias = "context",
            help = "Set auto-compact threshold for supported 1M providers (896k/768k/512k/256k)"
        )]
        auto_compact: Option<String>,

        /// API key to use (skips interactive selection)
        #[arg(
            long,
            visible_alias = "key",
            help = "API key to use (skips interactive selection)"
        )]
        api_key: Option<String>,

        /// Disable co-authored-by attribution in commits/PRs
        #[arg(long, help = "Disable co-authored-by attribution")]
        no_co_author: bool,

        /// Force the API-key picker even if a key is remembered
        #[arg(long, help = "Force the API-key picker (ignore remembered key)")]
        switch_key: bool,

        /// Preview the result without writing anything
        #[arg(long, help = "Preview changes without writing settings")]
        dry_run: bool,

        /// Specific variant alias for generic targets (e.g. zai-china, k2, kat-coder-air)
        #[arg(long, help = "Specific variant alias (e.g. zai-china, k2)")]
        variant: Option<String>,
    },

    /// Manage saved credentials [aliases: creds, cred]
    #[command(alias = "creds", alias = "cred")]
    Credentials {
        /// Subcommand for credential management
        #[command(subcommand)]
        command: CredentialCommands,
    },

    /// View or edit persistent preferences [alias: cfg]
    #[command(alias = "cfg")]
    Config(ConfigArgs),

    /// Show the currently-active provider [alias: status]
    #[command(alias = "status")]
    Current,
}

/// Arguments for `ccs config`
#[derive(Args, Clone, Debug)]
pub struct ConfigArgs {
    /// Set the default effort level (max/xhigh/high/medium/low)
    #[arg(long, help = "Set default effort (max/xhigh/high/medium/low)")]
    pub effort: Option<String>,

    /// Enable or disable co-authored-by in commits/PRs. Pass without a value to
    /// enable (`--co-author`), or `--co-author false` to disable.
    #[arg(
        long,
        action = clap::ArgAction::Set,
        num_args = 0..=1,
        default_missing_value = "true",
        help = "Enable/disable co-authored-by (true|false)"
    )]
    pub co_author: Option<bool>,

    /// Set the default apply scope (env/common/all)
    #[arg(long, help = "Set default apply scope (env/common/all)")]
    pub scope: Option<SnapshotScope>,

    /// Reset all preferences to defaults
    #[arg(long, help = "Reset all preferences to defaults")]
    pub reset: bool,
}

/// Credential management commands
#[derive(Subcommand)]
pub enum CredentialCommands {
    /// List saved credentials [aliases: l, ls]
    #[command(alias = "l", alias = "ls")]
    List,

    /// Clear all saved credentials
    Clear {
        /// Skip confirmation prompt
        #[arg(long, help = "Skip confirmation prompt")]
        yes: bool,
    },
}
