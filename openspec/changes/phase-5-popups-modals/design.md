## Context

The product spec defines floating popups as an integral part of the UI: Quick Documentation anchored to cursor/selection (`Ctrl+Q`/`K`), context-sensitive Keymap Help (`?`), and a Command Palette (`:`) that goes beyond simple commands to include keyword case toggling and Lua invocation. The current code has a basic Command mode with `:connect`, `:quit`, etc., and a ConnectDialog popup via `Mode::ConnectDialog`. Popup Z-index is handled via the `Clear` widget overlay pattern documented in the spec.

## Goals / Non-Goals

**Goals:**
- Quick Documentation popup anchored near the selected schema node, showing DDL, row count, table size
- Keymap Help popup listing context-sensitive shortcuts for active panel + mode
- Enhanced Command Palette with Tab completion, history, `:upper`/`:lower`/`:format`/`:help`
- All popups use `Clear` + overlay for correct Z-index
- Live-updating Keymap Help when panel/mode changes while popup is open

**Non-Goals:**
- Fuzzy finder or command search (full text search of commands)
- Customizable popup positioning (always anchored near source or centered)
- External command execution (e.g., `:!ls`)

## Decisions

### ADR-001: Popup State as Enum on AppState
**Choice:** `PopupState` enum on `AppState`: `None | QuickDoc { anchor_node: SchemaNode, ddl: Option<String>, row_count: Option<u64>, table_size: Option<String> } | KeymapHelp | CommandPalette`.

**Rationale:** A single `Option<PopupState>` field keeps the render function simple: draw popup on top if `Some`. Each variant carries its own data. The z-index is guaranteed by rendering the popup after all panels.

**Alternative considered:** Separate `Option` fields for each popup type — multiple `Option`s bloat AppState and make it possible to have two popups active simultaneously (undefined behavior).

### ADR-002: Quick Doc Anchor Positioning
**Choice:** Calculate popup rect based on the selected node's position in the explorer list. Use `centered_rect` with an offset toward the anchor. If near edges, flip to the opposite side.

**Rationale:** Ratatui renders to a fixed grid. Calculating the anchor position from `selected_idx * row_height` in the explorer area gives a pixel-approximate anchor. Edge detection uses the popup size vs available space.

**Alternative considered:** Fixed center position — works but loses the "anchored to cursor" UX described in the spec.

### ADR-003: Keymap Help as Static Lookup Table
**Choice:** Define keybinding lists as `&[(key: &str, description: &str)]` slices keyed by `(Panel, Mode)`. The render function picks the right slice.

**Rationale:** Keybindings are finite and known at compile time. A static lookup is zero-cost and easy to update when new bindings are added. The help popup re-renders on every frame, so it's always current.

### ADR-004: Command Palette with VecDeque History
**Choice:** `CommandHistory` as a `VecDeque<String>` on `AppState`, max 100 entries. Tab completion matches against a static `COMMANDS: &[&str]` array.

**Rationale:** Simple and sufficient for the ~20 commands expected. No need for a trie or fuzzy matcher for MVP.

### ADR-005: Keyword Case Toggle via Tree-sitter
**Choice:** Use Tree-sitter (Phase 2) to identify `keyword` nodes in the buffer. Iterate through them, toggling case of the source text.

**Rationale:** Regex-based case toggling would break on keywords inside strings/comments. Tree-sitter's AST distinguishes `keyword` tokens from `string` and `comment` tokens accurately.

## Risks / Trade-offs

- **[Risk] Quick Doc popup may overlap important UI elements** → Mitigation: popup is dismissible with one keypress; positioned to minimize overlap
- **[Risk] DDL reconstruction for complex tables may be slow** → Mitigation: fetched asynchronously; shows "Loading..." immediately
- **[Risk] Keymap Help table may overflow on small terminals (<80 columns)** → Mitigation: truncate descriptions; make popup scrollable

## Migration Plan

1. Add `PopupState` enum and `app_state.popup` field
2. Add `LoadTableInfo` DB command and event for Quick Doc data
3. Implement Quick Doc render function in `ui.rs`
4. Implement Keymap Help render function with lookup table
5. Enhance Command mode with history, Tab completion, `:upper`/`:lower`/`:help`
6. Wire keybindings: `K`/`Ctrl+Q` for Quick Doc, `?` for Keymap Help
7. No data migration needed

## Open Questions

- Should Quick Doc also show foreign keys and indexes? (Answer: show indexes in DDL; FKs deferred to post-MVP)
- Should Keymap Help be filterable by typing? (Answer: no for MVP; simple list is sufficient)
