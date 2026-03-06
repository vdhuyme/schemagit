use anyhow::{Context, Result};
use schemagit_snapshot::SnapshotManager;

use super::output;

/// Execute the history command.
pub fn execute(
    directory: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshots = manager.list().context("Failed to list snapshots")?;

    if snapshots.is_empty() {
        let content = format!("No snapshots found in {}\n", directory);
        output::write_or_stdout(
            &content,
            output_file,
            yes,
            no_create_dir,
            "History",
        )?;
        return Ok(());
    }

    let mut content = String::new();
    content.push_str("Snapshot History\n\n");
    content.push_str(&format!(
        "{:<25} {:<20} {:<10} {}",
        "ID", "CREATED", "TABLES", "DATABASE"
    ));
    content.push('\n');
    content.push_str(&"-".repeat(80));
    content.push('\n');

    for filename in &snapshots {
        // Extract ID from filename (e.g., "2026_03_05_071235.snapshot.json" -> "20260305071235")
        let id = filename
            .strip_suffix(".snapshot.json")
            .unwrap_or(filename)
            .replace('_', "");

        // Try to load snapshot to get metadata
        if let Ok(snapshot) = manager.load(filename) {
            content.push_str(&format!(
                "{:<25} {:<20} {:<10} {}",
                id,
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                snapshot.schema.tables.len(),
                snapshot.database_type
            ));
            content.push('\n');
        } else {
            content.push_str(&format!("{:<25} ERROR (failed to load)\n", id));
        }
    }

    content.push('\n');
    content.push_str(&format!("Total: {} snapshot(s)\n", snapshots.len()));

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "History",
    )?;

    Ok(())
}
