## ADR-001: Window enum replaces Panel

**Decision**: Replace `Panel { SchemaExplorer, QueryEditor, ResultGrid, Output }` with `Window { SchemaExplorer, QueryEditor, OutputResults }`. Output and Results merge into a single window with tabs because they share the same screen area and users treat them as one viewport with two content modes.

**Rationale**:
- Spatial navigation (Ctrl+h/j/k/l) needs 3 fixed layout regions, not 4 arbitrary panels
- Output and Results occupy the same bottom-right area; switching between them is a tab operation, not a window operation
- Simplifies the focus model: 3 windows vs 4 panels

**Alternatives considered**:
- Keep 4 panels and add window concept on top: Adds indirection without benefit
- 2 windows (left/right): Loses the editor/output vertical split

## ADR-002: EditorSplit with Vec<SqlEditor> tabs

**Decision**: Replace `editors: Vec<SqlEditor>` (flat, one buffer per split) with `editor_splits: Vec<EditorSplit>` where each `EditorSplit` contains `tabs: Vec<SqlEditor>` and `active_tab: usize`.

```
pub struct EditorSplit {
    pub tabs: Vec<SqlEditor>,
    pub active_tab: usize,
}
```

**Rationale**:
- Each split pane is a spatial region; tabs within it are content buffers
- SqlEditor is heavy (parser, history, etc.) but independent per tab — memory cost is acceptable for ≤8 tabs per split
- Maps directly to Neovim's "window contains buffers" model
- `SplitDirection` remains unchanged; layout applies to splits, not tabs

**Limits**: Max 8 splits, max 8 tabs per split (enforced in push methods).

## ADR-003: OutputResultsState merges Output + Results with tabs

**Decision**: Merge `output_pane: OutputPaneState` and `results: Vec<ResultTab>` into one `OutputResultsState`:

```
pub struct OutputResultsState {
    pub output: OutputPaneState,
    pub result_tabs: Vec<ResultTab>,
    pub active_tab: usize,  // 0 = output, >=1 = results
}
```

Tab 0 is always Output; tabs 1..N are result grids. Tab/Shift+Tab navigates between them.

**Rationale**:
- Both share the bottom-right 40% area; user expectation is tab switching, not panel switching
- Consistent with DataGrip's "Output / Results" tab bar
- No layout change needed; just a tab bar inside the existing output area

## ADR-004: Directional window navigation with Ctrl+h/j/k/l

**Decision**: Ctrl+h/j/k/l (without Ctrl+w prefix) navigates between windows directionally. Ctrl+w h/j/k/l does the same, plus navigates between editor splits within the QueryEditor window.

**Navigation matrix** (fixed layout: Schema left, Editor right-top, OutputResults right-bottom):

| From | Ctrl+h | Ctrl+j | Ctrl+k | Ctrl+l |
|------|--------|--------|--------|--------|
| SchemaExplorer | — | OutputResults | — | QueryEditor |
| QueryEditor | SchemaExplorer | OutputResults | — | — |
| OutputResults | SchemaExplorer | — | QueryEditor | — |

When QueryEditor has multiple splits active, Ctrl+w h/l navigates between them (h=previous, l=next); Ctrl+w j/k navigates to OutputResults/SchemaExplorer.

**Rationale**:
- Ctrl+arrow keys are the standard Neovim window navigation (vim-tmux-navigator pattern)
- h/j/k/l mnemonics: h=left, j=down, k=up, l=right — intuitive
- Ctrl+w prefix provides the "full" window command palette for splits and equals

## ADR-005: Tab navigation via Tab/Shift+Tab

**Decision**: Tab navigates to the next tab within the current window/split. Shift+Tab navigates to previous. Panel cycling is removed entirely.

In QueryEditor: Tab/Shift+Tab cycles through tabs in the active split.
In OutputResults: Tab/Shift+Tab cycles through Output → Result 1 → Result 2 → ... → Output.

**Rationale**:
- Tab key is universally understood as "next tab"
- Frees up the concept of "next panel" for window-specific commands
- Consistent with every modern editor/IDE

## ADR-006: Result-to-editor linkage

**Decision**: `ResultTab` gains `source_split: usize` and `source_tab: usize` fields. When an editor executes a query, the resulting ResultTab records the (split_index, tab_index) that produced it. When an editor tab is closed, any ResultTabs linked to (split_index, tab_index) are automatically closed. When an entire split is closed, all its tabs' results close.

**Rebalancing**: When tabs/splits are removed, remaining source references are re-indexed to keep them valid.

**Rationale**:
- Matches DataGrip's "query console → result" model
- Prevents orphaned results from deleted editor tabs
- Re-indexing prevents stale references after tab reordering

## ADR-007: Pure render, no side effects

All window/tab state lives in AppState. `render()` remains a pure function of `&AppState`. Tab bar rendering queries `editor_splits[*].tabs[*].buffer` for content previews without mutation.

## ADR-008: Session persistence extends to tabs

`SessionData` gains per-split tab arrays. `BufferSnapshot` already captures content/cursor/scroll. Session save/restore iterates all splits and all tabs.
