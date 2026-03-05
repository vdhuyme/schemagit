use serde::{Deserialize, Serialize};

/// Represents a database index.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Index {
    /// Index name
    pub name: String,
    /// Columns included in the index
    pub columns: Vec<String>,
    /// Whether the index enforces uniqueness
    pub unique: bool,
}
