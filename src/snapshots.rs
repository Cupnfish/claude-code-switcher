use crate::settings::ClaudeSettings;
use anyhow::{Result, anyhow};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

/// Scope for snapshots
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SnapshotScope {
    /// Only environment variables
    Env,
    /// Common settings (exclude environment)
    Common,
    /// All settings
    All,
}

impl std::str::FromStr for SnapshotScope {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "env" => Ok(SnapshotScope::Env),
            "common" => Ok(SnapshotScope::Common),
            "all" => Ok(SnapshotScope::All),
            _ => Err(anyhow!(
                "Invalid scope '{}'. Must be one of: env, common, all",
                s
            )),
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

/// A snapshot of Claude Code settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Unique identifier
    pub id: String,

    /// User-friendly name
    pub name: String,

    /// Optional description
    pub description: Option<String>,

    /// The settings data
    pub settings: ClaudeSettings,

    /// When this snapshot was created
    pub created_at: String,

    /// When this snapshot was last modified
    pub updated_at: String,

    /// Scope of this snapshot
    pub scope: SnapshotScope,

    /// Version for future compatibility
    pub version: u32,
}

impl Snapshot {
    /// Create a new snapshot
    pub fn new(
        name: String,
        settings: ClaudeSettings,
        scope: SnapshotScope,
        description: Option<String>,
    ) -> Self {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();

        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description,
            settings,
            created_at: now.clone(),
            updated_at: now,
            scope,
            version: 1,
        }
    }

    /// Update the timestamp
    pub fn touch(&mut self) {
        let now = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
        self.updated_at = now;
    }
}

/// Store for managing snapshots
#[derive(Debug, Clone)]
pub struct SnapshotStore {
    /// Directory where snapshots are stored
    pub snapshots_dir: PathBuf,
}

impl SnapshotStore {
    /// Create a new snapshot store
    pub fn new(snapshots_dir: PathBuf) -> Self {
        Self { snapshots_dir }
    }

    /// Ensure the snapshots directory exists
    pub fn ensure_dir(&self) -> Result<()> {
        if !self.snapshots_dir.exists() {
            fs::create_dir_all(&self.snapshots_dir).map_err(|e| {
                anyhow!(
                    "Failed to create snapshots directory {}: {}",
                    self.snapshots_dir.display(),
                    e
                )
            })?;
        }
        Ok(())
    }

    /// Get the path for a snapshot file
    pub fn snapshot_path(&self, snapshot_id: &str) -> PathBuf {
        self.snapshots_dir.join(format!("{}.json", snapshot_id))
    }

    /// Save a snapshot
    pub fn save(&self, snapshot: &Snapshot) -> Result<()> {
        self.ensure_dir()?;

        let path = self.snapshot_path(&snapshot.id);
        let content = serde_json::to_string_pretty(snapshot)
            .map_err(|e| anyhow!("Failed to serialize snapshot: {}", e))?;

        fs::write(&path, content)
            .map_err(|e| anyhow!("Failed to write snapshot file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Load a snapshot by ID
    pub fn load(&self, snapshot_id: &str) -> Result<Snapshot> {
        let path = self.snapshot_path(snapshot_id);

        if !path.exists() {
            return Err(anyhow!("Snapshot '{}' not found", snapshot_id));
        }

        let content = fs::read_to_string(&path)
            .map_err(|e| anyhow!("Failed to read snapshot file {}: {}", path.display(), e))?;

        let snapshot: Snapshot = serde_json::from_str(&content)
            .map_err(|e| anyhow!("Failed to parse snapshot file {}: {}", path.display(), e))?;

        Ok(snapshot)
    }

    /// Load a snapshot by name
    pub fn load_by_name(&self, name: &str) -> Result<Snapshot> {
        let snapshots = self.list()?;

        for snapshot in snapshots {
            if snapshot.name == name {
                return Ok(snapshot);
            }
        }

        Err(anyhow!("Snapshot '{}' not found", name))
    }

    /// List all snapshots
    pub fn list(&self) -> Result<Vec<Snapshot>> {
        if !self.snapshots_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in fs::read_dir(&self.snapshots_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                match self.load(path.file_stem().and_then(|s| s.to_str()).unwrap_or("")) {
                    Ok(snapshot) => snapshots.push(snapshot),
                    Err(_) => {
                        // Skip invalid snapshot files
                        continue;
                    }
                }
            }
        }

        // Sort by creation date (newest first)
        snapshots.sort_by(|a, b| b.created_at.cmp(&a.created_at));

        Ok(snapshots)
    }

    /// Delete a snapshot
    pub fn delete(&self, snapshot_id: &str) -> Result<()> {
        let path = self.snapshot_path(snapshot_id);

        if !path.exists() {
            return Err(anyhow!("Snapshot '{}' not found", snapshot_id));
        }

        fs::remove_file(&path)
            .map_err(|e| anyhow!("Failed to delete snapshot file {}: {}", path.display(), e))?;

        Ok(())
    }

    /// Delete a snapshot by name
    pub fn delete_by_name(&self, name: &str) -> Result<()> {
        let snapshots = self.list()?;

        for snapshot in snapshots {
            if snapshot.name == name {
                return self.delete(&snapshot.id);
            }
        }

        Err(anyhow!("Snapshot '{}' not found", name))
    }

    /// Check if a snapshot exists
    pub fn exists(&self, snapshot_id: &str) -> bool {
        self.snapshot_path(snapshot_id).exists()
    }

    /// Check if a snapshot with the given name exists
    pub fn exists_by_name(&self, name: &str) -> bool {
        self.list()
            .map(|snapshots| snapshots.iter().any(|s| s.name == name))
            .unwrap_or(false)
    }

    /// Get all snapshot names
    pub fn list_names(&self) -> Result<Vec<String>> {
        let snapshots = self.list()?;
        Ok(snapshots.into_iter().map(|s| s.name).collect())
    }
}

/// Filter settings by scope
pub fn filter_settings_by_scope(settings: ClaudeSettings, scope: &SnapshotScope) -> ClaudeSettings {
    match scope {
        SnapshotScope::Env => ClaudeSettings {
            environment: settings.environment,
            ..Default::default()
        },
        SnapshotScope::Common => ClaudeSettings {
            provider: settings.provider,
            model: settings.model,
            endpoint: settings.endpoint,
            http: settings.http,
            permissions: settings.permissions,
            hooks: settings.hooks,
            status_line: settings.status_line,
            environment: None,
        },
        SnapshotScope::All => settings,
    }
}

impl Default for SnapshotScope {
    fn default() -> Self {
        Self::Common
    }
}
