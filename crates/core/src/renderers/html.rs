use crate::renderers::GraphRenderer;
use crate::renderers::mermaid::MermaidRenderer;
use crate::schema_graph::SchemaGraph;
use askama::Template;

pub struct HtmlRenderer;

#[derive(Template)]
#[template(path = "graph.html")]
struct GraphTemplate {
    #[allow(dead_code)]
    mermaid: String,
    #[allow(dead_code)]
    schema_json: String,
}

#[derive(serde::Serialize)]
struct ColumnInfo {
    name: String,
    data_type: String,
    nullable: bool,
    is_primary: bool,
}

#[derive(serde::Serialize)]
struct TableInfo {
    name: String,
    columns: Vec<ColumnInfo>,
}

impl GraphRenderer for HtmlRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        let mermaid_renderer = MermaidRenderer;
        let mermaid_content = mermaid_renderer.render(graph);

        let tables: Vec<TableInfo> = graph
            .tables
            .iter()
            .map(|t| {
                let columns = t
                    .columns
                    .iter()
                    .map(|c| ColumnInfo {
                        name: c.name.clone(),
                        data_type: c.data_type.clone(),
                        nullable: c.nullable,
                        is_primary: c.is_primary,
                    })
                    .collect();

                TableInfo {
                    name: t.name.clone(),
                    columns,
                }
            })
            .collect();

        let schema_json =
            serde_json::to_string(&tables).unwrap_or_else(|_| "[]".to_string());

        let template = GraphTemplate {
            mermaid: mermaid_content,
            schema_json,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Error rendering template: {}", e))
    }
}
