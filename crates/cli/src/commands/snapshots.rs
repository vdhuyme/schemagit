use anyhow::{Context, Result};
use schemagit_snapshot::SnapshotManager;

use super::output;

const SNAPSHOT_SUFFIX: &str = ".snapshot.json";

/// Execute the snapshots command.
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
            "Snapshot list",
        )?;
        return Ok(());
    }

    let mut content = String::from("Available snapshots:\n\n");

    for filename in snapshots {
        content.push_str(filename.trim_end_matches(SNAPSHOT_SUFFIX));
        content.push('\n');
    }

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "Snapshot list",
    )?;

    Ok(())
}
