## ADDED Requirements

### Requirement: Multi-Engine Database Connectivity
The application SHALL connect to PostgreSQL, MySQL/MariaDB, and SQLite databases via a unified `DbBackend` abstraction. The user SHALL select the database type when creating or editing a connection profile.

#### Scenario: Connect to PostgreSQL
- **WHEN** the user opens the connection dialog, selects "PostgreSQL", enters host/port/database/user/password, and presses Enter
- **THEN** the application opens a PostgreSQL connection pool, queries system catalogs, and the status bar shows `● postgresql://user@host:5432/db`

#### Scenario: Connect to MySQL
- **WHEN** the user opens the connection dialog, selects "MySQL", enters host/port/database/user/password, and presses Enter
- **THEN** the application opens a MySQL connection pool, queries `information_schema`, and the status bar shows `● mysql://user@host:3306/db`

#### Scenario: Connect to SQLite
- **WHEN** the user opens the connection dialog, selects "SQLite", enters or browses to a `.db` file path, and presses Enter
- **THEN** the application opens the SQLite file and the status bar shows `● sqlite:///path/to/file.db`

#### Scenario: Connection dialog shows database type selector
- **WHEN** the user triggers the ConnectDialog mode (via `:connect` or `<leader>c`)
- **THEN** the first field in the dialog is a dropdown cycling through "PostgreSQL", "MySQL", "SQLite" with Tab/arrow keys

### Requirement: Unified Schema Introspection Across Engines
The application SHALL query each engine's system catalogs (`pg_catalog`, `information_schema`, `sqlite_master`) to produce the same `Vec<SchemaNode>` tree structure regardless of backend.

#### Scenario: PostgreSQL schema loaded
- **WHEN** connected to PostgreSQL and `LoadSchema` command is processed
- **THEN** the explorer shows schemas (from `pg_namespace`), tables (from `pg_class`), columns (from `pg_attribute`), and the tree renders with full type information

#### Scenario: MySQL schema loaded
- **WHEN** connected to MySQL and `LoadSchema` command is processed
- **THEN** the explorer shows databases as top-level nodes (from `information_schema.SCHEMATA`), tables beneath them, and columns with their MySQL types (e.g. `VARCHAR(255)`, `INT UNSIGNED`)

#### Scenario: SQLite schema loaded
- **WHEN** connected to SQLite and `LoadSchema` command is processed
- **THEN** the explorer shows a single database node (the file name), tables from `sqlite_master`, and columns via `PRAGMA table_info`

### Requirement: Per-Engine DSN Builder
The `ConnectForm::build_dsn()` method SHALL produce a valid connection string for the selected database type, respecting engine-specific URI schemes and quoting rules.

#### Scenario: PostgreSQL DSN includes all fields
- **WHEN** db type is PostgreSQL, host=localhost, port=5432, db=mydb, user=admin, pass=secret
- **THEN** `build_dsn()` returns `postgresql://admin:secret@localhost:5432/mydb`

#### Scenario: MySQL DSN with empty user uses root
- **WHEN** db type is MySQL, user is empty, pass is empty
- **THEN** `build_dsn()` returns `mysql://localhost:3306/` (or defaults to root if required)

#### Scenario: SQLite DSN is a file path
- **WHEN** db type is SQLite, path=/home/user/data.db
- **THEN** `build_dsn()` returns `sqlite:///home/user/data.db`

### Requirement: Connection Failure Handling Per Engine
The application SHALL surface connection errors with engine-specific diagnostics (e.g., "MySQL: Access denied for user 'root'@'localhost'").

#### Scenario: MySQL access denied
- **WHEN** MySQL connection fails with auth error
- **THEN** status bar shows red `✗ MySQL: Access denied for user '...'` and mode returns to Normal

#### Scenario: SQLite file not found (when creating new)
- **WHEN** SQLite connection targets a non-existent file path
- **THEN** the application opens the connection (SQLite creates the file if it doesn't exist) — no error

#### Scenario: Terminal resize during connection
- **WHEN** the terminal is resized while "Connecting..." spinner is shown
- **THEN** the connect dialog remains visible, re-centered, and the spinner continues

### Requirement: Concurrent Multi-Backend Support
The `DbClient` event loop SHALL handle commands for any backend type without blocking the UI. The `DbBackend` enum SHALL own the connection pool for the active engine.

#### Scenario: Switch from PostgreSQL to SQLite
- **WHEN** user disconnects from PostgreSQL and connects to SQLite
- **THEN** the PostgreSQL pool is gracefully closed, the SQLite pool opens, and schema loads immediately

#### Scenario: Query cancellation works across engines
- **WHEN** a long-running MySQL query is executing and user presses Ctrl+C
- **THEN** the query is cancelled via `CancellationToken`, the editor exits executing state, and no stale rows appear
