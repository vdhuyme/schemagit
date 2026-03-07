use crate::output::{self, OutputOptions};
use anyhow::{Context, Result};
use schemagit_diff::diff_schemas;
use schemagit_snapshot::SnapshotManager;

pub fn execute(
    directory: &str,
    format: &str,
    output_file: Option<&str>,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshots = manager.list().context("Failed to list snapshots")?;

    if snapshots.is_empty() {
        return Ok(());
    }

    let mut timeline = Vec::new();
    let mut prev_snapshot: Option<schemagit_snapshot::Snapshot> = None;

    for filename in snapshots {
        let current = manager.load(&filename)?;
        let id = filename
            .strip_suffix(".snapshot.json")
            .unwrap_or(&filename)
            .replace('_', "");

        let mut changes = Vec::new();
        if let Some(prev) = prev_snapshot {
            let diff = diff_schemas(&prev.schema, &current.schema);

            for table in &diff.tables_added {
                changes.push(format!("Added table \"{}\"", table.name));
            }
            for table in &diff.tables_removed {
                changes.push(format!("Removed table \"{}\"", table.name));
            }
            for table_diff in &diff.tables_modified {
                changes.push(format!(
                    "Modified table \"{}\"",
                    table_diff.table_name
                ));
            }
        } else {
            changes.push("Initial snapshot".to_string());
        }

        timeline.push((id, current.timestamp, changes));
        prev_snapshot = Some(current);
    }

    let mut content = String::new();
    content.push_str("Schema Evolution Timeline\n\n");

    for (id, timestamp, changes) in timeline {
        content.push_str(&format!(
            "{} [{}]\n",
            id,
            timestamp.format("%Y-%m-%d %H:%M:%S")
        ));
        if changes.is_empty() {
            content.push_str("  (no changes)\n");
        } else {
            for change in changes {
                content.push_str(&format!("  {}\n", change));
            }
        }
        content.push('\n');
    }

    let options = OutputOptions {
        format: format.to_string(),
        output_path: output_file.map(|s| s.to_string()),
        pretty: true,
    };

    output::write_output(&content, &options)?;

    Ok(())
}
