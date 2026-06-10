## Why

The product spec mandates a borderless, block-color visual design inspired by JetBrains Darcula: "no box-drawing characters, no wasted terminal cells." The current implementation uses Ratatui `Borders::ALL` on every panel, creating visual noise and box-drawing characters that the spec explicitly avoids. Additionally, the schema explorer lacks Nerd Font icons and Speed Search. This phase implements the full visual identity specified in the product doc: Darcula TrueColor palette, borderless panels with statusline focus indicators, Nerd Font entity icons, and type-to-filter node navigation.

## What Changes

- Remove all `Borders::ALL` from panels; replace with plain `Block::default()` or no block at all
- Implement Darcula TrueColor theme: background `#2B2B2B`, editor `#1E1F22`, SQL keywords `#CC7832`, strings `#6A8759`, numbers `#6897BB`, comments `#808080`
- Add Nerd Font icons to schema explorer nodes: `󰆼` for databases, `` for schemas, `` for tables, `󰈙` for views, `` for routines
- Add Speed Search to schema explorer: type characters to filter tree nodes in real-time
- Replace box-drawing separator lines with blank space or minimal column gaps
- Statusline communicates focus via background color (active: blue bg / black text, inactive: muted grey)
- Create a centralized `src/theme.rs` for all color/style constants

## Capabilities

### New Capabilities
- `visual-theme`: Darcula TrueColor palette, borderless rendering, centralized theme constants
- `nerd-font-icons`: Nerd Font icon rendering for database entity types in schema explorer
- `speed-search`: Real-time type-to-filter in schema explorer tree

### Modified Capabilities
- `schema-explorer`: Node rendering updated with Nerd Font icons and Speed Search filter
- `ui-rendering`: Borderless panel layout, focus-state via statusline color
- `sql-editor`: Syntax highlighting colors aligned with Darcula palette

## Impact

- `src/theme.rs`: New module with `Theme` struct, color constants, icon mappings
- `src/ui.rs`: Remove `Borders::ALL` from all panels; update statusline rendering for focus states; use theme colors
- `src/explorer.rs`: Add `search_query: String` field, `filter()` method for Speed Search; update icon rendering
- `src/editor.rs`: Update highlight color constants to use Darcula palette tokens
- `src/state.rs`: Speed Search key handling in explorer mode
