use crate::schema::DatabaseSchema;
use crate::schema_graph::{
    ColumnNode, ConstraintNode, IndexNode, RelationEdge, SchemaGraph, TableNode,
};

/// Build a SchemaGraph from a DatabaseSchema.
/// Note: The plan mentions taking a Snapshot, but to avoid circular dependencies
/// between core and snapshot crates, we take DatabaseSchema directly.
pub fn build_schema_graph(schema: &DatabaseSchema) -> SchemaGraph {
    let mut graph = SchemaGraph::new();

    for table in &schema.tables {
        let mut table_node = TableNode {
            name: table.name.clone(),
            schema: None, // DatabaseSchema doesn't store schema name (like 'public' or 'dbo') yet
            columns: Vec::new(),
            primary_key: Vec::new(),
            indexes: Vec::new(),
        };

        // Track primary key columns if we can detect them
        // (Currently introspectors filter them, so this might be empty)
        let mut pk_columns = Vec::new();

        for column in &table.columns {
            let is_pk = false; // Detection logic could be added here if we had metadata

            let column_node = ColumnNode {
                name: column.name.clone(),
                data_type: column.data_type.clone(),
                nullable: column.nullable,
                default: column.default.clone(),
                is_primary: is_pk,
            };

            if is_pk {
                pk_columns.push(column.name.clone());
            }

            table_node.columns.push(column_node);
        }

        table_node.primary_key = pk_columns.clone();

        // Add constraints for PK if found
        if !pk_columns.is_empty() {
            graph.constraints.push(ConstraintNode {
                name: format!("pk_{}", table.name),
                table: table.name.clone(),
                constraint_type: "primary_key".to_string(),
                columns: pk_columns,
            });
        }

        for index in &table.indexes {
            let index_node = IndexNode {
                name: index.name.clone(),
                table: table.name.clone(),
                columns: index.columns.clone(),
                unique: index.unique,
            };
            graph.indexes.push(index_node);
            table_node.indexes.push(index.name.clone());

            // If it's unique, also record as a unique constraint
            if index.unique {
                graph.constraints.push(ConstraintNode {
                    name: index.name.clone(),
                    table: table.name.clone(),
                    constraint_type: "unique".to_string(),
                    columns: index.columns.clone(),
                });
            }
        }

        for fk in &table.foreign_keys {
            let edge = RelationEdge {
                from_table: table.name.clone(),
                to_table: fk.ref_table.clone(),
                from_column: fk.column.clone(),
                to_column: fk.ref_column.clone(),
                constraint_name: fk.name.clone(),
            };
            graph.relations.push(edge);

            graph.constraints.push(ConstraintNode {
                name: fk.name.clone(),
                table: table.name.clone(),
                constraint_type: "foreign_key".to_string(),
                columns: vec![fk.column.clone()],
            });
        }

        graph.tables.push(table_node);
    }

    graph
}
