## 1. Popup State Infrastructure (`src/state.rs`)

- [x] 1.1 Define `PopupState` enum: `None | QuickDoc { schema, table, ddl, row_count, table_size, loading } | KeymapHelp | CommandPalette`
- [x] 1.2 Add `popup: PopupState` field to `AppState`
- [x] 1.3 Add `PopupState::is_open() -> bool` helper
- [x] 1.4 Add `AppState::close_popup()` method setting `popup = PopupState::None`
- [x] 1.5 Run `cargo build` and verify compilation

## 2. Quick Documentation Data Layer (`src/db/client.rs`, `src/events.rs`)

- [x] 2.1 Add `DbCommand::LoadTableInfo { schema: String, table: String }` variant
- [x] 2.2 Add `DbEvent::TableInfoLoaded { schema: String, table: String, ddl: Option<String>, row_count: Option<u64>, table_size: Option<String> }` variant
- [x] 2.3 In `src/db/pg_schema.rs` (or equivalent), implement `load_table_info` querying `pg_get_tabledef()` and `pg_stat_user_tables`
- [x] 2.4 In MySQL backend, implement `load_table_info` using `information_schema`
- [x] 2.5 In SQLite backend, implement `load_table_info` using `sqlite_master` and `SELECT COUNT(*)`
- [x] 2.6 Handle `DbEvent::TableInfoLoaded` in `state.rs::apply_db_event` to update `popup` state
- [x] 2.7 Run `cargo build` and verify DB commands compile

## 3. Quick Documentation UI (`src/ui.rs`, `src/app.rs`)

- [x] 3.1 Implement `render_quick_doc(f, area, popup_state)` rendering DDL in a scrollable paragraph, with row count and size as metadata lines
- [x] 3.2 Calculate anchor position from `explorer.selected_idx` and `explorer.flat_view`
- [x] 3.3 Position popup near the anchor, flipping if near terminal edges
- [x] 3.4 Use `Clear` widget behind the popup for Z-index correctness (transparent overlay)
- [x] 3.5 Handle scroll within the popup: `j`/`k` scroll DDL text, `g`/`G` top/bottom
- [x] 3.6 Wire `K` and `Ctrl+Q` keybindings in Normal mode (explorer focused) to open Quick Doc
- [x] 3.7 Send `DbCommand::LoadTableInfo` when opening the popup (if data not cached)
- [x] 3.8 Run `cargo build` and manually test Quick Doc popup

## 4. Keymap Help Lookup Table (`src/keymap_help.rs`)

- [x] 4.1 Create `src/keymap_help.rs` module
- [x] 4.2 Define `KeyBinding { keys: &'static str, description: &'static str }` struct
- [x] 4.3 Define lookup function `get_keybindings(panel: Panel, mode: Mode) -> &[KeyBinding]` returning a static slice
- [x] 4.4 Populate all known keybindings for: Explorer Normal, Editor Normal, Editor Insert, Result Normal, Command mode, ConnectDialog mode, Visual mode
- [x] 4.5 Run `cargo build` and verify module compiles

## 5. Keymap Help UI (`src/ui.rs`, `src/app.rs`)

- [x] 5.1 Implement `render_keymap_help(f, area, panel, mode)` rendering a bordered popup with a two-column table
- [x] 5.2 Title shows "Keymap Help — Schema Explorer [NORMAL]" (panel + mode)
- [x] 5.3 Handle scrolling within the popup if content exceeds viewport
- [x] 5.4 Wire `?` keybinding in Normal mode to toggle Keymap Help popup
- [x] 5.5 Ensure popup re-renders on every frame (live-updates when panel/mode changes)
- [x] 5.6 Run `cargo build` and manually test keymap help

## 6. Enhanced Command Palette (`src/state.rs`)

- [x] 6.1 Add `command_history: VecDeque<String>` and `history_index: Option<usize>` to `AppState`
- [x] 6.2 Add static `COMMANDS: &[&str]` array: `connect`, `disconnect`, `quit`, `export csv`, `upper`, `lower`, `format`, `help`, `h`
- [x] 6.3 Implement Tab completion in `handle_command_key`: on Tab, find the longest common prefix matching `COMMANDS`
- [x] 6.4 Implement Up/Down history navigation in `handle_command_key`
- [x] 6.5 After executing a command, push it to `command_history` (max 100, deduplicate consecutive identical)
- [x] 6.6 Run `cargo build` and test command completion

## 7. `:upper` / `:lower` Command (`src/state.rs`, Phase 2 tree-sitter)

- [x] 7.1 Implement `AppState::toggle_keyword_case(upper: bool)` method
- [x] 7.2 Use Tree-sitter (from Phase 2) to iterate through `keyword` nodes in the active editor buffer
- [x] 7.3 For each keyword node, replace the text in-place (preserving byte offsets) with upper/lower case
- [x] 7.4 Re-parse the buffer after toggling to keep highlighting accurate
- [x] 7.5 Run the `:upper` command handler in `execute_command`
- [x] 7.6 Run `cargo build` and manually test case toggling

## 8. `:help` Command and Unknown Command Handling

- [x] 8.1 Implement `:help` / `:h` command that appends available commands list to the output pane
- [x] 8.2 For unknown commands, show "Unknown command: X. Type :help for available commands." in output pane
- [x] 8.3 Ensure `:` command mode exits normally after `:help` (don't close immediately; show output)
- [x] 8.4 Run `cargo build` and `cargo clippy`

## 9. Verification

- [x] 9.1 Run `cargo build` and verify no errors
- [x] 9.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 9.3 Run `cargo fmt --check` and verify formatting
- [x] 9.4 Run `cargo test` and verify all tests pass
- [x] 9.5 Manually test Quick Doc popup on a table: DDL, row count, size visible
- [x] 9.6 Manually test Keymap Help with different panels/modes: content changes appropriately
- [x] 9.7 Manually test Command Palette Tab completion and history navigation
- [x] 9.8 Manually test `:upper` and `:lower` keyword case toggling
- [x] 9.9 Manually test Z-index: popup renders on top, no background bleed-through
