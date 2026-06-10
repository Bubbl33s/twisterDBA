## ADR-001: Remove backdrop, rely on dialog border for separation

**Decision**: Remove the `Clear` + dark paragraph backdrop entirely. The dialog will render directly on top of the existing UI without any overlay.

**Before**:
```rust
pub fn render_dialog_backdrop(f: &mut Frame, area: Rect) {
    use ratatui::widgets::Clear;
    f.render_widget(Clear, area);
    let backdrop = Paragraph::new(Text::from(""))
        .style(Style::default().bg(Color::Rgb(10, 10, 10)));
    f.render_widget(backdrop, area);
}
```

**After**: Empty function body (or remove call site entirely).

**Rationale**:
- `Clear` widget resets all cells in the area to terminal default, destroying the rendered UI
- The dark paragraph then paints black over everything — there's no way to "see through" it
- The dialog already has `theme.editor_bg` background and `theme.identifier` border, which provides clear visual separation from the underlying UI
- This matches nvim's popup behavior: the underlying buffer remains visible, the popup floats on top
- No performance impact — fewer widgets rendered

**Alternatives considered**:
- Using a semi-transparent color — ratatui doesn't support alpha/transparency
- Rendering the UI twice (dimmed + normal) — too complex, double render cost
- Using `Modifier::DIM` — not supported by all terminals

## ADR-002: Unified row background for active field

**Decision**: When `is_active` is true in `render_editable_field`, both the label span and all field spans use `theme.dialog_field_active_bg` as background, with `Color::White` for label text and field text.

**Before**:
- Label bg: `theme.dialog_cursor_bg` (light blue-gray ~169,183,198)
- Field bg: `theme.dialog_field_active_bg` (dark gray ~60,63,65)
- Result: two-tone row, no clear focus indicator

**After**:
- Label bg: `theme.dialog_field_active_bg` (same as field)
- Label fg: `Color::White` (for contrast on dark bg)
- Field bg: `theme.dialog_field_active_bg`
- Field fg: `Color::White`
- Result: unified dark row with white text, clear focus indicator

**Rationale**:
- A single background color across the full row creates a clear "this row is active" signal
- White text on dark gray provides sufficient contrast (WCAG AA)
- Matches the nvim reference where the active line has a uniform highlight
- Minimal code change — just align the label style with the field style

## ADR-003: j/k as aliases for Up/Down in Step 1

**Decision**: Add `KeyCode::Char('j')` and `KeyCode::Char('k')` as aliases for Down and Up respectively in `handle_step1_key`.

**Rationale**:
- Consistent with vim keybindings used throughout the application
- The schema explorer already supports j/k navigation
- No conflict — Step 1 is read-only except for navigation and selection, so j/k can't conflict with text input
- Minimal code change — two additional match arms that delegate to existing Up/Down logic
