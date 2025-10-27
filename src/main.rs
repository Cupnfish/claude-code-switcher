use anyhow::{Result, anyhow};
use chrono::Utc;
use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "claude-switcher")]
#[command(about = "Manage Claude Code settings snapshots with ease")]
#[command(
    long_about = "A CLI tool for creating, managing, and switching between Claude Code settings snapshots.

Perfect for developers who work with multiple AI models or need to switch between different Claude Code configurations.

Examples:
    ccs snap my-env         # Create snapshot of current settings
    ccs apply my-env        # Apply snapshot to current project
    ccs apply deepseek      # Apply DeepSeek template
    ccs apply minimax       # Apply MiniMax Anthropic template
    ccs ls -v               # List snapshots with details"
)]
#[command(version = "0.1.0")]
#[command(author = "Claude Code Switcher Team")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
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
        /// Snapshot name or template type (deepseek, glm, k2, longcat, minimax)
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
}

#[derive(Args)]
struct SnapArgs {
    name: String,
    scope: SnapshotScope,
    settings_path: Option<PathBuf>,
    description: Option<String>,
    overwrite: bool,
}

#[derive(Args)]
struct ApplyArgs {
    target: String,
    scope: SnapshotScope,
    model: Option<String>,
    settings_path: Option<PathBuf>,
    backup: bool,
    yes: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ClaudeSettings {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<std::collections::HashMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_style: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub include_co_authored_by: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permissions: Option<Permissions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hooks: Option<Hooks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_key_helper: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_period_days: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_all_hooks: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_login_org_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enable_all_project_mcp_servers: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled_mcpjson_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled_mcpjson_servers: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_auth_refresh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aws_credential_export: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_line: Option<StatusLine>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subagent_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Snapshot {
    pub id: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub scope: SnapshotScope,
    pub settings: ClaudeSettings,
    pub description: Option<String>,
    #[serde(skip)]
    pub show_api_key: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct SnapshotStore {
    pub snapshots: Vec<Snapshot>,
}

impl SnapshotStore {
    pub fn new() -> Self {
        Self {
            snapshots: Vec::new(),
        }
    }

    pub fn find_snapshot(&self, name: &str) -> Option<&Snapshot> {
        self.snapshots.iter().find(|s| s.name == name)
    }

    pub fn add_snapshot(&mut self, snapshot: Snapshot) {
        self.snapshots.push(snapshot);
    }

    pub fn delete_snapshot(&mut self, name: &str) -> Result<()> {
        let index = self
            .snapshots
            .iter()
            .position(|s| s.name == name)
            .ok_or_else(|| anyhow!("Snapshot '{}' not found", name))?;
        self.snapshots.remove(index);
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Permissions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub allow: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ask: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deny: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_directories: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disable_bypass_permissions_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct Hooks {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pre_command: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub post_command: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StatusLine {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
enum SnapshotScope {
    Env,
    Common,
    All,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum, PartialEq)]
#[clap(rename_all = "lowercase")]
enum TemplateType {
    DeepSeek,
    K2,
    Longcat,
    Zai,
    MiniMax,
}

#[derive(Debug, Clone, Serialize, Deserialize, clap::ValueEnum)]
#[clap(rename_all = "lowercase")]
enum ZaiRegion {
    China,
    International,
}

impl Default for ClaudeSettings {
    fn default() -> Self {
        Self {
            env: None,
            model: None,
            output_style: None,
            include_co_authored_by: None,
            permissions: None,
            hooks: None,
            api_key_helper: None,
            cleanup_period_days: None,
            disable_all_hooks: None,
            force_login_method: None,
            force_login_org_uuid: None,
            enable_all_project_mcp_servers: None,
            enabled_mcpjson_servers: None,
            disabled_mcpjson_servers: None,
            aws_auth_refresh: None,
            aws_credential_export: None,
            status_line: None,
            subagent_model: None,
        }
    }
}

impl std::fmt::Display for SnapshotScope {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SnapshotScope::Env => write!(f, "env"),
            SnapshotScope::Common => write!(f, "common"),
            SnapshotScope::All => write!(f, "all"),
        }
    }
}

impl std::fmt::Display for ZaiRegion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ZaiRegion::China => write!(f, "china"),
            ZaiRegion::International => write!(f, "international"),
        }
    }
}

impl std::fmt::Display for TemplateType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateType::DeepSeek => write!(f, "deepseek"),
            TemplateType::K2 => write!(f, "k2"),
            TemplateType::Longcat => write!(f, "longcat"),
            TemplateType::Zai => write!(f, "zai"),
            TemplateType::MiniMax => write!(f, "minimax"),
        }
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List { verbose } => list_command(verbose),
        Commands::Snap {
            name,
            scope,
            settings_path,
            description,
            overwrite,
        } => snap_command(SnapArgs {
            name,
            scope,
            settings_path,
            description,
            overwrite,
        }),
        Commands::Apply {
            target,
            scope,
            model,
            settings_path,
            backup,
            yes,
        } => apply_command(ApplyArgs {
            target,
            scope,
            model,
            settings_path,
            backup,
            yes,
        }),
        Commands::Delete { name, yes } => delete_command(name, yes),
    }
}

// Helper functions for file operations
fn get_settings_path(settings_path: Option<PathBuf>) -> Result<PathBuf> {
    match settings_path {
        Some(path) => Ok(path),
        None => {
            let current_dir = std::env::current_dir()?;
            let default_path = current_dir.join(".claude").join("settings.json");
            Ok(default_path)
        }
    }
}

