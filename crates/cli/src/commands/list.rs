use anyhow::{Context, Result};
use schemagit_snapshot::SnapshotManager;

use super::output;

/// Execute the list command.
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
            "List output",
        )?;
        return Ok(());
    }

    let mut content = String::new();
    content.push_str(&format!("Snapshots in {}:\n\n", directory));

    for (i, filename) in snapshots.iter().enumerate() {
        // Try to load snapshot to get metadata
        if let Ok(snapshot) = manager.load(filename) {
            content.push_str(&format!(
                "{}. {} - {} ({} tables)",
                i + 1,
                filename,
                snapshot.database_type,
                snapshot.schema.tables.len()
            ));
            content.push('\n');
            content.push_str(&format!(
                "   Created: {}\n",
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
            ));
        } else {
            content.push_str(&format!(
                "{}. {} (failed to load)\n",
                i + 1,
                filename
            ));
        }
    }

    content.push('\n');
    content.push_str(&format!("Total: {} snapshot(s)\n", snapshots.len()));

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "List output",
    )?;

    Ok(())
}
