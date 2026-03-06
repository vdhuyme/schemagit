use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_diff::diff_schemas;
use schemagit_introspector::create_introspector;
use schemagit_snapshot::SnapshotManager;

use super::{output, utils};

/// Execute the status command.
pub async fn execute(
    driver: Option<String>,
    connection: &str,
    snapshot_dir: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    println!("{}", "Checking database status...".cyan());

    // Auto-detect driver if not specified
    let driver = utils::resolve_driver(driver, connection)?;

    println!("Using driver: {}", driver.yellow());

    // Load latest snapshot
    let manager = SnapshotManager::new(snapshot_dir);
    let latest_snapshot = manager
        .latest()
        .context("Failed to load latest snapshot")?
        .ok_or_else(|| {
            anyhow::anyhow!("No snapshots found in {}", snapshot_dir)
        })?;

    println!(
        "Latest snapshot: {} ({})",
        latest_snapshot.timestamp.format("%Y-%m-%d %H:%M:%S"),
        latest_snapshot.database_type
    );

    // Create introspector and introspect current schema
    let introspector = create_introspector(&driver, connection)
        .map_err(|e| anyhow::anyhow!("Failed to create introspector: {}", e))?;

    println!("Introspecting current database schema...");

    let current_schema = introspector
        .introspect_schema()
        .await
        .context("Failed to introspect current schema")?;

    // Compare schemas
    let diff = diff_schemas(&latest_snapshot.schema, &current_schema);

    let mut rendered = String::from("\n=== Database Status ===\n");

    if diff.has_changes() {
        rendered.push_str("Database has drifted from snapshot!\n\n");
        rendered.push_str(&diff.summary());
        rendered.push('\n');
    } else {
        rendered.push_str("Database matches latest snapshot\n");
    }

    output::write_or_stdout(
        &rendered,
        output_file,
        yes,
        no_create_dir,
        "Status",
    )?;

    Ok(())
}
