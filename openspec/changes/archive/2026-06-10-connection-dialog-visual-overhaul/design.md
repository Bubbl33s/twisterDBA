## ADR-001: Backdrop dimming via Clear + subtle overlay

**Decision**: Replace the current `Clear` + `Paragraph` with solid `Color::Rgb(20, 20, 20)` background with a two-layer approach:

1. Render `Clear` widget over the full terminal area — this resets cells to terminal default background
2. Render a `Paragraph` with a very dark color (`Color::Rgb(10, 10, 10)`) that is close to but not identical to the terminal background

This creates a "curtain" effect where the underlying UI is dimmed but its shapes and colors remain faintly visible through the slight color difference.

```rust
pub fn render_dialog_backdrop(f: &mut Frame, area: Rect) {
    use ratatui::widgets::Clear;
    f.render_widget(Clear, area);
    let backdrop = Paragraph::new(Text::from(""))
        .style(Style::default().bg(Color::Rgb(10, 10, 10)));
    f.render_widget(backdrop, area);
}
```

**Rationale**:
- `Clear` alone would show the raw terminal background (no dimming)
- A solid dark color creates a "wall" that hides everything
- A very dark color close to the terminal bg creates a subtle dimming "curtain"
- No additional allocations — same widget types, just different color value
- Works on all terminals that support RGB colors

**Alternatives considered**:
- Using `Modifier::DIM` on the backdrop — not supported by all terminals
- Rendering the underlying UI with reduced brightness — too complex, would require re-rendering everything

## ADR-002: Cursor rendering with reverse-video style

**Decision**: The cursor in active input fields uses a **reverse-video** approach: the character at the cursor position (or a block character for empty fields) is rendered with foreground and background colors swapped relative to the field style.

For active fields:
- Field background: `Color::DarkGray` (or theme `dialog_field_active_bg`)
- Field text: `Color::White`
- Cursor character: background = `Color::White`, foreground = `Color::DarkGray` (inverted)
- Empty field cursor: block character `▌` (U+258C, left half block) with the same inverted colors

```
Active field:  [  hello█world  ]
                        ^^ cursor char has inverted fg/bg

Empty field:   [  ▌            ]
                  ^^ block cursor with inverted fg/bg
```

**Rationale**:
- Reverse-video is a well-understood terminal convention
- Works regardless of terminal color support level
- The block character `` is widely supported in monospace fonts
- Inversion provides maximum contrast without relying on color hue
- No dependency on cursor blinking — the static visual difference is sufficient

**Alternatives considered**:
- Using `█` (full block) — may render as double-width in some terminals
- Using underline cursor — not visible on empty fields
- Blinking cursor — adds complexity, not all terminals support it well

## ADR-003: Vertical engine list with combined navigation

**Decision**: Step 1 replaces the horizontal 3-column grid with a vertical list where engines and saved profiles share a single navigation cursor.

```
┌──────────────────────────────┐
│  New Connection              │
│                              │
│  > 🐘 PostgreSQL             │  ← engine entry
│    🐬 MySQL                  │
│    🪶 SQLite                 │
│  ─────────────────────       │  ← separator
│  Saved Connections:          │
│  > 🐘 local-postgres (localhost) │ ← profile entry
│                              │
│  ←→↑↓:navigate  Enter:select │
│  Esc:cancel                  │
└──────────────────────────────┘
```

The `ConnectForm` gains a single `cursor_position: usize` that indexes into a virtual combined list: `[PostgreSQL, MySQL, SQLite, profile_0, profile_1, ...]`. Positions 0-2 are engines; positions 3+ are profiles.

**Rationale**:
- Vertical list is more compact — dialog height matches content
- Single cursor simplifies navigation logic (no separate type_cursor + selected_profile)
- Matches DataGrip's dropdown-style data source picker
- Separator visually distinguishes engines from profiles
- Easier to add more engine types in the future (just append to list)

## ADR-004: Content-proportional dialog sizing

**Decision**: Dialog dimensions are calculated from content rather than fixed percentages:

- **Step 1 width**: `min(50, terminal_width * 80 / 100)` — enough for engine names + profile names
- **Step 1 height**: `3 (engines) + 1 (separator) + 1 (label) + profile_count + 1 (help) + 2 (padding)`, clamped to `min(20, terminal_height * 70 / 100)`
- **Step 2 width**: `min(60, terminal_width * 80 / 100)` — enough for longest label + input
- **Step 2 height**: `1 (name) + fields + 1 (DSN) + 1 (help) + 2 (padding)`, clamped to `min(20, terminal_height * 70 / 100)`

**Rationale**:
- Dialog never wastes screen space on large terminals
- Dialog never overflows on small terminals
- Height grows naturally as profiles are added
- Consistent with terminal UI best practices

## ADR-005: Theme-aware dialog styling

**Decision**: All dialog visual elements use colors from `Theme` rather than hardcoded values:

| Element | Current | New |
|---------|---------|-----|
| Dialog border | `Borders::ALL` (default white) | `Block::borders()` with `Style::fg(theme.identifier)` |
| Dialog background | `Color::Black` | `theme.editor_bg` |
| Selected item bg | `ENGINE_COLORS[i]` or `Color::Rgb(60,63,65)` | `theme.dialog_type_selected_bg` |
| Profile bg | `Color::Black` | `theme.dialog_profile_bg` |
| Help text key color | `Color::Cyan` | `theme.statusline_active_bg` |
| Cursor inverted bg | `Color::Cyan` | `theme.identifier` (light gray) |
| Cursor inverted fg | `Color::Black` | `theme.editor_bg` (dark) |

**Rationale**:
- Consistent visual language across the application
- Easier to create alternative themes in the future
- No magic numbers scattered in dialog rendering code
- Dialog feels like part of the app, not a foreign overlay
