use crate::renderers::GraphRenderer;
use crate::schema_graph::SchemaGraph;

pub struct MermaidRenderer;

impl GraphRenderer for MermaidRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        // Use graph diagram instead of ER diagram to avoid positioning issues
        let mut mermaid = String::from("graph TD\n");

        // Create nodes for each table
        for table in &graph.tables {
            let mut columns_text = String::new();
            let max_columns = 3; // Show fewer columns in graph view

            for (i, column) in table.columns.iter().enumerate() {
                if i >= max_columns {
                    columns_text.push_str(&format!("... +{} more<br>", table.columns.len() - max_columns));
                    break;
                }

                let _pk_mark = if column.is_primary { " 🔑" } else { "" };
                let data_type = simplify_type(&column.data_type);
                let col_name = column.name.replace([' ', '-'], "_");
                columns_text.push_str(&format!("{}: {}<br>", col_name, data_type));
                if column.is_primary {
                    columns_text.push_str("🔑<br>");
                }
            }

            mermaid.push_str(&format!(
                "    {0}[\"<b>{0}</b><br>{1}\"]\n",
                table.name,
                columns_text
            ));
        }

        mermaid.push('\n');

        // Create edges for relationships
        for relation in &graph.relations {
            mermaid.push_str(&format!(
                "    {} --> {}\n",
                relation.to_table,
                relation.from_table
            ));
        }

        mermaid
    }
}

/// Simplify SQL data types to valid Mermaid type names
fn simplify_type(sql_type: &str) -> String {
    let lower = sql_type.to_lowercase();
    
    // Extract base type (e.g., "nvarchar(255)" -> "nvarchar")
    let base_type = lower.split('(').next().unwrap_or(&lower).trim();
    
    // Map SQL types to Mermaid-friendly names
    match base_type {
        "int" | "integer" => "int".to_string(),
        "bigint" => "bigint".to_string(),
        "smallint" => "short".to_string(),
        "varchar" | "nvarchar" | "char" | "nchar" => "string".to_string(),
        "text" | "ntext" => "text".to_string(),
        "decimal" | "numeric" | "money" | "smallmoney" => "decimal".to_string(),
        "float" | "real" => "float".to_string(),
        "bit" => "boolean".to_string(),
        "date" => "date".to_string(),
        "time" => "time".to_string(),
        "datetime" | "datetime2" | "smalldatetime" | "datetimeoffset" => "datetime".to_string(),
        "timestamp" => "timestamp".to_string(),
        "guid" | "uniqueidentifier" => "uuid".to_string(),
        "binary" | "varbinary" => "binary".to_string(),
        _ => base_type.to_string(),
    }
}
