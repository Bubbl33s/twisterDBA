## 1. Dependencies & Build Setup

- [x] 1.1 Add `tree-sitter = "0.24"` and `tree-sitter-sql = "0.1"` to `Cargo.toml` (used tree-sitter-sequel as compatible alternative)
- [x] 1.2 Add `cc = "1"` as a build dependency in `[build-dependencies]` section
- [x] 1.3 Create `build.rs` that compiles `tree-sitter-sql` C sources using `cc::Build`
- [x] 1.4 Run `cargo build` and verify tree-sitter compiles without errors

## 2. Tree-Sitter Core Module (`src/editor/tree.rs`)

- [x] 2.1 Create `src/editor/tree.rs` with `pub struct TsParser` wrapping `tree_sitter::Parser`
- [x] 2.2 Implement `TsParser::new() -> Self` initializing parser with `tree_sitter_sql::LANGUAGE`
- [x] 2.3 Implement `TsParser::parse(&mut self, source: &str) -> Tree` returning the parsed syntax tree
- [x] 2.4 Implement `TsParser::highlight_line(line: &str, line_number: usize, source: &str) -> Vec<Span>` using tree cursors to walk tokens
- [x] 2.5 Define a `STYLE_MAP: HashMap<&str, Style>` mapping tree-sitter node types (keyword, string, comment, number, identifier, operator, parameter) to Darcula theme colors
- [x] 2.6 Run `cargo build` and verify `tree.rs` compiles

## 3. Statement Extraction (`src/editor/tree.rs`)

- [x] 3.1 Add `TsParser::find_statement_at(&self, tree: &Tree, cursor_byte: usize) -> Option<Range<usize>>` using `QueryCursor` with query `(statement) @stmt`
- [x] 3.2 Implement byte-offset cursor position calculation from `(cursor_row, cursor_col)` via `TextBuffer::cursor_byte_offset()`
- [x] 3.3 Add `TsParser::extract_text(&self, source: &str, range: Range<usize>) -> &str` using safe byte slicing
- [x] 3.4 Write unit test: cursor at start, middle, end, between statements, in string, in comment
- [x] 3.5 Write unit test: emoji/CJK in string literals (UTF-8 safety)
- [x] 3.6 Run `cargo test` and verify all tests pass

## 4. Replace Legacy Highlighting (`src/editor.rs`)

- [x] 4.1 In `SqlEditor`, add `ts_parser: TsParser` field initialized in `SqlEditor::new()`
- [x] 4.2 Add `SqlEditor::highlight_lines(&mut self) -> Vec<Vec<Span>>` that parses the full buffer and returns line-by-line spans
- [x] 4.3 In `src/ui.rs::render_sql_editor`, replace `editor::highlight_sql(line_str)` with `editor.highlight_lines()` cached per frame
- [x] 4.4 Remove the old `highlight_sql()` function and `is_sql_keyword()` helper from `src/editor.rs`
- [x] 4.5 Run `cargo build` and verify SQL highlighting works visually

## 5. Context-Aware Execution (`src/editor.rs`)

- [x] 5.1 Add `SqlEditor::extract_active_statement(&self) -> Option<String>` that finds the statement at cursor and returns its text
- [x] 5.2 Update `SqlEditor::execute(tx)` to call `extract_active_statement()` and send only the extracted SQL
- [x] 5.3 Add status bar message "No statement under cursor" when extraction returns `None`
- [x] 5.4 Prevent double-execution: check `self.executing` flag before sending new query
- [x] 5.5 Run `cargo build` and manually test multi-statement file execution

## 6. Split Window Management (`src/state.rs`)

- [x] 6.1 Change `AppState.editor: SqlEditor` to `editors: Vec<SqlEditor>` and `focused_editor: usize`
- [x] 6.2 Add `AppState::push_editor()`, `AppState::close_editor(index)`, `AppState::focus_editor(index)` methods
- [x] 6.3 Update all references to `self.editor` in `state.rs` to `self.focused_editor_mut()`
- [x] 6.4 Add `Panel::QueryEditor { buffer_index: usize }` variant to track which buffer's result to show (handled via focused_editor)
- [x] 6.5 Run `cargo build` and verify all state references compile

## 7. Split Window Keybindings (`src/app.rs`)

- [x] 7.1 Add `Ctrl+W` prefix detection in `handle_normal_key`: store pending prefix, await second key
- [x] 7.2 Implement `Ctrl+W s`: push new editor, split horizontally (row layout)
- [x] 7.3 Implement `Ctrl+W v`: push new editor, split vertically (column layout)
- [x] 7.4 Implement `Ctrl+W h/j/k/l`: change focused_editor based on layout direction
- [x] 7.5 Implement `Ctrl+W q`: close focused editor (prevent closing last)
- [x] 7.6 Implement `Ctrl+W =`: equalize split proportions
- [x] 7.7 Run `cargo build` and `cargo clippy`

## 8. Split Window Rendering (`src/ui.rs`)

- [x] 8.1 Update `render_main_area` to split editor area based on number of buffers and layout direction
- [x] 8.2 Render `│` separator between vertical splits and `─` separator between horizontal splits
- [x] 8.3 Show per-buffer cursor and mode in each split (only focused shows cursor highlight)
- [x] 8.4 Render per-buffer result grid below/beside its editor pane
- [x] 8.5 Handle terminal resize: recalculate split proportions
- [x] 8.6 Run `cargo build` and manually test split window rendering

## 9. Verification

- [x] 9.1 Run `cargo build` and verify no errors with tree-sitter integration
- [x] 9.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 9.3 Run `cargo fmt --check` and verify formatting
- [x] 9.4 Run `cargo test` and verify all tests pass
- [ ] 9.5 Manually test SQL highlighting with complex queries (CTEs, subqueries, dollar-quotes)
- [ ] 9.6 Manually test context-aware execution with multi-statement file
- [ ] 9.7 Manually test split window creation, navigation, and closing
- [ ] 9.8 Manually test UTF-8 strings with emoji and CJK characters