fn read_settings_file(path: &Path) -> Result<ClaudeSettings> {
    let content =
        fs::read_to_string(path).map_err(|e| anyhow!("Failed to read settings file: {}", e))?;

    let settings: ClaudeSettings = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse settings JSON: {}", e))?;

    Ok(settings)
}

fn write_settings_file(path: &Path, settings: &ClaudeSettings) -> Result<()> {
    let content = serde_json::to_string_pretty(settings)
        .map_err(|e| anyhow!("Failed to serialize settings: {}", e))?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| anyhow!("Failed to create directory: {}", e))?;
    }

    fs::write(path, content).map_err(|e| anyhow!("Failed to write settings file: {}", e))?;

    Ok(())
}

fn get_snapshot_store_path() -> Result<PathBuf> {
    let mut data_dir = dirs::data_dir().ok_or_else(|| anyhow!("Could not find data directory"))?;
    data_dir.push("claude-switcher");
    data_dir.push("snapshots.json");
    Ok(data_dir)
}

fn read_snapshot_store() -> Result<SnapshotStore> {
    let path = get_snapshot_store_path()?;

    if !path.exists() {
        return Ok(SnapshotStore::new());
    }

    let content =
        fs::read_to_string(&path).map_err(|e| anyhow!("Failed to read snapshot store: {}", e))?;

    let store: SnapshotStore = serde_json::from_str(&content)
        .map_err(|e| anyhow!("Failed to parse snapshot store: {}", e))?;

    Ok(store)
}

fn write_snapshot_store(store: &SnapshotStore) -> Result<()> {
    let path = get_snapshot_store_path()?;

    // Create parent directory if it doesn't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| anyhow!("Failed to create directory: {}", e))?;
    }

    let content = serde_json::to_string_pretty(store)
        .map_err(|e| anyhow!("Failed to serialize snapshot store: {}", e))?;

    fs::write(&path, content).map_err(|e| anyhow!("Failed to write snapshot store: {}", e))?;

    Ok(())
}

fn filter_settings_by_scope(settings: &ClaudeSettings, scope: &SnapshotScope) -> ClaudeSettings {
    match scope {
        SnapshotScope::Env => ClaudeSettings {
            env: settings.env.clone(),
            ..Default::default()
        },
        SnapshotScope::All => settings.clone(),
        SnapshotScope::Common => ClaudeSettings {
            env: settings.env.clone(),
            model: settings.model.clone(),
            output_style: settings.output_style.clone(),
            include_co_authored_by: settings.include_co_authored_by.clone(),
            permissions: settings.permissions.clone(),
            hooks: settings.hooks.clone(),
            status_line: settings.status_line.clone(),
            subagent_model: settings.subagent_model.clone(),
            ..Default::default()
        },
    }
}

// Template functions
fn get_template_type(target: &str) -> Option<TemplateType> {
    match target.to_lowercase().as_str() {
        "deepseek" | "ds" => Some(TemplateType::DeepSeek),
        "glm" | "zhipu" | "zai" => Some(TemplateType::Zai),
        "k2" | "moonshot" => Some(TemplateType::K2),
        "longcat" => Some(TemplateType::Longcat),
        "minimax" | "minimax-anthropic" => Some(TemplateType::MiniMax),
        _ => None,
    }
}

fn create_deepseek_template(api_key: &str) -> ClaudeSettings {
    let mut env = std::collections::HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        "https://api.deepseek.com/anthropic".to_string(),
    );
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
    env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
    env.insert("ANTHROPIC_MODEL".to_string(), "deepseek-chat".to_string());
    env.insert(
        "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
        "deepseek-chat".to_string(),
    );
    env.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        "1".to_string(),
    );

    let permissions = Permissions {
        allow: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebFetch".to_string(),
        ]),
        ask: None,
        deny: Some(vec!["WebSearch".to_string()]),
        additional_directories: None,
        default_mode: None,
        disable_bypass_permissions_mode: None,
    };

    ClaudeSettings {
        env: Some(env),
        model: Some("deepseek-chat".to_string()),
        output_style: None,
        include_co_authored_by: Some(true),
        permissions: Some(permissions),
        hooks: None,
        api_key_helper: None,
        cleanup_period_days: None,
        disable_all_hooks: None,
        force_login_method: None,
        force_login_org_uuid: None,
        enable_all_project_mcp_servers: None,
        enabled_mcpjson_servers: None,
        disabled_mcpjson_servers: None,
        aws_auth_refresh: None,
        aws_credential_export: None,
        status_line: None,
        subagent_model: None,
    }
}

