## Why

twisterDBA currently supports only one active database connection at a time. The `DbClient` holds a single `Option<DbBackend>`, and connecting to a new database closes the previous one. Users who work with multiple databases (e.g., dev + prod, or different projects) must disconnect and reconnect repeatedly. A multi-connection architecture allows multiple databases to remain connected simultaneously, each with its own schema tree, enabling the Database Explorer to show all connections as top-level sources.

## What Changes

- **Multi-backend DbClient**: Replace `backend: Option<DbBackend>` with `backends: HashMap<String, DbBackend>` keyed by connection name
- **Connection-named commands**: Add `connection_name: String` to all `DbCommand` variants that target a specific backend (`LoadSchema`, `LoadColumns`, `LoadTableInfo`, `ExecuteQuery`, `Disconnect`, `FetchNextPage`)
- **Connection-named events**: Add `connection_name: String` to all `DbEvent` variants that originate from a specific backend
- **Connection entries in AppState**: Replace single `connection_status: ConnectionStatus` with `connections: Vec<ConnectionEntry>` where each entry tracks name, engine type, status, and masked DSN
- **Event routing**: `apply_db_event` routes each event to the correct `ConnectionEntry` by name
- **Query routing**: `ExecuteQuery` and `FetchNextPage` are routed to the backend identified by `connection_name`
- **Active connection tracking**: AppState tracks `active_connection: Option<String>` to know which connection the user is currently working with

## Capabilities

### New Capabilities
- `multi-connection-backend`: DbClient manages multiple named backends concurrently via HashMap
- `connection-named-events`: All DbEvent variants carry a connection_name for routing
- `connection-entries`: AppState tracks multiple connections with individual status

### Modified Capabilities
- `multi-db-support`: DbCommand variants now require connection_name; DbClient dispatches by name
- `session-persistence`: Must persist active connection name instead of single connection_profile
- `credential-storage`: Password storage per profile unchanged; auto-reconnect iterates saved connections

## Impact

- `src/db/client.rs`: Major refactor — HashMap of backends, all handlers take connection_name
- `src/db/backend.rs`: No structural change; DbBackend enum unchanged
- `src/events.rs`: All DbEvent variants gain `connection_name: String` field
- `src/state/mod.rs`: Replace `connection_status` with `connections: Vec<ConnectionEntry>`; add `active_connection`
- `src/state/connection.rs`: Add `ConnectionEntry` struct; `ConnectionStatus` unchanged
- `src/state/events.rs`: Route events by connection_name; update all match arms
- `src/state/handlers/connect.rs`: Send connection_name with Connect command
- `src/state/handlers/command.rs`: Update :connect, :disconnect to use connection names
- `src/state/handlers/explorer.rs`: Send connection_name with LoadColumns
- `src/state/handlers/popup.rs`: Send connection_name with LoadTableInfo
- `src/editor/mod.rs`: Send connection_name with ExecuteQuery
- `src/state/session.rs`: Persist active_connection name; auto-reconnect logic updated
- `src/app.rs`: Auto-reconnect iterates saved connections; DbClient creation unchanged
- `src/ui/status.rs`: Status bar shows active connection name
