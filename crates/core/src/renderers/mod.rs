pub mod dot;
pub mod html;
pub mod json;
pub mod mermaid;

use crate::schema_graph::SchemaGraph;

pub trait GraphRenderer {
    fn render(&self, graph: &SchemaGraph) -> String;
}
