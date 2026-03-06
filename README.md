# Schemagit

Git-like schema versioning for databases.

Schemagit allows you to snapshot database schemas, track structural changes, generate migrations, and visualize relationships using a deterministic CLI workflow.

---

## Highlights

- Snapshot live database schemas into versioned JSON files
- Compare snapshots with deterministic diff output
- Generate SQL migrations from schema changes
- Validate schema integrity and modeling issues
- Visualize table relationships
- Flexible snapshot reference resolution
- CI-friendly output behavior with automation flags

---

## Current Support

| Database             | Status          | Driver   |
| -------------------- | --------------- | -------- |
| PostgreSQL           | Fully supported | postgres |
| Microsoft SQL Server | Fully supported | mssql    |
| MySQL                | Planned         | mysql    |
| SQLite               | Planned         | sqlite   |

Notes:

- MSSQL introspection and migration generation are fully supported.
- MySQL and SQLite command flags exist but introspection and migration are not yet implemented.

---

## Installation

### Requirements

- Rust 1.70+
- Cargo

### Build From Source

```
git clone https://github.com/vdhuyme/schemagit.git
cd schemagit
cargo build --release
```

Binary location:

Windows

```
target/release/schemagit.exe
```

Linux / macOS

```
target/release/schemagit
```

---

## Quick Start

### 1. Create a schema snapshot

PostgreSQL

```
schemagit snapshot -c "postgresql://user:password@localhost:5432/mydb"
```

MSSQL

```
schemagit snapshot -d mssql -c "mssql://sa:password@localhost:1433/mydb"
```

---

### 2. Modify your database schema

Example:

```
ALTER TABLE users ADD email NVARCHAR(255);
CREATE INDEX idx_users_email ON users(email);
```

---

### 3. Capture another snapshot

```
schemagit snapshot -d mssql -c "mssql://sa:password@localhost:1433/mydb"
```

---

### 4. Compare snapshots

```
schemagit diff previous latest
```

Example output:

```
+ column users.email NVARCHAR(255)
+ index idx_users_email(users.email)
```

---

### 5. Generate migration SQL

```
schemagit migrate previous latest -o ./migrations/001_init.sql --yes
```

---

## Commands

### Snapshot Management

Capture schema snapshots

```
schemagit snapshot -c <connection-string> [-d <driver>] [-o <output-dir>]
```

List snapshots with metadata

```
schemagit list [-d <directory>]
```

List snapshot IDs only

```
schemagit snapshots [-d <directory>]
```

Show snapshot timeline

```
schemagit history [-d <directory>]
```

Inspect snapshot contents

```
schemagit show <snapshot> [-d <directory>]
```

Schema summary statistics

```
schemagit summary <snapshot> [-d <directory>]
```

Validate schema structure

```
schemagit validate <snapshot> [-d <directory>]
```

Tag a snapshot

```
schemagit tag <snapshot> <tag-name> [-d <directory>]
```

---

### Comparison And Migration

Compare snapshots

```
schemagit diff <old> <new> [--snapshot-dir <dir>] [--format text|json]
```

Generate migration SQL

```
schemagit migrate <old> <new> [--snapshot-dir <dir>] [-o <output-file>]
```

Check drift against the latest snapshot

```
schemagit status -c <connection-string> [-d <driver>] [-s <snapshots-dir>]
```

---

### Visualization And Export

Render schema graph

```
schemagit graph <snapshot> [--format text|mermaid|dot]
```

Export snapshot schema

```
schemagit export <snapshot> --format <sql|json|yaml>
```

Show CLI version

```
schemagit --version
```

---

## Snapshot Input Resolution

Commands that accept snapshot references support multiple forms.

Full file path

```
schemagit diff ./snapshots/2026_03_05_115408.snapshot.json latest
```

Filename

```
schemagit diff 2026_03_05_115408.snapshot.json latest
```

Timestamp ID

```
schemagit diff 2026_03_05_115408 latest
```

Compact timestamp

```
schemagit diff 20260305115408 latest
```

Aliases

```
schemagit diff latest previous
```

Custom snapshot directory

```
schemagit diff latest previous --snapshot-dir ./db/snapshots
```

---

## Output File Behavior

Commands that produce files support safe output behavior.

If the output directory does not exist:

`--yes`

Automatically create the directory.

`--no-create-dir`

Fail with an error.

Default behavior

- Interactive terminal: prompt user
- Non-interactive environment: fail

Example

```
schemagit graph latest --format mermaid -o docs/schema.mmd --yes
```

---

## Output Formats

Graph formats

- text
- mermaid
- dot

Export formats

- sql
- json
- yaml

---

## Architecture

Workspace layout

```
schemagit/
  crates/
    cli/          CLI entrypoints
    core/         shared schema structures
    introspector/ database introspection
    snapshot/     snapshot storage and validation
    diff/         schema diff engine
    migration/    migration SQL generation
  snapshots/      default snapshot directory
```

Design principles

- modular crate boundaries
- deterministic output
- explicit error handling
- reproducible automation
- backward-compatible snapshot evolution

---

## Contributing

Build the project

```
cargo build
```

Format code

```
cargo fmt
```

Run tests

```
cargo test
```

Run lint checks

```
cargo clippy --all-targets --all-features -- -D warnings
```

Pull requests should include tests and documentation updates where applicable.

---

## Roadmap

Near term

- MySQL introspection and migration support
- SQLite introspection and migration support
- snapshot compression
- performance improvements for large schemas

Long term

- interactive diff viewer
- web visualization UI
- plugin system for custom drivers
- advanced migration planning

---

## License

MIT License.

See LICENSE for details.
