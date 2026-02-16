use crate::{
    Configurable, CredentialManager, cli,
    credentials::{CredentialStore, get_api_key_interactively},
    settings::{ClaudeSettings, format_settings_comparison, format_settings_for_display},
    snapshots::{self, SnapshotScope, SnapshotStore},
    templates::{Template, TemplateType, get_template_instance_with_input, get_template_type},
    utils::{
        backup_settings, confirm_action, get_credentials_dir, get_settings_path, get_snapshots_dir,
    },
};
use anyhow::{Result, anyhow};
use console::style;
use std::path::PathBuf;

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
        } => apply_command(target, scope, model, settings_path, *backup, *yes)?,
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
) -> Result<()> {
    let settings_path = get_settings_path(settings_path.clone());

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
) -> Result<()> {
    // Get template instance with the original input to handle specific variants
    let initial_template = get_template_instance_with_input(template_type, target);

    // If template has variants and user didn't specify a specific one, let user choose interactively
    let template_instance = if initial_template.has_variants()
        && ((target == "kat-coder" || target == "katcoder" || target == "kat")
            || (target == "kimi")
            || (target == "zai" || target == "glm" || target == "zhipu")
            || (target == "anyrouter" || target == "anyr" || target == "ar"))
    {
        // Use template's interactive creation method
        match template_type {
            crate::templates::TemplateType::KatCoder => {
                let kat_coder_template =
                    crate::templates::kat_coder::KatCoderTemplate::create_interactively()?;
                Box::new(kat_coder_template) as Box<dyn Template>
            }
            crate::templates::TemplateType::Kimi => {
                let kimi_template = crate::templates::kimi::KimiTemplate::create_interactively()?;
                Box::new(kimi_template) as Box<dyn Template>
            }
            crate::templates::TemplateType::Zai => {
                let zai_template = crate::templates::zai::ZaiTemplate::create_interactively()?;
                Box::new(zai_template) as Box<dyn Template>
            }
            crate::templates::TemplateType::AnyRouter => {
                let anyrouter_template =
                    crate::templates::anyrouter::AnyRouterTemplate::create_interactively()?;
                Box::new(anyrouter_template) as Box<dyn Template>
            }
            _ => initial_template,
        }
    } else {
        initial_template
    };

    // Get API key using the new multi-env-var support
    let api_key = {
        let env_var_names = template_instance.env_var_names();
        let mut env_vars_with_keys = Vec::new();

        // Check each environment variable name in order
        for env_var_name in &env_var_names {
            if let Some(api_key) = std::env::var(env_var_name)
                .ok()
                .filter(|key| !key.trim().is_empty())
            {
                env_vars_with_keys.push((env_var_name, api_key));
            }
        }

        // Let user choose between env var and custom API key if env var exists
        if !env_vars_with_keys.is_empty() {
            use inquire::Select;

            let mut options = Vec::new();
            for (env_var_name, _) in &env_vars_with_keys {
                options.push(format!(
                    "Use API key from environment variable {}",
                    env_var_name
                ));
            }
            options.push("Enter a custom API key".to_string());

            let choice = Select::new("API key source:", options)
                .prompt()
                .map_err(|e| anyhow!("Failed to get API key source selection: {}", e))?;

            // Find which env var was selected
            let mut selected_api_key: Option<String> = None;
            for (env_var_name, api_key) in &env_vars_with_keys {
                if choice.contains(&format!(
                    "Use API key from environment variable {}",
                    env_var_name
                )) {
                    println!("✓ Using API key from environment variable {}", env_var_name);
                    selected_api_key = Some(api_key.clone());
                    break;
                }
            }

            // Return the selected API key or get custom one
            if let Some(api_key) = selected_api_key {
                api_key
            } else {
                get_api_key_interactively(template_type.clone())?
            }
        } else {
            // No env vars found, use interactive credential selector
            get_api_key_interactively(template_type.clone())?
        }
    };

    let mut settings = template_instance.create_settings(&api_key, scope);

    // Override model if specified
    if let Some(model_name) = model {
        settings.model = Some(model_name.clone());
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

        if !confirm_action("Apply these changes?", false)? {
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

        if !confirm_action("Apply these settings?", false)? {
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
