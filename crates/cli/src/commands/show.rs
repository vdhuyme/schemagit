use anyhow::Result;
use schemagit_snapshot::SnapshotManager;

use super::{output, utils};

/// Execute the show command.
pub fn execute(
    snapshot_id: &str,
    directory: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

    let mut content = String::new();
    content.push_str("=== Snapshot Details ===\n\n");
    content.push_str(&format!("Database Type: {}\n", snapshot.database_type));
    content.push_str(&format!(
        "Created: {}\n",
        snapshot.timestamp.format("%Y-%m-%d %H:%M:%S")
    ));
    content.push_str(&format!("Tables: {}\n\n", snapshot.schema.tables.len()));

    content.push_str("=== Tables ===\n");
    for table in &snapshot.schema.tables {
        content.push('\n');
        content.push_str(&format!("  {}\n", table.name));
        content.push_str(&format!("    Columns: {}\n", table.columns.len()));

        for column in &table.columns {
            let nullable = if column.nullable { "NULL" } else { "NOT NULL" };
            let default_str = match &column.default {
                Some(d) => format!(" DEFAULT {}", d),
                None => String::new(),
            };

            content.push_str(&format!(
                "      - {} {} {}{}",
                column.name, column.data_type, nullable, default_str
            ));
            content.push('\n');
        }

        if !table.indexes.is_empty() {
            content
                .push_str(&format!("    Indexes: {}\n", table.indexes.len()));
            for index in &table.indexes {
                let unique = if index.unique { "UNIQUE" } else { "" };
                content.push_str(&format!(
                    "      - {} {} ({})",
                    index.name,
                    unique,
                    index.columns.join(", ")
                ));
                content.push('\n');
            }
        }

        if !table.foreign_keys.is_empty() {
            content.push_str(&format!(
                "    Foreign Keys: {}\n",
                table.foreign_keys.len()
            ));
            for fk in &table.foreign_keys {
                content.push_str(&format!(
                    "      - {} ({} -> {}.{})",
                    fk.name, fk.column, fk.ref_table, fk.ref_column
                ));
                content.push('\n');
            }
        }
    }

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "Snapshot detail",
    )?;

    Ok(())
}
