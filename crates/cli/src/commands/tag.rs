use anyhow::{Context, Result};
use colored::Colorize;
use schemagit_snapshot::SnapshotManager;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize)]
struct TagStorage {
    tags: HashMap<String, String>,
}

impl TagStorage {
    fn new() -> Self {
        Self {
            tags: HashMap::new(),
        }
    }

    fn load_from_file(path: &PathBuf) -> Result<Self> {
        if path.exists() {
            let content = fs::read_to_string(path)?;
            let storage: TagStorage = serde_json::from_str(&content)?;
            Ok(storage)
        } else {
            Ok(Self::new())
        }
    }

    fn save_to_file(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }
}

/// Execute the tag command.
pub fn execute(
    snapshot_id: &str,
    tag_name: &str,
    directory: &str,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let tags_file = PathBuf::from(directory).join("tags.json");

    // Load or create tag storage
    let mut storage = TagStorage::load_from_file(&tags_file)
        .context("Failed to load tags")?;

    // Resolve snapshot filename
    let snapshot_filename = match snapshot_id {
        id if id.ends_with(".snapshot.json") => id.to_string(),

        "latest" => {
            let snapshots =
                manager.list().context("Failed to list snapshots")?;
            snapshots
                .last()
                .ok_or_else(|| anyhow::anyhow!("No snapshots found"))?
                .clone()
        }

        id if id.len() == 14 => format!(
            "{}_{}_{}_{}.snapshot.json",
            &id[0..4],
            &id[4..6],
            &id[6..8],
            &id[8..14]
        ),

        id => format!("{}.snapshot.json", id),
    };

    // Verify snapshot exists
    manager
        .load(&snapshot_filename)
        .context("Snapshot not found")?;

    // Check if tag already exists
    if let Some(existing) = storage.tags.get(tag_name) {
        println!(
            "{}",
            format!(
                "Warning: Tag '{}' already points to '{}'",
                tag_name, existing
            )
            .yellow()
        );
        println!("Updating to point to '{}'", snapshot_filename);
    }

    // Add or update tag
    storage
        .tags
        .insert(tag_name.to_string(), snapshot_filename.clone());

    // Save tags
    storage
        .save_to_file(&tags_file)
        .context("Failed to save tags")?;

    println!(
        "{}",
        format!("✓ Tagged '{}' as '{}'", snapshot_filename, tag_name).green()
    );

    Ok(())
}
