use crate::MigrationGenerator;
use schemagit_core::{Column, ForeignKey, Index, Table};
use schemagit_diff::{ColumnModification, SchemaDiff, TableDiff};

/// PostgreSQL migration generator.
pub struct PostgresMigrationGenerator;

impl MigrationGenerator for PostgresMigrationGenerator {
    fn generate_sql(&self, diff: &SchemaDiff) -> Vec<String> {
        let mut statements = Vec::new();

        // Generate statements for added tables
        for table in &diff.tables_added {
            statements.push(self.generate_create_table(table));
        }

        // Generate statements for modified tables
        for table_diff in &diff.tables_modified {
            statements.extend(self.generate_table_modifications(table_diff));
        }

        // Generate statements for removed tables
        for table in &diff.tables_removed {
            statements.push(self.generate_drop_table(table));
        }

        statements
    }
}

impl PostgresMigrationGenerator {
    /// Generate CREATE TABLE statement.
    fn generate_create_table(&self, table: &Table) -> String {
        let mut lines = vec![format!(
            "CREATE TABLE {} (",
            Self::quote_identifier(&table.name)
        )];

        let mut column_defs = Vec::new();
        for column in &table.columns {
            column_defs.push(format!("  {}", self.column_definition(column)));
        }

        lines.push(column_defs.join(",\n"));
        lines.push(");".to_string());

        let mut statements = vec![lines.join("\n")];

        // Add index creation statements
        for index in &table.indexes {
            statements.push(self.generate_create_index(&table.name, index));
        }

        // Add foreign key constraints
        for fk in &table.foreign_keys {
            statements.push(self.generate_add_foreign_key(&table.name, fk));
        }

        statements.join("\n\n")
    }

    /// Generate DROP TABLE statement.
    fn generate_drop_table(&self, table: &Table) -> String {
        format!("DROP TABLE {};", Self::quote_identifier(&table.name))
    }

    /// Generate ALTER TABLE statements for table modifications.
    fn generate_table_modifications(
        &self,
        table_diff: &TableDiff,
    ) -> Vec<String> {
        let mut statements = Vec::new();
        let table_name = &table_diff.table_name;

        // Drop foreign keys first (they may depend on columns/indexes)
        for fk in &table_diff.foreign_keys_removed {
            statements.push(self.generate_drop_foreign_key(table_name, fk));
        }

        // Drop indexes
        for index in &table_diff.indexes_removed {
            statements.push(self.generate_drop_index(index));
        }

        // Drop columns
        for column in &table_diff.columns_removed {
            statements.push(self.generate_drop_column(table_name, column));
        }

        // Add columns
        for column in &table_diff.columns_added {
            statements.push(self.generate_add_column(table_name, column));
        }

        // Modify columns
        for col_mod in &table_diff.columns_modified {
            statements.extend(self.generate_modify_column(table_name, col_mod));
        }

        // Add indexes
        for index in &table_diff.indexes_added {
            statements.push(self.generate_create_index(table_name, index));
        }

        // Add foreign keys
        for fk in &table_diff.foreign_keys_added {
            statements.push(self.generate_add_foreign_key(table_name, fk));
        }

        statements
    }

    /// Generate column definition for CREATE TABLE.
    fn column_definition(&self, column: &Column) -> String {
        let mut parts = vec![
            Self::quote_identifier(&column.name),
            column.data_type.clone(),
        ];

        if !column.nullable {
            parts.push("NOT NULL".to_string());
        }

        if let Some(ref default) = column.default {
            parts.push(format!("DEFAULT {}", default));
        }

        parts.join(" ")
    }

    /// Generate ADD COLUMN statement.
    fn generate_add_column(&self, table_name: &str, column: &Column) -> String {
        format!(
            "ALTER TABLE {} ADD COLUMN {};",
            Self::quote_identifier(table_name),
            self.column_definition(column)
        )
    }

    /// Generate DROP COLUMN statement.
    fn generate_drop_column(
        &self,
        table_name: &str,
        column: &Column,
    ) -> String {
        format!(
            "ALTER TABLE {} DROP COLUMN {};",
            Self::quote_identifier(table_name),
            Self::quote_identifier(&column.name)
        )
    }

