mod commands;

use anyhow::Result;
use clap::{Parser, Subcommand};

const DEFAULT_SNAPSHOTS_DIR: &str = "./snapshots";
const DEFAULT_DIFF_FORMAT: &str = "text";
const DEFAULT_GRAPH_FORMAT: &str = "text";
const DEFAULT_EXPORT_FORMAT: &str = "sql";

#[derive(Parser)]
#[command(name = "schemagit")]
#[command(about = "Database schema versioning and inspection CLI", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Take a snapshot of the current database schema
    Snapshot {
        /// Database driver (postgres, mysql, sqlite, mssql). If not specified, auto-detected from connection string.
        #[arg(short, long)]
        driver: Option<String>,

        /// Database connection string
        #[arg(short, long)]
        connection: String,

        /// Directory to store snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        output: String,
    },

    /// Compare two schema snapshots
    Diff {
        /// Path to the old snapshot file
        old: String,

        /// Path to the new snapshot file
        new: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short = 's', long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        snapshot_dir: String,

        /// Output format (text, json)
        #[arg(short, long, default_value = DEFAULT_DIFF_FORMAT)]
        format: String,
    },

    /// Generate a migration script from two snapshots
    Migrate {
        /// Path to the old snapshot file
        old: String,

        /// Path to the new snapshot file
        new: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short = 's', long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        snapshot_dir: String,

        /// Output file for the migration (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Automatically create missing output directory
        #[arg(long, conflicts_with = "no_create_dir")]
        yes: bool,

        /// Do not create missing output directory
        #[arg(long = "no-create-dir", conflicts_with = "yes")]
        no_create_dir: bool,
    },

    /// Check database drift against the latest snapshot
    Status {
        /// Database driver (postgres, mysql, sqlite, mssql). If not specified, auto-detected from connection string.
        #[arg(short, long)]
        driver: Option<String>,

        /// Database connection string
        #[arg(short, long)]
        connection: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short = 's', long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        snapshots: String,
    },

    /// List all snapshots
    List {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// List snapshot IDs in timestamp order
    Snapshots {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// Show snapshot history timeline
    History {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// Show detailed information about a snapshot
    Show {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// Display schema summary statistics
    Summary {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// Visualize schema relationships as a graph
    Graph {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Output format (text, mermaid, dot)
        #[arg(short, long, default_value = DEFAULT_GRAPH_FORMAT)]
        format: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,

        /// Automatically create missing output directory
        #[arg(long, conflicts_with = "no_create_dir")]
        yes: bool,

        /// Do not create missing output directory
        #[arg(long = "no-create-dir", conflicts_with = "yes")]
        no_create_dir: bool,
    },

    /// Export snapshot to various formats
    Export {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short = 'd', long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Export format (sql, json, yaml)
        #[arg(short, long, default_value = DEFAULT_EXPORT_FORMAT)]
        format: String,
    },

    /// Validate schema for common issues
    Validate {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },

    /// Tag a snapshot with a meaningful name
    Tag {
        /// Snapshot filename or ID
        snapshot: String,

        /// Tag name
        tag: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Snapshot {
            driver,
            connection,
            output,
        } => commands::snapshot::execute(driver, &connection, &output).await,

        Commands::Diff {
            old,
            new,
            snapshot_dir,
            format,
        } => commands::diff::execute(&old, &new, &snapshot_dir, &format),

        Commands::Migrate {
            old,
            new,
            snapshot_dir,
            output,
            yes,
            no_create_dir,
        } => commands::migrate::execute(
            &old,
            &new,
            &snapshot_dir,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Status {
            driver,
            connection,
            snapshots,
        } => commands::status::execute(driver, &connection, &snapshots).await,

        Commands::List { directory } => commands::list::execute(&directory),

        Commands::Snapshots { directory } => {
            commands::snapshots::execute(&directory)
        }

        Commands::History { directory } => {
            commands::history::execute(&directory)
        }

        Commands::Show {
            snapshot,
            directory,
        } => commands::show::execute(&snapshot, &directory),

        Commands::Summary {
            snapshot,
            directory,
        } => commands::summary::execute(&snapshot, &directory),

        Commands::Graph {
            snapshot,
            directory,
            format,
            output,
            yes,
            no_create_dir,
        } => commands::graph::execute(
            &snapshot,
            &directory,
            &format,
            output.as_deref(),
            yes,
            no_create_dir,
        ),

        Commands::Export {
            snapshot,
            directory,
            format,
        } => commands::export::execute(&snapshot, &directory, &format),

        Commands::Validate {
            snapshot,
            directory,
        } => commands::validate::execute(&snapshot, &directory),

        Commands::Tag {
            snapshot,
            tag,
            directory,
        } => commands::tag::execute(&snapshot, &tag, &directory),
    }
}
