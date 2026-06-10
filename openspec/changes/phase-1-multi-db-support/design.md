## Context

The current `DbClient` in `src/db/client.rs` is tightly coupled to PostgreSQL via a concrete `sqlx::PgPool`. The product spec mandates PostgreSQL, MySQL/MariaDB, and SQLite support for MVP. Adding backends requires refactoring to a trait/enum-based abstraction while preserving the TEA pattern (message-passing, async isolation).

The `sqlx` crate already supports all three engines via feature flags. The challenge is creating a clean abstraction that minimizes duplicate code while respecting engine-specific catalog queries.

## Goals / Non-Goals

**Goals:**
- Single `DbClient` event loop handles commands for any backend via `enum DbBackend`
- Each backend implements catalog introspection returning the same `SchemaNode` tree
- Connect dialog UI adapts to the selected engine (SQLite hides host/port fields)
- TOML config file for connection profiles and keybinding overrides
- Zero compile-time cost for unused backends (feature-gated, but all three enabled for MVP)

**Non-Goals:**
- Oracle, SQL Server, or other engines (out of scope for MVP)
- Connection pooling across multiple simultaneous databases (single-connection MVP)
- ORM-like query builder or migration tooling

## Decisions

### ADR-001: Enum-based Backend Dispatch
**Choice:** `enum DbBackend { Pg(PgPool), Mysql(MySqlPool), Sqlite(SqlitePool) }` with match-internal dispatch.

**Rationale:** The `DbClient` owns a single active backend. An enum avoids dynamic dispatch overhead (`dyn DbBackend`), keeps the pool in the same allocation, and makes it clear at compile time which branches are taken. Each match arm delegates to a private async method (e.g., `handle_pg_load_schema`, `handle_mysql_load_schema`).

**Alternative considered:** `trait DbBackend` with `Box<dyn DbBackend>` — adds vtable overhead, makes the code harder to follow, and doesn't add value when there's exactly one active backend at a time.

### ADR-002: Catalog Query Per Engine
**Choice:** Three separate `async fn load_<engine>_schema(&self) -> Vec<SchemaNode>` functions, one per backend, each with engine-specific SQL.

**Rationale:** The system catalogs are fundamentally different:
- PostgreSQL: `pg_namespace`, `pg_class`, `pg_attribute`
- MySQL: `information_schema.SCHEMATA`, `TABLES`, `COLUMNS`
- SQLite: `sqlite_master` + `PRAGMA table_info`

A single abstraction would require an ORM-like SQL dialect translation layer — that's a separate product. Direct queries are simpler, faster, and easier to debug.

### ADR-003: TOML Config via Serde
**Choice:** Use `serde` + `toml` crate with derive macros for `struct Config { connections: Vec<ConnectionProfile>, keybindings: HashMap<String, String> }`.

**Rationale:** TOML is the Rust ecosystem standard. Serde derives are zero-cost at runtime. The config file is small (<100 lines for typical use), so parsing performance is irrelevant.

### ADR-004: Config File Location
**Choice:** `~/.config/twisterDBA/config.toml` (XDG-compliant via `dirs` crate).

**Rationale:** Follows XDG Base Directory spec, same convention as Neovim, Alacritty, Helix. The `dirs` crate handles platform differences (macOS `~/Library/Application Support`, Windows `%APPDATA%`).

### ADR-005: Feature Gates for Engines
**Choice:** All three sqlx features enabled by default in Cargo.toml.

**Rationale:** MVP needs all three. Feature gates would be premature optimization. If compile times become painful, users can disable unused backends via `--no-default-features`.

## Risks / Trade-offs

- **[Risk] sqlx MySQL driver is less mature than PostgreSQL** → Mitigation: basic queries (connect, schema introspection, query execution) are well-covered; edge cases like stored procedures deferred
- **[Risk] SQLite file locking can block the Tokio runtime** → Mitigation: `sqlx::sqlite` uses `libsqlite3-sys` with WAL mode default; no spawn_blocking needed for reads
- **[Risk] Compile time increases with three sqlx drivers** → Mitigation: acceptable for MVP (each driver compiles ~10s); optimize with feature gates post-MVP

## Migration Plan

1. Refactor `DbClient` to use `DbBackend` enum (existing PostgreSQL path continues working)
2. Add MySQL backend module
3. Add SQLite backend module
4. Update ConnectDialog UI
5. Add config module
6. No database migration needed (new features, not schema changes)

## Open Questions

- Should MySQL connections default to port 3306 or require explicit port? (Answer: default to 3306, same as PostgreSQL defaults to 5432)
- Should SQLite connections support `:memory:` for transient databases? (Answer: yes, for testing; add as explicit option in the dialog)
