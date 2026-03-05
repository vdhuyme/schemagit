use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;

/// Execute the list command.
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

    println!("{}", format!("Snapshots in {}:", directory).bold());
    println!();

    for (i, filename) in snapshots.iter().enumerate() {
        // Try to load snapshot to get metadata
        if let Ok(snapshot) = manager.load(filename) {
            println!(
                "{}. {} - {} ({} tables)",
                i + 1,
                filename.cyan(),
                snapshot.database_type,
                snapshot.schema.tables.len()
            );
            println!(
                "   Created: {}",
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
            );
        } else {
            println!("{}. {} (failed to load)", i + 1, filename.red());
        }
    }

    println!();
    println!("Total: {} snapshot(s)", snapshots.len());

    Ok(())
}
