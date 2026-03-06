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

        /// Output file for the migration (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
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

        Commands::Diff { old, new, format } => {
            commands::diff::execute(&old, &new, &format)
        }

        Commands::Migrate { old, new, output } => {
            commands::migrate::execute(&old, &new, output.as_deref())
        }

        Commands::Status {
            driver,
            connection,
            snapshots,
        } => commands::status::execute(driver, &connection, &snapshots).await,

        Commands::List { directory } => commands::list::execute(&directory),

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
        } => commands::graph::execute(&snapshot, &directory, &format),

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
