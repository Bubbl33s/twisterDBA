## Context

The product spec is prescriptive about visual design: "Borderless, block-color rendering — no box-drawing characters, no wasted terminal cells." The current `ui.rs` uses Ratatui's `Borders::ALL` on every panel, which produces `─│┌┐└┘` characters that the spec rejects. The spec also specifies a Darcula-inspired TrueColor palette and Nerd Font icons.

The challenge is replacing bordered blocks with borderless equivalents while maintaining visual clarity. Ratatui's `Block::default()` without borders is trivial, but panel separation must be communicated through color alone. This requires a centralized theme system and consistent color application.

## Goals / Non-Goals

**Goals:**
- Zero box-drawing characters in the UI
- Darcula TrueColor palette applied consistently
- Nerd Font icons with color coding for all entity types
- Speed Search with real-time filtering in schema explorer
- Theme constants centralized in `src/theme.rs` for Lua extensibility (Phase 7)
- Focus communicated via statusline background color

**Non-Goals:**
- Multiple theme presets (only Darcula for MVP)
- Theme hot-reload (reload on config change only)
- Custom font loading (expects Nerd Font installed in terminal)
- Animation/transition effects

## Decisions

### ADR-001: Centralized Theme Struct
**Choice:** `pub struct Theme { pub bg: Color, pub editor_bg: Color, pub keyword: Color, pub string: Color, pub number: Color, pub comment: Color, pub icons: IconMap }` in `src/theme.rs`.

**Rationale:** A single struct with public fields is zero-cost to read, easy to serialize/deserialize for TOML config, and trivially exposable to Lua via `mlua` in Phase 7. No dynamic dispatch needed.

**Alternative considered:** Lazy `LazyLock<Theme>` global — makes testing harder, prevents per-window themes.

### ADR-002: Borderless via Block::default()
**Choice:** Replace all `Block::default().borders(Borders::ALL)` with `Block::default()` and rely on background colors for visual separation.

**Rationale:** Ratatui's `Block::default()` renders only the inner area with the specified background. No box characters are drawn. Panel separation is achieved by leaving 1-2 columns of the global background color between panels.

**Alternative considered:** Custom `Clear` widget-based layout — unnecessary; Ratatui's `Layout` with `Constraint::Percentage` gaps works fine.

### ADR-003: Nerd Font Icon Rendering
**Choice:** Inline Nerd Font codepoints as `\u{F06FC}` in Rust string literals. Store the mapping in `Theme.icons` as `HashMap<NodeKind, (char, Color)>`.

**Rationale:** Nerd Font glyphs are valid Unicode Private Use Area characters. Rust handles them natively. Ratatui's `Span::raw()` renders them. No special rendering pipeline needed.

**Fallback:** Detect terminal font via environment variable or config flag. If Nerd Font is unavailable, use ASCII fallback strings.

**Alternative considered:** Rendering icons as separate `Span` with a custom font — over-engineered; terminal fonts handle glyph selection.

### ADR-004: Speed Search via Filtered Flat View
**Choice:** Add `search_query: String` and `search_timer: Instant` to `SchemaExplorer`. On rebuild, if query is non-empty, filter `flat_view` to nodes where `name.to_lowercase().contains(query.to_lowercase())`.

**Rationale:** The flat view is already built and O(n) to filter. Real-time filtering on <1000 nodes is instant. Timer resets after 1s of inactivity for a fresh search.

**Alternative considered:** Incremental tree walk — adds complexity for no performance gain at typical schema sizes (<5000 nodes).

### ADR-005: Darcula Color Mapping for Tree-Sitter
**Choice:** Update the `STYLE_MAP` in `src/editor/tree.rs` (Phase 2) to use Darcula colors: keywords `#CC7832`, strings `#6A8759`, numbers `#6897BB`, comments `#808080`.

**Rationale:** The spec explicitly defines these colors. Tree-sitter token types map cleanly: `keyword` → amber, `string` → olive, `number` → light blue, `comment` → grey.

## Risks / Trade-offs

- **[Risk] No borders may confuse users in small terminals** → Mitigation: 2-column gap between panels provides clear separation; statusline focus color is unambiguous
- **[Risk] Nerd Font may not be installed** → Mitigation: ASCII fallback; README documents Nerd Font requirement
- **[Risk] TrueColor may not work in all terminals** → Mitigation: crossterm's `supports_truecolor()` check at startup; fallback to 256-color approximation

## Migration Plan

1. Create `src/theme.rs` with `Theme` struct and default Darcula colors
2. Replace `Borders::ALL` with `Block::default()` in all panels
3. Update `explorer.rs` icon rendering to use Nerd Font codepoints and colors
4. Add Speed Search state and filtering to `SchemaExplorer`
5. Update statusline to use theme focus colors
6. No data migration needed

## Open Questions

- Should the theme be reloadable at runtime via command? (Answer: MVP via config file reload; Lua runtime theme in Phase 7)
- Should we detect Nerd Font automatically? (Answer: check for common Nerd Font patched font names in terminal; fallback gracefully)
