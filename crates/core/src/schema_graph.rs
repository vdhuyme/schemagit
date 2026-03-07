use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchemaGraph {
    pub tables: Vec<TableNode>,
    pub relations: Vec<RelationEdge>,
    pub indexes: Vec<IndexNode>,
    pub constraints: Vec<ConstraintNode>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableNode {
    pub name: String,
    pub schema: Option<String>,
    pub columns: Vec<ColumnNode>,
    pub primary_key: Vec<String>,
    pub indexes: Vec<String>, // Names of indexes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnNode {
    pub name: String,
    pub data_type: String,
    pub nullable: bool,
    pub default: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationEdge {
    pub from_table: String,
    pub to_table: String,
    pub from_column: String,
    pub to_column: String,
    pub constraint_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexNode {
    pub name: String,
    pub table: String,
    pub columns: Vec<String>,
    pub unique: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConstraintNode {
    pub name: String,
    pub table: String,
    pub constraint_type: String, // "primary_key", "foreign_key", "unique", "check"
    pub columns: Vec<String>,
}

impl Default for SchemaGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaGraph {
    pub fn new() -> Self {
        Self {
            tables: Vec::new(),
            relations: Vec::new(),
            indexes: Vec::new(),
            constraints: Vec::new(),
        }
    }
}
