use anyhow::Result;
use colored::Colorize;
use schemagit_core::{DatabaseSchema, Table};
use schemagit_snapshot::SnapshotManager;
use std::collections::{HashMap, HashSet};

use super::utils;

/// Execute the validate command.
pub fn execute(snapshot_id: &str, directory: &str) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

    println!("{}", "=== Schema Validation ===".bold().cyan());
    println!();

    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    add_duplicate_table_errors(&snapshot.schema, &mut errors);

    // Build set of all table names for foreign key validation
    let all_table_names: HashSet<String> = snapshot
        .schema
        .tables
        .iter()
        .map(|t| t.name.clone())
        .collect();

    // Validate each table
    for table in &snapshot.schema.tables {
        validate_table(
            table,
            &snapshot.schema,
            &all_table_names,
            &mut errors,
            &mut warnings,
        );
    }

    print_validation_results(&errors, &warnings);

    Ok(())
}

fn add_duplicate_table_errors(
    schema: &DatabaseSchema,
    errors: &mut Vec<String>,
) {
    let mut table_names: HashMap<String, usize> = HashMap::new();
    for table in &schema.tables {
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
}

fn validate_table(
    table: &Table,
    schema: &DatabaseSchema,
    all_table_names: &HashSet<String>,
    errors: &mut Vec<String>,
    warnings: &mut Vec<String>,
) {
    add_duplicate_column_errors(table, errors);
    add_primary_key_warning(table, warnings);
    add_foreign_key_errors(table, schema, all_table_names, errors);
    add_index_errors(table, errors);

    if table.columns.is_empty() {
        warnings.push(format!("Table '{}': Has no columns", table.name));
    }
}

fn add_duplicate_column_errors(table: &Table, errors: &mut Vec<String>) {
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
}

fn add_primary_key_warning(table: &Table, warnings: &mut Vec<String>) {
    let has_id_column = table
        .columns
        .iter()
        .any(|c| c.name.to_lowercase() == "id" && !c.nullable);

    if has_id_column {
        return;
    }

    let has_pk_candidate = table.columns.iter().any(|c| !c.nullable)
        && table.indexes.iter().any(|idx| idx.unique);

    if !has_pk_candidate {
        warnings.push(format!(
            "Table '{}': No obvious primary key found",
            table.name
        ));
    }
}

fn add_foreign_key_errors(
    table: &Table,
    schema: &DatabaseSchema,
    all_table_names: &HashSet<String>,
    errors: &mut Vec<String>,
) {
    for fk in &table.foreign_keys {
        if !all_table_names.contains(&fk.ref_table) {
            errors.push(format!(
                "Table '{}': Foreign key '{}' references non-existent table '{}'",
                table.name, fk.name, fk.ref_table
            ));
            continue;
        }

        if !table.columns.iter().any(|c| c.name == fk.column) {
            errors.push(format!(
                "Table '{}': Foreign key '{}' references non-existent column '{}'",
                table.name, fk.name, fk.column
            ));
        }

        if let Some(ref_table) =
            schema.tables.iter().find(|t| t.name == fk.ref_table)
        {
            let ref_column_exists =
                ref_table.columns.iter().any(|c| c.name == fk.ref_column);

            if !ref_column_exists {
                errors.push(format!(
                    "Table '{}': Foreign key '{}' references non-existent column '{}.{}'",
                    table.name, fk.name, fk.ref_table, fk.ref_column
                ));
            }
        }
    }
}

fn add_index_errors(table: &Table, errors: &mut Vec<String>) {
    for index in &table.indexes {
        for col in &index.columns {
            if !table.columns.iter().any(|c| &c.name == col) {
                errors.push(format!(
                    "Table '{}': Index '{}' references non-existent column '{}'",
                    table.name, index.name, col
                ));
            }
        }
    }
}

fn print_validation_results(errors: &[String], warnings: &[String]) {
    match (errors.is_empty(), warnings.is_empty()) {
        (true, true) => {
            println!("{}", "✓ Schema validation passed!".green().bold());
            println!("No errors or warnings found.");
        }
        (false, _) => {
            println!("{}", format!("ERRORS ({})", errors.len()).red().bold());
            for error in errors {
                println!("  {} {}", "✗".red(), error);
            }
            println!();
        }
        _ => {}
    }

    if !warnings.is_empty() {
        println!(
            "{}",
            format!("WARNINGS ({})", warnings.len()).yellow().bold()
        );
        for warning in warnings {
            println!("  {} {}", "⚠".yellow(), warning);
        }
        println!();
    }

    match errors.is_empty() {
        false => {
            println!("{}", "Schema validation failed with errors.".red().bold())
        }
        true if !warnings.is_empty() => {
            println!(
                "{}",
                "Schema validation passed with warnings.".yellow().bold()
            )
        }
        _ => {}
    }
}
