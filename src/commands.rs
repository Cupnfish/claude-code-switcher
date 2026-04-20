use crate::{
    Configurable, CredentialManager, cli,
    credentials::{CredentialStore, get_api_key_cli, get_api_key_interactively},
    settings::{ClaudeSettings, format_settings_comparison, format_settings_for_display},
    snapshots::{self, SnapshotScope, SnapshotStore},
    templates::{TemplateType, get_template_type, resolve_template_cli, resolve_template_interactive},
    utils::{
        backup_settings, confirm_action, get_credentials_dir, get_settings_path, get_snapshots_dir,
    },
};
use anyhow::Result;
use console::style;
use std::collections::HashMap;
use std::path::PathBuf;

/// Common environment variables that should be added to all templates
fn get_common_env_vars() -> HashMap<String, String> {
    let mut env = HashMap::new();
    env.insert("ENABLE_TOOL_SEARCH".to_string(), "true".to_string());
    env
}

/// Inject common environment variables into settings
/// Does not overwrite keys that are already set by the template
fn inject_common_env_vars(settings: &mut ClaudeSettings) {
    if let Some(ref mut env) = settings.env {
        for (key, value) in get_common_env_vars() {
            env.entry(key).or_insert(value);
        }
    } else {
        settings.env = Some(get_common_env_vars());
    }
}

/// Get Git Bash path for CLAUDE_CODE_GIT_BASH_PATH (Windows only)
///
/// Checks template settings and system env first. In CLI mode, auto-detects.
/// In interactive mode, prompts the user. Returns None if already set or skipped.
#[cfg(target_os = "windows")]
fn get_git_bash_path(settings: &ClaudeSettings, cli_mode: bool) -> Result<Option<String>> {
    if settings.env.as_ref().map_or(false, |e| e.contains_key("CLAUDE_CODE_GIT_BASH_PATH")) {
        return Ok(None);
    }

    if let Ok(value) = std::env::var("CLAUDE_CODE_GIT_BASH_PATH") {
        return Ok(Some(value));
    }

    let detected = crate::utils::detect_git_bash_paths();

    if cli_mode {
        return Ok(detected.into_iter().next().map(|p| p.display().to_string()));
    }

    let mut options: Vec<String> = detected.iter().map(|p| p.display().to_string()).collect();
    options.push("Enter custom path...".to_string());
    options.push("Skip".to_string());

    let selection = match inquire::Select::new(
        "Select Git Bash path for CLAUDE_CODE_GIT_BASH_PATH:",
        options,
    )
    .prompt()
    {
        Ok(s) => s,
        Err(_) => return Ok(None),
    };

    if selection == "Skip" {
        return Ok(None);
    }

    if selection == "Enter custom path..." {
        let custom_path = match inquire::Text::new("Enter Git Bash path:")
            .with_validator(|input: &str| {
                let path = PathBuf::from(input.trim());
                if path.exists() {
                    Ok(inquire::validator::Validation::Valid)
                } else {
                    Ok(inquire::validator::Validation::Invalid(
                        "Path does not exist".into(),
                    ))
                }
            })
            .prompt()
        {
            Ok(s) => s,
            Err(_) => return Ok(None),
        };
        return Ok(Some(custom_path));
    }

    Ok(Some(selection))
}

#[cfg(not(target_os = "windows"))]
fn get_git_bash_path(_settings: &ClaudeSettings, _cli_mode: bool) -> Result<Option<String>> {
    Ok(None)
}

/// Prompt user to select effort level
fn prompt_effort_setting(
    template_effort: Option<String>,
    existing_effort: Option<String>,
    effort_param: Option<&str>,
    cli_mode: bool,
) -> Result<Option<String>> {
    // CLI mode: use param or default to xhigh
    if cli_mode {
        return Ok(Some(effort_param.unwrap_or("xhigh").to_string()));
    }

    // If --effort provided in interactive mode, use it directly
    if let Some(level) = effort_param {
        return Ok(Some(level.to_string()));
    }

    // Interactive prompt
    let mut options = vec![
        "xhigh".to_string(),
        "high".to_string(),
        "medium".to_string(),
        "low".to_string(),
    ];

    if let Some(ref e) = existing_effort {
        options.insert(0, format!("Keep existing ({})", e));
    }

    options.push("Skip".to_string());

    let selection = match inquire::Select::new("Select effort level:", options)
        .with_help_message("Controls reasoning depth for Claude Code")
        .prompt()
    {
        Ok(s) => s,
        Err(_) => return Ok(template_effort),
    };

    if selection == "Skip" {
        return Ok(template_effort);
    }

    if selection.starts_with("Keep existing") {
        return Ok(existing_effort);
    }

    Ok(Some(selection))
}

