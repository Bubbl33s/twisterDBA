## 1. DbClient Multi-Backend Refactor (`src/db/client.rs`)

- [x] 1.1 Replace `backend: Option<DbBackend>` with `backends: HashMap<String, DbBackend>` in `DbClient`
- [x] 1.2 Add `connection_name: String` to all `DbCommand` variants
- [x] 1.3 Update `handle_connect` to store backend under `connection_name` key; close existing backend with same name if present
- [x] 1.4 Update `handle_disconnect` to take `connection_name`, remove from HashMap, close pool
- [x] 1.5 Update `handle_load_schema` to take `connection_name`, look up backend from HashMap
- [x] 1.6 Update `handle_load_columns` to take `connection_name`, look up backend from HashMap
- [x] 1.7 Update `handle_load_table_info` to take `connection_name`, look up backend from HashMap
- [x] 1.8 Update `handle_execute_query` to take `connection_name`, look up backend from HashMap
- [x] 1.9 Update `run()` match arms to pass `connection_name` to each handler
- [x] 1.10 Run `cargo build` and fix compile errors

## 2. DbEvent Connection Name (`src/events.rs`)

- [x] 2.1 Add `connection_name: String` to `DbEvent::Connected`
- [x] 2.2 Add `connection_name: String` and rename inner to `message: String` in `DbEvent::ConnectionFailed`
- [x] 2.3 Add `connection_name: String` to `DbEvent::Disconnected`
- [x] 2.4 Add `connection_name: String` to `DbEvent::SchemaLoaded`
- [x] 2.5 Add `connection_name: String` to `DbEvent::ColumnsLoaded`
- [x] 2.6 Add `connection_name: String` to `DbEvent::TableInfoLoaded`
- [x] 2.7 Add `connection_name: String` to `DbEvent::QueryStarted`
- [x] 2.8 Add `connection_name: String` to `DbEvent::ResultColumns`
- [x] 2.9 Add `connection_name: String` to `DbEvent::QueryRow`
- [x] 2.10 Add `connection_name: String` to `DbEvent::QueryCompleted`
- [x] 2.11 Add `connection_name: String` and rename inner to `message: String` in `DbEvent::QueryError`
- [x] 2.12 Add `connection_name: String` to `DbEvent::QueryCancelled`
- [x] 2.13 Run `cargo build` and fix compile errors

## 3. ConnectionEntry and AppState (`src/state/connection.rs`, `src/state/mod.rs`)

- [x] 3.1 Add `ConnectionEntry` struct with fields: `name`, `engine_type`, `status`, `masked_dsn`
- [x] 3.2 Replace `connection_status: ConnectionStatus` with `connections: Vec<ConnectionEntry>` in AppState
- [x] 3.3 Add `active_connection: Option<String>` to AppState
- [x] 3.4 Update `AppState::new()` to initialize empty connections vec and None active_connection
- [x] 3.5 Add helper method `active_connection_entry(&self) -> Option<&ConnectionEntry>`
- [x] 3.6 Add helper method `active_connection_entry_mut(&mut self) -> Option<&mut ConnectionEntry>`
- [x] 3.7 Add helper method `connection_by_name(&self, name: &str) -> Option<&ConnectionEntry>`
- [x] 3.8 Run `cargo build` and fix compile errors

## 4. Event Routing (`src/state/events.rs`)

- [x] 4.1 Update `Connected` handler: find or create ConnectionEntry by name, update status, set active_connection
- [x] 4.2 Update `ConnectionFailed` handler: find ConnectionEntry by name, update status to Error
- [x] 4.3 Update `Disconnected` handler: find ConnectionEntry by name, update status, clear active_connection if matching
- [x] 4.4 Update `SchemaLoaded` handler: only update explorer if connection_name matches active_connection
- [x] 4.5 Update `ColumnsLoaded` handler: only update explorer if connection_name matches active_connection
- [x] 4.6 Update `TableInfoLoaded` handler: route to popup regardless of active connection
- [x] 4.7 Update query event handlers (QueryStarted, ResultColumns, QueryRow, QueryCompleted, QueryError, QueryCancelled): process normally (queries only run on active connection)
- [x] 4.8 Run `cargo build` and fix compile errors

## 5. Command Senders Update (`src/state/handlers/`, `src/editor/`)

- [x] 5.1 Update `handle_connect_dialog_key`: send `connection_name` with `DbCommand::Connect`
- [x] 5.2 Update `handle_explorer_key`: send `active_connection` name with `DbCommand::LoadColumns`
- [x] 5.3 Update `handle_popup_key` (LoadTableInfo): send `active_connection` name
- [x] 5.4 Update `SqlEditor::execute_query`: send `active_connection` name with `DbCommand::ExecuteQuery`
- [x] 5.5 Update `:disconnect` command: send `active_connection` name with `DbCommand::Disconnect`
- [x] 5.6 Run `cargo build` and fix compile errors

## 6. UI Updates (`src/ui/`, `src/app.rs`)

- [x] 6.1 Update status bar to show active connection name instead of raw DSN
- [x] 6.2 Update `app.rs` auto-reconnect: iterate saved connections, send Connect for each with connection_name
- [x] 6.3 Update `app.rs` startup: set `active_connection` after successful auto-reconnect
- [x] 6.4 Run `cargo build` and fix compile errors

## 7. Session Persistence Update (`src/state/session.rs`)

- [x] 7.1 Update `SessionData`: replace `connection_profile` with `active_connection: Option<String>` and `saved_connections: Vec<String>`
- [x] 7.2 Update `to_session_data()`: serialize active_connection name and list of connected profile names
- [x] 7.3 Update `apply_session_data()`: restore active_connection; trigger reconnect for saved connections
- [x] 7.4 Run `cargo build` and fix compile errors

## 8. Verification

- [x] 8.1 Run `cargo build` and verify no errors
- [x] 8.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 8.3 Run `cargo fmt --check` and verify formatting
- [x] 8.4 Run `cargo test` and verify all tests pass
- [ ] 8.5 Manually test: connect to PostgreSQL, verify connection appears in state
- [ ] 8.6 Manually test: connect to second database, verify both exist simultaneously
- [ ] 8.7 Manually test: disconnect one, verify other remains connected
- [ ] 8.8 Manually test: reconnect to same name, verify old pool is replaced
- [ ] 8.9 Manually test: session save/restore with active connection
