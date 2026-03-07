use super::utils;
use crate::output::{self, OutputOptions};
use anyhow::Result;
use schemagit_core::DocsGenerator;
use schemagit_snapshot::SnapshotManager;

pub fn generate(
    snapshot_id: &str,
    directory: &str,
    output_path: &str,
) -> Result<()> {
    let manager = SnapshotManager::new(directory);
    let snapshot = utils::resolve_snapshot(&manager, snapshot_id, directory)?;

    let docs_content = DocsGenerator::generate(&snapshot.schema);

    let options = OutputOptions {
        format: "html".to_string(),
        output_path: Some(output_path.to_string()),
        pretty: true,
    };

    output::write_output(&docs_content, &options)?;

    println!("✓ Documentation generated: {}", output_path);

    Ok(())
}
