## ADR-001: HashMap<String, DbBackend> for multiple backends

**Decision**: Replace `backend: Option<DbBackend>` with `backends: HashMap<String, DbBackend>` in `DbClient`. The key is the connection name (e.g., `"local-postgres"`, `"prod-mysql"`).

**Rationale**:
- HashMap provides O(1) lookup by connection name
- Each backend is independent; no shared state between connections
- Connection name is the natural identifier (matches `ConnectionProfile.name`)
- Alternative `Vec<DbBackend>` would require linear scan on every command

**Limits**: No explicit limit on number of connections. Memory is bounded by pool size (5 connections per backend x N backends).

## ADR-002: Connection name on all DbCommand variants

**Decision**: Every `DbCommand` variant that targets a specific backend carries a `connection_name: String` field. `Connect` also carries `connection_name` so the DbClient knows which key to register the new backend under.

```rust
pub enum DbCommand {
    Connect { connection_name: String, dsn: SecretString, engine_type: EngineType },
    Disconnect { connection_name: String },
    LoadSchema { connection_name: String },
    LoadColumns { connection_name: String, schema: String, table: String },
    LoadTableInfo { connection_name: String, schema: String, table: String },
    ExecuteQuery { connection_name: String, sql: String, cancel: CancellationToken, auto_paginate: bool, page_size: usize },
    FetchNextPage { connection_name: String, page: usize, sql: String, cancel: CancellationToken },
}
```

**Rationale**:
- Explicit routing: every command knows which backend to target
- No ambiguity when multiple connections exist
- `Connect` includes name so the backend is registered under the correct key

## ADR-003: Connection name on all DbEvent variants

**Decision**: Every `DbEvent` variant carries `connection_name: String` so `apply_db_event` can route the event to the correct `ConnectionEntry`.

```rust
pub enum DbEvent {
    Connected { connection_name: String },
    ConnectionFailed { connection_name: String, message: String },
    Disconnected { connection_name: String },
    SchemaLoaded { connection_name: String, nodes: Vec<SchemaNode> },
    ColumnsLoaded { connection_name: String, schema: String, table: String, columns: Vec<ColumnInfo> },
    TableInfoLoaded { connection_name: String, schema: String, table: String, ddl: Option<String>, row_count: Option<u64>, table_size: Option<String> },
    QueryStarted { connection_name: String },
    ResultColumns { connection_name: String, columns: Vec<ColumnMeta> },
    QueryRow { connection_name: String, cells: Vec<String> },
    QueryCompleted { connection_name: String, rows_affected: u64, duration_ms: u64 },
    QueryError { connection_name: String, message: String },
    QueryCancelled { connection_name: String },
}
```

**Rationale**:
- `apply_db_event` must know which connection the event belongs to
- Query events (ResultColumns, QueryRow) must route to the correct result grid
- Without connection_name, events from different backends would be indistinguishable

## ADR-004: ConnectionEntry replaces single ConnectionStatus

**Decision**: AppState holds `connections: Vec<ConnectionEntry>` instead of `connection_status: ConnectionStatus`. Each entry tracks its own status independently.

```rust
pub struct ConnectionEntry {
    pub name: String,
    pub engine_type: EngineType,
    pub status: ConnectionStatus,
    pub masked_dsn: String,
}
```

AppState also gains `active_connection: Option<String>` to track which connection the user is currently interacting with.

**Rationale**:
- Each connection has independent lifecycle (connecting, connected, error, disconnected)
- `active_connection` determines which connection's schema the explorer displays
- Vec preserves insertion order for the explorer UI

## ADR-005: Event routing by connection name

**Decision**: `apply_db_event` matches on `connection_name` to find the correct `ConnectionEntry` and update its status. For schema/column events, the explorer is updated only if the event's connection matches `active_connection`.

**Rationale**:
- Prevents stale schema data from overwriting the active explorer view
- Query results are always routed to the active result grid (queries only run on the active connection)
- Connection status updates are always applied regardless of active connection

## ADR-006: Connect command registers backend by name

**Decision**: When `DbCommand::Connect` is processed, the DbClient stores the new backend under `connection_name` in the HashMap. If a backend with that name already exists, it is closed first (reconnect scenario).

**Rationale**:
- Reconnecting to the same profile replaces the old pool cleanly
- No orphaned connections in the HashMap
