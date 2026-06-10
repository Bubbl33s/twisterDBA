## 1. Theme Colors for Dialog (`src/theme.rs`)

- [x] 1.1 Add `dialog_backdrop_dim: Color` field to `Theme` struct (value: `Color::Rgb(10, 10, 10)`)
- [x] 1.2 Add `dialog_cursor_bg: Color` field to `Theme` struct (value: `Color::Rgb(169, 183, 198)` — same as `identifier`)
- [x] 1.3 Add `dialog_cursor_fg: Color` field to `Theme` struct (value: `Color::Rgb(30, 31, 34)` — same as `editor_bg`)
- [x] 1.4 Add `dialog_field_active_bg: Color` field to `Theme` struct (value: `Color::Rgb(60, 63, 65)`)
- [x] 1.5 Initialize all new fields in `Theme::darcula()`
- [x] 1.6 Run `cargo build` and fix compile errors

## 2. Backdrop Dimming (`src/ui/utils.rs`)

- [x] 2.1 Modify `render_dialog_backdrop` to use `Color::Rgb(10, 10, 10)` instead of `Color::Rgb(20, 20, 20)`
- [x] 2.2 Verify backdrop still uses `Clear` widget for proper terminal reset
- [x] 2.3 Run `cargo build` and fix compile errors
- [x] 2.4 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 2.5 Run `cargo fmt --check` and verify formatting

## 3. Cursor Visibility Improvement (`src/ui/dialog.rs`)

- [x] 3.1 Update `render_editable_field` to use reverse-video cursor: bg=`dialog_cursor_bg`, fg=`dialog_cursor_fg` for the cursor character
- [x] 3.2 Replace empty-field cursor character from `█` (full block) to `` (left half block, U+258C)
- [x] 3.3 Update cursor rendering for non-empty fields: invert fg/bg of the character at cursor position
- [x] 3.4 Accept `theme: &Theme` parameter in `render_editable_field` to use theme colors
- [x] 3.5 Run `cargo build` and fix compile errors
- [x] 3.6 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 3.7 Run `cargo fmt --check` and verify formatting

## 4. Vertical Engine List — ConnectForm Refactor (`src/state/connection.rs`)

- [x] 4.1 Add `cursor_position: usize` field to `ConnectForm` (replaces separate `type_cursor` and `selected_profile`)
- [x] 4.2 Add `ENGINE_COUNT: usize = 3` constant to `ConnectForm`
- [x] 4.3 Add method `is_engine_selected(&self) -> bool` (cursor_position < ENGINE_COUNT)
- [x] 4.4 Add method `selected_engine(&self) -> usize` (returns cursor_position when is_engine_selected)
- [x] 4.5 Add method `selected_profile_index(&self) -> Option<usize>` (returns cursor_position - ENGINE_COUNT when cursor is on a profile)
- [x] 4.6 Update `ConnectForm::default()` to set `cursor_position: 0`
- [x] 4.7 Update `ConnectForm::from_profile()` to set `cursor_position` appropriately
- [x] 4.8 Run `cargo build` and fix compile errors
- [x] 4.9 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 4.10 Run `cargo fmt --check` and verify formatting

## 5. Vertical Engine List — Key Handler Update (`src/state/handlers/connect.rs`)

- [x] 5.1 Rewrite Step 1 key handling to use single `cursor_position` instead of separate `type_cursor`/`selected_profile`
- [x] 5.2 Implement Up/Down navigation: cursor moves through engines (0..2) then profiles (3..3+profile_count), wrapping at boundaries
- [x] 5.3 Implement Left/Right as no-ops in Step 1 (or alias to Up/Down for accessibility)
- [x] 5.4 Update Enter handler: if `is_engine_selected()` → Step 2 with engine fields; if profile → Step 2 with profile data
- [x] 5.5 Update Esc handler: close dialog from Step 1
- [x] 5.6 Run `cargo build` and fix compile errors
- [x] 5.7 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 5.8 Run `cargo fmt --check` and verify formatting

