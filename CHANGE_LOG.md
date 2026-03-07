# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.3.0] - 2026-03-06

### Added
- **Interactive Schema Viewer**: Added `graph serve` command powered by Axum to provide an interactive, web-based view of your schema relationships.
- **Static Documentation Generator**: New `docs generate` command to create standalone HTML documentation files for snapshots.
- **Enhanced Visualization**: Added HTML/Json renderers for schema graphs and improved Mermaid/DOT output.
- **Schema Evolution Timeline**: New top-level `timeline` command to visualize how a database structure has evolved across multiple snapshots.
- **Generalized Graph Model**: Refactored `schemagit-core` with a unified `SchemaGraph` model that serves as a shared foundation for all visualization and documentation tools.

### Changed
- Refactored CLI: Split command arguments into a separate `args.rs` module to improve maintainability and strictly follow Rust project size guidelines.
- Simplified Graph CLI: Replaced top-level `graph` with subcommands `render` and `serve`.

### Fixed
- Resolved compilation errors related to type inference in snapshot processing.
- Fixed `cargo test` suite to ensure all 26+ integration and unit tests pass across the workspace.

## [0.2.3] - 2026-03-06

### Added

- FEAT: Improve CLI output, snapshot validation, and documentation ([#8](https://github.com/vdhuyme/schemagit/pull/8))
- Standardized output writing across output-producing commands with shared `-o/--output` behavior and directory creation controls (`--yes`, `--no-create-dir`)
- Release helper script: `scripts/release.sh` for workspace version bump, commit, and tag flow

### Changed

- Removed custom `schemagit version` subcommand; CLI now reports version from Cargo metadata via `schemagit --version`
- Migrated crate versioning to workspace-level metadata (`[workspace.package]`) so all crates share one source of truth

### Refactored

- Refactor: Apply Rust Coding Conventions and Strict Code Quality Rules ([#7](https://github.com/vdhuyme/schemagit/pull/7))

### Documentation

- Updated README release/versioning instructions to use `cargo-edit` (`cargo set-version --workspace --bump <patch|minor|major>`)

## [0.2.1] - 2026-03-05

### Added

- `snapshots` command to list available snapshot IDs sorted by timestamp
- `previous` snapshot alias across snapshot-loading commands
- `--snapshot-dir` option for `diff` and `migrate`
- `-o/--output` option for `graph` command
- Snapshot metadata fields: `database_name` and `snapshot_version`
- `--yes` and `--no-create-dir` flags for output-writing commands (`graph`, `migrate`)

### Improved

- Consistent snapshot resolution for full path, filename, timestamp ID, `latest`, and `previous`
- Snapshot loading now validates deserialized structure and reports clear invalid-file errors
- Diff output ordering is deterministic to keep generated migration SQL stable
- Diff detection now treats changed index and foreign key definitions as remove+add operations
- Migration generation now verifies table/column/foreign key/index references before output
- Output file UX now supports interactive directory creation prompts and non-interactive-safe failures

### Fixed

- Graph DOT output now includes isolated tables (tables without relationships)
- Replaced production `unwrap()` calls in introspector connection getters with explicit errors

## [0.2.0] - 2026-03-05

### Added

- **History Command**: Display timeline view of all snapshots with metadata
- **Show Command**: Inspect detailed snapshot content including tables, columns, indexes, and foreign keys
- **Summary Command**: Display statistical overview of schema with column counts and type distribution
- **Graph Command**: Visualize table relationships based on foreign keys
  - Support for text (ASCII tree) format
  - Support for Mermaid ER diagram format
  - Support for Graphviz DOT format
- **Export Command**: Export snapshots to various formats
  - SQL CREATE statements export
  - JSON format export
  - YAML format export
- **Validate Command**: Check schema for common issues
  - Detect duplicate table names
  - Detect duplicate column names
  - Detect missing primary keys
  - Validate foreign key references
  - Validate index references
- **Tag Command**: Label snapshots with meaningful names for easier reference
  - Tag storage in tags.json file
  - Support for tagging by snapshot ID, filename, or "latest"
- **Driver Auto-detection**: Automatically detect database driver from connection string
  - Support for PostgreSQL (postgresql:// or postgres://)
  - Support for MySQL (mysql://)
  - Support for SQLite (sqlite:// or file:)
  - Support for MS SQL Server (mssql:// or sqlserver://)

### Changed

- Made `--driver` flag optional for `snapshot` and `status` commands
- Improved graph visualization to handle closure tables and complex relationships
- Enhanced error messages with colored output

### Fixed

- Fixed clippy warnings related to wildcard pattern matching
- Improved snapshot ID resolution logic across all commands

## [0.1.0] - 2026-03-01

### Added

- **Snapshot Command**: Capture database schema at any point in time
- **Diff Command**: Compare two schema snapshots and display differences
  - Support for text format output
  - Support for JSON format output
- **Migrate Command**: Generate SQL migration scripts from schema diffs
- **Status Command**: Check if database has drifted from latest snapshot
- **List Command**: Display all available snapshots
- PostgreSQL introspection support
  - Tables, columns, indexes, foreign keys
  - Data types, nullability, defaults
- PostgreSQL migration generation
  - CREATE/DROP TABLE statements
  - ALTER TABLE for modifications
  - CREATE/DROP INDEX statements
  - ADD/DROP CONSTRAINT for foreign keys

### Core Features

- Modular crate architecture:
  - `schemagit-core`: Core data structures
  - `schemagit-introspector`: Database introspection
  - `schemagit-snapshot`: Snapshot management
  - `schemagit-diff`: Schema comparison
  - `schemagit-migration`: Migration generation
  - `schemagit-cli`: Command-line interface
- Snapshot storage in JSON format with timestamps
- Colored terminal output for better readability
- Comprehensive error handling with anyhow

[Unreleased]: https://github.com/vdhuyme/schemagit/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/vdhuyme/schemagit/compare/v0.2.3...v0.3.0
[0.2.3]: https://github.com/vdhuyme/schemagit/compare/v0.2.2...v0.2.3
[0.2.1]: https://github.com/vdhuyme/schemagit/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/vdhuyme/schemagit/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/vdhuyme/schemagit/releases/tag/v0.1.0
