use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_introspector::create_introspector;
use schemagit_snapshot::{Snapshot, SnapshotManager};

use super::utils;

/// Execute the snapshot command.
pub async fn execute(
    driver: Option<String>,
    connection: &str,
    output_dir: &str,
) -> Result<()> {
    println!("{}", "Creating snapshot...".cyan());

    // Auto-detect driver if not specified
    let driver = utils::resolve_driver(driver, connection)?;

    println!("Using driver: {}", driver.yellow());

    // Create introspector
    let introspector = create_introspector(&driver, connection)
        .map_err(|e| anyhow::anyhow!("Failed to create introspector: {}", e))?;

    // Introspect schema
    println!("Introspecting database schema...");
    let schema = introspector
        .introspect_schema()
        .await
        .context("Failed to introspect schema")?;

    let db_name = introspector
        .database_name()
        .await
        .context("Failed to get database name")?;

    println!("Found {} tables", schema.tables.len());

    // Create snapshot
    let snapshot = Snapshot::new(driver.to_string(), db_name.clone(), schema);

    // Save snapshot
    let manager = SnapshotManager::new(output_dir);
    let filename =
        manager.save(&snapshot).context("Failed to save snapshot")?;

    println!(
        "{}",
        format!("✓ Snapshot saved: {}/{}", output_dir, filename).green()
    );
    println!("Database: {}", db_name);
    println!("Tables: {}", snapshot.schema.tables.len());

    Ok(())
}
