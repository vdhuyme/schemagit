use crate::graph_builder::build_schema_graph;
use crate::renderers::GraphRenderer;
use crate::renderers::html::HtmlRenderer;
use crate::schema::DatabaseSchema;

pub struct DocsGenerator;

impl DocsGenerator {
    pub fn generate(schema: &DatabaseSchema) -> String {
        let graph = build_schema_graph(schema);
        let renderer = HtmlRenderer;

        // For phase 3, we can just use the HtmlRenderer but maybe enhance it
        // Or create a more "documentation" focused HTML.
        // For now, let's reuse HtmlRenderer as it already has the diagram and metadata.
        renderer.render(&graph)
    }
}