fn create_zai_template(api_key: &str, region: &ZaiRegion) -> ClaudeSettings {
    let mut env = std::collections::HashMap::new();

    // Set base URL based on region
    let base_url = match region {
        ZaiRegion::China => "https://open.bigmodel.cn/api/anthropic",
        ZaiRegion::International => "https://api.z.ai/api/anthropic",
    };

    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
    env.insert("ANTHROPIC_BASE_URL".to_string(), base_url.to_string());
    env.insert("API_TIMEOUT_MS".to_string(), "3000000".to_string());
    env.insert("ANTHROPIC_MODEL".to_string(), "glm-4.6".to_string());
    env.insert(
        "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
        "glm-4.6-air".to_string(),
    );
    env.insert("ENABLE_THINKING".to_string(), "true".to_string());
    env.insert("REASONING_EFFORT".to_string(), "ultrathink".to_string());
    env.insert("MAX_THINKING_TOKENS".to_string(), "32000".to_string());
    env.insert("ENABLE_STREAMING".to_string(), "true".to_string());
    env.insert("MAX_OUTPUT_TOKENS".to_string(), "96000".to_string());
    env.insert("MAX_MCP_OUTPUT_TOKENS".to_string(), "64000".to_string());
    env.insert("AUTH_HEADER_MODE".to_string(), "x-api-key".to_string());

    let permissions = Permissions {
        allow: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebFetch".to_string(),
        ]),
        ask: None,
        deny: Some(vec!["WebSearch".to_string()]),
        additional_directories: None,
        default_mode: None,
        disable_bypass_permissions_mode: None,
    };

    ClaudeSettings {
        env: Some(env),
        model: Some("glm-4.6".to_string()),
        output_style: None,
        include_co_authored_by: Some(true),
        permissions: Some(permissions),
        hooks: None,
        api_key_helper: None,
        cleanup_period_days: None,
        disable_all_hooks: None,
        force_login_method: None,
        force_login_org_uuid: None,
        enable_all_project_mcp_servers: None,
        enabled_mcpjson_servers: None,
        disabled_mcpjson_servers: None,
        aws_auth_refresh: None,
        aws_credential_export: None,
        status_line: None,
        subagent_model: None,
    }
}

fn create_longcat_template(api_key: &str, _think: bool) -> ClaudeSettings {
    let mut env = std::collections::HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        "https://api.longcat.chat/anthropic".to_string(),
    );
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
    env.insert(
        "ANTHROPIC_MODEL".to_string(),
        "LongCat-Flash-Chat".to_string(),
    );
    env.insert(
        "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
        "LongCat-Flash-Chat".to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
        "LongCat-Flash-Chat".to_string(),
    );
    env.insert(
        "CLAUDE_CODE_MAX_OUTPUT_TOKENS".to_string(),
        "8192".to_string(),
    );
    env.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        "1".to_string(),
    );

    let permissions = Permissions {
        allow: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebFetch".to_string(),
        ]),
        ask: None,
        deny: Some(vec!["WebSearch".to_string()]),
        additional_directories: None,
        default_mode: None,
        disable_bypass_permissions_mode: None,
    };

    ClaudeSettings {
        env: Some(env),
        model: Some("LongCat-Flash-Chat".to_string()),
        output_style: None,
        include_co_authored_by: Some(true),
        permissions: Some(permissions),
        hooks: None,
        api_key_helper: None,
        cleanup_period_days: None,
        disable_all_hooks: None,
        force_login_method: None,
        force_login_org_uuid: None,
        enable_all_project_mcp_servers: None,
        enabled_mcpjson_servers: None,
        disabled_mcpjson_servers: None,
        aws_auth_refresh: None,
        aws_credential_export: None,
        status_line: None,
        subagent_model: None,
    }
}

fn create_minimax_template(api_key: &str) -> ClaudeSettings {
    let mut env = std::collections::HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        "https://api.minimax.io/anthropic".to_string(),
    );
    env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
    env.insert("ANTHROPIC_MODEL".to_string(), "MiniMax-M2".to_string());
    env.insert(
        "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
        "MiniMax-M2".to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
        "MiniMax-M2".to_string(),
    );
    env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
    env.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        "1".to_string(),
    );

    let permissions = Permissions {
        allow: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebFetch".to_string(),
        ]),
        ask: None,
        deny: Some(vec!["WebSearch".to_string()]),
        additional_directories: None,
        default_mode: None,
        disable_bypass_permissions_mode: None,
    };

    ClaudeSettings {
        env: Some(env),
        model: Some("MiniMax-M2".to_string()),
        output_style: None,
        include_co_authored_by: Some(true),
        permissions: Some(permissions),
        hooks: None,
        api_key_helper: None,
        cleanup_period_days: None,
        disable_all_hooks: None,
        force_login_method: None,
        force_login_org_uuid: None,
        enable_all_project_mcp_servers: None,
        enabled_mcpjson_servers: None,
        disabled_mcpjson_servers: None,
        aws_auth_refresh: None,
        aws_credential_export: None,
        status_line: None,
        subagent_model: None,
    }
}

