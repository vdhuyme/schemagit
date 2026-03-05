use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;

/// Execute the show command.
pub fn execute(snapshot_id: &str, directory: &str) -> Result<()> {
    let manager = SnapshotManager::new(directory);

    // Try to find the snapshot - could be full filename or just ID
    let snapshot = if snapshot_id.ends_with(".snapshot.json") {
        manager.load(snapshot_id)?
    } else if snapshot_id == "latest" {
        manager
            .latest()
            .context("Failed to load latest snapshot")?
            .ok_or_else(|| {
                anyhow::anyhow!("No snapshots found in {}", directory)
            })?
    } else {
        // Convert ID to filename (e.g., "20260305071235" -> "2026_03_05_071235.snapshot.json")
        let filename = if snapshot_id.len() == 14 {
            format!(
                "{}_{}_{}_{}.snapshot.json",
                &snapshot_id[0..4],
                &snapshot_id[4..6],
                &snapshot_id[6..8],
                &snapshot_id[8..14]
            )
        } else {
            format!("{}.snapshot.json", snapshot_id)
        };
        manager.load(&filename)?
    };

    println!("{}", "=== Snapshot Details ===".bold().cyan());
    println!();
    println!("{}: {}", "Database Type".bold(), snapshot.database_type);
    println!(
        "{}: {}",
        "Created".bold(),
        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
    println!("{}: {}", "Tables".bold(), snapshot.schema.tables.len());
    println!();

    println!("{}", "=== Tables ===".bold());
    for table in &snapshot.schema.tables {
        println!();
        println!("  {}", table.name.green().bold());
        println!("    Columns: {}", table.columns.len().to_string().cyan());

        // Show columns
        for column in &table.columns {
            let nullable = if column.nullable { "NULL" } else { "NOT NULL" };
            let default_str = match &column.default {
                Some(d) => format!(" DEFAULT {}", d),
                None => String::new(),
            };
            println!(
                "      - {} {} {}{}",
                column.name,
                column.data_type.yellow(),
                nullable,
                default_str
            );
        }

        // Show indexes
        if !table.indexes.is_empty() {
            println!("    Indexes: {}", table.indexes.len().to_string().cyan());
            for index in &table.indexes {
                let unique = if index.unique { "UNIQUE" } else { "" };
                println!(
                    "      - {} {} ({})",
                    index.name,
                    unique.yellow(),
                    index.columns.join(", ")
                );
            }
        }

        // Show foreign keys
        if !table.foreign_keys.is_empty() {
            println!(
                "    Foreign Keys: {}",
                table.foreign_keys.len().to_string().cyan()
            );
            for fk in &table.foreign_keys {
                println!(
                    "      - {} ({} -> {}.{})",
                    fk.name, fk.column, fk.ref_table, fk.ref_column
                );
            }
        }
    }

    Ok(())
}
