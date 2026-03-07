use anyhow::Result;
use std::fs;
use std::path::Path;

#[allow(dead_code)]
pub struct OutputOptions {
    pub format: String,
    pub output_path: Option<String>,
    pub pretty: bool,
}

/// Unified output writer for the CLI.
/// Handles writing to stdout or a file, and ensures parent directories exist.
pub fn write_output(content: &str, options: &OutputOptions) -> Result<()> {
    let content = if options.pretty {
        content.to_string()
    } else {
        content.replace('\n', "")
    };

    match &options.output_path {
        Some(path_str) => {
            let path = Path::new(path_str);

            if let Some(parent) = path.parent()
                && !parent.exists()
                && !parent.as_os_str().is_empty()
            {
                fs::create_dir_all(parent)?;
            }

            fs::write(path, content)?;
        }
        None => {
            println!("{}", content);
        }
    }

    Ok(())
}
