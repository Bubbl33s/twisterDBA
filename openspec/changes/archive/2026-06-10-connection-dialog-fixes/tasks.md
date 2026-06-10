## 1. Remove Backdrop Overlay (`src/ui/utils.rs`, `src/ui/mod.rs`)

- [x] 1.1 Make `render_dialog_backdrop` a no-op (empty function body, keep signature for API stability)
- [x] 1.2 Run `cargo build` and fix compile errors
- [x] 1.3 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 1.4 Run `cargo fmt --check` and verify formatting

## 2. Unified Active Row Highlight (`src/ui/dialog.rs`)

- [x] 2.1 Update `render_editable_field`: when `is_active`, use `theme.dialog_field_active_bg` for label background (instead of `theme.dialog_cursor_bg`)
- [x] 2.2 Update `render_editable_field`: when `is_active`, use `Color::White` for label foreground (instead of `theme.editor_bg`)
- [x] 2.3 Run `cargo build` and fix compile errors
- [x] 2.4 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 2.5 Run `cargo fmt --check` and verify formatting

## 3. j/k Navigation in Step 1 (`src/state/handlers/connect.rs`, `src/ui/dialog.rs`)

- [x] 3.1 Add `KeyCode::Char('j')` match arm in `handle_step1_key` that delegates to Down logic
- [x] 3.2 Add `KeyCode::Char('k')` match arm in `handle_step1_key` that delegates to Up logic
- [x] 3.3 Update `render_step1_help` to show `↑↓/jk` instead of just `↑↓`
- [x] 3.4 Run `cargo build` and fix compile errors
- [x] 3.5 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 3.6 Run `cargo fmt --check` and verify formatting

## 4. Verification

- [x] 4.1 Run `cargo build` and verify no errors
- [x] 4.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 4.3 Run `cargo fmt --check` and verify formatting
- [x] 4.4 Run `cargo test` and verify all tests pass
- [x] 4.5 Manually test: open dialog, verify underlying UI is visible behind dialog
- [x] 4.6 Manually test: navigate to a field in Step 2, verify entire row (label + input) has unified highlight
- [x] 4.7 Manually test: press j/k in Step 1, verify cursor moves down/up
- [x] 4.8 Manually test: verify all existing functionality (engine selection, profile selection, field editing, SSL mode, connection) works without regression
