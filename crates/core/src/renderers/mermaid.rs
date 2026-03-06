use crate::renderers::GraphRenderer;
use crate::schema_graph::SchemaGraph;

pub struct MermaidRenderer;

impl GraphRenderer for MermaidRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        let mut mermaid = String::from("erDiagram\n");

        for table in &graph.tables {
            mermaid.push_str(&format!("    \"{}\" {{\n", table.name));
            for column in &table.columns {
                let pk_mark = if column.is_primary { " PK" } else { "" };
                // Mermaid column format: [type] [name] [PK/FK]
                // We sanitize name if it has spaces or special chars if Mermaid needs it
                mermaid.push_str(&format!(
                    "        {} \"{}\"{}\n",
                    column.data_type.replace(' ', "_"),
                    column.name,
                    pk_mark
                ));
            }
            mermaid.push_str("    }\n\n");
        }

        for relation in &graph.relations {
            // Relation format: table1 relation-type table2 : label
            // Example: users ||--o{ posts : has
            // We use ||--o{ (one to many) for foreign keys generally
            mermaid.push_str(&format!(
                "    \"{}\" ||--o{{ \"{}\" : \"{}\"\n",
                relation.to_table,
                relation.from_table,
                relation.constraint_name
            ));
        }

        mermaid
    }
}
