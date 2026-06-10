## 1. DbSource Struct and Explorer Refactor (`src/explorer.rs`)

- [x] 1.1 Add `DbSource` struct with fields: `name`, `engine_type`, `status`, `masked_dsn`, `tree`, `expanded`
- [x] 1.2 Add `SourceNode` variant to `SchemaNode` enum (or handle sources separately in flattening)
- [x] 1.3 Add `NodeKind::Source` variant with engine-specific icon mapping
- [x] 1.4 Replace `tree: Vec<SchemaNode>` in `SchemaExplorer` with `sources: Vec<DbSource>`
- [x] 1.5 Update `flatten()` to render source nodes at depth 0, their trees at depth 1+
- [x] 1.6 Add `set_tree_for_source(&mut self, name: &str, nodes: Vec<SchemaNode>)` method
- [x] 1.7 Add `add_source(&mut self, source: DbSource)` method
- [x] 1.8 Add `remove_source(&mut self, name: &str)` method
- [x] 1.9 Add `set_source_status(&mut self, name: &str, status: ConnectionStatus)` method
- [x] 1.10 Add `expand_source(&mut self, name: &str)` / `collapse_source(&mut self, name: &str)` methods
- [x] 1.11 Update `insert_columns` to work within a specific source's tree
- [x] 1.12 Update `set_loading_child` to work within a specific source's tree
- [x] 1.13 Run `cargo build` and fix compile errors

## 2. Explorer Rendering Update (`src/ui/explorer.rs`)

- [x] 2.1 Rename `render_schema_explorer` to `render_database_explorer`
- [x] 2.2 Change block title from "Schema Explorer" to "Database Explorer"
- [x] 2.3 Render source nodes at depth 0 with engine icon, name, and status indicator
- [x] 2.4 Add status indicator rendering: green dot (connected), yellow spinner (connecting), red dot (error), gray dot (disconnected)
- [x] 2.5 Update empty state text from "(no schema)" to "(no connections)"
- [x] 2.6 Update ASCII fallbacks: `[PG]` for PostgreSQL, `[MY]` for MySQL, `[SQ]` for SQLite
- [x] 2.7 Run `cargo build` and fix compile errors

## 3. Theme Engine Icons (`src/theme.rs`)

- [x] 3.1 Add `postgres: (char, Color)` to `IconMap` (teal database icon)
- [x] 3.2 Add `mysql: (char, Color)` to `IconMap` (blue cylinder icon)
- [x] 3.3 Add `sqlite: (char, Color)` to `IconMap` (gray file icon)
- [x] 3.4 Add `status_connected: Color` (green) to Theme
- [x] 3.5 Add `status_connecting: Color` (yellow) to Theme
- [x] 3.6 Add `status_error: Color` (red) to Theme
- [x] 3.7 Add `status_disconnected: Color` (gray) to Theme
- [x] 3.8 Run `cargo build` and fix compile errors

## 4. Event Handler Updates (`src/state/events.rs`)

- [x] 4.1 Update `Connected` handler: add source to explorer with `DbSource` struct
- [x] 4.2 Update `SchemaLoaded` handler: set tree on the correct source by connection_name
- [x] 4.3 Update `Disconnected` handler: remove source from explorer
- [x] 4.4 Update `ConnectionFailed` handler: update source status to Error
- [x] 4.5 Run `cargo build` and fix compile errors

## 5. AppState Source Management (`src/state/mod.rs`)

- [x] 5.1 Add `active_source(&self) -> Option<&DbSource>` method
- [x] 5.2 Add `set_active_source(&mut self, name: &str)` method
- [x] 5.3 Update explorer key handler to handle source node selection (Enter = set active + expand)
- [x] 5.4 Run `cargo build` and fix compile errors

## 6. Explorer Key Handler Update (`src/state/handlers/explorer.rs`)

- [x] 6.1 Update expand logic: if node is a source, expand/collapse the source; if schema/table, existing logic
- [x] 6.2 Update Enter on source: set as active_connection, expand source, send LoadSchema if needed
- [x] 6.3 Update LoadColumns command to include active_connection name
- [x] 6.4 Run `cargo build` and fix compile errors

## 7. Session Persistence Update (`src/state/session.rs`)

- [x] 7.1 Update `SessionData`: add `saved_sources: Vec<String>` and `active_connection: Option<String>`
- [x] 7.2 Update `to_session_data()`: serialize connected source names and active connection
- [x] 7.3 Update `apply_session_data()`: store saved sources for auto-reconnect; restore active_connection
- [x] 7.4 Run `cargo build` and fix compile errors

## 8. App Startup Auto-Reconnect (`src/app.rs`)

- [x] 8.1 Update startup: iterate saved sources from session, send Connect for each with connection_name
- [x] 8.2 After all auto-reconnects, set active_connection from session
- [x] 8.3 Run `cargo build` and fix compile errors

## 9. Status Bar Update (`src/ui/status.rs`)

- [x] 9.1 Update status bar to show active connection name with engine icon
- [x] 9.2 Show connection count when multiple sources exist (e.g., "2 connections")
- [x] 9.3 Run `cargo build` and fix compile errors

## 10. Verification

- [x] 10.1 Run `cargo build` and verify no errors
- [x] 10.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 10.3 Run `cargo fmt --check` and verify formatting
- [x] 10.4 Run `cargo test` and verify all tests pass
- [x] 10.5 Manually test: connect to PostgreSQL, verify source appears in explorer
- [x] 10.6 Manually test: connect to MySQL, verify both sources visible
- [x] 10.7 Manually test: expand/collapse sources independently
- [x] 10.8 Manually test: select source sets active connection
- [x] 10.9 Manually test: disconnect removes source from explorer
- [x] 10.10 Manually test: quit and restart, verify auto-reconnect restores sources
