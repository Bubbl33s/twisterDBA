## Why

The connection dialog has three visual/UX issues reported after the previous overhaul:

1. **Backdrop hides underlying UI**: `render_dialog_backdrop` uses `Clear` widget which wipes the entire terminal buffer, then paints `Color::Rgb(10, 10, 10)` (essentially black) over everything. The schema explorer, SQL editor, output panel, and status bar are completely hidden. The nvim reference shows that popups should let the underlying UI remain visible.

2. **Active field row not highlighted**: When a field is active in Step 2, the label span uses `bg(theme.dialog_cursor_bg)` (light blue-gray) while the field spans use `bg(theme.dialog_field_active_bg)` (darker gray). Two different backgrounds on the same row means there's no unified row highlight — the user can't easily see which row has focus.

3. **No j/k navigation in Step 1**: The engine/profile selection only responds to arrow keys (`Up`/`Down`). Vim users expect `j`/`k` to work for vertical navigation, consistent with the schema explorer tree.

## What Changes

- **Remove backdrop overlay**: Eliminate the `Clear` + dark paragraph that hides the underlying UI. The dialog's own border and background provide sufficient visual separation.
- **Unified active row highlight**: When a field is active, use `theme.dialog_field_active_bg` for the entire row (label + field), with white text on the label for contrast.
- **Add j/k navigation**: Map `Char('j')` to Down and `Char('k')` to Up in Step 1 key handler. Update help text to show `↑↓/jk`.

## Capabilities

### New Capabilities
- `dialog-no-backdrop`: Dialog renders without opaque backdrop, underlying UI remains visible
- `dialog-active-row-highlight`: Active field row has unified background highlight across label and input
- `dialog-jk-navigation`: Step 1 supports j/k keys for vertical navigation

### Modified Capabilities
- None — these are pure visual/UX fixes, no capability changes

## Impact

- `src/ui/utils.rs`: Remove or no-op `render_dialog_backdrop` body
- `src/ui/mod.rs`: Remove or keep (no-op) call to `render_dialog_backdrop`
- `src/ui/dialog.rs`: Fix active row highlight in `render_editable_field`; update Step 1 help text
- `src/state/handlers/connect.rs`: Add `Char('j')`/`Char('k')` match arms in `handle_step1_key`
