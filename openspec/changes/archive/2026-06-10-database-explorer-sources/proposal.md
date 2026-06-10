## Why

The current schema explorer shows a single flat tree under "Schema Explorer" title. With multi-connection support (from the `multi-connection-architecture` change), users need to see all their database connections as top-level sources in the explorer, similar to DataGrip's Database Explorer. Each connection source shows its engine icon, name, and connection status. The explorer title should change from "Schema Explorer" to "Database Explorer" to reflect this broader scope. Additionally, the archived `window-tab-navigation` change's `Panel` to `Window` rename needs to be incorporated here since both changes touch the explorer.

## What Changes

- **Rename "Schema Explorer" to "Database Explorer"**: Update all UI labels, titles, and references
- **DbSource struct**: New struct wrapping a connection's schema tree with metadata (name, engine_type, status, icon)
- **Explorer holds Vec<DbSource>**: Instead of a single tree, the explorer manages multiple sources
- **Top-level rendering**: Each DbSource renders as an expandable root node with engine icon, name, and status indicator
- **Source selection**: Clicking/entering a source sets it as `active_connection` and shows its schema
- **Connection persistence**: Sources persist across sessions; auto-reconnect restores them on startup
- **Window enum**: Incorporate `Panel` to `Window` rename from archived `window-tab-navigation` change (already applied in current codebase as `focused_window: Window`)

## Capabilities

### New Capabilities
- `database-explorer-sources`: Explorer shows multiple connection sources as top-level nodes
- `source-persistence`: Connection sources persist in session and auto-reconnect on startup
- `source-status-indicators`: Each source shows connection status (connected, connecting, error, disconnected)

### Modified Capabilities
- `schema-explorer-rendering`: Explorer title changes to "Database Explorer"; renders source nodes at root level
- `window-tab-navigation`: Window enum already in use; ensure all references are consistent

## Impact

- `src/explorer.rs`: Major refactor — add `DbSource` struct, `SchemaExplorer` holds `Vec<DbSource>`, flattening includes source nodes
- `src/ui/explorer.rs`: Rename function, update title, render source nodes with engine icons and status
- `src/state/events.rs`: Update `SchemaLoaded` handling to set tree on the correct DbSource
- `src/state/mod.rs`: Add methods to manage sources (add, remove, set_active)
- `src/state/session.rs`: Persist sources list; auto-reconnect logic
- `src/app.rs`: Auto-reconnect all saved sources on startup
- `src/theme.rs`: Add engine-specific icons (PostgreSQL, MySQL, SQLite) to IconMap
- `src/ui/status.rs`: Show active source name in status bar
