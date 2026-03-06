use clap::{Parser, Subcommand};

pub const DEFAULT_SNAPSHOTS_DIR: &str = "./snapshots";
pub const DEFAULT_DIFF_FORMAT: &str = "text";
pub const DEFAULT_GRAPH_FORMAT: &str = "text";
pub const DEFAULT_EXPORT_FORMAT: &str = "sql";

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Parser)]
#[command(name = "schemagit")]
#[command(version = VERSION)]
#[command(about = "Database schema versioning and inspection CLI", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
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

    /// List all snapshots
    List {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// List snapshot IDs in timestamp order
    Snapshots {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// Show snapshot history timeline
    History {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// Show detailed information about a snapshot
    Show {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// Display schema summary statistics
    Summary {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// Visualize schema relationships as a graph
    Graph {
        #[command(subcommand)]
        subcommand: GraphSubcommand,
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

    /// Validate schema for common issues
    Validate {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

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

    /// Generate schema documentation
    Docs {
        #[command(subcommand)]
        subcommand: DocsSubcommand,
    },

    /// Visualize schema evolution across snapshots
    Timeline {
        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Output format (text, json, html)
        #[arg(short, long, default_value = "text")]
        format: String,

        /// Output file (default: stdout)
        #[arg(short, long)]
        output: Option<String>,
    },
}

#[derive(Subcommand)]
pub enum GraphSubcommand {
    /// Render graph in various formats
    Render {
        /// Snapshot filename or ID
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Output format (text, mermaid, dot, html, json)
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

    /// Launch interactive schema viewer server
    Serve {
        /// Snapshot filename or ID (default: latest)
        #[arg(default_value = "latest")]
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Port to listen on (default: 7420)
        #[arg(short, long, default_value_t = 7420)]
        port: u16,
    },
}

#[derive(Subcommand)]
pub enum DocsSubcommand {
    /// Generate static schema documentation
    Generate {
        /// Snapshot filename or ID (default: latest)
        #[arg(default_value = "latest")]
        snapshot: String,

        /// Directory containing snapshots (default: ./snapshots)
        #[arg(short, long, default_value = DEFAULT_SNAPSHOTS_DIR)]
        directory: String,

        /// Output file (default: docs/schema.html)
        #[arg(short, long, default_value = "docs/schema.html")]
        output: String,
    },
}
