//! Snapshot selector using the unified selector framework

use crate::selectors::{
    base::{SelectableItem, SelectionResult, Selector, SelectorConfig, prompt_rename},
    confirmation::ConfirmationService,
    error::{SelectorError, SelectorResult},
};
use crate::{
    Configurable,
    settings::{ClaudeSettings, format_settings_for_display},
    snapshots::{Snapshot, SnapshotScope, SnapshotStore},
    utils::get_snapshots_dir,
};
use std::io::Write;

/// Action for snapshot management
#[derive(Debug, Clone)]
pub enum SnapshotManagementAction {
    Apply(usize),
    Delete(usize),
    Rename(usize),
    CreateSnapshot,
    Back,
    Exit,
}

/// Snapshot selector using the unified framework
pub struct SnapshotSelector {
    snapshots: Vec<Snapshot>,
    store: SnapshotStore,
}

/// Display wrapper for snapshots
#[derive(Clone, Debug)]
struct SnapshotDisplayItem {
    index: usize,
    snapshot: Snapshot,
}

impl SelectableItem for SnapshotDisplayItem {
    fn display_name(&self) -> String {
        format!("{} ({})", self.snapshot.name, self.snapshot.scope)
    }

    fn format_for_list(&self) -> String {
        self.display_name()
    }

    fn id(&self) -> Option<String> {
        Some(self.snapshot.id.clone())
    }
}

impl SnapshotSelector {
    /// Create a new snapshot selector
    pub fn new() -> SelectorResult<Self> {
        let snapshots_dir = get_snapshots_dir();
        let store = SnapshotStore::new(snapshots_dir);
        let snapshots = store
            .list()
            .map_err(|e| SelectorError::Storage(format!("Failed to load snapshots: {}", e)))?;

        Ok(Self { snapshots, store })
    }

    /// Run interactive snapshot management
    pub fn run_management(&mut self) -> SelectorResult<()> {
        if self.snapshots.is_empty() {
            println!("No snapshots found. Let's create your first snapshot!");
            if self.create_snapshot()? {
                self.snapshots = self.store.list().map_err(|e| {
                    SelectorError::Storage(format!("Failed to reload snapshots: {}", e))
                })?;
            }

            if self.snapshots.is_empty() {
                return Ok(());
            }
        }

        loop {
            match self.select_snapshot_action()? {
                Some(SnapshotManagementAction::Apply(index)) => {
                    if self.apply_snapshot(index)? {
                        break;
                    }
                }
                Some(SnapshotManagementAction::Delete(index)) => {
                    if self.delete_snapshot(index)? && index < self.snapshots.len() {
                        self.snapshots.remove(index);
                    }
                }
                Some(SnapshotManagementAction::CreateSnapshot) => {
                    if self.create_snapshot()? {
                        self.snapshots = self.store.list().map_err(|e| {
                            SelectorError::Storage(format!("Failed to reload snapshots: {}", e))
                        })?;
                    }
                }
                Some(SnapshotManagementAction::Rename(index)) => {
                    if let Some(true) = self.rename_snapshot(index)? {
                        self.snapshots = self.store.list().map_err(|e| {
                            SelectorError::Storage(format!("Failed to reload snapshots: {}", e))
                        })?;
                    }
                }
                Some(SnapshotManagementAction::Back) => continue,
                Some(SnapshotManagementAction::Exit) => break,
                None => break,
            }

            if self.snapshots.is_empty() {
                println!("No more snapshots found.");
                break;
            }
        }

        Ok(())
    }

    /// Simple snapshot selection (for applying snapshots)
    pub fn select_snapshot() -> SelectorResult<Option<Snapshot>> {
        let selector = Self::new()?;

        if selector.snapshots.is_empty() {
            println!("No snapshots available.");
            return Ok(None);
        }

        let items: Vec<SnapshotDisplayItem> = selector
            .snapshots
            .iter()
            .enumerate()
            .map(|(i, s)| SnapshotDisplayItem {
                index: i,
                snapshot: s.clone(),
            })
            .collect();

        let config = SelectorConfig {
            allow_management: false,
            ..SelectorConfig::default()
        };

        let mut sel = Selector::new("Select a snapshot to apply:", items).with_config(config);

        match sel.prompt()? {
            SelectionResult::Selected(item) => Ok(Some(item.snapshot)),
            SelectionResult::Back => Ok(None),
            SelectionResult::Exit => {
                println!("Operation cancelled.");
                std::process::exit(0);
            }
            _ => Ok(None),
        }
    }

