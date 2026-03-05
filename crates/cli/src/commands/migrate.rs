use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_diff::diff_schemas;
use schemagit_migration::create_generator;
use schemagit_snapshot::SnapshotManager;
use std::fs;

/// Execute the migrate command.
pub fn execute(
    old_path: &str,
    new_path: &str,
    output_file: Option<&str>,
) -> Result<()> {
    println!("{}", "Generating migration...".cyan());

    // Load snapshots
    let old_snapshot = SnapshotManager::load_from_path(old_path)
        .context(format!("Failed to load old snapshot: {}", old_path))?;

    let new_snapshot = SnapshotManager::load_from_path(new_path)
        .context(format!("Failed to load new snapshot: {}", new_path))?;

    // Compare schemas
    let diff = diff_schemas(&old_snapshot.schema, &new_snapshot.schema);

    if !diff.has_changes() {
        println!("{}", "No changes detected - no migration needed".yellow());
        return Ok(());
    }

    // Create migration generator
    let generator =
        create_generator(&new_snapshot.database_type).ok_or_else(|| {
            anyhow::anyhow!(
                "Unsupported database type: {}",
                new_snapshot.database_type
            )
        })?;

    // Generate migration SQL
    let migration = generator.generate_migration(&diff);

    // Output migration
    match output_file {
        Some(path) => {
            fs::write(path, &migration)
                .context("Failed to write migration file")?;
            println!("{}", format!("✓ Migration saved: {}", path).green());
        }
        None => {
            println!("\n{}", "=== Migration SQL ===".bold());
            println!("{}", migration);
        }
    }

    Ok(())
}
