## Why

The current connection dialog shows a DB type dropdown + 5 inline fields + saved profiles listed below. This is functional but not intuitive. DataGrip uses a two-step approach: Step 1 shows a grid of database type icons (PostgreSQL, MySQL, SQLite) with saved connections listed below each type. Step 2 shows engine-specific connection fields. This is more discoverable and matches user expectations from the most popular database IDE. Additionally, the current dialog auto-generates connection names (`postgres-localhost`) which are not user-friendly. Users should be able to set a custom connection name.

## What Changes

- **Two-step dialog flow**: Step 1 = database type selection grid + saved profiles list; Step 2 = engine-specific connection fields
- **Database type grid**: Visual grid showing PostgreSQL, MySQL, SQLite with icons; keyboard navigation between types
- **Saved profiles in Step 1**: Profiles grouped by engine type; select a profile to auto-fill Step 2
- **Connection name field**: Editable name field in Step 2, auto-generated from `user@host/database` but overridable
- **Engine-specific fields**: PostgreSQL gets SSL mode dropdown; field sets vary by engine
- **Dialog title**: Shows current step ("New Connection" or "Edit Connection")
- **Navigation**: Enter on type/profile in Step 1 → Step 2; Esc in Step 2 → Step 1; Esc in Step 1 → close dialog

## Capabilities

### New Capabilities
- `two-step-connection-dialog`: Dialog has Step 1 (type/profile selection) and Step 2 (connection fields)
- `engine-specific-fields`: Each database engine shows its own set of connection fields
- `custom-connection-name`: User can set a custom name for the connection

### Modified Capabilities
- `multi-db-support`: Connection dialog flow redesigned; DSN building unchanged
- `credential-storage`: Keychain integration unchanged; profile selection from Step 1

## Impact

- `src/state/connection.rs`: Major refactor — `ConnectForm` becomes two-step with `step` field; add `ConnectionName` field; engine-specific field definitions
- `src/state/handlers/connect.rs`: Complete rewrite of key handling for two-step flow
- `src/ui/dialog.rs`: Complete rewrite of rendering for two-step layout
- `src/config/mod.rs`: `ConnectionProfile` may need `connection_name` field (or derive from profile name)
- `src/state/handlers/command.rs`: `:connect` command enters Step 1; `:connect <name>` pre-fills from profile
- `src/theme.rs`: Add larger engine icons for the type grid
