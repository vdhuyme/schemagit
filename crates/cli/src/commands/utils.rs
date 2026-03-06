use anyhow::{Context, Result};
use schemagit_snapshot::{Snapshot, SnapshotManager};

const POSTGRESQL_SCHEME: &str = "postgresql://";
const POSTGRES_SCHEME: &str = "postgres://";
const MYSQL_SCHEME: &str = "mysql://";
const SQLITE_SCHEME: &str = "sqlite://";
const SQLITE_FILE_SCHEME: &str = "file:";
const MSSQL_SCHEME: &str = "mssql://";
const SQLSERVER_SCHEME: &str = "sqlserver://";

const DRIVER_POSTGRES: &str = "postgres";
const DRIVER_MYSQL: &str = "mysql";
const DRIVER_SQLITE: &str = "sqlite";
const DRIVER_MSSQL: &str = "mssql";

const SNAPSHOT_SUFFIX: &str = ".snapshot.json";
const LATEST_SNAPSHOT_KEY: &str = "latest";
const AUTO_DETECT_DRIVER_ERROR: &str =
    "Could not auto-detect database driver from connection string. Please specify --driver explicitly.";

/// Detect database driver from connection string.
pub fn detect_driver(connection_string: &str) -> Option<String> {
    let lower = connection_string.to_lowercase();

    match lower.as_str() {
        s if s.starts_with(POSTGRESQL_SCHEME)
            || s.starts_with(POSTGRES_SCHEME) =>
        {
            Some(DRIVER_POSTGRES.to_string())
        }
        s if s.starts_with(MYSQL_SCHEME) => Some(DRIVER_MYSQL.to_string()),
        s if s.starts_with(SQLITE_SCHEME)
            || s.starts_with(SQLITE_FILE_SCHEME) =>
        {
            Some(DRIVER_SQLITE.to_string())
        }
        s if s.starts_with(MSSQL_SCHEME) || s.starts_with(SQLSERVER_SCHEME) => {
            Some(DRIVER_MSSQL.to_string())
        }
        _ => None,
    }
}

pub fn resolve_snapshot(
    manager: &SnapshotManager,
    snapshot_id: &str,
    directory: &str,
) -> Result<Snapshot> {
    match snapshot_id {
        id if id.ends_with(SNAPSHOT_SUFFIX) => {
            manager.load(id).map_err(Into::into)
        }
        LATEST_SNAPSHOT_KEY => manager
            .latest()
            .context("Failed to load latest snapshot")?
            .ok_or_else(|| {
                anyhow::anyhow!("No snapshots found in {}", directory)
            }),
        id => manager
            .load(&build_snapshot_filename(id))
            .map_err(Into::into),
    }
}

pub fn resolve_snapshot_filename(
    manager: &SnapshotManager,
    snapshot_id: &str,
    directory: &str,
) -> Result<String> {
    match snapshot_id {
        id if id.ends_with(SNAPSHOT_SUFFIX) => Ok(id.to_string()),
        LATEST_SNAPSHOT_KEY => {
            let snapshots =
                manager.list().context("Failed to list snapshots")?;
            snapshots.last().cloned().ok_or_else(|| {
                anyhow::anyhow!("No snapshots found in {}", directory)
            })
        }
        id => Ok(build_snapshot_filename(id)),
    }
}

pub fn resolve_driver(
    driver: Option<String>,
    connection: &str,
) -> Result<String> {
    match driver {
        Some(driver) => Ok(driver),
        None => detect_driver(connection)
            .ok_or_else(|| anyhow::anyhow!(AUTO_DETECT_DRIVER_ERROR)),
    }
}

fn build_snapshot_filename(snapshot_id: &str) -> String {
    match snapshot_id.len() {
        14 => format!(
            "{}_{}_{}_{}.snapshot.json",
            &snapshot_id[0..4],
            &snapshot_id[4..6],
            &snapshot_id[6..8],
            &snapshot_id[8..14]
        ),
        _ => format!("{}.snapshot.json", snapshot_id),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_postgres() {
        assert_eq!(
            detect_driver("postgresql://user:pass@localhost/db"),
            Some("postgres".to_string())
        );
        assert_eq!(
            detect_driver("postgres://user:pass@localhost/db"),
            Some("postgres".to_string())
        );
    }

    #[test]
    fn test_detect_mysql() {
        assert_eq!(
            detect_driver("mysql://user:pass@localhost/db"),
            Some("mysql".to_string())
        );
    }

    #[test]
    fn test_detect_sqlite() {
        assert_eq!(
            detect_driver("sqlite://path/to/db.sqlite"),
            Some("sqlite".to_string())
        );
    }

    #[test]
    fn test_detect_mssql() {
        assert_eq!(
            detect_driver("mssql://server/database"),
            Some("mssql".to_string())
        );
        assert_eq!(
            detect_driver("sqlserver://server/database"),
            Some("mssql".to_string())
        );
    }

    #[test]
    fn test_detect_unknown() {
        assert_eq!(detect_driver("unknown://something"), None);
    }
}
