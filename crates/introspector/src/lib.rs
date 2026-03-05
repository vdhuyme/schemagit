mod mssql;
mod postgres;

use schemagit_core::DatabaseSchema;
use thiserror::Error;

pub use mssql::MssqlIntrospector;
pub use postgres::PostgresIntrospector;

/// Errors that can occur during schema introspection.
#[derive(Debug, Error)]
pub enum IntrospectorError {
    #[error("Database connection error: {0}")]
    ConnectionError(String),

    #[error("Query execution error: {0}")]
    QueryError(String),

    #[error("Schema introspection error: {0}")]
    IntrospectionError(String),

    #[error("Unsupported database feature: {0}")]
    UnsupportedFeature(String),
}

pub type IntrospectorResult<T> = Result<T, IntrospectorError>;

/// Trait that all database introspectors must implement.
///
/// This trait provides a database-agnostic interface for introspecting
/// schema information from different database systems.
#[async_trait::async_trait]
pub trait Introspector: Send + Sync {
    /// Introspect the database schema.
    ///
    /// This method connects to the database, reads metadata, and returns
    /// a standardized `DatabaseSchema` representation.
    async fn introspect_schema(&self) -> IntrospectorResult<DatabaseSchema>;

    /// Get the name of the database being introspected.
    async fn database_name(&self) -> IntrospectorResult<String>;

    /// Get the database type identifier (e.g., "postgres", "mysql", "mssql").
    fn database_type(&self) -> &str;
}

/// Create an introspector based on the database type.
///
/// # Arguments
/// * `db_type` - Database type ("postgres", "mysql", "mssql")
/// * `connection_string` - Connection string for the database
///
/// # Returns
/// A boxed introspector, or an error if the database type is not supported
pub fn create_introspector(
    db_type: &str,
    connection_string: &str,
) -> IntrospectorResult<Box<dyn Introspector>> {
    match db_type.to_lowercase().as_str() {
        "postgres" | "postgresql" => Ok(Box::new(PostgresIntrospector::new(
            connection_string.to_string(),
        ))),
        "mssql" | "sqlserver" => Ok(Box::new(MssqlIntrospector::new(
            connection_string.to_string(),
        ))),
        _ => Err(IntrospectorError::UnsupportedFeature(format!(
            "Unsupported database type: {}",
            db_type
        ))),
    }
}