/// Prompt user to configure attribution setting
fn prompt_attribution_setting(
    template_value: Option<crate::settings::Attribution>,
    existing_value: Option<crate::settings::Attribution>,
    no_co_author: bool,
    cli_mode: bool,
) -> Result<Option<crate::settings::Attribution>> {
    use crate::settings::Attribution;

    // If --no-co-author flag is set, disable attribution
    if no_co_author {
        return Ok(Some(Attribution {
            commit: Some(String::new()),
            pr: Some(String::new()),
        }));
    }

    // CLI mode without flag: disable co-author by default
    if cli_mode {
        return Ok(Some(Attribution {
            commit: Some(String::new()),
            pr: Some(String::new()),
        }));
    }

    // Interactive prompt
    let mut options = vec![
        "Disable co-author".to_string(),
        "Enable co-author".to_string(),
    ];

    if existing_value.is_some() {
        let display = match &existing_value {
            Some(a) if a.commit.as_deref() == Some("") => "disabled".to_string(),
            Some(_) => "custom".to_string(),
            None => "default".to_string(),
        };
        options.insert(0, format!("Keep existing ({})", display));
    }

    options.push("Skip".to_string());

    let selection = match inquire::Select::new(
        "Configure attribution for commits and PRs?",
        options,
    )
    .with_help_message("Controls whether Claude adds co-authored-by to git commits and PRs")
    .prompt()
    {
        Ok(s) => s,
        Err(_) => return Ok(template_value),
    };

    if selection == "Skip" {
        return Ok(template_value);
    }

    if selection.starts_with("Keep existing") {
        return Ok(existing_value);
    }

    if selection == "Disable co-author" {
        Ok(Some(Attribution {
            commit: Some(String::new()),
            pr: Some(String::new()),
        }))
    } else {
        // Enable - omit attribution to use Claude Code's default behavior
        Ok(None)
    }
}

/// Run a command based on CLI arguments
pub fn run_command(args: &crate::Cli) -> Result<()> {
    match &args.command {
        cli::Commands::List => {
            list_command()?;
        }
        cli::Commands::Apply {
            target,
            scope,
            model,
            settings_path,
            backup,
            yes,
            cli,
            effort,
            api_key,
            no_co_author,
        } => apply_command(target, scope, model, settings_path, *backup, *yes, *cli, effort, api_key, *no_co_author)?,
        cli::Commands::Credentials { command } => match command {
            cli::CredentialCommands::List => credentials_list_command()?,
            cli::CredentialCommands::Clear { yes } => credentials_clear_command(*yes)?,
        },
    }
    Ok(())
}

