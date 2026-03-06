use anyhow::{Context, Result};
use colored::Colorize;
use std::fs;

use super::utils;

pub fn write_or_stdout(
    content: &str,
    output_path: Option<&str>,
    yes: bool,
    no_create_dir: bool,
    content_name: &str,
) -> Result<()> {
    match output_path {
        Some(path) => {
            write_file(content, path, yes, no_create_dir, content_name)
        }
        None => {
            print!("{}", content);
            Ok(())
        }
    }
}

fn write_file(
    content: &str,
    output_path: &str,
    yes: bool,
    no_create_dir: bool,
    content_name: &str,
) -> Result<()> {
    utils::prepare_output_path(output_path, yes, no_create_dir)?;
    fs::write(output_path, content)
        .with_context(|| format!("Failed to write {} file", content_name))?;

    println!(
        "{}",
        format!("✓ {} saved: {}", content_name, output_path).green()
    );

    Ok(())
}
