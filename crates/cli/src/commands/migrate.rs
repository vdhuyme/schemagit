use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_core::{DatabaseSchema, ForeignKey, Index};
use schemagit_diff::diff_schemas;
use schemagit_migration::create_generator;
use schemagit_snapshot::SnapshotManager;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use super::utils;

/// Execute the migrate command.
pub fn execute(
    old_path: &str,
    new_path: &str,
    snapshot_dir: &str,
    output_file: Option<&str>,
) -> Result<()> {
    println!("{}", "Generating migration...".cyan());

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

    verify_migration_references(&new_snapshot.schema, &diff)?;

    // Generate migration SQL
    let statements = generator.generate_sql(&diff);
    verify_duplicate_operations(&statements)?;
    let migration = statements.join("\n\n");

    // Output migration
    match output_file {
        Some(path) => {
            ensure_output_directory_exists(path)?;
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

fn verify_migration_references(
    schema: &DatabaseSchema,
    diff: &schemagit_diff::SchemaDiff,
) -> Result<()> {
    let table_names: HashSet<&str> = schema
        .tables
        .iter()
        .map(|table| table.name.as_str())
        .collect();

    for table in &diff.tables_added {
        verify_table_references(
            table.name.as_str(),
            &table.foreign_keys,
            schema,
        )?;
        verify_table_indexes(table.name.as_str(), &table.indexes, schema)?;
    }

    for table in &diff.tables_modified {
        verify_table_references(
            table.table_name.as_str(),
            &table.foreign_keys_added,
            schema,
        )?;
        verify_table_indexes(
            table.table_name.as_str(),
            &table.indexes_added,
            schema,
        )?;

        for fk in &table.foreign_keys_added {
            if !table_names.contains(fk.ref_table.as_str()) {
                return Err(anyhow::anyhow!(
                    "Migration generation error:\nReferenced table \"{}\" not found.",
                    fk.ref_table
                ));
            }
        }
    }

    Ok(())
}

fn verify_table_references(
    table_name: &str,
    foreign_keys: &[ForeignKey],
    schema: &DatabaseSchema,
) -> Result<()> {
    let table = schema
        .tables
        .iter()
        .find(|table| table.name == table_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Migration generation error:\nTable \"{}\" not found.",
                table_name
            )
        })?;

    for fk in foreign_keys {
        if !table.columns.iter().any(|column| column.name == fk.column) {
            return Err(anyhow::anyhow!(
                "Migration generation error:\nReferenced column \"{}.{}\" not found.",
                table_name,
                fk.column
            ));
        }

        let referenced_table = schema
            .tables
            .iter()
            .find(|candidate| candidate.name == fk.ref_table)
            .ok_or_else(|| {
                anyhow::anyhow!(
                    "Migration generation error:\nReferenced table \"{}\" not found.",
                    fk.ref_table
                )
            })?;

        if !referenced_table
            .columns
            .iter()
            .any(|column| column.name == fk.ref_column)
        {
            return Err(anyhow::anyhow!(
                "Migration generation error:\nReferenced column \"{}.{}\" not found.",
                fk.ref_table,
                fk.ref_column
            ));
        }
    }

    Ok(())
}

fn verify_table_indexes(
    table_name: &str,
    indexes: &[Index],
    schema: &DatabaseSchema,
) -> Result<()> {
    let table = schema
        .tables
        .iter()
        .find(|table| table.name == table_name)
        .ok_or_else(|| {
            anyhow::anyhow!(
                "Migration generation error:\nTable \"{}\" not found.",
                table_name
            )
        })?;

    for index in indexes {
        for index_column in &index.columns {
            if !table
                .columns
                .iter()
                .any(|column| column.name == *index_column)
            {
                return Err(anyhow::anyhow!(
                    "Migration generation error:\nReferenced column \"{}.{}\" not found.",
                    table_name,
                    index_column
                ));
            }
        }
    }

    Ok(())
}

fn verify_duplicate_operations(statements: &[String]) -> Result<()> {
    let mut seen = HashSet::new();

    for statement in statements {
        let key = statement.trim();
        if !seen.insert(key.to_string()) {
            return Err(anyhow::anyhow!(
                "Migration generation error:\nDuplicate operation detected: {}",
                key
            ));
        }
    }

    Ok(())
}

fn ensure_output_directory_exists(output_path: &str) -> Result<()> {
    let path = Path::new(output_path);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            return Err(anyhow::anyhow!(
                "Output directory does not exist: {}",
                parent.display()
            ));
        }
    }

    Ok(())
}
