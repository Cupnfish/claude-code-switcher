use crate::{
    Configurable, CredentialManager,
    credentials::{CredentialStore, get_api_key_interactively},
    settings::{ClaudeSettings, format_settings_for_display},
    snapshots::{SnapshotScope, SnapshotStore},
    templates::{TemplateType, get_template, get_template_type},
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
        crate::Commands::List { verbose } => list_command(*verbose)?,
        crate::Commands::Snap {
            name,
            scope,
            settings_path,
            description,
            overwrite,
        } => snap_command(name, scope, settings_path, description, *overwrite)?,
        crate::Commands::Apply {
            target,
            scope,
            model,
            settings_path,
            backup,
            yes,
        } => apply_command(target, scope, model, settings_path, *backup, *yes)?,
        crate::Commands::Delete { name, yes } => delete_command(name, *yes)?,
        crate::Commands::Credentials(credential_commands) => match credential_commands {
            crate::CredentialCommands::List => credentials_list_command()?,
            crate::CredentialCommands::Delete { id } => credentials_delete_command(id)?,
            crate::CredentialCommands::Clear { yes } => credentials_clear_command(*yes)?,
        },
    }
    Ok(())
}

/// List available snapshots
pub fn list_command(verbose: bool) -> Result<()> {
    let snapshots_dir = crate::utils::get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);
    let snapshots = store.list()?;

    if snapshots.is_empty() {
        println!("No snapshots found.");
        return Ok(());
    }

    println!("Available snapshots ({} total):", snapshots.len());

    for snapshot in &snapshots {
        if verbose {
            println!("\n{} {}", style("Name:").bold(), snapshot.name);
            println!("{} {}", style("ID:").bold(), snapshot.id);
            if let Some(ref desc) = snapshot.description {
                println!("{} {}", style("Description:").bold(), desc);
            }
            println!("{} {}", style("Scope:").bold(), snapshot.scope);
            println!("{} {}", style("Created:").bold(), snapshot.created_at);
            println!("{} {}", style("Updated:").bold(), snapshot.updated_at);

            let masked_settings = snapshot.settings.clone().mask_sensitive_data();
            println!(
                "{}\n{}",
                style("Settings:").bold(),
                format_settings_for_display(&masked_settings, true)
            );
        } else {
            println!(
                "{}: {} (scope: {}, created: {})",
                style(&snapshot.name).cyan().bold(),
                snapshot.id,
                snapshot.scope,
                snapshot.created_at
            );
        }
        println!();
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
        snapshot_settings.environment = Some(ClaudeSettings::capture_environment());
    }

    let snapshots_dir = crate::utils::get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    if store.exists_by_name(name) && !overwrite {
        if !confirm_action(
            &format!("Snapshot '{}' already exists. Overwrite?", name),
            false,
        )? {
            return Ok(());
        }
    }

    let snapshot = crate::Snapshot::new(
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
        return apply_template_command(&template_type, scope, model, &settings_path, backup, yes);
    }

    // Otherwise treat as snapshot name
    apply_snapshot_command(target, scope, model, &settings_path, backup, yes)
}

/// Apply a template
fn apply_template_command(
    template_type: &TemplateType,
    scope: &SnapshotScope,
    model: &Option<String>,
    settings_path: &PathBuf,
    backup: bool,
    yes: bool,
) -> Result<()> {
    let template = get_template(template_type);
    // Get API key
    let api_key = get_api_key_interactively(template_type.clone())?;

    let mut settings = template(&api_key, scope);

    // Override model if specified
    if let Some(model_name) = model {
        if let Some(ref mut model_config) = settings.model {
            model_config.name = model_name.clone();
        }
    }

    // Load existing settings and merge
    let existing_settings = ClaudeSettings::from_file(settings_path)?;

    // Backup current settings if requested
    if backup {
        backup_settings(settings_path)?;
    }

    // Confirm overwrite
    if !yes {
        let existing_masked = existing_settings.clone().mask_sensitive_data();
        let new_masked = settings.clone().mask_sensitive_data();

        println!("Current settings:");
        println!("{}", format_settings_for_display(&existing_masked, false));
        println!("\nNew settings:");
        println!("{}", format_settings_for_display(&new_masked, false));

        if !confirm_action("Apply these settings?", false)? {
            return Ok(());
        }
    }

    let final_settings = settings.merge_with(existing_settings);

    // Save settings
    final_settings.to_file(settings_path)?;

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
        if let Some(ref mut model_config) = snapshot.settings.model {
            model_config.name = model_name.clone();
        }
    }

    // Load existing settings and merge
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

    let final_settings = snapshot.settings.merge_with(existing_settings);

    // Save settings
    final_settings.to_file(settings_path)?;

    println!(
        "{} Applied snapshot '{}' successfully!",
        style("✓").green().bold(),
        snapshot_name
    );

    Ok(())
}

/// Delete a snapshot
pub fn delete_command(name: &str, yes: bool) -> Result<()> {
    let snapshots_dir = get_snapshots_dir();
    let store = SnapshotStore::new(snapshots_dir);

    if !store.exists_by_name(name) {
        return Err(anyhow!("Snapshot '{}' not found", name));
    }

    if !yes {
        if !confirm_action(&format!("Delete snapshot '{}'?", name), false)? {
            return Ok(());
        }
    }

    store.delete_by_name(name)?;
    println!(
        "{} Deleted snapshot '{}' successfully!",
        style("✓").green().bold(),
        name
    );

    Ok(())
}

/// List saved credentials
pub fn credentials_list_command() -> Result<()> {
    let _credentials_dir = get_credentials_dir();
    let credential_store = CredentialStore::new()?;

    let credentials = credential_store.load_credentials()?;

    if credentials.is_empty() {
        println!("No saved credentials found.");
        return Ok(());
    }

    println!("Saved credentials ({} total):", credentials.len());

    for credential in &credentials {
        let template_type = credential.template_type();
        let masked_key = mask_api_key(credential.api_key());

        println!(
            "{}: {} ({} - {})",
            style(credential.id()).cyan().bold(),
            credential.name(),
            template_type,
            masked_key
        );
    }

    Ok(())
}

/// Delete a credential
pub fn credentials_delete_command(id: &str) -> Result<()> {
    let _credentials_dir = get_credentials_dir();
    let credential_store = CredentialStore::new()?;

    if credential_store.delete_credential(id).is_err() {
        return Err(anyhow!("Credential '{}' not found", id));
    }

    println!(
        "{} Deleted credential '{}' successfully!",
        style("✓").green().bold(),
        id
    );

    Ok(())
}

/// Clear all credentials
pub fn credentials_clear_command(yes: bool) -> Result<()> {
    if !yes {
        if !confirm_action("Clear all saved credentials?", false)? {
            return Ok(());
        }
    }

    let _credentials_dir = get_credentials_dir();
    let credential_store = CredentialStore::new()?;

    credential_store.clear_credentials()?;

    println!("{} Cleared all credentials!", style("✓").green().bold());

    Ok(())
}

/// Helper function to mask API key for display
fn mask_api_key(api_key: &str) -> String {
    if api_key.len() <= 8 {
        "••••••••".to_string()
    } else {
        format!("{}••••••••", &api_key[..api_key.len().min(8)])
    }
}
