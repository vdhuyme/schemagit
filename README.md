# Schemagit

**Database schema versioning and inspection CLI tool**

Track database schema changes like you track code with Git.

---

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Command Reference](#command-reference)
- [Usage Examples](#usage-examples)
- [Snapshot ID Formats](#snapshot-id-formats)
- [Supported Databases](#supported-databases)
- [Output Formats](#output-formats)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [License](#license)

---

## Features

Schemagit is a powerful CLI tool for managing database schema versions, inspired by Git's workflow.

### Core Features

- **Snapshot** - Capture database schema at any point in time
- **Diff** - Compare schemas and see exactly what changed
- **Migrate** - Auto-generate SQL migration scripts from diffs
- **Status** - Check if your database has drifted from the latest snapshot
- **List** - View all your schema snapshots

### Advanced Features

- **History** - Timeline view of all snapshots
- **Show** - Detailed inspection of any snapshot
- **Summary** - Statistical overview of your schema
- **Graph** - Visualize table relationships (text/mermaid/dot)
- **Export** - Export snapshots to SQL, JSON, or YAML
- **Validate** - Detect schema problems and best practice violations
- **Tag** - Label snapshots with meaningful names
- **Auto-detect** - Automatically detect database driver from connection string

---

## Installation

### Prerequisites

- Rust 1.70 or later
- Cargo (comes with Rust)

### Build from Source

```bash
git clone https://github.com/yourusername/schemagit.git
cd schemagit
cargo build --release
```

The binary will be available at `target/release/schemagit` (or `schemagit.exe` on Windows).

### Add to PATH

**Linux/macOS:**

```bash
sudo cp target/release/schemagit /usr/local/bin/
```

**Windows:**
Copy `target/release/schemagit.exe` to a directory in your PATH.

---

## Quick Start

### 1. Take a snapshot

```bash
# Auto-detect driver from connection string
schemagit snapshot -c "postgresql://user:password@localhost/mydb"

# Or specify driver explicitly
schemagit snapshot -d postgres -c "postgresql://user:password@localhost/mydb"
```

### 2. Make changes to your database

```sql
-- Make some schema changes
ALTER TABLE users ADD COLUMN phone VARCHAR(20);
CREATE TABLE orders (...);
```

### 3. Take another snapshot

```bash
schemagit snapshot -c "postgresql://user:password@localhost/mydb"
```

### 4. Compare snapshots

```bash
schemagit diff old.snapshot.json new.snapshot.json
```

### 5. Generate migration

```bash
schemagit migrate old.snapshot.json new.snapshot.json -o migration.sql
```

---

## Command Reference

### Snapshot Management

```bash
# Take a snapshot
schemagit snapshot -c <connection-string> [-d <driver>] [-o <output-dir>]

# List all snapshots
schemagit list [-d <directory>]

# List snapshot IDs only
schemagit snapshots [-d <directory>]

# Show snapshot history
schemagit history [-d <directory>]

# Show detailed snapshot info
schemagit show <snapshot-id> [-d <directory>]

# Tag a snapshot
schemagit tag <snapshot-id> <tag-name> [-d <directory>]
```

### Schema Analysis

```bash
# Compare two snapshots
schemagit diff <old> <new> [--snapshot-dir <dir>] [--format text|json]

# Check database drift
schemagit status -c <connection-string> [-d <driver>] [-s <snapshots-dir>]

# Display schema statistics
schemagit summary <snapshot-id> [-d <directory>]

# Validate schema for issues
schemagit validate <snapshot-id> [-d <directory>]

# Visualize table relationships
schemagit graph <snapshot-id> --format <text|mermaid|dot> [-d <directory>] [-o <output-file>] [--yes|--no-create-dir]
```

### Migration

```bash
# Generate migration SQL
schemagit migrate <old> <new> [--snapshot-dir <dir>] [-o <output-file>] [--yes|--no-create-dir]
```

### Export

```bash
# Export to SQL
schemagit export <snapshot-id> --format sql [-d <directory>]

# Export to JSON
schemagit export <snapshot-id> --format json [-d <directory>]

# Export to YAML
schemagit export <snapshot-id> --format yaml [-d <directory>]
```

---

## Usage Examples

### Example 1: Production Deployment Workflow

```bash
# Before deployment: take snapshot and tag it
schemagit snapshot -c "postgres://prod-db/myapp"
schemagit tag latest pre-deploy-2026-03-05

# Make changes, take new snapshot
schemagit snapshot -c "postgres://prod-db/myapp"
schemagit tag latest post-deploy-2026-03-05

# Generate migration script
schemagit migrate pre-deploy-2026-03-05 post-deploy-2026-03-05 -o migration.sql

# Validate before applying
schemagit validate post-deploy-2026-03-05
```

### Example 2: Documentation Generation

```bash
# Generate SQL schema documentation
schemagit export latest --format sql > docs/schema.sql

# Create ER diagram (Mermaid)
schemagit graph latest --format mermaid -o docs/schema.mmd

# Non-interactive/CI-safe directory creation
schemagit graph latest --format mermaid -o docs/schema.mmd --yes

# Generate statistics
schemagit summary latest > docs/schema-stats.txt
```

### Example 3: Continuous Integration

```bash
# In CI pipeline: check for schema drift
schemagit status -c "$DATABASE_URL"

# If drift detected, fail the build
if [ $? -ne 0 ]; then
  echo "Schema drift detected!"
  exit 1
fi
```

### Example 4: Schema Validation

```bash
# Validate before deploying
schemagit validate latest

# Example output:
# Schema validation passed!
# No errors or warnings found.
```

### Example 5: Graph Visualization

**Text Format:**

```bash
schemagit graph latest --format text
```

Output:

```
=== Schema Relationship Graph ===

categories_closure
  └── categories
comments
  └── posts
    └── users
```

**Mermaid Format:**

```bash
schemagit graph latest --format mermaid
```

Output:

```mermaid
erDiagram
    users ||--o{ posts : references
    posts ||--o{ comments : references
```

**DOT Format:**

```bash
schemagit graph latest --format dot
```

Output:

```dot
digraph schema {
    rankdir=LR;
    node [shape=box];
    "posts" -> "users";
    "comments" -> "posts";
}
```

### Output Directory Behavior

When `-o/--output` is provided and the parent directory does not exist:

- Default (interactive terminal): prompt to create the directory
- Default (non-interactive, CI, or redirected input): return an error
- `--yes`: create directory automatically without prompting
- `--no-create-dir`: never create directory, always return an error

Examples:

```bash
schemagit graph latest --format mermaid -o docs/schema.mmd --yes
schemagit migrate latest previous -o migrations/001_init.sql --no-create-dir
```

---

## Snapshot ID Formats

Snapshots can be referenced in multiple ways:

| Format            | Example                                       | Description                 |
| ----------------- | --------------------------------------------- | --------------------------- |
| **latest**        | `latest`                                      | Most recent snapshot        |
| **previous**      | `previous`                                    | Snapshot before latest      |
| **Full filename** | `2026_03_05_072538.snapshot.json`             | Complete filename           |
| **Timestamp ID**  | `2026_03_05_072538`                           | Snapshot timestamp ID       |
| **Short ID**      | `20260305072538`                              | Timestamp-based ID          |
| **Full path**     | `./snapshots/2026_03_05_072538.snapshot.json` | Absolute/relative file path |

All snapshot-accepting commands resolve these formats consistently.

You can also override the snapshot directory for `diff` and `migrate`:

```bash
schemagit diff latest previous --snapshot-dir ./db/snapshots
schemagit migrate latest previous --snapshot-dir ./db/snapshots -o ./migrations/001_init.sql
```

---

## Supported Databases

| Database             | Status          | Driver Name |
| -------------------- | --------------- | ----------- |
| PostgreSQL           | Fully supported | `postgres`  |
| MySQL                | Planned         | `mysql`     |
| SQLite               | Planned         | `sqlite`    |
| Microsoft SQL Server | Fully supported | `mssql`     |

---

## Output Formats

### Graph Formats

**Text** - ASCII tree representation

```
users
 └── orders
      └── order_items
```

**Mermaid** - ER diagram syntax

```mermaid
erDiagram
  users ||--o{ orders : has
  orders ||--o{ order_items : contains
```

**DOT** - Graphviz format

```dot
digraph schema {
  users -> orders;
  orders -> order_items;
}
```

### Export Formats

- **SQL** - CREATE TABLE statements with indexes and foreign keys
- **JSON** - Complete snapshot with metadata
- **YAML** - Human-readable structured format

---

## Architecture

Schemagit is built as a modular Rust workspace with the following crates:

```
schemagit/
├── crates/
│   ├── cli/           # Command-line interface
│   ├── core/          # Core data structures (Table, Column, etc.)
│   ├── introspector/  # Database introspection (postgres, mysql, etc.)
│   ├── snapshot/      # Snapshot management and storage
│   ├── diff/          # Schema comparison logic
│   └── migration/     # SQL migration generation
└── snapshots/         # Default snapshot storage directory
```

### Design Principles

- **Modular**: Each crate has a single, well-defined responsibility
- **Type-safe**: Leverages Rust's type system for correctness
- **Extensible**: Easy to add support for new databases
- **Fast**: Compiled binary with minimal runtime overhead
- **Reliable**: Comprehensive error handling

---

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/yourusername/schemagit.git
cd schemagit

# Build in debug mode
cargo build

# Run tests
cargo test

# Run clippy for linting
cargo clippy --all-targets --all-features -- -D warnings

# Run with debug output
cargo run -- snapshot -c "postgresql://localhost/test"
```

### Code Guidelines

- Follow Rust standard style (use `rustfmt`)
- Write tests for new features
- Update documentation
- Ensure all tests pass before submitting PR
- Follow the existing code structure

### Reporting Issues

Please use GitHub Issues to report bugs or request features. Include:

- Steps to reproduce the issue
- Expected behavior
- Actual behavior
- Schemagit version
- Operating system and version

---

## Documentation

- [CHANGE_LOG.md](CHANGE_LOG.md) - Version history and changes
- [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) - Community guidelines
- [LICENSE](LICENSE) - MIT License

---

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

---

## Acknowledgments

- Inspired by Git's simple and powerful workflow
- Built with [Rust](https://www.rust-lang.org/) for performance and safety
- Uses [SQLx](https://github.com/launchbadge/sqlx) for database connectivity
- Uses [Clap](https://github.com/clap-rs/clap) for CLI parsing

---

## Roadmap

### Near-term

- Support for MySQL database introspection and migration
- Support for SQLite database introspection and migration
- Support for MS SQL Server database introspection and migration
- Snapshot compression (gzip)
- Performance optimizations for large schemas

### Long-term

- Interactive diff viewer
- Web UI for visualization
- Plugin system for custom drivers
- Schema rollback capabilities
- Multi-database comparison
- Schema diff visualization in CI/CD pipelines

---

**Made with Rust by the Schemagit team**

[Report Bug](https://github.com/yourusername/schemagit/issues) | [Request Feature](https://github.com/yourusername/schemagit/issues) | [Documentation](https://github.com/yourusername/schemagit/wiki)
