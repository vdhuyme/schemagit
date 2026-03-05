use serde::{Deserialize, Serialize};

/// Represents a database column with its properties.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Column {
    /// Column name
    pub name: String,
    /// Data type (e.g., "VARCHAR", "INTEGER", "TEXT")
    pub data_type: String,
    /// Whether the column accepts NULL values
    pub nullable: bool,
    /// Default value expression, if any
    pub default: Option<String>,
}
