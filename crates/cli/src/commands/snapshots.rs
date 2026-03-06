use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;

const SNAPSHOT_SUFFIX: &str = ".snapshot.json";

/// Execute the snapshots command.
pub fn execute(directory: &str) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshots = manager.list().context("Failed to list snapshots")?;

    if snapshots.is_empty() {
        println!(
            "{}",
            format!("No snapshots found in {}", directory).yellow()
        );
        return Ok(());
    }

    println!("Available snapshots:");
    println!();

    for filename in snapshots {
        println!("{}", filename.trim_end_matches(SNAPSHOT_SUFFIX));
    }

    Ok(())
}
