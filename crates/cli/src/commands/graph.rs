use anyhow::Result;
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;
use std::collections::{HashMap, HashSet};

use super::{output, utils};

/// Execute the graph command.
pub fn execute(
    snapshot_id: &str,
    directory: &str,
    format: &str,
    output_file: Option<&str>,
    yes: bool,
    no_create_dir: bool,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

    let mut relationships: HashSet<(String, String, String, String)> =
        HashSet::new();
    let mut all_tables: HashSet<String> = HashSet::new();

    for table in &snapshot.schema.tables {
        all_tables.insert(table.name.clone());

        for fk in &table.foreign_keys {
            relationships.insert((
                table.name.clone(),
                fk.ref_table.clone(),
                fk.column.clone(),
                fk.ref_column.clone(),
            ));
        }
    }

    verify_graph_relationships(&all_tables, &snapshot.schema.tables)?;

    let graph = match format.to_lowercase().as_str() {
        "text" => render_text(&all_tables, &relationships),
        "mermaid" => render_mermaid(&all_tables, &relationships),
        "dot" => render_dot(&all_tables, &relationships),
        _ => {
            return Err(anyhow::anyhow!(
                "Unknown format: {}. Use text, mermaid, or dot",
                format
            ));
        }
    };

    output::write_or_stdout(&graph, output_file, yes, no_create_dir, "Graph")?;

    Ok(())
}

/// Render graph as text tree.
fn render_text(
    all_tables: &HashSet<String>,
    relationships: &HashSet<(String, String, String, String)>,
) -> String {
    let mut output = String::new();
    output.push_str(&format!(
        "{}\n\n",
        "=== Schema Relationship Graph ===".bold().cyan()
    ));

    let mut adjacency: HashMap<String, Vec<String>> = HashMap::new();

    for (from, to, _, _) in relationships {
        adjacency.entry(from.clone()).or_default().push(to.clone());
    }

    let referenced_tables: HashSet<String> = relationships
        .iter()
        .map(|(_, to, _, _)| to.clone())
        .collect();

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

    let mut remaining: Vec<String> =
        all_tables.difference(&visited).cloned().collect();
    remaining.sort();

    if !remaining.is_empty() {
        output.push('\n');
        output.push_str(&format!(
            "{}\n",
            "Circular dependencies or isolated tables:".yellow()
        ));
        for table in remaining {
            output.push_str(&format!("  {}\n", table.cyan()));
        }
    }

    output
}

/// Recursive tree printer.
fn push_tree(
    table: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    depth: usize,
    output: &mut String,
) {
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

/// Render Mermaid ER diagram.
fn render_mermaid(
    all_tables: &HashSet<String>,
    relationships: &HashSet<(String, String, String, String)>,
) -> String {
    let mut output = String::from("erDiagram\n");

    let mut tables: Vec<String> = all_tables.iter().cloned().collect();
    tables.sort();
    for table in tables {
        output.push_str(&format!("    {}\n", table));
    }

    let mut sorted_relationships: Vec<_> =
        relationships.iter().cloned().collect();
    sorted_relationships.sort();
    for (from, to, column, ref_column) in sorted_relationships {
        output.push_str(&format!(
            "    {} ||--o{{ {} : \"{} to {}\"",
            to, from, ref_column, column
        ));
        output.push('\n');
    }

    output
}

/// Render Graphviz DOT format.
fn render_dot(
    all_tables: &HashSet<String>,
    relationships: &HashSet<(String, String, String, String)>,
) -> String {
    let mut output = String::from("digraph schema {\n");
    output.push_str("    rankdir=LR;\n");
    output.push_str("    node [shape=box];\n\n");

    let mut tables: Vec<String> = all_tables.iter().cloned().collect();
    tables.sort();
    for table in tables {
        output.push_str(&format!("    \"{}\";\n", table));
    }
    output.push('\n');

    let mut sorted_relationships: Vec<_> =
        relationships.iter().cloned().collect();
    sorted_relationships.sort();
    for (from, to, column, ref_column) in sorted_relationships {
        output.push_str(&format!(
            "    \"{}\" -> \"{}\" [label=\"{} to {}\"];",
            from, to, column, ref_column
        ));
        output.push('\n');
    }

    output.push_str("}\n");
    output
}

fn verify_graph_relationships(
    all_tables: &HashSet<String>,
    tables: &[schemagit_core::Table],
) -> Result<()> {
    for table in tables {
        for fk in &table.foreign_keys {
            if !all_tables.contains(&fk.ref_table) {
                return Err(anyhow::anyhow!(
                    "Graph generation error:\nReferenced table \"{}\" not found.",
                    fk.ref_table
                ));
            }

            if !table.columns.iter().any(|column| column.name == fk.column) {
                return Err(anyhow::anyhow!(
                    "Graph generation error:\nReferenced column \"{}.{}\" not found.",
                    table.name,
                    fk.column
                ));
            }
        }
    }

    Ok(())
}
