use crate::MigrationGenerator;
use schemagit_core::{Column, ForeignKey, Index, Table};
use schemagit_diff::{ColumnModification, SchemaDiff, TableDiff};

const UNIQUE_PREFIX: &str = "UNIQUE ";
const EMPTY: &str = "";
const NULL_LITERAL: &str = "NULL";
const NOT_NULL_LITERAL: &str = "NOT NULL";

/// SQL Server (MSSQL) migration generator.
pub struct MssqlMigrationGenerator;

impl MigrationGenerator for MssqlMigrationGenerator {
    fn generate_sql(&self, diff: &SchemaDiff) -> Vec<String> {
        let mut statements = Vec::new();

        for table in &diff.tables_added {
            statements.push(self.generate_create_table(table));
        }

        for table_diff in &diff.tables_modified {
            statements.extend(self.generate_table_modifications(table_diff));
        }

        for table in &diff.tables_removed {
            statements.push(self.generate_drop_table(table));
        }

        statements
    }
}

impl MssqlMigrationGenerator {
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

        for index in &table.indexes {
            statements.push(self.generate_create_index(&table.name, index));
        }

        for fk in &table.foreign_keys {
            statements.push(self.generate_add_foreign_key(&table.name, fk));
        }

        statements.join("\n\n")
    }

    fn generate_drop_table(&self, table: &Table) -> String {
        format!("DROP TABLE {};", Self::quote_identifier(&table.name))
    }

    fn generate_table_modifications(
        &self,
        table_diff: &TableDiff,
    ) -> Vec<String> {
        let mut statements = Vec::new();
        let table_name = &table_diff.table_name;

        for fk in &table_diff.foreign_keys_removed {
            statements.push(self.generate_drop_foreign_key(table_name, fk));
        }

        for index in &table_diff.indexes_removed {
            statements.push(self.generate_drop_index(table_name, index));
        }

        for column in &table_diff.columns_removed {
            statements.push(self.generate_drop_column(table_name, column));
        }

        for column in &table_diff.columns_added {
            statements.push(self.generate_add_column(table_name, column));
        }

        for col_mod in &table_diff.columns_modified {
            statements.extend(self.generate_modify_column(table_name, col_mod));
        }

        for index in &table_diff.indexes_added {
            statements.push(self.generate_create_index(table_name, index));
        }

        for fk in &table_diff.foreign_keys_added {
            statements.push(self.generate_add_foreign_key(table_name, fk));
        }

        statements
    }

    fn column_definition(&self, column: &Column) -> String {
        let mut parts = vec![
            Self::quote_identifier(&column.name),
            column.data_type.clone(),
        ];

        parts.push(Self::nullability_literal(column.nullable).to_string());

        // SQL Server defaults are modeled as constraints; we emit an ADD DEFAULT later in modify/add flows
        // to keep output deterministic and reversible.
        parts.join(" ")
    }

    fn generate_add_column(&self, table_name: &str, column: &Column) -> String {
        let mut statements = vec![format!(
            "ALTER TABLE {} ADD {};",
            Self::quote_identifier(table_name),
            self.column_definition(column)
        )];

        if let Some(ref default) = column.default {
            statements.push(self.generate_set_default(
                table_name,
                &column.name,
                default,
            ));
        }

        statements.join("\n")
    }

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

        if old.data_type != new.data_type || old.nullable != new.nullable {
            statements.push(format!(
                "ALTER TABLE {} ALTER COLUMN {} {} {};",
                table,
                column,
                new.data_type,
                Self::nullability_literal(new.nullable)
            ));
        }

        if old.default != new.default {
            match new.default.as_deref() {
                Some(default) => {
                    statements.push(self.generate_set_default(
                        table_name,
                        &col_mod.column_name,
                        default,
                    ));
                }
                None => {
                    statements.push(self.generate_drop_default(
                        table_name,
                        &col_mod.column_name,
                    ));
                }
            }
        }

        statements
    }

    fn generate_create_index(&self, table_name: &str, index: &Index) -> String {
        let unique = if index.unique { UNIQUE_PREFIX } else { EMPTY };
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

    fn generate_drop_index(&self, table_name: &str, index: &Index) -> String {
        format!(
            "DROP INDEX {} ON {};",
            Self::quote_identifier(&index.name),
            Self::quote_identifier(table_name)
        )
    }

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

    fn generate_drop_default(
        &self,
        table_name: &str,
        column_name: &str,
    ) -> String {
        let table_literal = escape_tsql_string(table_name);
        let column_literal = escape_tsql_string(column_name);
        let table_ident = Self::quote_identifier(table_name);

        format!(
            "DECLARE @df sysname;\n\
            SELECT @df = dc.name\n\
            FROM sys.default_constraints dc\n\
            JOIN sys.columns c ON c.default_object_id = dc.object_id\n\
            JOIN sys.tables t ON t.object_id = c.object_id\n\
            JOIN sys.schemas s ON s.schema_id = t.schema_id\n\
            WHERE s.name = 'dbo' AND t.name = N'{table_literal}' AND c.name = N'{column_literal}';\n\
            IF @df IS NOT NULL EXEC(N'ALTER TABLE {table_ident} DROP CONSTRAINT [' + @df + ']');"
        )
    }

    fn generate_set_default(
        &self,
        table_name: &str,
        column_name: &str,
        default: &str,
    ) -> String {
        let drop = self.generate_drop_default(table_name, column_name);
        format!(
            "{}\nALTER TABLE {} ADD DEFAULT {} FOR {};",
            drop,
            Self::quote_identifier(table_name),
            default,
            Self::quote_identifier(column_name)
        )
    }

    fn quote_identifier(name: &str) -> String {
        let escaped = name.replace(']', "]]");
        format!("[{}]", escaped)
    }

    fn nullability_literal(nullable: bool) -> &'static str {
        if nullable {
            return NULL_LITERAL;
        }

        NOT_NULL_LITERAL
    }
}

