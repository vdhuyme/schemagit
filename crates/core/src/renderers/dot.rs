use crate::renderers::GraphRenderer;
use crate::schema_graph::SchemaGraph;

pub struct DotRenderer;

impl GraphRenderer for DotRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        let mut dot = String::from("digraph schema {\n");
        dot.push_str("    rankdir=LR;\n");
        dot.push_str("    node [shape=record];\n\n");

        for table in &graph.tables {
            // Reciprocal node: { table_name | col1\lcol2\l }
            let mut columns_label = String::new();
            for column in &table.columns {
                let pk_mark = if column.is_primary { " (PK)" } else { "" };
                columns_label.push_str(&format!(
                    "{}: {}{}\\l",
                    column.name, column.data_type, pk_mark
                ));
            }

            dot.push_str(&format!(
                "    \"{}\" [label=\"{{ {} | {} }}\", shape=Mrecord];\n",
                table.name, table.name, columns_label
            ));
        }

        dot.push('\n');

        for relation in &graph.relations {
            dot.push_str(&format!(
                "    \"{}\" -> \"{}\" [label=\"{}\"];\n",
                relation.from_table,
                relation.to_table,
                relation.constraint_name
            ));
        }

        dot.push_str("}\n");
        dot
    }
}