## 6. Vertical Engine List — Rendering (`src/ui/dialog.rs`)

- [x] 6.1 Rewrite `render_step1` to use vertical list layout instead of horizontal grid
- [x] 6.2 Render each engine as a row: icon + name, with highlight if cursor is on it
- [x] 6.3 Render separator line between engines and profiles (only if profiles exist)
- [x] 6.4 Render "Saved Connections:" label above profile entries
- [x] 6.5 Render each profile as a row: icon + name + host, with highlight if cursor is on it
- [x] 6.6 Update `render_type_grid` calls to use new vertical list approach (or remove and inline)
- [x] 6.7 Update `render_profile_list` to work with combined cursor model
- [x] 6.8 Use theme colors for borders, backgrounds, and highlights
- [x] 6.9 Update dialog sizing: width = `min(50, area.width * 80 / 100)`, height = content-proportional with max `area.height * 70 / 100`
- [x] 6.10 Run `cargo build` and fix compile errors
- [x] 6.11 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 6.12 Run `cargo fmt --check` and verify formatting

## 7. Step 2 Theme-Aware Rendering (`src/ui/dialog.rs`)

- [x] 7.1 Update `render_step2` to accept `theme: &Theme` parameter
- [x] 7.2 Replace hardcoded `Color::Black` dialog background with `theme.editor_bg`
- [x] 7.3 Replace hardcoded dialog border with `theme.identifier` color
- [x] 7.4 Update `render_step2_fields` to use `theme.dialog_field_active_bg` for active field background
- [x] 7.5 Update `render_step2_help` to use `theme.statusline_active_bg` for key highlights
- [x] 7.6 Update `render_editable_field` label highlight to use theme colors
- [x] 7.7 Run `cargo build` and fix compile errors
- [x] 7.8 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 7.9 Run `cargo fmt --check` and verify formatting

## 8. Dialog Sizing Improvement (`src/ui/dialog.rs`, `src/ui/utils.rs`)

- [x] 8.1 Add `centered_rect_bounded(percent_x: u16, percent_y: u16, max_width: u16, max_height: u16, r: Rect) -> Rect` to `utils.rs`
- [x] 8.2 Update Step 1 to use bounded sizing: max 50 cols width, content-based height, max 70% terminal height
- [x] 8.3 Update Step 2 to use bounded sizing: max 60 cols width, content-based height, max 70% terminal height
- [x] 8.4 Verify dialog remains centered when bounded
- [x] 8.5 Run `cargo build` and fix compile errors
- [x] 8.6 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 8.7 Run `cargo fmt --check` and verify formatting

## 9. Integration: Pass Theme to Dialog Rendering (`src/ui/mod.rs`, `src/ui/dialog.rs`)

- [x] 9.1 Update `render_connect_dialog` signature to accept `theme: &Theme`
- [x] 9.2 Update call site in `src/ui/mod.rs` to pass `theme` to `render_connect_dialog`
- [x] 9.3 Propagate `theme` parameter through all internal render functions in `dialog.rs`
- [x] 9.4 Run `cargo build` and fix compile errors
- [x] 9.5 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 9.6 Run `cargo fmt --check` and verify formatting

## 10. Verification

- [x] 10.1 Run `cargo build` and verify no errors
- [x] 10.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 10.3 Run `cargo fmt --check` and verify formatting
- [x] 10.4 Run `cargo test` and verify all tests pass
- [ ] 10.5 Manually test: open dialog, verify backdrop dims but doesn't hide underlying UI
- [ ] 10.6 Manually test: verify cursor is clearly visible in all input fields (empty and non-empty)
- [ ] 10.7 Manually test: verify Step 1 shows vertical engine list with combined navigation
- [ ] 10.8 Manually test: verify dialog sizing is content-proportional and bounded
- [ ] 10.9 Manually test: verify theme colors are used consistently throughout dialog
- [ ] 10.10 Manually test: verify all existing functionality (engine selection, profile selection, field editing, SSL mode, connection) works without regression
