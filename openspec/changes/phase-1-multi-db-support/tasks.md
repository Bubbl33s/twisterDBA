## 1. Dependencies & Cargo Setup

- [x] 1.1 Add `mysql` and `sqlite` features to sqlx in `Cargo.toml`: `sqlx = { version = "0.9.0", features = ["postgres", "mysql", "sqlite", "runtime-tokio", "tls-rustls"] }`
- [x] 1.2 Add `toml = "0.8"` and `dirs = "5"` dependencies to `Cargo.toml`
- [x] 1.3 Run `cargo build` and verify all three sqlx drivers compile without errors

## 2. DB Backend Trait & Enum (`src/db/`)

- [x] 2.1 Define `DbBackend` enum in `src/db/backend.rs` with variants `Pg(PgPool)`, `Mysql(MySqlPool)`, `Sqlite(SqlitePool)` and a `Disconnected` variant
- [x] 2.2 Implement `DbBackend::connect(dsn, engine_type)` factory that returns the correct variant
- [x] 2.3 Implement `DbBackend::close()` that gracefully closes the pool for each variant
- [x] 2.4 Implement `DbBackend::is_connected()` -> bool helper
- [x] 2.5 Run `cargo build` and verify `src/db/` compiles

## 3. PostgreSQL Schema Refactor (`src/db/client.rs`)

- [x] 3.1 Extract `handle_load_schema` PostgreSQL logic into `src/db/pg_schema.rs::load_pg_schema(pool) -> Vec<SchemaNode>`
- [x] 3.2 Extract `handle_load_columns` PostgreSQL logic into `src/db/pg_schema.rs::load_pg_columns(pool, schema, table) -> Vec<ColumnInfo>`
- [x] 3.3 Extract `handle_execute_query` PostgreSQL logic into `src/db/pg_exec.rs::execute_pg_query(pool, sql, cancel, tx)`
- [x] 3.4 Update `DbClient::handle_connect` to use `DbBackend::connect` and store `Option<DbBackend>` instead of `Option<PgPool>`
- [x] 3.5 Update all command handlers in `DbClient::run` to match on `DbBackend` variant and delegate to engine-specific functions
- [x] 3.6 Run `cargo build` and verify PostgreSQL connect + schema load still works

## 4. MySQL Backend (`src/db/mysql.rs`)

- [x] 4.1 Implement `load_mysql_schema(pool: &MySqlPool) -> Vec<SchemaNode>` querying `information_schema.SCHEMATA`, `TABLES`, `COLUMNS`
- [x] 4.2 Implement `load_mysql_columns(pool, schema, table) -> Vec<ColumnInfo>` with column type mappings (VARCHAR, INT, TEXT, etc.)
- [x] 4.3 Implement `execute_mysql_query(pool, sql, cancel, tx)` sending `DbEvent::QueryStarted/ResultColumns/QueryRow/QueryCompleted`
- [x] 4.4 Handle MySQL-specific type display (e.g., `VARCHAR(255)`, `INT UNSIGNED`, `ENUM('a','b')`)
- [x] 4.5 Run `cargo build` and `cargo clippy`

## 5. SQLite Backend (`src/db/sqlite.rs`)

- [x] 5.1 Implement `load_sqlite_schema(pool: &SqlitePool) -> Vec<SchemaNode>` querying `sqlite_master` for tables/views/indexes
- [x] 5.2 Implement `load_sqlite_columns(pool, table) -> Vec<ColumnInfo>` using `PRAGMA table_info('<table>')`
- [x] 5.3 Implement `execute_sqlite_query(pool, sql, cancel, tx)` streaming results
- [x] 5.4 Handle SQLite-specific quirks: column types are affinity-based (TEXT, INTEGER, REAL, BLOB), `sqlite_master` has no schema column
- [x] 5.5 Run `cargo build` and `cargo clippy`

## 6. Configuration Module (`src/config/`)

- [x] 6.1 Create `src/config/mod.rs` with `Config` struct: `connections: Vec<ConnectionProfile>`, `keybindings: HashMap<String, String>`
- [x] 6.2 Define `ConnectionProfile` struct: `name`, `db_type` (enum `postgres`|`mysql`|`sqlite`), `host`, `port`, `database`, `user`, `password` (optional, for keychain)
- [x] 6.3 Implement `Config::load()` reading from `~/.config/twisterDBA/config.toml` via `dirs::config_dir()`
- [x] 6.4 Implement `Config::default()` with commented-out example profile and default keybindings
- [x] 6.5 Implement `Config::save()` writing back to TOML file
- [x] 6.6 Add `Config::add_profile(profile)` and auto-save after successful connection
- [x] 6.7 Run `cargo build` and verify config file generation

## 7. ConnectDialog UI Updates (`src/state.rs`, `src/ui.rs`)

- [x] 7.1 Add `db_type: usize` field to `ConnectForm` cycling through 0=PostgreSQL, 1=MySQL, 2=SQLite
- [x] 7.2 Update `ConnectForm::default()` to include db_type field at index 0
- [x] 7.3 Update `ConnectForm::build_dsn()` to produce engine-specific URI based on `db_type`
- [x] 7.4 Update `ConnectForm::masked_dsn()` for engine-specific masking
- [x] 7.5 In `src/ui.rs`, render the db_type field as a dropdown-style selector at the top of the dialog
- [x] 7.6 When SQLite is selected, hide Host/Port/Database/User/Password fields and show a single "File Path" field
- [x] 7.7 Run `cargo build` and manually test connect dialog

## 8. Config Integration with ConnectDialog

- [x] 8.1 Load `Config` in `App::new()` and store on `AppState`
- [x] 8.2 Populate profile list in ConnectDialog from `state.config.connections`
- [x] 8.3 After successful connection, call `Config::add_profile()` (omit password)
- [x] 8.4 Wire keybinding overrides from config into `AppState::handle_key` (check config map before hardcoded defaults)
- [x] 8.5 Run `cargo build` and `cargo clippy`

## 9. Verification

- [x] 9.1 Run `cargo build` and verify no errors with all three sqlx drivers
- [x] 9.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 9.3 Run `cargo fmt --check` and verify formatting
- [x] 9.4 Run `cargo test` and verify all tests pass
- [ ] 9.5 Manually test PostgreSQL connection (existing functionality preserved)
- [ ] 9.6 Manually test MySQL connection to a local MySQL instance
- [ ] 9.7 Manually test SQLite connection to a local `.db` file
- [ ] 9.8 Manually verify config file is created at `~/.config/twisterDBA/config.toml`
