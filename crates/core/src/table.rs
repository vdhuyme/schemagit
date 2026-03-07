use crate::column::Column;
use crate::foreign_key::ForeignKey;
use crate::index::Index;
use serde::{Deserialize, Serialize};

/// Represents a database table with its structure.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Table {
    /// Table name
    pub name: String,
    /// Columns in the table
    pub columns: Vec<Column>,
    /// Indexes defined on the table
    pub indexes: Vec<Index>,
    /// Foreign key constraints
    pub foreign_keys: Vec<ForeignKey>,
}
