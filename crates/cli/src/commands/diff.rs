use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_diff::diff_schemas;
use schemagit_snapshot::SnapshotManager;

/// Execute the diff command.
pub fn execute(old_path: &str, new_path: &str, format: &str) -> Result<()> {
    println!("{}", "Comparing snapshots...".cyan());

    // Load snapshots
    let old_snapshot = SnapshotManager::load_from_path(old_path)
        .context(format!("Failed to load old snapshot: {}", old_path))?;

    let new_snapshot = SnapshotManager::load_from_path(new_path)
        .context(format!("Failed to load new snapshot: {}", new_path))?;

    println!(
        "Old snapshot: {} ({})",
        old_path, old_snapshot.database_type
    );
    println!(
        "New snapshot: {} ({})",
        new_path, new_snapshot.database_type
    );

    // Compare schemas
    let diff = diff_schemas(&old_snapshot.schema, &new_snapshot.schema);

    // Output based on format
    match format.to_lowercase().as_str() {
        "json" => {
            let json = serde_json::to_string_pretty(&diff)
                .context("Failed to serialize diff to JSON")?;
            println!("{}", json);
        }
        _ => {
            println!("\n{}", "=== Schema Differences ===".bold());
            if diff.has_changes() {
                println!("{}", diff.summary());
            } else {
                println!("{}", "No changes detected".green());
            }
        }
    }

    Ok(())
}
