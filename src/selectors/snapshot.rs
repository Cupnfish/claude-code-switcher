//! Snapshot selector using the unified selector framework

use crate::selectors::{
    base::{
        BaseSelector, SelectableItem, SelectionResult, Selector, SelectorConfig, prompt_rename,
    },
    confirmation::ConfirmationService,
    error::{SelectorError, SelectorResult},
};
use crate::{
    Configurable,
    settings::{ClaudeSettings, format_settings_for_display},
    snapshots::{Snapshot, SnapshotScope, SnapshotStore},
    utils::get_snapshots_dir,
};
use std::io::{self, Write};

/// Simple text input function
fn get_text_input(
    prompt: &str,
    default: Option<&str>,
    _description: Option<&str>,
) -> SelectorResult<String> {
    print!("{}", prompt);
    if let Some(default) = default {
        print!(" [{}]", default);
    }
    print!(": ");
    io::stdout().flush()?;

    let mut input = String::new();
    io::stdin().read_line(&mut input)?;

    let input = input.trim();
    if input.is_empty() && default.is_some() {
        Ok(default.unwrap().to_string())
    } else {
        Ok(input.to_string())
    }
}

/// Action for snapshot management
#[derive(Debug, Clone)]
pub enum SnapshotManagementAction {
    ViewDetails(usize),
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
                Some(SnapshotManagementAction::Rename(index)) => {
                    if let Some(should_continue) = self.rename_snapshot(index)?
                        && !should_continue
                    {
                        continue;
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

    /// Select snapshot action using new selector framework
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
            SelectionResult::Selected(item) => self.show_snapshot_actions(item.index).map(Some),
            SelectionResult::Create => Ok(Some(SnapshotManagementAction::CreateSnapshot)),
            SelectionResult::Back => Ok(None),
            SelectionResult::Exit => std::process::exit(0),
            _ => Ok(None),
        }
    }

    fn show_snapshot_actions(&self, index: usize) -> SelectorResult<SnapshotManagementAction> {
        use inquire::{InquireError, Select};

        let snapshot = &self.snapshots[index];

        // Create snapshot details string
        let mut details = format!(
            "Snapshot: {} ({})\n\
             Scope: {}\n\
             Created: {}\n\
             Updated: {}",
            snapshot.name, snapshot.scope, snapshot.scope, snapshot.created_at, snapshot.updated_at
        );

        // Add description if available
        if let Some(description) = &snapshot.description
            && !description.is_empty()
        {
            details.push_str(&format!("\nDescription: {}", description));
        }

        // Create options for the select
        let options = vec![
            "📋 View Details",
            "🔄 Apply",
            "✏️  Rename",
            "🗑️  Delete",
            "⬅️  Back",
        ];

        let help_message = format!("{}\n\n↑↓ to move, enter to select, esc to cancel", details);

        match Select::new("Manage Snapshot:", options)
            .with_help_message(&help_message)
            .with_page_size(5)
            .prompt_skippable()
        {
            Ok(Some(action)) => match action {
                "📋 View Details" => Ok(SnapshotManagementAction::ViewDetails(index)),
                "🔄 Apply" => Ok(SnapshotManagementAction::Apply(index)),
                "✏️  Rename" => Ok(SnapshotManagementAction::Rename(index)),
                "🗑️  Delete" => Ok(SnapshotManagementAction::Delete(index)),
                "⬅️  Back" => Ok(SnapshotManagementAction::Back),
                _ => Ok(SnapshotManagementAction::Exit),
            },
            Ok(None) => Ok(SnapshotManagementAction::Exit),
            Err(InquireError::OperationCanceled) => Ok(SnapshotManagementAction::Exit),
            Err(e) => Err(SelectorError::failed(e.to_string())),
        }
    }

