//! Snapshot selector using the unified selector framework

use crate::selectors::{
    base::{SelectableItem, Selector},
    confirmation::ConfirmationService,
    error::{SelectorError, SelectorResult},
    navigation::NavigationManager,
};
use crate::{
    Configurable,
    settings::{ClaudeSettings, format_settings_for_display},
    snapshots::{Snapshot, SnapshotScope, SnapshotStore},
    utils::get_snapshots_dir,
};

/// Action for snapshot management
#[derive(Debug, Clone)]
pub enum SnapshotManagementAction {
    ViewDetails(usize),
    Apply(usize),
    Delete(usize),
    CreateSnapshot,
    Back,
    Exit,
}

/// Snapshot selector using the unified framework
pub struct SnapshotSelector {
    snapshots: Vec<Snapshot>,
    store: SnapshotStore,
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
                Some(SnapshotManagementAction::ViewDetails(index)) => {
                    if !self.show_snapshot_details_with_navigation(index)? {
                        break;
                    }
                    continue;
                }
                Some(SnapshotManagementAction::Apply(index)) => {
                    if self.apply_snapshot(index)? {
                        break; // Successfully applied, exit browser
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

        // Use framework for selection
        let mut base_selector = crate::selectors::base::BaseSelector::new(
            selector.snapshots.clone(),
            "Select a snapshot to apply:",
        );

        base_selector
            .run()
            .map_err(|e| SelectorError::Failed(format!("Snapshot selection failed: {}", e)))
    }

    /// Select snapshot action
    fn select_snapshot_action(&mut self) -> SelectorResult<Option<SnapshotManagementAction>> {
        // Use special handling for create option
        let items: Vec<SnapshotListItem> = self
            .snapshots
            .iter()
            .enumerate()
            .map(|(index, snapshot)| SnapshotListItem {
                index,
                snapshot: snapshot.clone(),
                is_create: false,
            })
            .collect();

        let title = format!(
            "Select a snapshot to manage ({} total):",
            self.snapshots.len() + 1
        );

        match NavigationManager::select_from_list_with_create(
            &items,
            &title,
            true,
            Some("↑/↓: Navigate, →: Select, ←/Esc: Back"),
        )? {
            crate::selectors::navigation::NavigationResult::Selected(item) => {
                if item.is_create {
                    Ok(Some(SnapshotManagementAction::CreateSnapshot))
                } else {
                    self.show_snapshot_actions(item.index).map(Some)
                }
            }
            crate::selectors::navigation::NavigationResult::CreateNew => {
                Ok(Some(SnapshotManagementAction::CreateSnapshot))
            }
            crate::selectors::navigation::NavigationResult::Back
            | crate::selectors::navigation::NavigationResult::Exit => Ok(None),
        }
    }

    /// Show actions for a snapshot
    fn show_snapshot_actions(&self, index: usize) -> SelectorResult<SnapshotManagementAction> {
        let snapshot = &self.snapshots[index];

        let actions = vec!["📋 View Details", "🔄 Apply", "🗑️  Delete", "⬅️  Back"];

        let title = format!("Managing: {} ({})", snapshot.name, snapshot.scope);

        match NavigationManager::select_option(&title, &actions, None)? {
            action if action == "📋 View Details" => {
                Ok(SnapshotManagementAction::ViewDetails(index))
            }
            action if action == "🔄 Apply" => Ok(SnapshotManagementAction::Apply(index)),
            action if action == "🗑️  Delete" => Ok(SnapshotManagementAction::Delete(index)),
            action if action == "⬅️  Back" => Ok(SnapshotManagementAction::Back),
            _ => Ok(SnapshotManagementAction::Exit),
        }
    }

    /// Show snapshot details with navigation
    fn show_snapshot_details_with_navigation(&self, index: usize) -> SelectorResult<bool> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        self.display_snapshot_info(index)?;

        let actions = vec!["⬅️  Back to Snapshot List", "🚪 Exit Program"];

        match NavigationManager::select_option("Choose an action:", &actions, None)? {
            action if action == "⬅️  Back to Snapshot List" => Ok(true),
            action if action == "🚪 Exit Program" => Ok(false),
            _ => Ok(true),
        }
    }