fn create_k2_template(api_key: &str) -> ClaudeSettings {
    let mut env = std::collections::HashMap::new();
    env.insert(
        "ANTHROPIC_BASE_URL".to_string(),
        "https://api.moonshot.cn/v1".to_string(),
    );
    env.insert("ANTHROPIC_API_KEY".to_string(), api_key.to_string());
    env.insert("ANTHROPIC_AUTH_TOKEN".to_string(), api_key.to_string());
    env.insert(
        "ANTHROPIC_MODEL".to_string(),
        "kimi-k2-0905-preview".to_string(),
    );
    env.insert(
        "ANTHROPIC_SMALL_FAST_MODEL".to_string(),
        "kimi-k2-0905-preview".to_string(),
    );
    env.insert(
        "ANTHROPIC_DEFAULT_SONNET_MODEL".to_string(),
        "kimi-k2-0905-preview".to_string(),
    );
    env.insert("API_TIMEOUT_MS".to_string(), "600000".to_string());
    env.insert(
        "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC".to_string(),
        "1".to_string(),
    );

    let permissions = Permissions {
        allow: Some(vec![
            "Bash".to_string(),
            "Read".to_string(),
            "Write".to_string(),
            "Edit".to_string(),
            "MultiEdit".to_string(),
            "Glob".to_string(),
            "Grep".to_string(),
            "WebFetch".to_string(),
        ]),
        ask: None,
        deny: Some(vec!["WebSearch".to_string()]),
        additional_directories: None,
        default_mode: None,
        disable_bypass_permissions_mode: None,
    };

    ClaudeSettings {
        env: Some(env),
        model: Some("kimi-k2-0905-preview".to_string()),
        output_style: None,
        include_co_authored_by: Some(true),
        permissions: Some(permissions),
        hooks: None,
        api_key_helper: None,
        cleanup_period_days: None,
        disable_all_hooks: None,
        force_login_method: None,
        force_login_org_uuid: None,
        enable_all_project_mcp_servers: None,
        enabled_mcpjson_servers: None,
        disabled_mcpjson_servers: None,
        aws_auth_refresh: None,
        aws_credential_export: None,
        status_line: None,
        subagent_model: None,
    }
}

fn get_template_api_key(template: &TemplateType) -> Result<String> {
    // Try to get from environment first
    let env_var = match template {
        TemplateType::DeepSeek => "DEEPSEEK_API_KEY",
        TemplateType::Zai => "Z_AI_API_KEY",
        TemplateType::K2 => "MOONSHOT_API_KEY",
        TemplateType::Longcat => "LONGCAT_API_KEY",
        TemplateType::MiniMax => "MINIMAX_API_KEY",
    };

    if let Ok(key) = std::env::var(env_var) {
        println!("  ✓ Using API key from environment variable {}", env_var);
        return Ok(key);
    }

    // If not found and we're in non-interactive mode, error
    if !atty::is(atty::Stream::Stdin) {
        return Err(anyhow!(
            "API key required for {} template. Set {} environment variable or use interactive mode.",
            template,
            env_var
        ));
    }

    // Prompt user for API key
    use inquire::{Confirm, Text};

    let prompt = format!("Enter API key for {} template:", template);
    let key = Text::new(&prompt)
        .prompt()
        .map_err(|e| anyhow!("Failed to read input: {}", e))?;

    if key.trim().is_empty() {
        return Err(anyhow!("API key cannot be empty"));
    }

    // Ask if user wants to save to environment
    let save_env = Confirm::new(&format!(
        "Save {} to environment variable for future use?",
        env_var
    ))
    .with_default(false)
    .prompt()
    .unwrap_or(false);

    if save_env {
        println!("  💡 To save permanently, add this to your shell profile:");
        println!("     export {}=\"***\"", env_var);
    }

    Ok(key)
}

fn apply_template(
    template: &TemplateType,
    scope: &SnapshotScope,
    model_override: Option<String>,
) -> Result<Snapshot> {
    // Get API key from environment or prompt user
    let api_key = get_template_api_key(template)?;

    // Create template settings
    let mut template_settings = match template {
        TemplateType::DeepSeek => create_deepseek_template(&api_key),
        TemplateType::Zai => create_zai_template(&api_key, &ZaiRegion::China),
        TemplateType::K2 => create_k2_template(&api_key),
        TemplateType::Longcat => create_longcat_template(&api_key, false),
        TemplateType::MiniMax => create_minimax_template(&api_key),
    };

    // Apply model override if specified
    if let Some(model) = model_override {
        if let Some(ref mut env) = template_settings.env {
            env.insert("ANTHROPIC_MODEL".to_string(), model.clone());
        } else {
            let mut env = std::collections::HashMap::new();
            env.insert("ANTHROPIC_MODEL".to_string(), model.clone());
            template_settings.env = Some(env);
        }
        if template_settings.model.is_none() {
            template_settings.model = Some(model.clone());
        }
    }

    // Filter settings based on scope
    let filtered_settings = filter_settings_by_scope(&template_settings, scope);

    Ok(Snapshot {
        id: Uuid::new_v4().to_string(),
        name: format!("{}-template", template),
        created_at: Utc::now(),
        scope: scope.clone(),
        settings: filtered_settings,
        description: Some(format!("Applied from {} template", template)),
        show_api_key: false,
    })
}

// Environment and settings functions
fn read_user_settings() -> Result<ClaudeSettings> {
    let home_dir = dirs::home_dir().ok_or_else(|| anyhow!("Could not find home directory"))?;
    let user_settings_path = home_dir.join(".claude").join("settings.json");

    if user_settings_path.exists() {
        read_settings_file(&user_settings_path)
    } else {
        Ok(ClaudeSettings::default())
    }
}

fn read_project_settings(project_dir: &Path) -> Result<ClaudeSettings> {
    let shared_path = project_dir.join(".claude").join("settings.json");
    let local_path = project_dir.join(".claude").join("settings.local.json");

    let shared_settings = if shared_path.exists() {
        read_settings_file(&shared_path)?
    } else {
        ClaudeSettings::default()
    };

    let local_settings = if local_path.exists() {
        read_settings_file(&local_path)?
    } else {
        ClaudeSettings::default()
    };

    Ok(merge_settings(shared_settings, local_settings))
}

