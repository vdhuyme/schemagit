use chrono::{DateTime, Utc};
use schemagit_core::DatabaseSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors that can occur during snapshot operations.
#[derive(Debug, Error)]
pub enum SnapshotError {
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Snapshot not found: {0}")]
    NotFound(String),

    #[error("Invalid snapshot format: {0}")]
    InvalidFormat(String),
}

pub type SnapshotResult<T> = Result<T, SnapshotError>;

/// Represents a schema snapshot with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    /// Database type (e.g., "postgres", "mysql")
    pub database_type: String,
    /// Timestamp when the snapshot was created
    pub timestamp: DateTime<Utc>,
    /// The actual database schema
    pub schema: DatabaseSchema,
}

impl Snapshot {
    /// Create a new snapshot.
    pub fn new(database_type: String, schema: DatabaseSchema) -> Self {
        Self {
            database_type,
            timestamp: Utc::now(),
            schema,
        }
    }
}

/// Manages schema snapshots on the filesystem.
pub struct SnapshotManager {
    snapshot_dir: PathBuf,
}

impl SnapshotManager {
    /// Create a new snapshot manager.
    ///
    /// # Arguments
    /// * `snapshot_dir` - Directory where snapshots will be stored
    pub fn new<P: AsRef<Path>>(snapshot_dir: P) -> Self {
        Self {
            snapshot_dir: snapshot_dir.as_ref().to_path_buf(),
        }
    }

    /// Ensure the snapshot directory exists.
    fn ensure_directory(&self) -> SnapshotResult<()> {
        fs::create_dir_all(&self.snapshot_dir)?;
        Ok(())
    }

    /// Generate a snapshot filename based on the current timestamp.
    fn generate_filename(&self) -> String {
        let now = Utc::now();
        format!("{}.snapshot.json", now.format("%Y_%m_%d_%H%M%S"))
    }

    /// Get the full path for a snapshot file.
    fn snapshot_path(&self, filename: &str) -> PathBuf {
        self.snapshot_dir.join(filename)
    }

    /// Save a snapshot to disk.
    ///
    /// # Arguments
    /// * `snapshot` - The snapshot to save
    ///
    /// # Returns
    /// The filename of the saved snapshot
    pub fn save(&self, snapshot: &Snapshot) -> SnapshotResult<String> {
        self.ensure_directory()?;

        let filename = self.generate_filename();
        let path = self.snapshot_path(&filename);

        let json = serde_json::to_string_pretty(snapshot)?;
        fs::write(&path, json)?;

        Ok(filename)
    }

    /// Load a snapshot from disk.
    ///
    /// # Arguments
    /// * `filename` - Name of the snapshot file to load
    pub fn load(&self, filename: &str) -> SnapshotResult<Snapshot> {
        let path = self.snapshot_path(filename);

        if !path.exists() {
            return Err(SnapshotError::NotFound(filename.to_string()));
        }

        let json = fs::read_to_string(&path)?;
        let snapshot: Snapshot = serde_json::from_str(&json)?;

        Ok(snapshot)
    }

    /// Load a snapshot from an arbitrary path.
    ///
    /// # Arguments
    /// * `path` - Full path to the snapshot file
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> SnapshotResult<Snapshot> {
        let path = path.as_ref();

        if !path.exists() {
            return Err(SnapshotError::NotFound(
                path.to_string_lossy().to_string(),
            ));
        }

        let json = fs::read_to_string(path)?;
        let snapshot: Snapshot = serde_json::from_str(&json)?;

        Ok(snapshot)
    }

    /// List all snapshot files in the snapshot directory.
    ///
    /// # Returns
    /// A vector of snapshot filenames, sorted by name (which is timestamp-based)
    pub fn list(&self) -> SnapshotResult<Vec<String>> {
        if !self.snapshot_dir.exists() {
            return Ok(Vec::new());
        }

        let mut snapshots = Vec::new();

        for entry in fs::read_dir(&self.snapshot_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_file() {
                if let Some(filename) = path.file_name() {
                    let filename = filename.to_string_lossy().to_string();
                    if filename.ends_with(".snapshot.json") {
                        snapshots.push(filename);
                    }
                }
            }
        }

        snapshots.sort();
        Ok(snapshots)
    }

    /// Get the most recent snapshot.
    pub fn latest(&self) -> SnapshotResult<Option<Snapshot>> {
        let snapshots = self.list()?;

        if let Some(latest_filename) = snapshots.last() {
            Ok(Some(self.load(latest_filename)?))
        } else {
            Ok(None)
        }
    }

    /// Delete a snapshot file.
    ///
    /// # Arguments
    /// * `filename` - Name of the snapshot file to delete
    pub fn delete(&self, filename: &str) -> SnapshotResult<()> {
        let path = self.snapshot_path(filename);

        if !path.exists() {
            return Err(SnapshotError::NotFound(filename.to_string()));
        }

        fs::remove_file(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemagit_core::{Column, Table};

    #[test]
    fn test_snapshot_creation() {
        let schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    default: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let snapshot = Snapshot::new("postgres".to_string(), schema);
        assert_eq!(snapshot.database_type, "postgres");
        assert_eq!(snapshot.schema.tables.len(), 1);
    }
}
