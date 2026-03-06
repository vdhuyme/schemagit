use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_diff::diff_schemas;
use schemagit_snapshot::SnapshotManager;

use super::utils;

/// Execute the diff command.
pub fn execute(
    old_path: &str,
    new_path: &str,
    snapshot_dir: &str,
    format: &str,
) -> Result<()> {
    println!("{}", "Comparing snapshots...".cyan());

    let manager = SnapshotManager::new(snapshot_dir);

    let resolved_old =
        utils::resolve_snapshot_path(&manager, old_path, snapshot_dir)?;
    let resolved_new =
        utils::resolve_snapshot_path(&manager, new_path, snapshot_dir)?;

    // Load snapshots
    let old_snapshot =
        utils::resolve_snapshot(&manager, old_path, snapshot_dir).context(
            format!("Failed to load old snapshot: {}", resolved_old),
        )?;

    let new_snapshot =
        utils::resolve_snapshot(&manager, new_path, snapshot_dir).context(
            format!("Failed to load new snapshot: {}", resolved_new),
        )?;

    println!(
        "Old snapshot: {} ({})",
        resolved_old, old_snapshot.database_type
    );
    println!(
        "New snapshot: {} ({})",
        resolved_new, new_snapshot.database_type
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