fn read_enterprise_settings() -> Result<ClaudeSettings> {
    #[cfg(target_os = "windows")]
    let enterprise_path = Path::new("C:\\ProgramData\\ClaudeCode\\managed-settings.json");

    #[cfg(target_os = "macos")]
    let enterprise_path =
        Path::new("/Library/Application Support/ClaudeCode/managed-settings.json");

    #[cfg(target_os = "linux")]
    let enterprise_path = Path::new("/etc/claude-code/managed-settings.json");

    if enterprise_path.exists() {
        read_settings_file(enterprise_path)
    } else {
        Ok(ClaudeSettings::default())
    }
}

// List of Claude Code environment variables to capture
const CLAUDE_ENV_VARS: &[&str] = &[
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "ANTHROPIC_CUSTOM_HEADERS",
    "ANTHROPIC_DEFAULT_HAIKU_MODEL",
    "ANTHROPIC_DEFAULT_OPUS_MODEL",
    "ANTHROPIC_DEFAULT_SONNET_MODEL",
    "ANTHROPIC_MODEL",
    "ANTHROPIC_SMALL_FAST_MODEL",
    "CLAUDE_CODE_MAX_OUTPUT_TOKENS",
    "CLAUDE_CODE_SUBAGENT_MODEL",
    "DISABLE_TELEMETRY",
];

fn capture_environment_variables() -> ClaudeSettings {
    let mut env_map = std::collections::HashMap::new();

    for &var_name in CLAUDE_ENV_VARS {
        if let Ok(value) = std::env::var(var_name) {
            env_map.insert(var_name.to_string(), value);
        }
    }

    ClaudeSettings {
        env: if env_map.is_empty() {
            None
        } else {
            Some(env_map)
        },
        ..Default::default()
    }
}

fn merge_settings(base: ClaudeSettings, override_settings: ClaudeSettings) -> ClaudeSettings {
    ClaudeSettings {
        env: merge_hashmaps(base.env, override_settings.env),
        model: override_settings.model.or(base.model),
        output_style: override_settings.output_style.or(base.output_style),
        include_co_authored_by: override_settings
            .include_co_authored_by
            .or(base.include_co_authored_by),
        permissions: merge_permissions(base.permissions, override_settings.permissions),
        hooks: merge_hooks(base.hooks, override_settings.hooks),
        api_key_helper: override_settings.api_key_helper.or(base.api_key_helper),
        cleanup_period_days: override_settings
            .cleanup_period_days
            .or(base.cleanup_period_days),
        disable_all_hooks: override_settings
            .disable_all_hooks
            .or(base.disable_all_hooks),
        force_login_method: override_settings
            .force_login_method
            .or(base.force_login_method),
        force_login_org_uuid: override_settings
            .force_login_org_uuid
            .or(base.force_login_org_uuid),
        enable_all_project_mcp_servers: override_settings
            .enable_all_project_mcp_servers
            .or(base.enable_all_project_mcp_servers),
        enabled_mcpjson_servers: merge_vec(
            base.enabled_mcpjson_servers,
            override_settings.enabled_mcpjson_servers,
        ),
        disabled_mcpjson_servers: merge_vec(
            base.disabled_mcpjson_servers,
            override_settings.disabled_mcpjson_servers,
        ),
        aws_auth_refresh: override_settings.aws_auth_refresh.or(base.aws_auth_refresh),
        aws_credential_export: override_settings
            .aws_credential_export
            .or(base.aws_credential_export),
        status_line: override_settings.status_line.or(base.status_line),
        subagent_model: override_settings.subagent_model.or(base.subagent_model),
    }
}

fn merge_hashmaps<K: Clone + Eq + std::hash::Hash, V: Clone>(
    base: Option<HashMap<K, V>>,
    override_settings: Option<HashMap<K, V>>,
) -> Option<HashMap<K, V>> {
    match (base, override_settings) {
        (Some(mut base_map), Some(override_map)) => {
            base_map.extend(override_map);
            Some(base_map)
        }
        (Some(base_map), None) => Some(base_map),
        (None, Some(override_map)) => Some(override_map),
        (None, None) => None,
    }
}

fn merge_permissions(
    base: Option<Permissions>,
    override_settings: Option<Permissions>,
) -> Option<Permissions> {
    match (base, override_settings) {
        (Some(base_perms), Some(override_perms)) => Some(Permissions {
            allow: merge_vec(base_perms.allow, override_perms.allow),
            ask: merge_vec(base_perms.ask, override_perms.ask),
            deny: merge_vec(base_perms.deny, override_perms.deny),
            additional_directories: merge_vec(
                base_perms.additional_directories,
                override_perms.additional_directories,
            ),
            default_mode: override_perms.default_mode.or(base_perms.default_mode),
            disable_bypass_permissions_mode: override_perms
                .disable_bypass_permissions_mode
                .or(base_perms.disable_bypass_permissions_mode),
        }),
        (Some(base_perms), None) => Some(base_perms),
        (None, Some(override_perms)) => Some(override_perms),
        (None, None) => None,
    }
}

fn merge_hooks(base: Option<Hooks>, override_settings: Option<Hooks>) -> Option<Hooks> {
    match (base, override_settings) {
        (Some(base_hooks), Some(override_hooks)) => Some(Hooks {
            pre_command: merge_vec(base_hooks.pre_command, override_hooks.pre_command),
            post_command: merge_vec(base_hooks.post_command, override_hooks.post_command),
        }),
        (Some(base_hooks), None) => Some(base_hooks),
        (None, Some(override_hooks)) => Some(override_hooks),
        (None, None) => None,
    }
}

