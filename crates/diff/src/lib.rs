use schemagit_core::{Column, DatabaseSchema, ForeignKey, Index, Table};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Represents a difference in a table structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableDiff {
    pub table_name: String,
    pub columns_added: Vec<Column>,
    pub columns_removed: Vec<Column>,
    pub columns_modified: Vec<ColumnModification>,
    pub indexes_added: Vec<Index>,
    pub indexes_removed: Vec<Index>,
    pub foreign_keys_added: Vec<ForeignKey>,
    pub foreign_keys_removed: Vec<ForeignKey>,
}

/// Represents a modification to a column.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ColumnModification {
    pub column_name: String,
    pub old_column: Column,
    pub new_column: Column,
}

/// Represents the differences between two database schemas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SchemaDiff {
    pub tables_added: Vec<Table>,
    pub tables_removed: Vec<Table>,
    pub tables_modified: Vec<TableDiff>,
}

impl SchemaDiff {
    /// Check if there are any differences.
    pub fn has_changes(&self) -> bool {
        !self.tables_added.is_empty()
            || !self.tables_removed.is_empty()
            || !self.tables_modified.is_empty()
    }

    /// Get a human-readable summary of changes.
    pub fn summary(&self) -> String {
        let mut lines = Vec::new();

        if !self.tables_added.is_empty() {
            lines.push(format!("Tables Added: {}", self.tables_added.len()));
            for table in &self.tables_added {
                lines.push(format!("  + {}", table.name));
            }
        }

        if !self.tables_removed.is_empty() {
            lines
                .push(format!("Tables Removed: {}", self.tables_removed.len()));
            for table in &self.tables_removed {
                lines.push(format!("  - {}", table.name));
            }
        }

        if !self.tables_modified.is_empty() {
            lines.push(format!(
                "Tables Modified: {}",
                self.tables_modified.len()
            ));
            for table_diff in &self.tables_modified {
                lines.push(format!("  ~ {}", table_diff.table_name));

                for col in &table_diff.columns_added {
                    lines.push(format!("      + Column: {}", col.name));
                }

                for col in &table_diff.columns_removed {
                    lines.push(format!("      - Column: {}", col.name));
                }

                for col_mod in &table_diff.columns_modified {
                    lines.push(format!(
                        "      ~ Column: {}",
                        col_mod.column_name
                    ));
                }

                for idx in &table_diff.indexes_added {
                    lines.push(format!("      + Index: {}", idx.name));
                }

                for idx in &table_diff.indexes_removed {
                    lines.push(format!("      - Index: {}", idx.name));
                }

                for fk in &table_diff.foreign_keys_added {
                    lines.push(format!("      + Foreign Key: {}", fk.name));
                }

                for fk in &table_diff.foreign_keys_removed {
                    lines.push(format!("      - Foreign Key: {}", fk.name));
                }
            }
        }

        if lines.is_empty() {
            "No changes detected".to_string()
        } else {
            lines.join("\n")
        }
    }
}

/// Compare two database schemas and generate a diff.
///
/// # Arguments
/// * `old_schema` - The original schema
/// * `new_schema` - The new schema to compare against
///
/// # Returns
/// A `SchemaDiff` describing all differences between the schemas
pub fn diff_schemas(
    old_schema: &DatabaseSchema,
    new_schema: &DatabaseSchema,
) -> SchemaDiff {
    // Create maps for quick lookup
    let old_tables: HashMap<String, &Table> = old_schema
        .tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    let new_tables: HashMap<String, &Table> = new_schema
        .tables
        .iter()
        .map(|t| (t.name.clone(), t))
        .collect();

    let old_table_names: HashSet<String> = old_tables.keys().cloned().collect();
    let new_table_names: HashSet<String> = new_tables.keys().cloned().collect();

    // Find added tables
    let tables_added: Vec<Table> = new_table_names
        .difference(&old_table_names)
        .filter_map(|name| new_tables.get(name).map(|&t| t.clone()))
        .collect();

    // Find removed tables
    let tables_removed: Vec<Table> = old_table_names
        .difference(&new_table_names)
        .filter_map(|name| old_tables.get(name).map(|&t| t.clone()))
        .collect();

    // Find modified tables
    let mut tables_modified = Vec::new();
    for table_name in old_table_names.intersection(&new_table_names) {
        if let (Some(&old_table), Some(&new_table)) =
            (old_tables.get(table_name), new_tables.get(table_name))
        {
            let table_diff = diff_tables(old_table, new_table);
            if table_diff.has_changes() {
                tables_modified.push(table_diff);
            }
        }
    }

    SchemaDiff {
        tables_added,
        tables_removed,
        tables_modified,
    }
}

