use crate::table::Table;
use serde::{Deserialize, Serialize};

/// Represents a complete database schema.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseSchema {
    /// All tables in the database
    pub tables: Vec<Table>,
}
