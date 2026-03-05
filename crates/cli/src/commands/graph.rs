use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;
use std::collections::{HashMap, HashSet};

/// Execute the graph command.
pub fn execute(snapshot_id: &str, directory: &str, format: &str) -> Result<()> {
    let manager = SnapshotManager::new(directory);

    // Load snapshot
    let snapshot = if snapshot_id.ends_with(".snapshot.json") {
        manager.load(snapshot_id)?
    } else if snapshot_id == "latest" {
        manager
            .latest()
            .context("Failed to load latest snapshot")?
            .ok_or_else(|| {
                anyhow::anyhow!("No snapshots found in {}", directory)
            })?
    } else {
        let filename = if snapshot_id.len() == 14 {
            format!(
                "{}_{}_{}_{}.snapshot.json",
                &snapshot_id[0..4],
                &snapshot_id[4..6],
                &snapshot_id[6..8],
                &snapshot_id[8..14]
            )
        } else {
            format!("{}.snapshot.json", snapshot_id)
        };
        manager.load(&filename)?
    };

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

    match format.to_lowercase().as_str() {
        "text" => render_text(&all_tables, &relationships),
        "mermaid" => render_mermaid(&all_tables, &relationships),
        "dot" => render_dot(&relationships),
        _ => {
            println!(
                "{}",
                format!(
                    "Unknown format: {}. Use text, mermaid, or dot",
                    format
                )
                .red()
            );
        }
    }

    Ok(())
}

/// Render graph as text tree.
fn render_text(
    all_tables: &HashSet<String>,
    relationships: &HashSet<(String, String, String, String)>,
) {
    println!("{}", "=== Schema Relationship Graph ===".bold().cyan());
    println!();

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
        print_tree(root, &adjacency, &mut visited, 0);
    }

    let remaining: Vec<String> =
        all_tables.difference(&visited).cloned().collect();

    if !remaining.is_empty() {
        println!();
        println!("{}", "Circular dependencies or isolated tables:".yellow());
        for table in remaining {
            println!("  {}", table.cyan());
        }
    }
}

/// Recursive tree printer.
fn print_tree(
    table: &str,
    adjacency: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    depth: usize,
) {
    if visited.contains(table) {
        return;
    }

    visited.insert(table.to_string());

    let indent = "  ".repeat(depth);
    let prefix = if depth > 0 { "└── " } else { "" };

    println!("{}{}{}", indent, prefix, table.green());

    if let Some(children) = adjacency.get(table) {
        for child in children {
            print_tree(child, adjacency, visited, depth + 1);
        }
    }
}

/// Render Mermaid ER diagram.
fn render_mermaid(
    all_tables: &HashSet<String>,
    relationships: &HashSet<(String, String, String, String)>,
) {
    println!("erDiagram");

    for table in all_tables {
        println!("    {}", table);
    }

    for (from, to, column, ref_column) in relationships {
        println!(
            "    {} ||--o{{ {} : \"{} to {}\"",
            to, from, ref_column, column
        );
    }
}

/// Render Graphviz DOT format.
fn render_dot(relationships: &HashSet<(String, String, String, String)>) {
    println!("digraph schema {{");
    println!("    rankdir=LR;");
    println!("    node [shape=box];");
    println!();

    for (from, to, column, ref_column) in relationships {
        println!(
            "    \"{}\" -> \"{}\" [label=\"{} to {}\"];",
            from, to, column, ref_column
        );
    }

    println!("}}");
}
