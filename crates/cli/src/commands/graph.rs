use super::utils;
use crate::output::{self, OutputOptions};
use anyhow::Result;
use schemagit_core::{
    build_schema_graph,
    renderers::{
        GraphRenderer, dot::DotRenderer, html::HtmlRenderer,
        json::JsonRenderer, mermaid::MermaidRenderer,
    },
};
use schemagit_snapshot::SnapshotManager;
use std::collections::{HashMap, HashSet};

/// Execute the graph command.
pub fn execute(
    snapshot_id: &str,
    directory: &str,
    format: &str,
    output_file: Option<&str>,
    _yes: bool,
    _no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

    let schema_graph = build_schema_graph(&snapshot.schema);

    let content = match format.to_lowercase().as_str() {
        "mermaid" => MermaidRenderer.render(&schema_graph),
        "dot" => DotRenderer.render(&schema_graph),
        "html" => HtmlRenderer.render(&schema_graph),
        "json" => JsonRenderer.render(&schema_graph),
        "text" => {
            // Keep existing text renderer logic for now until Phase 3/4
            render_text_legacy(&snapshot.schema)
        }
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown format: {}. Use mermaid, dot, html, json, or text",
                format
            ));
        }
    };

    let options = OutputOptions {
        format: format.to_string(),
        output_path: output_file.map(|s| s.to_string()),
        pretty: true,
    };

    output::write_output(&content, &options)?;

    Ok(())
}

/// Serve the interactive schema viewer.
pub async fn serve(
    snapshot_id: &str,
    directory: &str,
    port: u16,
) -> Result<()> {
    use axum::{Router, response::Html, routing::get};
    use schemagit_core::renderers::html::HtmlRenderer;

    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;
    let schema_graph = build_schema_graph(&snapshot.schema);
    let html_content = HtmlRenderer.render(&schema_graph);

    let app =
        Router::new().route("/", get(move || async { Html(html_content) }));

    let addr = std::net::SocketAddr::from(([127, 0, 0, 1], port));
    println!("🚀 SchemaGit Viewer running at http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn render_text_legacy(schema: &schemagit_core::DatabaseSchema) -> String {
    use colored::Colorize;
    use std::collections::{HashMap, HashSet};

    let mut output = String::new();
    output.push_str(&format!(
        "{}\n\n",
        "=== Schema Relationship Graph ===".bold().cyan()
    ));

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();
    let mut all_tables = HashSet::new();
    let mut referenced_tables = HashSet::new();

    for table in &schema.tables {
        all_tables.insert(table.name.clone());
        for fk in &table.foreign_keys {
            adjacency
                .entry(table.name.clone())
                .or_default()
                .push(fk.ref_table.clone());
            referenced_tables.insert(fk.ref_table.clone());
        }
    }

    let mut root_tables: Vec<String> =
        all_tables.difference(&referenced_tables).cloned().collect();

    root_tables.sort();

    if root_tables.is_empty() {
        root_tables = all_tables.iter().cloned().collect();
        root_tables.sort();
    }

    let mut visited = HashSet::new();

    for root in &root_tables {
        push_tree(root, &adjacency, &mut visited, 0, &mut output);
    }

    output
}

fn push_tree(
    table: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    depth: usize,
    output: &mut String,
) {
    use colored::Colorize;
    if visited.contains(table) {
        return;
    }

    visited.insert(table.to_string());

    let indent = "  ".repeat(depth);
    let prefix = if depth > 0 { "└── " } else { "" };

    output.push_str(&format!("{}{}{}\n", indent, prefix, table.green()));

    if let Some(children) = adjacency.get(table) {
        let mut sorted_children = children.clone();
        sorted_children.sort();

        for child in sorted_children {
            push_tree(&child, adjacency, visited, depth + 1, output);
        }
    }
}
