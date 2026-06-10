## ADR-001: DbSource wraps connection schema tree

**Decision**: Introduce `DbSource` struct that wraps a connection's schema tree with metadata:

```rust
pub struct DbSource {
    pub name: String,
    pub engine_type: EngineType,
    pub status: ConnectionStatus,
    pub masked_dsn: String,
    pub tree: Vec<SchemaNode>,
    pub expanded: bool,
}
```

`SchemaExplorer` changes from holding a single `tree: Vec<SchemaNode>` to `sources: Vec<DbSource>`.

**Rationale**:
- Each source is self-contained: its own tree, expansion state, and status
- Sources are ordered (Vec) matching the connection entries in AppState
- When a source is expanded, its schema tree is shown as children
- Alternative: separate explorer per connection — too complex, loses unified navigation

## ADR-002: Source nodes at depth 0 in flat view

**Decision**: In the flattened view, each DbSource appears at depth 0 with an engine-specific icon and status indicator. When expanded, its schema tree nodes appear at depth 1+.

**Rendering format**:
```
▾  local-postgres (connected)        <- depth 0, source node
  ▸ public                              <- depth 1, schema node
  ▸ information_schema                  <- depth 1, schema node
▸ 🐬 prod-mysql (disconnected)          <- depth 0, collapsed source
▾ 🪶 local-sqlite (connected)           <- depth 0, source node
   main                                <- depth 1, schema node
    ▸ users                             <- depth 2, table node
```

**Rationale**:
- Matches DataGrip's visual hierarchy: connection > database > schema > objects
- Status indicator helps users see which connections are active at a glance
- Engine icons provide immediate visual identification

## ADR-003: Source expansion triggers schema load

**Decision**: When a user expands a disconnected or never-loaded source, the app sends `DbCommand::LoadSchema { connection_name }` for that source. If the source is already connected and has a loaded tree, expansion just reveals the existing tree.

**Rationale**:
- Lazy loading: schema is only fetched when user expands the source
- Avoids loading schemas for all connections on startup
- Matches DataGrip behavior where expanding a connection loads its metadata

## ADR-004: Source selection sets active connection

**Decision**: When a user presses Enter on a source node (or navigates to it), that source becomes the `active_connection`. The explorer then shows that source's schema tree. Query execution targets the active connection.

**Rationale**:
- Clear mental model: "I'm working with this database"
- Prevents accidental queries on wrong connection
- Matches DataGrip's console-to-connection binding

## ADR-005: Engine-specific icons in IconMap

**Decision**: Add engine-specific icons to `IconMap`:
- PostgreSQL: `'\u{F06FC}'` (database icon, teal) — reuse existing `database` icon
- MySQL: `'\u{F07C0}'` (cylinder icon, blue) — new icon
- SQLite: `'\u{F021A}'` (file icon, gray) — new icon

Also add status indicator colors:
- Connected: green
- Connecting: yellow (with spinner)
- Error: red
- Disconnected: gray

**Rationale**:
- Visual differentiation between engine types
- Status colors provide immediate connection health feedback
- Nerd Font icons when available, ASCII fallbacks otherwise

## ADR-006: Source persistence in session

**Decision**: `SessionData` stores `saved_sources: Vec<String>` (connection profile names). On startup, the app iterates saved sources, attempts to reconnect each, and adds them to the explorer. The `active_connection` is restored last.

**Rationale**:
- Users expect their connections to persist across sessions
- Auto-reconnect saves time; users can still manually disconnect unwanted sources
- Reconnection failures are shown as "error" status, not silently dropped
