## Why

Users report confusion between the current flat panel-cycling model (Tab hops SchemaExplorer → QueryEditor → ResultGrid → Output) and the well-established Neovim window/tab navigation paradigm. The current model conflates window focus (spatial layout areas) with buffer/content switching (tabs within a panel), making it impossible to have multiple SQL buffers per editor split or to navigate directionally between panels. A hierarchical Window → Split → Tab navigation model, with distinct keybindings for each level (Ctrl+h/j/k/l for windows, Ctrl+w for window commands, Tab/Shift+Tab for tabs), brings predictable Neovim-like navigation to twisterDBA.

## What Changes

- **New ``Window`` enum**: Replaces ``Panel`` with 3 spatial windows: SchemaExplorer, QueryEditor, OutputResults
- **Editor tabs per split**: Each editor split pane gets a tab bar (BufferLine); Space+b creates a new buffer, Space+x closes the current one
- **Merge Output + Results**: Output and ResultGrid become tabs within a single OutputResults window; Tab/Shift+Tab navigates between them
- **Directional window navigation**: Ctrl+h/j/k/l navigates between windows spatially (left/down/up/right)
- **Extended Ctrl+w commands**: Ctrl+w o closes other editor splits; Ctrl+w w cycles all windows; Ctrl+w h/j/k/l navigates all windows including editor splits
- **Tab navigation replaces panel cycling**: Tab/Shift+Tab navigates tabs/buffers within the current window/split, not panels
- **Buffer reordering**: Ctrl+Shift+h/l moves the current tab left/right in the tab bar
- **Result-to-editor linkage**: Each ResultTab records which editor (split + tab) generated it; closing that editor closes its results too
- **Breaking**: Tab key no longer cycles panels; focused_panel field replaced by focused_window; editors field becomes editor_splits: Vec\<EditorSplit\>

## Capabilities

### New Capabilities
- `window-tab-navigation`: Hierarchical Window/Split/Tab navigation with Neovim-style directional window movement and tab management
- `editor-tab-management`: Per-split editor tabs with create (Space+b), close (Space+x), reorder (Ctrl+Shift+h/l)
- `output-results-tabs`: Merged Output+Results window with tab bar separating Output log from Result grids

### Modified Capabilities
- `split-window-editing`: Editors restructured as splits containing tabs instead of flat Vec of single-buffer editors; Ctrl+w navigation extended to all windows
- `output-services-pane`: Output pane merged into OutputResults window as a tab alongside result tabs
- `session-persistence`: Must persist tab state per split plus result-to-editor linkage

## Impact

- `src/state.rs`: Major refactor — Window enum, EditorSplit struct, OutputResultsState struct, all keybindings, push/close/focus for tabs and splits
- `src/ui.rs`: BufferLine rendering for editor splits and OutputResults, merged output+results layout
- `src/keymap_help.rs`: Update all keybinding definitions for new model
- `src/editor/mod.rs`: Minimal change (SqlEditor unchanged, used as tab content)
- `src/result.rs`: Add source_split/source_tab fields to ResultTab
- `src/explorer.rs`: Adapt to Window-focused model (remove Panel references)
- `src/config/mod.rs`: May need new configurable keybindings
