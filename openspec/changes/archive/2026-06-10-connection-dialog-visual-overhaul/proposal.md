## Why

The connection dialog currently has two critical visual bugs:

1. **Opaque backdrop**: `render_dialog_backdrop` in `ui/utils.rs` fills the entire terminal with `Color::Rgb(20, 20, 20)`, completely hiding the schema explorer, SQL editor, output panel, and status bar. The user loses all spatial context.
2. **Invisible cursor**: The cursor indicator in input fields is not clearly distinguishable — the cyan background on dark fields doesn't provide enough contrast, and the block cursor character may not render in all terminals.

Additionally, the dialog layout is oversized (fixed 70%/55% and 60%/60% of terminal) and uses a horizontal 3-column grid for engine selection that wastes vertical space. The design should be modernized to match DataGrip's compact, form-style connection dialog.

## What Changes

- **Semi-transparent backdrop**: Replace solid black backdrop with a dimming effect that keeps the underlying UI faintly visible
- **High-contrast cursor**: Use reverse-video style or bright block character for cursor visibility in all terminals
- **Vertical engine list**: Replace horizontal 3-column grid with a compact vertical list (icon + name per row)
- **Combined navigation**: Single cursor moves through engines and saved profiles as one list
- **Content-proportional sizing**: Dialog height matches content (min/max bounds), not fixed percentages
- **Theme-aware styling**: Dialog border, highlights, and backgrounds use theme colors instead of hardcoded values
- **Form-style Step 2**: Host+Port on same row, DSN preview at bottom, consistent label alignment

## Capabilities

### New Capabilities
- `dialog-backdrop-dimming`: Backdrop dims underlying UI without fully obscuring it
- `dialog-cursor-visibility`: Cursor is always clearly visible in active input fields
- `dialog-compact-sizing`: Dialog sizes proportionally to content with min/max bounds

### Modified Capabilities
- `two-step-connection-dialog`: Step 1 layout changes from horizontal grid to vertical list; combined engine+profile navigation
- `visual-theme`: Dialog rendering uses theme colors for borders, backgrounds, and highlights

## Impact

- `src/ui/utils.rs`: Modify `render_dialog_backdrop` — remove solid background, use dimming style
- `src/ui/dialog.rs`: Rewrite Step 1 rendering (vertical list, combined navigation); improve cursor rendering in Step 2; use theme colors; content-proportional sizing
- `src/state/connection.rs`: Add `cursor_position: usize` to track combined engine+profile cursor; possibly add DSN display field
- `src/state/handlers/connect.rs`: Update Step 1 key handling for combined vertical list navigation
- `src/theme.rs`: Add `dialog_backdrop_dim`, `dialog_cursor_bg`, `dialog_field_active_bg` colors
