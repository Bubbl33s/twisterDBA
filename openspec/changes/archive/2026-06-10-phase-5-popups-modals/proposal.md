## Why

The product spec defines four floating popups/modals: Connection Manager (`<leader>c`), Quick Documentation (`Ctrl+Q` / `K`), Keymap Help (`?`), and Command Palette (`:`). The current codebase has a basic connection dialog (ConnectDialog mode) and a minimal command palette via `:`. Missing are: floating Quick Documentation anchored to cursor/selected node showing DDL, row count, and table size; a context-sensitive Keymap Help popup that changes per active panel; and a full Command Palette for actions like toggle keyword case, run formatters, and invoke Lua commands (Phase 7).

## What Changes

- **Quick Documentation popup**: Press `K` or `Ctrl+Q` on a schema node (table/view) to show DDL, row count, and estimated table size in a floating window anchored near the entity
- **Keymap Help popup**: Press `?` to show a context-sensitive list of shortcuts for the currently active panel and mode
- **Enhanced Command Palette**: Extend the `:` command mode to support tab-completion, history navigation, and additional commands (toggle keyword case, run formatters, invoke Lua)
- All popups use the Clear + overlay pattern for Z-index correctness
- Context-sensitive: help text changes based on whether Schema Explorer, Query Editor, or Result Grid is focused

## Capabilities

### New Capabilities
- `quick-documentation`: Floating popup showing DDL, row count, and table size for selected schema entities
- `keymap-help`: Context-sensitive shortcut reference popup, triggered by `?`
- `enhanced-command-palette`: Tab-completion, history, and extended command set for the `:` prompt

### Modified Capabilities
- None (all new capabilities layered on top of existing modes)

## Impact

- `src/state.rs`: New `PopupState` enum (QuickDoc, KeymapHelp); AppState gains `popup: Option<PopupState>` field
- `src/ui.rs`: New popup render functions: `render_quick_doc`, `render_keymap_help`; Z-index via Clear overlay
- `src/explorer.rs`: SchemaNode gains `row_count: Option<u64>`, `table_size: Option<String>`, `ddl: Option<String>` fields
- `src/db/client.rs`: New `DbCommand::LoadTableInfo { schema, table }` for fetching DDL/row_count/size
- `src/events.rs`: New `DbEvent::TableInfoLoaded { ddl, row_count, table_size }`
- `src/app.rs`: New keybindings `K`/`Ctrl+Q` for Quick Doc, `?` for Keymap Help; enhanced command processing