fn merge_vec<T: Clone>(base: Option<Vec<T>>, override_settings: Option<Vec<T>>) -> Option<Vec<T>> {
    match (base, override_settings) {
        (Some(mut base_vec), Some(override_vec)) => {
            base_vec.extend(override_vec);
            Some(base_vec)
        }
        (Some(base_vec), None) => Some(base_vec),
        (None, Some(override_vec)) => Some(override_vec),
        (None, None) => None,
    }
}

fn get_merged_settings(
    settings_path: Option<PathBuf>,
    capture_env: bool,
) -> Result<ClaudeSettings> {
    // 1. User settings (lowest priority)
    let user_settings = read_user_settings()?;

    // 2. Project settings (local overrides shared)
    let project_dir = std::env::current_dir()?;
    let project_settings = read_project_settings(&project_dir)?;

    // 3. Enterprise settings
    let enterprise_settings = read_enterprise_settings()?;

    // Merge in priority order: user -> project -> enterprise
    let mut merged = merge_settings(user_settings, project_settings);
    merged = merge_settings(merged, enterprise_settings);

    // If a specific settings file is provided, treat it as highest priority
    if let Some(custom_path) = settings_path {
        if custom_path.exists() {
            let custom_settings = read_settings_file(&custom_path)?;
            merged = merge_settings(merged, custom_settings);
        }
    }

    // 4. Environment variables (highest priority when capturing)
    if capture_env {
        let env_settings = capture_environment_variables();
        merged = merge_settings(merged, env_settings);
    }

    Ok(merged)
}

fn format_settings_for_display(settings: &ClaudeSettings, scope: &SnapshotScope) -> String {
    let filtered = filter_settings_by_scope(settings, scope);
    let mut output = String::new();

    output.push_str(&format!("Scope: {}\n", scope));
    output.push_str("=\n");

    if let Some(env) = &filtered.env {
        output.push_str("Environment Variables:\n");
        for (key, value) in env {
            let display_value = if key.contains("API_KEY")
                || key.contains("AUTH_TOKEN")
                || key.contains("TOKEN")
                || key.contains("SECRET")
                || key.contains("PASSWORD")
                || key.contains("PRIVATE_KEY")
            {
                mask_api_key(value)
            } else {
                value.clone()
            };
            output.push_str(&format!("  {} = {}\n", key, display_value));
        }
        output.push('\n');
    }

    if let Some(model) = &filtered.model {
        output.push_str(&format!("Model: {}\n\n", model));
    }

    if let Some(output_style) = &filtered.output_style {
        output.push_str(&format!("Output Style: {}\n\n", output_style));
    }

    if let Some(include_co_authored_by) = filtered.include_co_authored_by {
        output.push_str(&format!(
            "Include Co-Authored-By: {}\n\n",
            include_co_authored_by
        ));
    }

    if let Some(permissions) = &filtered.permissions {
        output.push_str("Permissions:\n");
        if let Some(allow) = &permissions.allow {
            output.push_str("  Allow:\n");
            for item in allow {
                output.push_str(&format!("    - {}\n", item));
            }
        }
        if let Some(ask) = &permissions.ask {
            output.push_str("  Ask:\n");
            for item in ask {
                output.push_str(&format!("    - {}\n", item));
            }
        }
        if let Some(deny) = &permissions.deny {
            output.push_str("  Deny:\n");
            for item in deny {
                output.push_str(&format!("    - {}\n", item));
            }
        }
        output.push('\n');
    }

    if output.ends_with("\n\n") {
        output.pop();
        output.pop();
    }

    output
}

fn mask_api_key(api_key: &str) -> String {
    if api_key.starts_with("sk-") {
        let actual_key = &api_key[3..];
        let actual_len = actual_key.len();

        if actual_len <= 6 {
            format!("sk-{}", "*".repeat(actual_len))
        } else if actual_len <= 14 {
            format!(
                "sk-{}***{}",
                &actual_key[..2],
                &actual_key[actual_len - 3..]
            )
        } else {
            format!(
                "sk-{}{}...{} ({} chars)",
                &actual_key[..3],
                "*".repeat(std::cmp::min(actual_len - 7, 8)),
                &actual_key[actual_len - 4..],
                api_key.len()
            )
        }
    } else {
        if api_key.len() <= 8 {
            "*".repeat(api_key.len())
        } else if api_key.len() <= 16 {
            format!("{}***{}", &api_key[..3], &api_key[api_key.len() - 3..])
        } else {
            let visible_start = &api_key[..4];
            let visible_end = &api_key[api_key.len() - 4..];
            let masked_length = api_key.len() - 8;
            format!(
                "{}{}...{} ({} chars)",
                visible_start,
                "*".repeat(std::cmp::min(masked_length, 8)),
                visible_end,
                api_key.len()
            )
        }
    }
}

impl ClaudeSettings {
    fn mask_api_keys(&self) -> Self {
        let mut masked = self.clone();
        if let Some(ref mut env) = masked.env {
            let keys_to_mask: Vec<String> = env
                .keys()
                .filter(|key| {
                    key.contains("API_KEY") || key.contains("AUTH_TOKEN") || key.contains("TOKEN")
                })
                .cloned()
                .collect();

            for key in keys_to_mask {
                if let Some(value) = env.get(&key) {
                    env.insert(key, mask_api_key(value));
                }
            }
        }
        masked
    }
}