    /// Select snapshot action using the selector framework
    fn select_snapshot_action(&mut self) -> SelectorResult<Option<SnapshotManagementAction>> {
        let snapshot_items: Vec<SnapshotDisplayItem> = self
            .snapshots
            .iter()
            .enumerate()
            .map(|(i, s)| SnapshotDisplayItem {
                index: i,
                snapshot: s.clone(),
            })
            .collect();

        let title = format!(
            "Select a snapshot to manage ({} total):",
            self.snapshots.len()
        );

        let config = SelectorConfig {
            allow_create: true,
            show_filter: true,
            ..SelectorConfig::default()
        };

        let mut selector = Selector::new(&title, snapshot_items).with_config(config);

        match selector.prompt()? {
            SelectionResult::Selected(item) | SelectionResult::ViewDetails(item) => {
                // Clear screen before showing details
                print!("\x1b[2J\x1b[H");
                std::io::stdout().flush().ok();
                self.manage_snapshot(item.index).map(Some)
            }
            SelectionResult::Delete(item) => {
                Ok(Some(SnapshotManagementAction::Delete(item.index)))
            }
            SelectionResult::Rename(item) => {
                Ok(Some(SnapshotManagementAction::Rename(item.index)))
            }
            SelectionResult::Create => Ok(Some(SnapshotManagementAction::CreateSnapshot)),
            SelectionResult::Back => Ok(None),
            SelectionResult::Exit => std::process::exit(0),
            _ => Ok(None),
        }
    }

    /// Show snapshot details and action menu
    fn manage_snapshot(&self, index: usize) -> SelectorResult<SnapshotManagementAction> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        let snapshot = &self.snapshots[index];

        // Print snapshot details
        println!("\n📋 Snapshot: {} ({})", snapshot.name, snapshot.scope);
        println!("  Created: {}", snapshot.created_at);
        println!("  Updated: {}", snapshot.updated_at);
        if let Some(ref desc) = snapshot.description {
            println!("  Description: {}", desc);
        }

        // Show action menu
        let options = vec!["Apply", "Rename", "Delete", "Back"];

