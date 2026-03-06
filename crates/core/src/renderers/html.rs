use crate::renderers::GraphRenderer;
use crate::renderers::mermaid::MermaidRenderer;
use crate::schema_graph::SchemaGraph;
use askama::Template;

pub struct HtmlRenderer;

#[derive(Template)]
#[template(path = "graph.html")]
struct GraphTemplate {
    mermaid: String,
    tables: Vec<TableInfo>,
}

struct TableInfo {
    name: String,
    columns_count: usize,
}

impl GraphRenderer for HtmlRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        let mermaid_renderer = MermaidRenderer;
        let mermaid_content = mermaid_renderer.render(graph);

        let tables = graph
            .tables
            .iter()
            .map(|t| TableInfo {
                name: t.name.clone(),
                columns_count: t.columns.len(),
            })
            .collect();

        let template = GraphTemplate {
            mermaid: mermaid_content,
            tables,
        };

        template
            .render()
            .unwrap_or_else(|e| format!("Error rendering template: {}", e))
    }
}