    /// Show snapshot details with navigation, including configuration and operations
    fn show_snapshot_details_with_navigation(&mut self, index: usize) -> SelectorResult<bool> {
        if index >= self.snapshots.len() {
            return Err(SelectorError::NotFound);
        }

        use crossterm::{
            QueueableCommand,
            cursor::{Hide, Show},
            event::{Event, KeyCode, KeyEvent, read},
            style::{Color, Print, ResetColor, SetForegroundColor},
            terminal::{self, Clear, ClearType},
        };
        use std::io::{Write, stdout};

        let snapshot = &self.snapshots[index];
        let actions = vec!["🔄 Apply", "✏️  Rename", "🗑️  Delete", "⬅️  Back"];

        // Setup terminal
        terminal::enable_raw_mode()?;
        let mut stdout = stdout();

        // Initialize selected_action outside the loop
        let mut selected_action = 0;

        loop {
            // Clear screen and move cursor to top-left
            stdout.queue(Clear(ClearType::All))?;
            stdout.queue(crossterm::cursor::MoveTo(0, 0))?;

            // Render header
            stdout
                .queue(SetForegroundColor(Color::Cyan))?
                .queue(Print(format!(
                    "? Managing: {} ({})\n\n",
                    snapshot.name, snapshot.scope
                )))?
                .queue(ResetColor)?;

            // Render detailed snapshot information
            stdout.queue(SetForegroundColor(Color::White))?;

            // Display basic snapshot details
            self.display_snapshot_info(index)?;

            // Display snapshot configuration
            stdout.queue(Print("\n📝 Configuration:\n"))?;
            stdout.queue(Print(&crate::settings::format_settings_for_display(
                &snapshot.settings,
                false,
            )))?;

            stdout.queue(Print("\n"))?;
            stdout.queue(ResetColor)?;

            // Render action list
            for (i, action) in actions.iter().enumerate() {
                if i == selected_action {
                    stdout
                        .queue(SetForegroundColor(Color::Yellow))?
                        .queue(Print("❯ "))?
                        .queue(Print(action))?
                        .queue(ResetColor)?;
                } else {
                    stdout.queue(Print(format!("  {}", action)))?;
                }
                stdout.queue(Print("\n"))?;
            }

            // Render help
            stdout.queue(Print("\n"))?;
            stdout
                .queue(SetForegroundColor(Color::DarkGrey))?
                .queue(Print("↑/↓: Navigate, Enter: Select, Esc: Back"))?
                .queue(ResetColor)?;

            // Flush all queued commands
            stdout.flush()?;
            stdout.queue(Hide)?;
            stdout.flush()?;

            // Handle input
            if let Event::Key(KeyEvent { code, .. }) = read()? {
                match code {
                    KeyCode::Up => {
                        if selected_action > 0 {
                            selected_action -= 1;
                        }
                    }
                    KeyCode::Down => {
                        if selected_action < actions.len() - 1 {
                            selected_action += 1;
                        }
                    }
                    KeyCode::Left => {
                        // Left arrow acts as back
                        stdout.queue(Show)?;
                        stdout.flush()?;
                        terminal::disable_raw_mode()?;
                        return Ok(true);
                    }
                    KeyCode::Enter => {
                        stdout.queue(Show)?;
                        stdout.flush()?;
                        terminal::disable_raw_mode()?;

                        match actions[selected_action] {
                            "🔄 Apply" => {
                                // Apply the snapshot
                                if let Err(e) = self.apply_snapshot(index) {
                                    println!("❌ Failed to apply snapshot: {}", e);
                                    println!("Press Enter to continue...");
                                    let mut input = String::new();
                                    std::io::stdin().read_line(&mut input)?;
                                    return self.show_snapshot_details_with_navigation(index);
                                }
                                println!("✅ Snapshot applied successfully!");
                                println!("Press Enter to continue...");
                                let mut input = String::new();
                                std::io::stdin().read_line(&mut input)?;
                                return Ok(true);
                            }
                            "✏️  Rename" => {
                                // Rename the snapshot
                                if let Some(true) = self.rename_snapshot(index)? {
                                    // Re-run to show updated name
                                    terminal::enable_raw_mode()?;
                                    return self.show_snapshot_details_with_navigation(index);
                                }
                                return Ok(true);
                            }
                            "🗑️  Delete" => {
                                // Delete the snapshot
                                if self.delete_snapshot(index)? {
                                    return Ok(false); // Exit since snapshot was deleted
                                }
                                return Ok(true);
                            }
                            "⬅️  Back" => {
                                return Ok(true);
                            }
                            _ => {
                                return Ok(true);
                            }
                        }
                    }
                    KeyCode::Esc => {
                        stdout.queue(Show)?;
                        stdout.flush()?;
                        terminal::disable_raw_mode()?;
                        return Ok(true);
                    }
                    _ => {}
                }
            }
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
            "Local (.claude/settings.json) - Project-specific settings".to_string(),
            "Global (~/.claude/settings.json) - User-wide settings".to_string(),
        ];

        let mut selector =
            BaseSelector::new(config_path_options, "Select configuration to snapshot:")
                .with_show_filter(false);
        let config_selection = selector.run()?.unwrap_or_else(|| {
            "Local (.claude/settings.json) - Project-specific settings".to_string()
        });

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
        let name = get_text_input(
            "Enter snapshot name",
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
        let description = get_text_input(
            "Enter description (optional)",
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
            "common - Common settings only (model, hooks, permissions)".to_string(),
            "env - Environment variables only".to_string(),
            "all - All settings (common + environment)".to_string(),
        ];

        let mut selector =
            BaseSelector::new(scope_options, "Select snapshot scope:").with_show_filter(false);
        let scope_selection = selector.run()?.unwrap_or_else(|| {
            "common - Common settings only (model, hooks, permissions)".to_string()
        });

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
                    return Ok(Some(true)); // Continue with management
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
        } else {
            println!("Name unchanged.");
        }

        Ok(Some(true)) // Continue with management
    }
}

// Also implement SelectableItem directly for Snapshot
impl SelectableItem for Snapshot {
    fn display_name(&self) -> String {
        format!("{} ({})", self.name, self.scope)
    }

    fn format_for_list(&self) -> String {
        // Show detailed information when displayed as a single item in list-like UI
        let mut details = format!(
            "Name: {}\nScope: {}\nCreated: {}\nUpdated: {}\n",
            self.name, self.scope, self.created_at, self.updated_at
        );

        // Add description if available
        if let Some(desc) = &self.description {
            details.push_str(&format!("Description: {}\n", desc));
        }

        details
    }

    fn id(&self) -> Option<String> {
        Some(self.id.clone())
    }
}
