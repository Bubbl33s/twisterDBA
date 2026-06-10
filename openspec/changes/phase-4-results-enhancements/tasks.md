## 1. Auto-Pagination (`src/editor.rs`, `src/db/client.rs`)

- [x] 1.1 Add `auto_paginate: bool` and `page_size: usize` fields to `SqlEditor`
- [x] 1.2 Implement `SqlEditor::inject_pagination(sql: &str, page: usize, page_size: usize) -> String` wrapping SELECT queries in a subquery with LIMIT/OFFSET
- [x] 1.3 Detect existing LIMIT clause: if query already has LIMIT (case-insensitive regex check), skip injection
- [x] 1.4 Update `SqlEditor::execute()` to use `inject_pagination` when `auto_paginate` is true
- [x] 1.5 Add `auto_paginate` and `page_size` fields to `DbCommand::ExecuteQuery`
- [x] 1.6 In `db/client.rs`, pass pagination parameters through to query execution
- [x] 1.7 Run `cargo build` and verify injection logic

## 2. Scroll-to-Bottom Page Trigger (`src/result.rs`, `src/state.rs`)

- [x] 2.1 Add `current_page: usize`, `has_more: bool` fields to `ResultGrid`
- [x] 2.2 Implement `ResultGrid::needs_next_page() -> bool` checking if selected_row is at last visible row and `has_more` is true
- [x] 2.3 In `state.rs::handle_result_normal_key`, after `move_selection`, check `needs_next_page()` and trigger next page fetch
- [x] 2.4 Add `DbCommand::FetchNextPage { page: usize }` variant for explicit page requests
- [x] 2.5 Handle `DbEvent::QueryCompleted` to set `has_more = false` when rows returned < page_size
- [x] 2.6 Run `cargo build` and `cargo clippy`

## 3. Primary Key Detection (`src/db/client.rs`, `src/result.rs`)

- [x] 3.1 Add `is_primary_key: bool` field to `ColumnMeta` struct in `src/result.rs`
- [x] 3.2 In PostgreSQL schema loading (`pg_schema.rs`), query `pg_constraint` + `pg_attribute` to detect PK columns
- [x] 3.3 In MySQL schema loading, query `information_schema.KEY_COLUMN_USAGE` for PK detection
- [x] 3.4 In SQLite schema loading, parse `PRAGMA table_info` result (pk column is non-zero)
- [x] 3.5 Pass PK info through `DbEvent::ColumnsLoaded` and store in `ColumnMeta`
- [x] 3.6 Run `cargo build` and verify PK detection compiles

## 4. Inline Cell Editing (`src/result.rs`, `src/state.rs`)

- [x] 4.1 Define `CellEditState` enum: `None | Editing { row: usize, col: usize, value: String, cursor: usize }`
- [x] 4.2 Add `cell_edit: CellEditState` field to `ResultGrid`
- [x] 4.3 Implement `ResultGrid::enter_edit(row, col, current_value)` populating edit state
- [x] 4.4 Implement `ResultGrid::edit_insert_char(c)`, `edit_delete_backward()`, `edit_move_cursor(dir)`
- [x] 4.5 Implement `ResultGrid::commit_edit() -> Option<(String, Vec<(String, String)>)>` returning new value and PK WHERE clause
- [x] 4.6 In `state.rs`, add `HandleEditMode` key routing: characters insert, Backspace deletes, Enter commits, Escape cancels
- [x] 4.7 On commit, construct `UPDATE <table> SET <col> = <new_value> WHERE <pk_col> = <pk_value>` SQL and send via `DbCommand::ExecuteQuery`
- [x] 4.8 On success (QueryCompleted), update the cell value in the grid; on error, show error in output pane
- [x] 4.9 Prevent editing PK columns: check `columns[col].is_primary_key` before entering edit mode
- [x] 4.10 Run `cargo build` and `cargo clippy`

## 5. Output/Services Pane (`src/state.rs`, `src/ui.rs`)

- [x] 5.1 Define `OutputPaneState` struct: `lines: VecDeque<String>`, `max_lines: usize (500)`, `scroll: usize`
- [x] 5.2 Add `OutputPaneState` to `SqlEditor` (per-buffer output) and `AppState` (global output for connection events)
- [x] 5.3 Implement `OutputPaneState::push(message: String)` with ring buffer behavior
- [x] 5.4 In `state.rs::apply_db_event`, append connection/query/error events to output pane
- [x] 5.5 In `ui.rs`, update `render_main_area` to include fourth panel (Output) in bottom-left
- [x] 5.6 Implement `render_output_pane(f, area, output_state)` showing timestamped log lines
- [x] 5.7 Implement output pane scrolling: `j`/`k` for line scroll, `g`/`G` for top/bottom
- [x] 5.8 Update `Panel` enum to include `Output` variant
- [x] 5.9 Update Tab cycling in `app.rs`: Explorer → Editor → Results → Output → Explorer
- [x] 5.10 Run `cargo build` and visually verify output pane

## 6. Per-Buffer Output Independence

- [x] 6.1 Ensure each `SqlEditor` (split buffer) has its own `OutputPaneState`
- [x] 6.2 When focus changes between buffers, `render_output_pane` shows the focused buffer's output
- [x] 6.3 When a buffer is closed, its output state is dropped
- [x] 6.4 Run `cargo build` and `cargo clippy`

## 7. Verification

- [x] 7.1 Run `cargo build` and verify no errors
- [x] 7.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 7.3 Run `cargo fmt --check` and verify formatting
- [x] 7.4 Run `cargo test` and verify all tests pass
- [ ] 7.5 Manually test auto-pagination: execute `SELECT * FROM large_table`, scroll to bottom, verify next page loads
- [ ] 7.6 Manually test inline editing: edit a cell, commit, verify UPDATE executes and cell updates
- [ ] 7.7 Manually test output pane: run queries, check timestamps and error messages appear
- [ ] 7.8 Manually test Tab cycling through 4 panels
- [ ] 7.9 Manually test per-buffer output independence with two split windows
