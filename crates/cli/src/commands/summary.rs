use anyhow::Result;
use schemagit_snapshot::SnapshotManager;
use std::collections::HashMap;

use super::{output, utils};

/// Execute the summary command.
pub fn execute(
    snapshot_id: &str,
    directory: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

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

    let mut content = String::new();
    content.push_str("=== Schema Summary ===\n\n");
    content.push_str(&format!("Database: {}\n", snapshot.database_type));
    content.push_str(&format!(
        "Snapshot: {}\n",
        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
    ));
    content.push('\n');

    content.push_str("Overview:\n");
    content.push_str(&format!("  Tables:       {}\n", total_tables));
    content.push_str(&format!("  Columns:      {}\n", total_columns));
    content.push_str(&format!("  Indexes:      {}\n", total_indexes));
    content.push_str(&format!("  Foreign Keys: {}\n\n", total_foreign_keys));

    let mut table_sizes: Vec<(&str, usize)> = snapshot
        .schema
        .tables
        .iter()
        .map(|t| (t.name.as_str(), t.columns.len()))
        .collect();
    table_sizes.sort_by(|a, b| b.1.cmp(&a.1));

    content.push_str("Top tables by column count:\n");
    for (i, (name, count)) in table_sizes.iter().take(10).enumerate() {
        content.push_str(&format!("  {}. {}: {}\n", i + 1, name, count));
    }
    content.push('\n');

    let mut type_counts: HashMap<String, usize> = HashMap::new();
    for table in &snapshot.schema.tables {
        for column in &table.columns {
            *type_counts.entry(column.data_type.clone()).or_insert(0) += 1;
        }
    }

    let mut type_vec: Vec<(String, usize)> = type_counts.into_iter().collect();
    type_vec.sort_by(|a, b| b.1.cmp(&a.1));

    content.push_str("Column type distribution:\n");
    for (data_type, count) in type_vec.iter().take(10) {
        content.push_str(&format!("  {}: {}\n", data_type, count));
    }

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "Summary",
    )?;

    Ok(())
}