    /// Display snapshot information
    fn display_snapshot_info(&self, index: usize) -> SelectorResult<()> {
        let snapshot = &self.snapshots[index];

        println!("\n📋 Snapshot Details:");
        println!("  Name: {}", snapshot.name);
        println!("  ID: {}", snapshot.id);
        println!("  Scope: {}", snapshot.scope);
        println!("  Created: {}", snapshot.created_at);
        println!("  Updated: {}", snapshot.updated_at);

        if let Some(ref desc) = snapshot.description {
            println!("  Description: {}", desc);
        }

        println!("  Version: {}", snapshot.version);

        // Show masked settings
        let masked_settings = snapshot.settings.clone().mask_sensitive_data();
        println!("  Settings:");
        println!("{}", format_settings_for_display(&masked_settings, true));

        Ok(())
    }

    /// Create a new snapshot interactively
    fn create_snapshot(&self) -> SelectorResult<bool> {
        println!("\n📝 Creating a new snapshot...\n");

        // Step 1: Select configuration path
        let config_path_options = vec![
            "Local (.claude/settings.json) - Project-specific settings",
            "Global (~/.claude/settings.json) - User-wide settings",
        ];

        let config_selection = NavigationManager::select_option(
            "Select configuration to snapshot:",
            &config_path_options,
            Some("Choose which settings file to create snapshot from"),
        )?;

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
        let name = NavigationManager::get_text_input(
            "Enter snapshot name:",
            None,
            Some(
                "A descriptive name for your snapshot (e.g., 'development-setup', 'production-config')",
            ),
        )?;

        if name.trim().is_empty() {
            println!("❌ Snapshot name cannot be empty.");
            return Ok(false);
        }

        // Get description (optional)
        let description = NavigationManager::get_text_input(
            "Enter description (optional):",
            Some(""),
            Some("Optional description to help you remember what this snapshot is for"),
        )?;

        let description = if description.trim().is_empty() {
            None
        } else {
            Some(description.trim().to_string())
        };

        // Select scope
        let scope_options = vec![
            "common - Common settings only (model, hooks, permissions)",
            "env - Environment variables only",
            "all - All settings (common + environment)",
        ];

        let scope_selection = NavigationManager::select_option(
            "Select snapshot scope:",
            &scope_options,
            Some("Choose what to include in the snapshot"),
        )?;

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
}

/// Wrapper for snapshots in selection lists
#[derive(Debug, Clone)]
struct SnapshotListItem {
    index: usize,
    snapshot: Snapshot,
    is_create: bool,
}

impl SelectableItem for SnapshotListItem {
    fn display_name(&self) -> String {
        if self.is_create {
            "➕ Create New Snapshot".to_string()
        } else {
            format!("{} ({})", self.snapshot.name, self.snapshot.scope)
        }
    }

    fn format_for_list(&self) -> String {
        if self.is_create {
            "➕ Create New Snapshot".to_string()
        } else {
            let description_part = if let Some(desc) = &self.snapshot.description {
                format!(" - {}", desc)
            } else {
                String::new()
            };

            format!(
                "{} (scope: {}, created: {}){}",
                self.snapshot.name, self.snapshot.scope, self.snapshot.created_at, description_part
            )
        }
    }

    fn id(&self) -> Option<String> {
        if self.is_create {
            Some("create".to_string())
        } else {
            Some(self.snapshot.id.clone())
        }
    }
}

// Also implement SelectableItem directly for Snapshot
impl SelectableItem for Snapshot {
    fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.scope)
    }

    fn format_for_list(&self) -> String {
        let description_part = if let Some(desc) = &self.description {
            format!(" - {}", desc)
        } else {
            String::new()
        };

        format!(
            "{} (scope: {}, created: {}){}",
            self.name, self.scope, self.created_at, description_part
        )
    }

    fn id(&self) -> Option<String> {
        Some(self.id.clone())
    }
}
