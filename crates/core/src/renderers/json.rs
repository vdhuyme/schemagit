use crate::renderers::GraphRenderer;
use crate::schema_graph::SchemaGraph;

pub struct JsonRenderer;

impl GraphRenderer for JsonRenderer {
    fn render(&self, graph: &SchemaGraph) -> String {
        serde_json::to_string_pretty(graph).unwrap_or_else(|_| "[]".to_string())
    }
}