    /// Generate ALTER COLUMN statements for column modifications.
    fn generate_modify_column(
        &self,
        table_name: &str,
        col_mod: &ColumnModification,
    ) -> Vec<String> {
        let mut statements = Vec::new();
        let old = &col_mod.old_column;
        let new = &col_mod.new_column;
        let table = Self::quote_identifier(table_name);
        let column = Self::quote_identifier(&col_mod.column_name);

        // Change data type
        if old.data_type != new.data_type {
            statements.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} TYPE {};",
                table, column, new.data_type
            ));
        }

        // Change nullability
        if old.nullable != new.nullable {
            if new.nullable {
                statements.push(format!(
                    "ALTER TABLE {} ALTER COLUMN {} DROP NOT NULL;",
                    table, column
                ));
            } else {
                statements.push(format!(
                    "ALTER TABLE {} ALTER COLUMN {} SET NOT NULL;",
                    table, column
                ));
            }
        }

        // Change default
        if old.default != new.default {
            if let Some(ref default) = new.default {
                statements.push(format!(
                    "ALTER TABLE {} ALTER COLUMN {} SET DEFAULT {};",
                    table, column, default
                ));
            } else {
                statements.push(format!(
                    "ALTER TABLE {} ALTER COLUMN {} DROP DEFAULT;",
                    table, column
                ));
            }
        }

        statements
    }

    /// Generate CREATE INDEX statement.
    fn generate_create_index(&self, table_name: &str, index: &Index) -> String {
        let unique = if index.unique { "UNIQUE " } else { "" };
        let columns = index
            .columns
            .iter()
            .map(|c| Self::quote_identifier(c))
            .collect::<Vec<_>>()
            .join(", ");

        format!(
            "CREATE {}INDEX {} ON {} ({});",
            unique,
            Self::quote_identifier(&index.name),
            Self::quote_identifier(table_name),
            columns
        )
    }

    /// Generate DROP INDEX statement.
    fn generate_drop_index(&self, index: &Index) -> String {
        format!("DROP INDEX {};", Self::quote_identifier(&index.name))
    }

    /// Generate ADD CONSTRAINT statement for foreign key.
    fn generate_add_foreign_key(
        &self,
        table_name: &str,
        fk: &ForeignKey,
    ) -> String {
        format!(
            "ALTER TABLE {} ADD CONSTRAINT {} FOREIGN KEY ({}) REFERENCES {} ({});",
            Self::quote_identifier(table_name),
            Self::quote_identifier(&fk.name),
            Self::quote_identifier(&fk.column),
            Self::quote_identifier(&fk.ref_table),
            Self::quote_identifier(&fk.ref_column)
        )
    }

    /// Generate DROP CONSTRAINT statement for foreign key.
    fn generate_drop_foreign_key(
        &self,
        table_name: &str,
        fk: &ForeignKey,
    ) -> String {
        format!(
            "ALTER TABLE {} DROP CONSTRAINT {};",
            Self::quote_identifier(table_name),
            Self::quote_identifier(&fk.name)
        )
    }

    /// Quote an identifier for PostgreSQL.
    fn quote_identifier(name: &str) -> String {
        format!("\"{}\"", name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemagit_core::DatabaseSchema;
    use schemagit_diff::diff_schemas;

    #[test]
    fn test_create_table() {
        let generator = PostgresMigrationGenerator;
        let table = Table {
            name: "users".to_string(),
            columns: vec![
                Column {
                    name: "id".to_string(),
                    data_type: "INTEGER".to_string(),
                    nullable: false,
                    default: None,
                },
                Column {
                    name: "email".to_string(),
                    data_type: "TEXT".to_string(),
                    nullable: false,
                    default: None,
                },
            ],
            indexes: vec![],
            foreign_keys: vec![],
        };

        let sql = generator.generate_create_table(&table);
        assert!(sql.contains("CREATE TABLE"));
        assert!(sql.contains("\"users\""));
        assert!(sql.contains("\"id\""));
        assert!(sql.contains("\"email\""));
    }

    #[test]
    fn test_diff_to_migration() {
        let generator = PostgresMigrationGenerator;

        let old_schema = DatabaseSchema { tables: vec![] };
        let new_schema = DatabaseSchema {
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

        let diff = diff_schemas(&old_schema, &new_schema);
        let statements = generator.generate_sql(&diff);

        assert!(!statements.is_empty());
        assert!(statements[0].contains("CREATE TABLE"));
    }
}