/// Compare two tables and generate a diff.
fn diff_tables(old_table: &Table, new_table: &Table) -> TableDiff {
    let table_name = old_table.name.clone();

    // Diff columns
    let (columns_added, columns_removed, columns_modified) =
        diff_columns(&old_table.columns, &new_table.columns);

    // Diff indexes
    let (indexes_added, indexes_removed) =
        diff_indexes(&old_table.indexes, &new_table.indexes);

    // Diff foreign keys
    let (foreign_keys_added, foreign_keys_removed) =
        diff_foreign_keys(&old_table.foreign_keys, &new_table.foreign_keys);

    TableDiff {
        table_name,
        columns_added,
        columns_removed,
        columns_modified,
        indexes_added,
        indexes_removed,
        foreign_keys_added,
        foreign_keys_removed,
    }
}

impl TableDiff {
    /// Check if this table has any changes.
    fn has_changes(&self) -> bool {
        !self.columns_added.is_empty()
            || !self.columns_removed.is_empty()
            || !self.columns_modified.is_empty()
            || !self.indexes_added.is_empty()
            || !self.indexes_removed.is_empty()
            || !self.foreign_keys_added.is_empty()
            || !self.foreign_keys_removed.is_empty()
    }
}

/// Compare two lists of columns.
fn diff_columns(
    old_columns: &[Column],
    new_columns: &[Column],
) -> (Vec<Column>, Vec<Column>, Vec<ColumnModification>) {
    let old_cols: HashMap<String, &Column> =
        old_columns.iter().map(|c| (c.name.clone(), c)).collect();

    let new_cols: HashMap<String, &Column> =
        new_columns.iter().map(|c| (c.name.clone(), c)).collect();

    let old_names: HashSet<String> = old_cols.keys().cloned().collect();
    let new_names: HashSet<String> = new_cols.keys().cloned().collect();

    // Added columns
    let columns_added: Vec<Column> = new_names
        .difference(&old_names)
        .filter_map(|name| new_cols.get(name).map(|&c| c.clone()))
        .collect();

    // Removed columns
    let columns_removed: Vec<Column> = old_names
        .difference(&new_names)
        .filter_map(|name| old_cols.get(name).map(|&c| c.clone()))
        .collect();

    // Modified columns
    let mut columns_modified = Vec::new();
    for name in old_names.intersection(&new_names) {
        if let (Some(&old_col), Some(&new_col)) =
            (old_cols.get(name), new_cols.get(name))
        {
            if old_col != new_col {
                columns_modified.push(ColumnModification {
                    column_name: name.clone(),
                    old_column: old_col.clone(),
                    new_column: new_col.clone(),
                });
            }
        }
    }

    (columns_added, columns_removed, columns_modified)
}

/// Compare two lists of indexes.
fn diff_indexes(
    old_indexes: &[Index],
    new_indexes: &[Index],
) -> (Vec<Index>, Vec<Index>) {
    let old_idxs: HashMap<String, &Index> =
        old_indexes.iter().map(|i| (i.name.clone(), i)).collect();

    let new_idxs: HashMap<String, &Index> =
        new_indexes.iter().map(|i| (i.name.clone(), i)).collect();

    let old_names: HashSet<String> = old_idxs.keys().cloned().collect();
    let new_names: HashSet<String> = new_idxs.keys().cloned().collect();

    let indexes_added: Vec<Index> = new_names
        .difference(&old_names)
        .filter_map(|name| new_idxs.get(name).map(|&i| i.clone()))
        .collect();

    let indexes_removed: Vec<Index> = old_names
        .difference(&new_names)
        .filter_map(|name| old_idxs.get(name).map(|&i| i.clone()))
        .collect();

    (indexes_added, indexes_removed)
}

/// Compare two lists of foreign keys.
fn diff_foreign_keys(
    old_fks: &[ForeignKey],
    new_fks: &[ForeignKey],
) -> (Vec<ForeignKey>, Vec<ForeignKey>) {
    let old_fks_map: HashMap<String, &ForeignKey> =
        old_fks.iter().map(|fk| (fk.name.clone(), fk)).collect();

    let new_fks_map: HashMap<String, &ForeignKey> =
        new_fks.iter().map(|fk| (fk.name.clone(), fk)).collect();

    let old_names: HashSet<String> = old_fks_map.keys().cloned().collect();
    let new_names: HashSet<String> = new_fks_map.keys().cloned().collect();

    let foreign_keys_added: Vec<ForeignKey> = new_names
        .difference(&old_names)
        .filter_map(|name| new_fks_map.get(name).map(|&fk| fk.clone()))
        .collect();

    let foreign_keys_removed: Vec<ForeignKey> = old_names
        .difference(&new_names)
        .filter_map(|name| old_fks_map.get(name).map(|&fk| fk.clone()))
        .collect();

    (foreign_keys_added, foreign_keys_removed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_no_diff() {
        let schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    default: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let diff = diff_schemas(&schema, &schema);
        assert!(!diff.has_changes());
    }

    #[test]
    fn test_table_added() {
        let old_schema = DatabaseSchema { tables: vec![] };
        let new_schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let diff = diff_schemas(&old_schema, &new_schema);
        assert_eq!(diff.tables_added.len(), 1);
        assert_eq!(diff.tables_added[0].name, "users");
    }
}
