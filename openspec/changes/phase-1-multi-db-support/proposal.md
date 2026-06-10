## Why

The product spec targets three database engines for MVP: PostgreSQL, MySQL/MariaDB, and SQLite. Currently only PostgreSQL is wired via `sqlx`. Backend/data engineers routinely work across all three engines, and a TUI that supports only one is a non-starter. Adding MySQL and SQLite unlocks the full MVP scope and validates the DB abstraction layer design.

## What Changes

- **BREAKING**: `src/db/client.rs` refactored from concrete PostgreSQL pool to a trait-based `DbBackend` enum
- New `src/db/mysql.rs` backend using `sqlx` MySQL driver
- New `src/db/sqlite.rs` backend using `sqlx` SQLite driver
- New `src/config/` module for TOML-based connection profile parsing
- `Cargo.toml` gains `sqlx` features for `mysql`, `sqlite`, `runtime-tokio`
- Connect dialog gains a "Database Type" dropdown (PostgreSQL / MySQL / SQLite)
- Connection DSN builder adapts per engine (postgresql:// vs mysql:// vs file: path)

## Capabilities

### New Capabilities
- `multi-db-support`: Connect to and query PostgreSQL, MySQL/MariaDB, and SQLite databases from the same TUI session
- `config-management`: Read connection profiles and user preferences from `~/.config/twisterDBA/config.toml`

### Modified Capabilities
- None (all net-new on top of existing PostgreSQL-only code)

## Impact

- `Cargo.toml`: sqlx features `mysql`, `sqlite` (compile time increases ~30s)
- `src/db/client.rs`: Replace concrete `PgPool` with `enum DbBackend { Pg(...), Mysql(...), Sqlite(...) }`
- `src/db/mysql.rs`, `src/db/sqlite.rs`: New modules
- `src/db/mod.rs`: Re-export backend types
- `src/config/`: New module tree for TOML parsing (serde)
- `src/state.rs`: `ConnectForm` gains `db_type` field; `ConnectionStatus` gains db type
- `src/events.rs`: `DbCommand::Connect` gains backend discriminator
- `src/ui.rs`: Connect dialog renders db type selector
