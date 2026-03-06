use anyhow::Result;
use schemagit_snapshot::SnapshotManager;
use std::collections::{HashMap, HashSet};

use super::{output, utils};

/// Execute the validate command.
pub fn execute(
    snapshot_id: &str,
    directory: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);

    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;
    let mut content = String::from("=== Schema Validation ===\n\n");

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Check for duplicate table names
    let mut table_names: HashMap<String, usize> = HashMap::new();
    for table in &snapshot.schema.tables {
        *table_names.entry(table.name.clone()).or_insert(0) += 1;
    }

    for (name, count) in &table_names {
        if *count > 1 {
            errors.push(format!(
                "Duplicate table name: {} (appears {} times)",
                name, count
            ));
        }
    }

    // Build set of all table names for foreign key validation
    let all_table_names: HashSet<String> = snapshot
        .schema
        .tables
        .iter()
        .map(|t| t.name.clone())
        .collect();

    // Validate each table
    for table in &snapshot.schema.tables {
        // Check for duplicate column names
        let mut column_names: HashMap<String, usize> = HashMap::new();
        for column in &table.columns {
            *column_names.entry(column.name.clone()).or_insert(0) += 1;
        }

        for (name, count) in &column_names {
            if *count > 1 {
                errors.push(format!(
                    "Table '{}': Duplicate column name: {} (appears {} times)",
                    table.name, name, count
                ));
            }
        }

        // Check for missing primary key (heuristic: no 'id' column or no NOT NULL unique column)
        let has_id_column = table
            .columns
            .iter()
            .any(|c| c.name.to_lowercase() == "id" && !c.nullable);

        if !has_id_column {
            // Check if there's a unique NOT NULL column
            let has_pk_candidate = table.columns.iter().any(|c| !c.nullable)
                && table.indexes.iter().any(|idx| idx.unique);

            if !has_pk_candidate {
                warnings.push(format!(
                    "Table '{}': No obvious primary key found",
                    table.name
                ));
            }
        }

        // Validate foreign keys
        for fk in &table.foreign_keys {
            // Check if referenced table exists
            if !all_table_names.contains(&fk.ref_table) {
                errors.push(format!(
                    "Table '{}': Foreign key '{}' references non-existent table '{}'",
                    table.name, fk.name, fk.ref_table
                ));
            } else {
                // Check if column exists in current table
                let column_exists =
                    table.columns.iter().any(|c| c.name == fk.column);

                if !column_exists {
                    errors.push(format!(
                        "Table '{}': Foreign key '{}' references non-existent column '{}'",
                        table.name, fk.name, fk.column
                    ));
                }

                // Check if referenced column exists in referenced table
                if let Some(ref_table) = snapshot
                    .schema
                    .tables
                    .iter()
                    .find(|t| t.name == fk.ref_table)
                {
                    let ref_column_exists = ref_table
                        .columns
                        .iter()
                        .any(|c| c.name == fk.ref_column);

                    if !ref_column_exists {
                        errors.push(format!(
                            "Table '{}': Foreign key '{}' references non-existent column '{}.{}'",
                            table.name, fk.name, fk.ref_table, fk.ref_column
                        ));
                    }
                }
            }
        }

        // Validate indexes
        for index in &table.indexes {
            for col in &index.columns {
                let column_exists =
                    table.columns.iter().any(|c| &c.name == col);

                if !column_exists {
                    errors.push(format!(
                        "Table '{}': Index '{}' references non-existent column '{}'",
                        table.name, index.name, col
                    ));
                }
            }
        }

        // Check for tables with no columns
        if table.columns.is_empty() {
            warnings.push(format!("Table '{}': Has no columns", table.name));
        }
    }

    // Print results
    match (errors.is_empty(), warnings.is_empty()) {
        (true, true) => {
            content.push_str("Schema validation passed!\n");
            content.push_str("No errors or warnings found.\n");
        }

        (false, _) => {
            content.push_str(&format!("ERRORS ({})\n", errors.len()));
            for error in &errors {
                content.push_str(&format!("  - {}\n", error));
            }
            content.push('\n');
        }

        _ => {}
    }

    if !warnings.is_empty() {
        content.push_str(&format!("WARNINGS ({})\n", warnings.len()));
        for warning in &warnings {
            content.push_str(&format!("  - {}\n", warning));
        }
        content.push('\n');
    }

    match errors.is_empty() {
        false => content.push_str("Schema validation failed with errors.\n"),
        true if !warnings.is_empty() => {
            content.push_str("Schema validation passed with warnings.\n")
        }
        _ => {}
    }

    output::write_or_stdout(
        &content,
        output_file,
        yes,
        no_create_dir,
        "Validation report",
    )?;

    Ok(())
}