// Command implementations
fn snap_command(args: SnapArgs) -> Result<()> {
    let SnapArgs {
        name,
        scope,
        settings_path,
        description,
        overwrite,
    } = args;

    println!("📸 Creating snapshot '{}' with scope '{}'", name, scope);

    // Get merged settings from all sources including environment variables
    let merged_settings = get_merged_settings(settings_path.clone(), true)?;

    // Filter settings based on scope
    let filtered_settings = filter_settings_by_scope(&merged_settings, &scope);

    // Create snapshot
    let snapshot = Snapshot {
        id: Uuid::new_v4().to_string(),
        name: name.clone(),
        created_at: Utc::now(),
        scope: scope.clone(),
        settings: filtered_settings,
        description,
        show_api_key: false, // Default to masked for snapshots
    };

    // Load existing snapshots
    let mut store = read_snapshot_store()?;

    // Check if snapshot with same name already exists
    if let Some(existing) = store.find_snapshot(&name) {
        if !overwrite {
            return Err(anyhow!(
                "Snapshot '{}' already exists (created {}). Use --overwrite to replace it.",
                name,
                existing.created_at.format("%Y-%m-%d %H:%M:%S UTC")
            ));
        } else {
            // Remove existing snapshot
            store.delete_snapshot(&name)?;
            println!("  ✏️  Replacing existing snapshot");
        }
    }

    // Add new snapshot
    store.add_snapshot(snapshot);

    // Save snapshot store
    write_snapshot_store(&store)?;

    println!("✅ Snapshot '{}' created successfully!", name);
    println!("  📦 Scope: {}", scope);
    println!(
        "  🕒 Created: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );
    println!(
        "  📖 Source: Merged settings from user, project, enterprise, and environment variables"
    );

    Ok(())
}

fn apply_command(args: ApplyArgs) -> Result<()> {
    let ApplyArgs {
        target,
        scope,
        model,
        settings_path,
        backup,
        yes,
    } = args;

    // Check if target is a template or snapshot
    let template_type = get_template_type(&target);

    let snapshot = if let Some(template) = template_type {
        println!("🎯 Applying template '{}' with scope '{}'", target, scope);
        apply_template(&template, &scope, model)?
    } else {
        println!("🎯 Applying snapshot '{}' with scope '{}'", target, scope);
        // Load snapshots
        let store = read_snapshot_store()?;

        // Find the snapshot
        let found_snapshot = store.find_snapshot(&target).ok_or_else(|| {
            anyhow!(
                "Snapshot '{}' not found. Use 'ccs ls' to see available snapshots.",
                target
            )
        })?;

        // Apply model override if specified
        let mut settings = found_snapshot.settings.clone();
        if let Some(model_override) = model {
            if let Some(ref mut env) = settings.env {
                env.insert("ANTHROPIC_MODEL".to_string(), model_override.clone());
            } else {
                let mut env = std::collections::HashMap::new();
                env.insert("ANTHROPIC_MODEL".to_string(), model_override.clone());
                settings.env = Some(env);
            }
            if settings.model.is_none() {
                settings.model = Some(model_override.clone());
            }
        }

        Snapshot {
            id: found_snapshot.id.clone(),
            name: found_snapshot.name.clone(),
            created_at: found_snapshot.created_at,
            scope: found_snapshot.scope.clone(),
            settings,
            description: found_snapshot.description.clone(),
            show_api_key: found_snapshot.show_api_key,
        }
    };

    // Get the target settings file path
    let target_path = get_settings_path(settings_path)?;

    // Show preview of what will be applied with better UI
    if !yes && atty::is(atty::Stream::Stdin) {
        use inquire::Select;

        println!("\n🎯 Target: {}", target_path.display());
        println!("{}", "─".repeat(60));

        let preview = format_settings_for_display(&snapshot.settings, &snapshot.scope);
        println!("{}", preview);
        println!("{}", "─".repeat(60));

        let options = vec!["✅ Apply these settings", "❌ Cancel"];
        let choice = Select::new("What would you like to do?", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get user input: {}", e))?;

        if choice == "❌ Cancel" {
            println!("❌ Operation cancelled.");
            return Ok(());
        }
    }

    // Create backup if requested
    if backup {
        let backup_path =
            target_path.with_extension(format!("json.backup.{}", Utc::now().timestamp()));

        // Read current settings for backup
        if target_path.exists() {
            let current_settings = read_settings_file(&target_path)?;
            write_settings_file(&backup_path, &current_settings)?;
            println!("  💾 Backup created at: {}", backup_path.display());
        } else {
            println!("  ℹ️  No existing settings file to backup");
        }
    }

    // Apply the snapshot settings
    write_settings_file(&target_path, &snapshot.settings)?;

    println!("✅ Settings applied successfully!");
    println!("  📁 Target: {}", target_path.display());
    println!("  🎯 Source: {}", snapshot.name);
    println!("  📦 Scope: {}", snapshot.scope);
    if let Some(desc) = &snapshot.description {
        println!("  📝 {}", desc);
    }
    println!(
        "  🕒 Applied: {}",
        Utc::now().format("%Y-%m-%d %H:%M:%S UTC")
    );

    Ok(())
}

fn delete_command(name: String, yes: bool) -> Result<()> {
    let mut store = read_snapshot_store()?;

    // Check if snapshot exists
    let snapshot = match store.find_snapshot(&name) {
        Some(s) => s,
        None => {
            return Err(anyhow!(
                "Snapshot '{}' not found. Use 'ccs ls' to see available snapshots.",
                name
            ));
        }
    };

    // Show confirmation prompt if not bypassed
    if !yes && atty::is(atty::Stream::Stdin) {
        use inquire::Select;

        println!("🗑️  Delete snapshot '{}'", name);
        println!(
            "📅 Created: {}",
            snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC")
        );
        println!("🎯 Scope: {}", snapshot.scope);
        if let Some(desc) = &snapshot.description {
            println!("📝 {}", desc);
        }

        let options = vec!["🗑️  Yes, delete it", "❌ No, keep it"];
        let choice = Select::new("Are you sure you want to delete this snapshot?", options)
            .prompt()
            .map_err(|e| anyhow!("Failed to get user input: {}", e))?;

        if choice == "❌ No, keep it" {
            println!("❌ Operation cancelled.");
            return Ok(());
        }
    }

    // Delete the snapshot
    store.delete_snapshot(&name)?;

    // Save the updated store
    write_snapshot_store(&store)?;

    println!("✅ Snapshot '{}' deleted successfully", name);

    Ok(())
}

fn list_command(verbose: bool) -> Result<()> {
    let store = read_snapshot_store()?;

    println!("📋 Claude Code Switcher");

    if store.snapshots.is_empty() {
        println!("\n❌ No snapshots found.");
        println!("💡 Use 'ccs snap <name>' to create your first snapshot.");
        println!("💡 Or use 'ccs apply deepseek' to apply a template directly.");
        return Ok(());
    }

    if verbose {
        println!(
            "\n📦 Available snapshots ({} total):",
            store.snapshots.len()
        );
        println!("{}", "═".repeat(80));

        for (index, snapshot) in store.snapshots.iter().enumerate() {
            let age = Utc::now().signed_duration_since(snapshot.created_at);
            let age_text = if age.num_days() > 0 {
                format!("{}d ago", age.num_days())
            } else if age.num_hours() > 0 {
                format!("{}h ago", age.num_hours())
            } else if age.num_minutes() > 0 {
                format!("{}m ago", age.num_minutes())
            } else {
                "just now".to_string()
            };

            println!("{}. {}", index + 1, console::style(&snapshot.name).bold());
            println!(
                "   📅 Created: {} ({})",
                snapshot.created_at.format("%Y-%m-%d %H:%M:%S UTC"),
                age_text
            );
            println!("   🎯 Scope: {}", snapshot.scope);

            if let Some(desc) = &snapshot.description {
                println!("   📝 {}", desc);
            }

            // Show what settings are included with icons
            let settings = &snapshot.settings;
            let mut included = Vec::new();

            if settings.env.is_some() {
                included.push("🔧 env");
            }
            if settings.model.is_some() {
                included.push("🤖 model");
            }
            if settings.permissions.is_some() {
                included.push("🔐 permissions");
            }
            if settings.output_style.is_some() {
                included.push("🎨 output_style");
            }
            if settings.hooks.is_some() {
                included.push("⚡ hooks");
            }

            if !included.is_empty() {
                println!("   📦 {}", included.join(" • "));
            }

            // Show detailed content in verbose mode
            println!("   📋 Content:");
            let display_settings = if snapshot.show_api_key {
                snapshot.settings.clone()
            } else {
                snapshot.settings.mask_api_keys()
            };
            let settings_preview = format_settings_for_display(&display_settings, &snapshot.scope);
            let preview_lines: Vec<&str> = settings_preview.lines().collect();
            for line in preview_lines {
                if !line.trim().is_empty() && !line.starts_with("Scope:") && !line.starts_with("=")
                {
                    println!("     {}", line);
                }
            }

            println!();
        }
    } else {
        println!("\nAvailable snapshots ({}):", store.snapshots.len());
        println!("{}", "─".repeat(50));

        for snapshot in &store.snapshots {
            let age = Utc::now().signed_duration_since(snapshot.created_at);
            let age_text = if age.num_days() > 0 {
                format!("{}d", age.num_days())
            } else if age.num_hours() > 0 {
                format!("{}h", age.num_hours())
            } else if age.num_minutes() > 0 {
                format!("{}m", age.num_minutes())
            } else {
                "now".to_string()
            };

            println!(
                "• {}  ({}, {})",
                console::style(&snapshot.name).bold(),
                age_text,
                snapshot.scope
            );
        }

        println!("\nUse -v or --verbose for more details.");
    }

    // Show available templates
    println!("\n🎯 Available templates (use 'ccs apply <template>'):");
    println!("  🚀 deepseek          - DeepSeek Chat API");
    println!("  🤖 glm               - GLM/Zhipu AI");
    println!("  🌙 k2                - Moonshot K2 API");
    println!("  🐱 longcat           - Longcat Chat API");
    println!("  🔥 minimax           - MiniMax API (recommended)");

    println!("\n💡 Examples:");
    println!("  ccs apply deepseek                    # Apply DeepSeek template");
    println!("  ccs apply glm --model glm-4-plus     # Apply GLM with custom model");
    println!("  ccs apply k2                          # Apply Moonshot K2 template");
    println!("  ccs apply minimax                     # Apply MiniMax template");
    println!("  ccs apply my-snapshot --backup       # Apply snapshot with backup");

    Ok(())
}
