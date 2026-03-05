use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;
use std::collections::HashMap;

/// Execute the summary command.
pub fn execute(snapshot_id: &str, directory: &str) -> Result<()> {
    let manager = SnapshotManager::new(directory);

    let snapshot = match snapshot_id {
        id if id.ends_with(".snapshot.json") => manager.load(id)?,

        "latest" => manager
            .latest()
            .context("Failed to load latest snapshot")?
            .ok_or_else(|| {
                anyhow::anyhow!("No snapshots found in {}", directory)
            })?,

        id => {
            let filename = match id.len() {
                14 => format!(
                    "{}_{}_{}_{}.snapshot.json",
                    &id[0..4],
                    &id[4..6],
                    &id[6..8],
                    &id[8..14]
                ),
                _ => format!("{}.snapshot.json", id),
            };

            manager.load(&filename)?
        }
    };

    // Calculate statistics
    let total_tables = snapshot.schema.tables.len();
    let total_columns: usize =
        snapshot.schema.tables.iter().map(|t| t.columns.len()).sum();
    let total_indexes: usize =
        snapshot.schema.tables.iter().map(|t| t.indexes.len()).sum();
    let total_foreign_keys: usize = snapshot
        .schema
        .tables
        .iter()
        .map(|t| t.foreign_keys.len())
        .sum();

    println!("{}", "=== Schema Summary ===".bold().cyan());
    println!();
    println!("{}: {}", "Database".bold(), snapshot.database_type);
    println!(
        "{}: {}",
        "Snapshot".bold(),
        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
    );
    println!();

    println!("{}", "Overview:".bold());
    println!("  Tables:       {}", total_tables.to_string().green());
    println!("  Columns:      {}", total_columns.to_string().cyan());
    println!("  Indexes:      {}", total_indexes.to_string().yellow());
    println!(
        "  Foreign Keys: {}",
        total_foreign_keys.to_string().magenta()
    );
    println!();

    let mut table_sizes: Vec<(&str, usize)> = snapshot
        .schema
        .tables
        .iter()
        .map(|t| (t.name.as_str(), t.columns.len()))
        .collect();
    table_sizes.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "Top tables by column count:".bold());
    for (i, (name, count)) in table_sizes.iter().take(10).enumerate() {
        println!("  {}. {}: {}", i + 1, name.green(), count);
    }
    println!();

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for table in &snapshot.schema.tables {
        for column in &table.columns {
            *type_counts.entry(column.data_type.clone()).or_insert(0) += 1;
        }
    }

    let mut type_vec: Vec<(String, usize)> = type_counts.into_iter().collect();
    type_vec.sort_by(|a, b| b.1.cmp(&a.1));

    println!("{}", "Column type distribution:".bold());
    for (data_type, count) in type_vec.iter().take(10) {
        println!("  {}: {}", data_type.yellow(), count);
    }

    Ok(())
}
