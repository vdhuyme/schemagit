use anyhow::{Context, Result};
use schemagit_snapshot::{Snapshot, SnapshotManager};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::Path;

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
const PREVIOUS_SNAPSHOT_KEY: &str = "previous";
const AUTO_DETECT_DRIVER_ERROR: &str = "Could not auto-detect database driver from connection string. Please specify --driver explicitly.";
const OUTPUT_DIRECTORY_MISSING_ERROR: &str = "Output directory does not exist";
const OPERATION_CANCELLED_MESSAGE: &str =
    "Operation cancelled. Output directory was not created.";

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
    let resolved = resolve_snapshot_target(manager, snapshot_id, directory)?;

    if is_path_reference(snapshot_id) {
        return SnapshotManager::load_from_path(&resolved).map_err(|error| {
            anyhow::anyhow!("Invalid snapshot file:\n{}\n{}", resolved, error)
        });
    }

    manager.load(&resolved).map_err(|error| {
        anyhow::anyhow!("Invalid snapshot file:\n{}\n{}", resolved, error)
    })
}

pub fn resolve_snapshot_filename(
    manager: &SnapshotManager,
    snapshot_id: &str,
    directory: &str,
) -> Result<String> {
    let resolved = resolve_snapshot_target(manager, snapshot_id, directory)?;

    if is_path_reference(snapshot_id) {
        return Path::new(&resolved)
            .file_name()
            .map(|filename| filename.to_string_lossy().to_string())
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Failed to resolve snapshot filename from path: {}",
                    resolved
                )
            });
    }

    Ok(resolved)
}

pub fn resolve_snapshot_path(
    manager: &SnapshotManager,
    snapshot_id: &str,
    directory: &str,
) -> Result<String> {
    resolve_snapshot_target(manager, snapshot_id, directory)
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

pub fn prepare_output_path(
    output_path: &str,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    if yes && no_create_dir {
        return Err(anyhow::anyhow!(
            "--yes and --no-create-dir cannot be used together"
        ));
    }

    let path = Path::new(output_path);
    let Some(parent) = path.parent() else {
        return Ok(());
    };

    if parent.as_os_str().is_empty() {
        return Ok(());
    }

    if parent.exists() {
        if parent.is_dir() {
            return Ok(());
        }

        return Err(anyhow::anyhow!(
            "Output path parent exists but is not a directory: {}",
            parent.display()
        ));
    }

    if yes {
        fs::create_dir_all(parent)?;
        return Ok(());
    }

    if no_create_dir {
        return Err(anyhow::anyhow!(
            "{}: {}",
            OUTPUT_DIRECTORY_MISSING_ERROR,
            parent.display()
        ));
    }

    if !is_interactive_terminal() {
        return Err(anyhow::anyhow!(
            "{}: {}",
            OUTPUT_DIRECTORY_MISSING_ERROR,
            parent.display()
        ));
    }

    if prompt_create_directory(parent)? {
        fs::create_dir_all(parent)?;
        return Ok(());
    }

    Err(anyhow::anyhow!(OPERATION_CANCELLED_MESSAGE))
}

fn is_interactive_terminal() -> bool {
    io::stdin().is_terminal() && io::stdout().is_terminal()
}

fn prompt_create_directory(path: &Path) -> Result<bool> {
    let mut stdout = io::stdout();
    writeln!(
        stdout,
        "Output directory \"{}\" does not exist.",
        path.display()
    )?;

    loop {
        write!(stdout, "Do you want to create it? (y/n): ")?;
        stdout.flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let normalized = input.trim().to_ascii_lowercase();

        match normalized.as_str() {
            "y" | "yes" => return Ok(true),
            "n" | "no" => return Ok(false),
            _ => {
                writeln!(stdout, "Please answer with 'y'/'yes' or 'n'/'no'.")?;
            }
        }
    }
}

fn build_snapshot_filename(snapshot_id: &str) -> String {
    if snapshot_id.ends_with(SNAPSHOT_SUFFIX) {
        return snapshot_id.to_string();
    }

    if is_compact_timestamp(snapshot_id) {
        return format!(
            "{}_{}_{}_{}.snapshot.json",
            &snapshot_id[0..4],
            &snapshot_id[4..6],
            &snapshot_id[6..8],
            &snapshot_id[8..14]
        );
    }

    if is_underscored_timestamp(snapshot_id) {
        return format!("{}.snapshot.json", snapshot_id);
    }

    format!("{}.snapshot.json", snapshot_id)
}

fn resolve_snapshot_target(
    manager: &SnapshotManager,
    snapshot_id: &str,
    directory: &str,
) -> Result<String> {
    match snapshot_id {
        LATEST_SNAPSHOT_KEY => resolve_relative_snapshot(manager, directory, 1),
        PREVIOUS_SNAPSHOT_KEY => {
            resolve_relative_snapshot(manager, directory, 2)
        }
        id if is_path_reference(id) => Ok(id.to_string()),
        id => Ok(build_snapshot_filename(id)),
    }
}

fn resolve_relative_snapshot(
    manager: &SnapshotManager,
    directory: &str,
    from_end: usize,
) -> Result<String> {
    let snapshots = manager.list().context("Failed to list snapshots")?;
    if snapshots.len() < from_end {
        let label = if from_end == 1 { "latest" } else { "previous" };
        return Err(anyhow::anyhow!(
            "No {} snapshot found in {}",
            label,
            directory
        ));
    }

    Ok(snapshots[snapshots.len() - from_end].clone())
}

fn is_path_reference(value: &str) -> bool {
    let path = Path::new(value);
    path.is_absolute() || value.contains('/') || value.contains('\\')
}

fn is_compact_timestamp(value: &str) -> bool {
    value.len() == 14 && value.chars().all(|c| c.is_ascii_digit())
}

fn is_underscored_timestamp(value: &str) -> bool {
    if value.len() != 17 {
        return false;
    }

    let bytes = value.as_bytes();
    bytes[4] == b'_'
        && bytes[7] == b'_'
        && bytes[10] == b'_'
        && value
            .chars()
            .enumerate()
            .all(|(idx, c)| matches!(idx, 4 | 7 | 10) || c.is_ascii_digit())
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