/// List available snapshots
pub fn list_command() -> Result<()> {
    // Interactive snapshot browser
    println!("📸 Snapshot Browser");
    println!();

    let mut selector = crate::selectors::snapshot::SnapshotSelector::new()?;

    // Run the interactive selector
    match selector.run_management() {
        Ok(()) => {
            println!("\n👋 Goodbye!");
        }
        Err(e) => {
            // Check if this is a selector cancellation error
            let error_str = e.to_string();
            if error_str.contains("User cancelled selection") {
                println!("\n👋 Cancelled. See you next time!");
            } else {
                println!("\n❌ Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Create a snapshot
pub fn snap_command(
    name: &str,
    scope: &SnapshotScope,
    settings_path: &Option<PathBuf>,
    description: &Option<String>,
    overwrite: bool,
) -> Result<()> {
    let settings_path = get_settings_path(settings_path.clone());
    let settings = ClaudeSettings::from_file(&settings_path)?;

    // Capture environment variables if needed
    let mut snapshot_settings = settings;

    if matches!(scope, SnapshotScope::All | SnapshotScope::Env) {
        snapshot_settings.env = Some(ClaudeSettings::capture_environment());
    }

    let snapshots_dir = crate::utils::get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    if store.exists_by_name(name)
        && !overwrite
        && !confirm_action(
            &format!("Snapshot '{}' already exists. Overwrite?", name),
            false,
        )?
    {
        return Ok(());
    }

    let snapshot = snapshots::Snapshot::new(
        name.to_string(),
        snapshot_settings,
        scope.clone(),
        description.clone(),
    );

    store.save(&snapshot)?;
    println!(
        "{} Snapshot '{}' created successfully!",
        style("✓").green().bold(),
        name
    );

    Ok(())
}

/// Apply a snapshot or template
pub fn apply_command(
    target: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &Option<PathBuf>,
    backup: bool,
    yes: bool,
    cli: bool,
    effort: &Option<String>,
    api_key: &Option<String>,
    no_co_author: bool,
) -> Result<()> {
    let settings_path = get_settings_path(settings_path.clone());
    let yes = yes || cli; // CLI mode implies --yes

    // Try to parse as template type first
    if let Ok(template_type) = get_template_type(target) {
        return apply_template_command(
            &template_type,
            target,
            scope,
            model,
            &settings_path,
            backup,
            yes,
            cli,
            effort,
            api_key,
            no_co_author,
        );
    }

    // Otherwise treat as snapshot name
    apply_snapshot_command(target, scope, model, &settings_path, backup, yes)
}

/// Apply a template
fn apply_template_command(
    template_type: &TemplateType,
    target: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &PathBuf,
    backup: bool,
    yes: bool,
    cli: bool,
    effort: &Option<String>,
    api_key: &Option<String>,
    no_co_author: bool,
) -> Result<()> {
    // Resolve template instance
    let template_instance = if cli {
        resolve_template_cli(template_type, target)?
    } else {
        resolve_template_interactive(template_type, target)?
    };

    // Get API key
    let api_key = if cli {
        get_api_key_cli(template_type.clone(), api_key.as_deref())?
    } else {
        get_api_key_interactively(template_type.clone(), api_key.as_deref())?
    };

    let mut settings = template_instance.create_settings(&api_key, scope);

    // Inject common environment variables
    inject_common_env_vars(&mut settings);

    // Add Windows-specific CLAUDE_CODE_GIT_BASH_PATH
    if let Some(git_bash_path) = get_git_bash_path(&settings, cli)? {
        settings
            .env
            .get_or_insert_with(HashMap::new)
            .insert("CLAUDE_CODE_GIT_BASH_PATH".to_string(), git_bash_path);
    }

    // Override model if specified
    if let Some(model_name) = model {
        settings.model = Some(model_name.clone());
    }

    // Load existing settings for effort prompt and diff comparison
    let existing_settings = ClaudeSettings::from_file(settings_path)?;

    // Set effort level
    settings.effort_level = prompt_effort_setting(
        settings.effort_level.clone(),
        existing_settings.effort_level.clone(),
        effort.as_deref(),
        cli,
    )?;

    // Set attribution
    settings.attribution = prompt_attribution_setting(
        settings.attribution,
        existing_settings.attribution.clone(),
        no_co_author,
        cli,
    )?;

    // Backup current settings if requested
    if backup {
        backup_settings(settings_path)?;
    }

    // Confirm overwrite
    if !yes {
        let existing_masked = existing_settings.clone().mask_sensitive_data();
        let new_masked = settings.clone().mask_sensitive_data();

        let comparison = format_settings_comparison(&existing_masked, &new_masked);

        if comparison == "Settings are identical." {
            println!(
                "{}",
                style("Settings are already configured as requested.").green()
            );
            // Even if settings are identical, we still need to save them in case the user
            // explicitly wanted to ensure these settings are applied (replace mode)
            settings.to_file(settings_path)?;
            return Ok(());
        }

        println!("Changes to be applied:");
        println!("{}", comparison);

        if !confirm_action("Apply these changes?", true)? {
            return Ok(());
        }
    }

    // Save settings (replace mode - no merging)
    settings.to_file(settings_path)?;

    println!(
        "{} Applied template '{}' successfully!",
        style("✓").green().bold(),
        template_type
    );

    Ok(())
}

/// Apply a snapshot
fn apply_snapshot_command(
    snapshot_name: &str,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &PathBuf,
    backup: bool,
    yes: bool,
) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    let mut snapshot = store.load_by_name(snapshot_name)?;

    // Filter settings by scope
    snapshot.settings = snapshot.settings.filter_by_scope(scope);

    // Override model if specified
    if let Some(model_name) = model {
        snapshot.settings.model = Some(model_name.clone());
    }

    // Load existing settings for comparison (will be replaced, not merged)
    let existing_settings = ClaudeSettings::from_file(settings_path)?;

    // Backup current settings if requested
    if backup {
        backup_settings(settings_path)?;
    }

    // Confirm overwrite
    if !yes {
        let existing_masked = existing_settings.clone().mask_sensitive_data();
        let snapshot_masked = snapshot.settings.clone().mask_sensitive_data();

        println!("Current settings:");
        println!("{}", format_settings_for_display(&existing_masked, false));
        println!("\nSnapshot settings:");
        println!("{}", format_settings_for_display(&snapshot_masked, false));

        if !confirm_action("Apply these settings?", true)? {
            return Ok(());
        }
    }

    // Save settings (replace mode - no merging)
    snapshot.settings.to_file(settings_path)?;

    println!(
        "{} Applied snapshot '{}' successfully!",
        style("✓").green().bold(),
        snapshot_name
    );

    Ok(())
}

/// List saved credentials interactively
pub fn credentials_list_command() -> Result<()> {
    println!("🔐 Credential Browser");
    println!();

    let mut selector = crate::selectors::credential::CredentialSelector::new_all()?;

    // Run the interactive selector
    match selector.run_management() {
        Ok(()) => {
            println!("\n👋 Goodbye!");
        }
        Err(e) => {
            // Check if this is a selector cancellation error
            let error_str = e.to_string();
            if error_str.contains("User cancelled selection") {
                println!("\n👋 Cancelled. See you next time!");
            } else {
                println!("\n❌ Error: {}", e);
            }
        }
    }

    Ok(())
}

/// Clear all credentials
pub fn credentials_clear_command(yes: bool) -> Result<()> {
    if !yes && !confirm_action("Clear all saved credentials?", false)? {
        return Ok(());
    }

    let _credentials_dir = get_credentials_dir();
    let credential_store = CredentialStore::new()?;

    credential_store.clear_credentials()?;

    println!("{} Cleared all credentials!", style("✓").green().bold());

    Ok(())
}
