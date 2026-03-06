# Schemagit

Database schema versioning and inspection CLI.
Track schema changes with a Git-like workflow.

---

## Table Of Contents

- [Highlights](#highlights)
- [Current Support](#current-support)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Snapshot Input Resolution](#snapshot-input-resolution)
- [Output File Behavior](#output-file-behavior)
- [Examples](#examples)
- [Output Formats](#output-formats)
- [Architecture](#architecture)
- [Contributing](#contributing)
- [Roadmap](#roadmap)
- [License](#license)

---

## Highlights

- Snapshot live database schema into JSON files.
- Compare snapshots with deterministic diff output.
- Generate SQL migrations from schema diffs.
- Validate schema integrity and common modeling issues.
- Render relationship graphs in `text`, `mermaid`, and `dot`.
- Resolve snapshot references consistently:
  - file path
  - filename
  - timestamp ID
  - `latest`
  - `previous`
- Safe output writing with automation flags for CI:
  - `--yes`
  - `--no-create-dir`

---

## Current Support

| Database             | Status          | Driver     |
| -------------------- | --------------- | ---------- |
| PostgreSQL           | Fully supported | `postgres` |
| Microsoft SQL Server | Fully supported | `mssql`    |
| MySQL                | Planned         | `mysql`    |
| SQLite               | Planned         | `sqlite`   |

Notes:

- MSSQL introspection and migration generation are fully supported.
- MySQL/SQLite command-level flags exist, but introspection/migration are not yet complete.

---

## Installation

### Requirements

- Rust 1.70+
- Cargo

### Build

```bash
git clone https://github.com/vdhuyme/schemagit.git
cd schemagit
cargo build --release
```

Binary output:

- Windows: `target/release/schemagit.exe`
- Linux/macOS: `target/release/schemagit`

---

## Quick Start

### 1. Create a snapshot

```bash
# PostgreSQL (auto-detect driver)
schemagit snapshot -c "postgresql://user:password@localhost:5432/mydb"

# MSSQL (explicit driver)
schemagit snapshot -d mssql -c "mssql://sa:password@localhost:1433/mydb"
```

### 2. Apply schema changes in DB

```sql
ALTER TABLE users ADD email NVARCHAR(255);
CREATE INDEX idx_users_email ON users(email);
```

### 3. Create another snapshot

```bash
schemagit snapshot -d mssql -c "mssql://sa:password@localhost:1433/mydb"
```

### 4. Diff

```bash
schemagit diff previous latest
```

### 5. Generate migration

```bash
schemagit migrate previous latest -o ./migrations/001_init.sql --yes
```

---

## Commands

### Snapshot Management

```bash
# Capture schema snapshot
schemagit snapshot -c <connection-string> [-d <driver>] [-o <output-dir>]

# Detailed list with metadata
schemagit list [-d <directory>]

# IDs only (timestamp list)
schemagit snapshots [-d <directory>]

# Timeline view
schemagit history [-d <directory>]

# Detailed snapshot content
schemagit show <snapshot> [-d <directory>]

# Schema summary stats
schemagit summary <snapshot> [-d <directory>]

# Validate schema issues
schemagit validate <snapshot> [-d <directory>]

# Tag snapshot
schemagit tag <snapshot> <tag-name> [-d <directory>]
```

### Comparison And Migration

```bash
# Diff snapshots
schemagit diff <old> <new> [--snapshot-dir <dir>] [--format text|json] [-o <output-file>] [--yes|--no-create-dir]

# Generate migration SQL (stdout by default)
schemagit migrate <old> <new> [--snapshot-dir <dir>] [-o <output-file>] [--yes|--no-create-dir]

# Drift check against latest snapshot
schemagit status -c <connection-string> [-d <driver>] [-s <snapshots-dir>] [-o <output-file>] [--yes|--no-create-dir]
```

### Visualization And Export

```bash
# Graph output (stdout by default)
schemagit graph <snapshot> [--format text|mermaid|dot] [-d <directory>] [-o <output-file>] [--yes|--no-create-dir]

# Export snapshot
schemagit export <snapshot> --format <sql|json|yaml> [-d <directory>] [-o <output-file>] [--yes|--no-create-dir]

# Show CLI version from Cargo metadata
schemagit --version
```

---

## Snapshot Input Resolution

All snapshot-accepting commands resolve input consistently.

Accepted forms:

1. Full path

```bash
schemagit diff ./snapshots/2026_03_05_115408.snapshot.json latest
```

2. Filename

```bash
schemagit diff 2026_03_05_115408.snapshot.json latest
```

3. Timestamp ID

```bash
schemagit diff 2026_03_05_115408 latest
```

4. Compact timestamp ID

```bash
schemagit diff 20260305115408 latest
```

5. Aliases

```bash
schemagit diff latest previous
```

Custom snapshot directory:

```bash
schemagit diff latest previous --snapshot-dir ./db/snapshots
schemagit migrate latest previous --snapshot-dir ./db/snapshots -o ./migrations/001_init.sql
```

---

## Output File Behavior

Applies to output-producing commands (`diff`, `migrate`, `status`, `list`, `snapshots`, `history`, `show`, `summary`, `graph`, `export`, `validate`).

When output parent directory does not exist:

1. `--yes`

- Create directory automatically.
- No prompt.

2. `--no-create-dir`

- Do not create.
- Return clear error.

3. Default behavior

- Interactive terminal: ask for confirmation.
- Non-interactive (CI, redirected input): fail without prompt.

Examples:

```bash
# Auto-create output directory
schemagit graph latest --format mermaid -o docs/schema.mmd --yes

# Strict mode
schemagit migrate previous latest -o ./migrations/001_init.sql --no-create-dir
```

---

## Examples

### MSSQL Workflow

```bash
# Snapshot current schema
schemagit snapshot -d mssql -c "mssql://sa:password@localhost:1433/schemagit"

# Compare snapshots
schemagit diff previous latest --format text

# Generate migration script
schemagit migrate previous latest -o ./migrations/002_change.sql --yes

# Render Mermaid graph to file
schemagit graph latest --format mermaid -o ./docs/schema.mmd --yes
```

### CI Workflow

```bash
# Fail fast on drift
schemagit status -d mssql -c "$DATABASE_URL"

# Non-interactive output writing
schemagit migrate previous latest -o ./artifacts/migration.sql --yes
```

### Validation And Export

```bash
schemagit validate latest
schemagit validate latest -o ./reports/validation.txt --yes
schemagit export latest --format sql -o ./reports/schema.sql --yes
schemagit export latest --format json -o ./reports/schema.json --yes
```

### Release Versioning

```bash
# Install cargo-edit once
cargo install cargo-edit

# Bump workspace version
cargo set-version --workspace --bump patch
cargo set-version --workspace --bump minor
cargo set-version --workspace --bump major

# Run helper script (bump + commit + tag)
./scripts/release.sh patch
./scripts/release.sh minor
./scripts/release.sh major
```

Workspace versioning is centralized in root `Cargo.toml`:

```toml
[workspace.package]
version = "0.1.0"
edition = "2021"
```

All crates inherit from workspace metadata:

```toml
[package]
version.workspace = true
edition.workspace = true
```

---

## Output Formats

### Graph

- `text`: ASCII tree
- `mermaid`: Mermaid ER diagram
- `dot`: Graphviz DOT

### Export

- `sql`: DDL-style schema export
- `json`: full snapshot metadata + schema
- `yaml`: readable structured export

---

## Architecture

Workspace layout:

```text
schemagit/
  crates/
    cli/          # CLI entrypoints and command wiring
    core/         # Shared schema structures (table/column/index/fk)
    introspector/ # Database-specific introspection
    snapshot/     # Snapshot serialization, loading, validation
    diff/         # Schema diff engine
    migration/    # SQL migration generators
  snapshots/      # Default snapshot storage
```

Design principles:

- Modular crates with clear boundaries
- Strong typing and explicit error handling
- Deterministic output for reproducible automation
- Backward-compatible snapshot metadata evolution

---

## Contributing

```bash
cargo build
cargo fmt
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Please include tests and documentation updates in your pull request.

---

## Roadmap

### Near Term

- MySQL introspection and migration parity
- SQLite introspection and migration parity
- Snapshot compression (gzip)
- Performance tuning for very large schemas

### Long Term

- Interactive diff viewer
- Web visualization UI
- Plugin system for custom drivers
- Rollback and forward planning enhancements

---

## License

MIT. See `LICENSE` for details.

---
