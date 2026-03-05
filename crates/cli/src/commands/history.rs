use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;

/// Execute the history command.
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

    println!("{}", "Snapshot History".bold());
    println!();
    println!(
        "{:<25} {:<20} {:<10} {}",
        "ID".bold(),
        "CREATED".bold(),
        "TABLES".bold(),
        "DATABASE".bold()
    );
    println!("{}", "-".repeat(80));

    for filename in &snapshots {
        // Extract ID from filename (e.g., "2026_03_05_071235.snapshot.json" -> "20260305071235")
        let id = filename
            .strip_suffix(".snapshot.json")
            .unwrap_or(filename)
            .replace('_', "");

        // Try to load snapshot to get metadata
        if let Ok(snapshot) = manager.load(filename) {
            println!(
                "{:<25} {:<20} {:<10} {}",
                id.cyan(),
                snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
                snapshot.schema.tables.len(),
                snapshot.database_type
            );
        } else {
            println!("{:<25} {} (failed to load)", id.red(), "ERROR".red());
        }
    }

    println!();
    println!("Total: {} snapshot(s)", snapshots.len());

    Ok(())
}
