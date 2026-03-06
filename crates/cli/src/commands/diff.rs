use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_diff::diff_schemas;
use schemagit_snapshot::SnapshotManager;

use super::{output, utils};

/// Execute the diff command.
pub fn execute(
    old_path: &str,
    new_path: &str,
    snapshot_dir: &str,
    format: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
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

    let rendered = match format.to_lowercase().as_str() {
        "json" => serde_json::to_string_pretty(&diff)
            .context("Failed to serialize diff to JSON")?,
        _ => {
            let mut text =
                format!("\n{}\n", "=== Schema Differences ===".bold());
            if diff.has_changes() {
                text.push_str(&format!("{}\n", diff.summary()));
            } else {
                text.push_str(&format!("{}\n", "No changes detected".green()));
            }
            text
        }
    };

    output::write_or_stdout(
        &rendered,
        output_file,
        yes,
        no_create_dir,
        "Diff output",
    )?;

    Ok(())
}