fn escape_tsql_string(s: &str) -> String {
    s.replace('\'', "''")
}

#[cfg(test)]
mod tests {
    use super::*;
    use schemagit_core::DatabaseSchema;
    use schemagit_diff::diff_schemas;

    #[test]
    fn test_create_table_quotes_brackets() {
        let generator = MssqlMigrationGenerator;
        let table = Table {
            name: "users".to_string(),
            columns: vec![Column {
                name: "id".to_string(),
                data_type: "int".to_string(),
                nullable: false,
                default: None,
            }],
            indexes: vec![],
            foreign_keys: vec![],
        };

        let sql = generator.generate_create_table(&table);
        assert!(sql.contains("CREATE TABLE [users]"));
        assert!(sql.contains("[id] int NOT NULL"));
    }

    #[test]
    fn test_add_column_uses_add_not_add_column() {
        let generator = MssqlMigrationGenerator;

        let old_schema = DatabaseSchema { tables: vec![] };
        let new_schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "id".to_string(),
                    data_type: "int".to_string(),
                    nullable: false,
                    default: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let diff = diff_schemas(&old_schema, &new_schema);
        let sql = generator.generate_migration(&diff);
        assert!(sql.contains("CREATE TABLE [users]"));
        assert!(!sql.contains("ADD COLUMN"));
    }

    #[test]
    fn test_drop_index_includes_table() {
        let generator = MssqlMigrationGenerator;
        let stmt = generator.generate_drop_index(
            "users",
            &Index {
                name: "idx_users_email".to_string(),
                columns: vec!["email".to_string()],
                unique: false,
            },
        );
        assert_eq!(stmt, "DROP INDEX [idx_users_email] ON [users];");
    }

    #[test]
    fn test_alter_column_syntax() {
        let generator = MssqlMigrationGenerator;

        let old_schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "email".to_string(),
                    data_type: "nvarchar(100)".to_string(),
                    nullable: true,
                    default: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let new_schema = DatabaseSchema {
            tables: vec![Table {
                name: "users".to_string(),
                columns: vec![Column {
                    name: "email".to_string(),
                    data_type: "nvarchar(200)".to_string(),
                    nullable: false,
                    default: None,
                }],
                indexes: vec![],
                foreign_keys: vec![],
            }],
        };

        let diff = diff_schemas(&old_schema, &new_schema);
        let sql = generator.generate_migration(&diff);
        assert!(sql.contains(
            "ALTER TABLE [users] ALTER COLUMN [email] nvarchar(200) NOT NULL;"
        ));
        assert!(!sql.contains(" TYPE "));
    }
}
