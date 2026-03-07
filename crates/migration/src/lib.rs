pub mod mssql;
pub mod postgres;

use schemagit_diff::SchemaDiff;

/// Trait for generating SQL migrations from schema diffs.
pub trait MigrationGenerator {
    /// Generate SQL statements from a schema diff.
    ///
    /// # Arguments
    /// * `diff` - The schema diff to generate migrations for
    ///
    /// # Returns
    /// A vector of SQL statements
    fn generate_sql(&self, diff: &SchemaDiff) -> Vec<String>;

    /// Generate a complete migration script with all statements.
    fn generate_migration(&self, diff: &SchemaDiff) -> String {
        let statements = self.generate_sql(diff);
        statements.join("\n\n")
    }
}

/// Create a migration generator for the specified database type.
///
/// # Arguments
/// * `db_type` - Database type ("postgres", "mysql", "sqlite")
///
/// # Returns
/// A boxed migration generator, or None if the database type is not supported
pub fn create_generator(db_type: &str) -> Option<Box<dyn MigrationGenerator>> {
    match db_type.to_lowercase().as_str() {
        "postgres" | "postgresql" => {
            Some(Box::new(postgres::PostgresMigrationGenerator))
        }
        "mssql" | "sqlserver" => Some(Box::new(mssql::MssqlMigrationGenerator)),
        _ => None,
    }
}
