## 1. Theme Module (`src/theme.rs`)

- [x] 1.1 Create `src/theme.rs` with `pub struct Theme { pub bg: Color, pub editor_bg: Color, pub keyword: Color, pub string: Color, pub number: Color, pub comment: Color, pub identifier: Color, pub operator: Color, pub statusline_active_bg: Color, pub statusline_inactive_bg: Color, pub icons: IconMap }`
- [x] 1.2 Define `pub struct IconMap` with `database: (char, Color)`, `schema: (char, Color)`, `table: (char, Color)`, `view: (char, Color)`, `routine: (char, Color)`, `column: (char, Color)`
- [x] 1.3 Implement `Theme::darcula()` returning the default Darcula palette with all icon codepoints
- [x] 1.4 Add `pub mod theme;` to `src/main.rs` and ensure module compiles
- [x] 1.5 Run `cargo build` and verify `theme.rs` compiles

## 2. Borderless Rendering (`src/ui.rs`)

- [x] 2.1 In `render_schema_explorer`, replace `Block::default().borders(Borders::ALL)` with `Block::default().style(Style::default().bg(theme.bg))`
- [x] 2.2 In `render_sql_editor`, remove `editor_block` border; use background color only
- [x] 2.3 In `render_result_grid`, remove `grid_block` border; use background color only
- [x] 2.4 In `render_status_bar`, remove border; render statusline as full-width colored bar
- [x] 2.5 Add 2-column gap between panels using `Layout` spacing (gap between percentage constraints)
- [x] 2.6 In `render_connect_dialog`, keep popup border (floating modals are an exception per spec)
- [x] 2.7 Run `cargo build` and visually verify no box-drawing characters

## 3. Statusline Focus State (`src/ui.rs`)

- [x] 3.1 Pass `theme: &Theme` to `render_status_bar` and all panel render functions
- [x] 3.2 Render the focused panel's section of the statusline with `theme.statusline_active_bg` (vibrant blue)
- [x] 3.3 Render inactive panels' statusline sections with `theme.statusline_inactive_bg` (muted grey)
- [x] 3.4 Update mode indicator colors to use theme constants
- [x] 3.5 Run `cargo build` and visually test focus states

## 4. Nerd Font Icons (`src/explorer.rs`, `src/ui.rs`)

- [x] 4.1 In `explorer.rs`, add `NodeKind` → `(icon_char, color)` mapping using `Theme.icon_map`
- [x] 4.2 Update `SchemaExplorer::rebuild_flat_view` to include icon and color in `FlatNode` struct
- [x] 4.3 In `ui.rs::render_schema_explorer`, render icon before node name using the mapped color
- [x] 4.4 Add Nerd Font availability check using `std::env::var("TERM")` or config flag
- [x] 4.5 Implement ASCII fallback: `[DB]`, `[S]`, `[T]`, `[V]`, `[R]` when Nerd Font unavailable
- [x] 4.6 Run `cargo build` and visually verify Nerd Font icons render

## 5. Speed Search (`src/explorer.rs`, `src/state.rs`)

- [x] 5.1 Add `search_query: String`, `search_active: bool`, `last_key_time: Instant` fields to `SchemaExplorer`
- [x] 5.2 Implement `SchemaExplorer::apply_search(&mut self)` filtering `flat_view` by `search_query` (case-insensitive substring match)
- [x] 5.3 Implement `SchemaExplorer::push_search_char(c: char)` and `SchemaExplorer::pop_search_char()`
- [x] 5.4 Implement auto-reset: if `last_key_time.elapsed() > 1s`, clear `search_query` before pushing new char
- [x] 5.5 In `state.rs::handle_explorer_key`, route alphanumeric chars in Normal mode to search (not vim commands)
- [x] 5.6 Handle `Escape` to clear search filter
- [x] 5.7 Show "(no matches)" text when filter yields zero results
- [x] 5.8 Run `cargo build` and `cargo clippy`

## 6. Darcula Color Integration in Editor (`src/editor.rs`)

- [x] 6.1 Replace hardcoded `Color::Cyan`, `Color::Green`, etc. in `highlight_sql` with theme colors (Phase 2 tree-sitter integration will use these)
- [x] 6.2 In `editor.rs`, pass `theme: &Theme` to the highlight function via `SqlEditor`
- [x] 6.3 Update tree-sitter `STYLE_MAP` (Phase 2 `tree.rs`) to use theme colors from `Theme`
- [x] 6.4 Run `cargo build` and verify editor colors match Darcula spec

## 7. Verification

- [x] 7.1 Run `cargo build` and verify no errors
- [x] 7.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 7.3 Run `cargo fmt --check` and verify formatting
- [x] 7.4 Run `cargo test` and verify all tests pass
- [x] 7.5 Manually verify zero box-drawing characters in all panels
- [x] 7.6 Manually verify Darcula colors (amber keywords, green strings, blue numbers, grey comments)
- [x] 7.7 Manually verify Nerd Font icons in schema explorer with a Nerd Font terminal
- [x] 7.8 Manually verify Speed Search filters tree nodes in real-time
- [x] 7.9 Manually verify statusline focus state changes when switching panels
