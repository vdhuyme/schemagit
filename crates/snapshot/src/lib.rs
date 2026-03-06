use chrono::{DateTime, Utc};
use schemagit_core::DatabaseSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

const SNAPSHOT_EXTENSION: &str = ".snapshot.json";
const TIMESTAMP_FORMAT: &str = "%Y_%m_%d_%H%M%S";
const DEFAULT_DATABASE_NAME: &str = "unknown";
const DEFAULT_SNAPSHOT_VERSION: &str = "1";

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
    /// Database name from the connection context
    #[serde(default = "default_database_name")]
    pub database_name: String,
    /// Snapshot format version for forward-compatible evolution
    #[serde(default = "default_snapshot_version")]
    pub snapshot_version: String,
    /// Timestamp when the snapshot was created
    pub timestamp: DateTime<Utc>,
    /// The actual database schema
    pub schema: DatabaseSchema,
}

impl Snapshot {
    /// Create a new snapshot.
    pub fn new(
        database_type: String,
        database_name: String,
        schema: DatabaseSchema,
    ) -> Self {
        Self {
            database_type,
            database_name,
            snapshot_version: DEFAULT_SNAPSHOT_VERSION.to_string(),
            timestamp: Utc::now(),
            schema,
        }
    }

    fn validate(&self) -> SnapshotResult<()> {
        if self.database_type.trim().is_empty() {
            return Err(SnapshotError::InvalidFormat(
                "missing or empty \"database_type\" field".to_string(),
            ));
        }

        if self.database_name.trim().is_empty() {
            return Err(SnapshotError::InvalidFormat(
                "missing or empty \"database_name\" field".to_string(),
            ));
        }

        if self.snapshot_version.trim().is_empty() {
            return Err(SnapshotError::InvalidFormat(
                "missing or empty \"snapshot_version\" field".to_string(),
            ));
        }

        for table in &self.schema.tables {
            if table.name.trim().is_empty() {
                return Err(SnapshotError::InvalidFormat(
                    "table has missing or empty \"name\" field".to_string(),
                ));
            }

            for column in &table.columns {
                if column.name.trim().is_empty() {
                    return Err(SnapshotError::InvalidFormat(format!(
                        "table \"{}\" has a column with missing or empty \"name\" field",
                        table.name
                    )));
                }

                if column.data_type.trim().is_empty() {
                    return Err(SnapshotError::InvalidFormat(format!(
                        "table \"{}\" column \"{}\" has missing or empty \"data_type\" field",
                        table.name, column.name
                    )));
                }
            }

            for index in &table.indexes {
                if index.name.trim().is_empty() {
                    return Err(SnapshotError::InvalidFormat(format!(
                        "table \"{}\" has an index with missing or empty \"name\" field",
                        table.name
                    )));
                }
            }

            for foreign_key in &table.foreign_keys {
                if foreign_key.name.trim().is_empty() {
                    return Err(SnapshotError::InvalidFormat(format!(
                        "table \"{}\" has a foreign key with missing or empty \"name\" field",
                        table.name
                    )));
                }

                if foreign_key.column.trim().is_empty()
                    || foreign_key.ref_table.trim().is_empty()
                    || foreign_key.ref_column.trim().is_empty()
                {
                    return Err(SnapshotError::InvalidFormat(format!(
                        "table \"{}\" foreign key \"{}\" has missing required reference fields",
                        table.name, foreign_key.name
                    )));
                }
            }
        }

        Ok(())
    }
}

fn default_database_name() -> String {
    DEFAULT_DATABASE_NAME.to_string()
}

fn default_snapshot_version() -> String {
    DEFAULT_SNAPSHOT_VERSION.to_string()
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
        format!("{}{}", now.format(TIMESTAMP_FORMAT), SNAPSHOT_EXTENSION)
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
        Self::load_snapshot_from_path(&path, filename)
    }

    /// Load a snapshot from an arbitrary path.
    ///
    /// # Arguments
    /// * `path` - Full path to the snapshot file
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> SnapshotResult<Snapshot> {
        let path = path.as_ref();
        Self::load_snapshot_from_path(path, &path.to_string_lossy())
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

            if path.is_file()
                && let Some(filename) = path.file_name()
            {
                let filename = filename.to_string_lossy().to_string();
                if filename.ends_with(SNAPSHOT_EXTENSION) {
                    snapshots.push(filename);
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

    fn load_snapshot_from_path(
        path: &Path,
        not_found_name: &str,
    ) -> SnapshotResult<Snapshot> {
        if !path.exists() {
            return Err(SnapshotError::NotFound(not_found_name.to_string()));
        }

        let json = fs::read_to_string(path)?;
        let snapshot: Snapshot = serde_json::from_str(&json)
            .map_err(|error| SnapshotError::InvalidFormat(error.to_string()))?;
        snapshot.validate()?;

        Ok(snapshot)
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

        let snapshot = Snapshot::new(
            "postgres".to_string(),
            "test_db".to_string(),
            schema,
        );
        assert_eq!(snapshot.database_type, "postgres");
        assert_eq!(snapshot.database_name, "test_db");
        assert_eq!(snapshot.snapshot_version, "1");
        assert_eq!(snapshot.schema.tables.len(), 1);
    }
}
