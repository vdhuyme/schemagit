use serde::{Deserialize, Serialize};

/// Represents a foreign key constraint.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForeignKey {
    /// Constraint name
    pub name: String,
    /// Column in the current table
    pub column: String,
    /// Referenced table name
    pub ref_table: String,
    /// Referenced column name
    pub ref_column: String,
}