        let action = inquire::Select::new(&format!("Action for '{}':", snapshot.name), options)
            .with_help_message("↑/↓: Navigate, Enter: Select, Esc: Back")
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Selection failed: {}", e))
                }
            })?;

        match action {
            "Apply" => Ok(SnapshotManagementAction::Apply(index)),
            "Rename" => Ok(SnapshotManagementAction::Rename(index)),
            "Delete" => Ok(SnapshotManagementAction::Delete(index)),
            _ => Ok(SnapshotManagementAction::Back),
        }
    }

    /// Create a new snapshot interactively
    fn create_snapshot(&self) -> SelectorResult<bool> {
        println!("\n📝 Creating a new snapshot...\n");

        // Step 1: Select configuration path
        let config_options = vec![
            "Local (.claude/settings.json) - Project-specific settings",
            "Global (~/.claude/settings.json) - User-wide settings",
        ];

        let config_selection = inquire::Select::new("Select configuration to snapshot:", config_options)
            .with_help_message("↑/↓: Navigate, Enter: Select")
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Selection failed: {}", e))
                }
            })?;

        let settings_path = if config_selection.starts_with("Local") {
            crate::utils::get_local_settings_path()
        } else {
            crate::utils::get_settings_path(None)
        };

        // Step 2: Show current configuration preview
        println!("\n📋 Current Configuration Preview:");
        println!("📁 Path: {}", settings_path.display());

        if !settings_path.exists() {
            println!("⚠️  Settings file not found at this location.");

            let continue_confirmation =
                ConfirmationService::confirm_action("Continue creating snapshot anyway?")?;

            if !continue_confirmation {
                println!("Snapshot creation cancelled.");
                return Ok(false);
            }
        } else {
            // Load and display masked settings
            match ClaudeSettings::from_file(&settings_path) {
                Ok(settings) => {
                    let masked_settings = settings.clone().mask_sensitive_data();
                    println!("{}", format_settings_for_display(&masked_settings, false));

                    // Show additional info
                    if let Some(model) = &settings.model {
                        println!("🤖 Model: {}", model);
                    }
                    if let Some(hooks) = &settings.hooks {
                        let hook_count = hooks.pre_command.as_ref().map_or(0, |v| v.len())
                            + hooks.post_command.as_ref().map_or(0, |v| v.len());
                        if hook_count > 0 {
                            println!("🪝 Hooks: {} configured", hook_count);
                        }
                    }
                    if let Some(permissions) = &settings.permissions {
                        let rule_count = permissions.allow.as_ref().map_or(0, |v| v.len())
                            + permissions.ask.as_ref().map_or(0, |v| v.len())
                            + permissions.deny.as_ref().map_or(0, |v| v.len());
                        if rule_count > 0 {
                            println!("🔐 Permissions: {} rules", rule_count);
                        }
                    }
                }
                Err(e) => {
                    println!("❌ Failed to load settings: {}", e);

                    let continue_confirmation =
                        ConfirmationService::confirm_action("Continue anyway?")?;

                    if !continue_confirmation {
                        println!("Snapshot creation cancelled.");
                        return Ok(false);
                    }
                }
            }
        }

        println!(); // Add spacing

        // Step 3: Get snapshot name
        let name = inquire::Text::new("Enter snapshot name:")
            .with_help_message("A descriptive name (e.g., 'development-setup', 'production-config')")
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Input failed: {}", e))
                }
            })?;

        if name.trim().is_empty() {
            println!("❌ Snapshot name cannot be empty.");
            return Ok(false);
        }

        // Get description (optional)
        let description = inquire::Text::new("Enter description (optional):")
            .with_help_message("Optional description to help you remember what this snapshot is for")
            .prompt()
            .ok()
            .and_then(|d| {
                let trimmed = d.trim().to_string();
                if trimmed.is_empty() {
                    None
                } else {
                    Some(trimmed)
                }
            });

        // Select scope
        let scope_options = vec![
            "common - Common settings only (model, hooks, permissions)",
            "env - Environment variables only",
            "all - All settings (common + environment)",
        ];

        let scope_selection = inquire::Select::new("Select snapshot scope:", scope_options)
            .with_help_message("↑/↓: Navigate, Enter: Select")
            .prompt()
            .map_err(|e| {
                let msg = e.to_string();
                if msg.contains("canceled") || msg.contains("cancelled") {
                    SelectorError::Cancelled
                } else {
                    SelectorError::Failed(format!("Selection failed: {}", e))
                }
            })?;

        let scope = match scope_selection.split_once(" - ") {
            Some((scope_name, _)) => scope_name
                .parse::<SnapshotScope>()
                .map_err(|e| SelectorError::InvalidInput(format!("Invalid scope: {}", e)))?,
            None => SnapshotScope::Common,
        };

        // Show summary before confirmation
        println!("\n📋 Snapshot Summary:");
        println!("  Name: {}", name);
        println!("  Path: {}", settings_path.display());
        println!("  Scope: {}", scope);
        if let Some(ref desc) = description {
            println!("  Description: {}", desc);
        }

        // Confirm creation
        let confirmation = ConfirmationService::confirm_action("Create this snapshot?")?;

        if !confirmation {
            println!("Snapshot creation cancelled.");
            return Ok(false);
        }

        // Check if snapshot already exists
        if self.store.exists_by_name(&name) {
            let overwrite_confirmation = ConfirmationService::confirm_overwrite(&name, "snapshot")?;

            if !overwrite_confirmation {
                println!("Snapshot creation cancelled.");
                return Ok(false);
            }
        }

        // Create the snapshot
        let settings = if settings_path.exists() {
            ClaudeSettings::from_file(&settings_path)
                .map_err(|e| SelectorError::Failed(format!("Failed to load settings: {}", e)))?
        } else {
            ClaudeSettings::default()
        };

        // Capture environment variables if needed
        let mut snapshot_settings = settings;
        if matches!(scope, SnapshotScope::All | SnapshotScope::Env) {
            snapshot_settings.env = Some(ClaudeSettings::capture_environment());
        }

        let snapshot = Snapshot::new(name.clone(), snapshot_settings, scope, description);

        self.store.save(&snapshot).map_err(|e| {
            SelectorError::OperationFailed(format!("Failed to save snapshot: {}", e))
        })?;

        println!("✓ Snapshot '{}' created successfully!", name);

        Ok(true)
    }

    /// Apply a snapshot with confirmation
    fn apply_snapshot(&self, index: usize) -> SelectorResult<bool> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        let snapshot = &self.snapshots[index];

        let confirmation =
            ConfirmationService::confirm_action(&format!("Apply snapshot '{}'?", snapshot.name))?;

        if confirmation {
            // Load current settings for comparison
            let settings_path = crate::utils::get_settings_path(None);
            let _existing_settings = ClaudeSettings::from_file(&settings_path).map_err(|e| {
                SelectorError::Failed(format!("Failed to load current settings: {}", e))
            })?;

            // Backup current settings
            let backup_path = settings_path.with_extension("json.backup");
            std::fs::copy(&settings_path, &backup_path).map_err(|e| {
                SelectorError::OperationFailed(format!("Failed to create backup: {}", e))
            })?;

            println!("✓ Settings backed up to: {}", backup_path.display());

            // Apply snapshot settings
            snapshot
                .settings
                .clone()
                .to_file(&settings_path)
                .map_err(|e| {
                    SelectorError::OperationFailed(format!("Failed to apply snapshot: {}", e))
                })?;

            println!("✓ Applied snapshot '{}' successfully!", snapshot.name);
            Ok(true)
        } else {
            println!("Apply cancelled.");
            Ok(false)
        }
    }

    /// Delete a snapshot with confirmation
    fn delete_snapshot(&self, index: usize) -> SelectorResult<bool> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        let snapshot = &self.snapshots[index];

        let confirmation = ConfirmationService::confirm_deletion(&snapshot.name, "snapshot")?;

        if confirmation {
            self.store.delete(&snapshot.id).map_err(|e| {
                SelectorError::OperationFailed(format!("Failed to delete snapshot: {}", e))
            })?;
            println!("✓ Snapshot deleted successfully!");
            Ok(true)
        } else {
            println!("Deletion cancelled.");
            Ok(false)
        }
    }

    /// Rename a snapshot with confirmation
    fn rename_snapshot(&self, index: usize) -> SelectorResult<Option<bool>> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        let snapshot = &self.snapshots[index];
        let new_name = prompt_rename(&snapshot.name, "snapshot")?;

        if new_name != snapshot.name {
            // Check if snapshot already exists with new name
            if self.store.exists_by_name(&new_name) {
                let overwrite_confirmation =
                    ConfirmationService::confirm_overwrite(&new_name, "snapshot")?;
                if !overwrite_confirmation {
                    println!("Rename cancelled.");
                    return Ok(Some(true));
                }
            }

            // Create new snapshot with updated name
            let mut updated_snapshot = snapshot.clone();
            updated_snapshot.name = new_name.clone();
            updated_snapshot.updated_at = chrono::Utc::now().to_rfc3339();

            // Save updated snapshot
            self.store.save(&updated_snapshot).map_err(|e| {
                SelectorError::OperationFailed(format!("Failed to rename snapshot: {}", e))
            })?;

            // Delete old snapshot
            self.store.delete(&snapshot.id).map_err(|e| {
                SelectorError::OperationFailed(format!("Failed to delete old snapshot: {}", e))
            })?;

            println!("✓ Snapshot renamed to '{}' successfully!", new_name);
        }

        Ok(Some(true))
    }
}
