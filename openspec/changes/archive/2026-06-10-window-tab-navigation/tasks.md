## 1. Data Model Refactor (`src/state.rs`)

- [ ] 1.1 Define `Window` enum: `SchemaExplorer | QueryEditor | OutputResults`
- [ ] 1.2 Replace `focused_panel: Panel` with `focused_window: Window`
- [ ] 1.3 Define `EditorSplit` struct: `{ tabs: Vec<SqlEditor>, active_tab: usize }`
- [ ] 1.4 Replace `editors: Vec<SqlEditor>` with `editor_splits: Vec<EditorSplit>`
- [ ] 1.5 Replace `focused_editor: usize` with `active_split: usize`
- [ ] 1.6 Define `OutputResultsState` struct merging output + result_tabs + active_tab
- [ ] 1.7 Replace `output_pane: OutputPaneState` and `results: Vec<ResultTab>` with `output_results: OutputResultsState`
- [ ] 1.8 Add `source_split: usize` and `source_tab: usize` to `ResultTab`
- [ ] 1.9 Update `AppState::new()` to initialize new structures
- [ ] 1.10 Run `cargo build` — expect many errors from dependent code; fix all compile errors in state.rs first

## 2. Editor Tab Management (`src/state.rs`)

- [ ] 2.1 Implement `push_tab(split_idx)` — add new SqlEditor tab to split
- [ ] 2.2 Implement `close_tab(split_idx, tab_idx)` — remove tab, prevent last tab closure, re-index linked results
- [ ] 2.3 Implement `focus_tab(split_idx, tab_idx)` — set active tab for split
- [ ] 2.4 Implement `next_tab()` / `prev_tab()` — wrap-around tab navigation in active split
- [ ] 2.5 Implement `move_tab_left()` / `move_tab_right()` — reorder tabs in active split
- [ ] 2.6 Update `push_editor(direction)` to push a new EditorSplit with 1 default tab
- [ ] 2.7 Update `close_editor(index)` to remove EditorSplit and its linked results
- [ ] 2.8 Update `close_other_editors(index)` — new method for Ctrl+w o
- [ ] 2.9 Add `focused_split()` / `focused_split_mut()` accessors
- [ ] 2.10 Run `cargo build` and fix compile errors

## 3. OutputResults Tab Management (`src/state.rs`)

- [ ] 3.1 Implement `OutputResultsState::new()` with empty output + 1 default result tab
- [ ] 3.2 Implement `next_tab()` / `prev_tab()` for OutputResults (wrap around output + results)
- [ ] 3.3 Implement `create_result_tab()` recording source_split and source_tab
- [ ] 3.4 Implement `close_result_tab(index)` with linked-results cleanup
- [ ] 3.5 Add `active_result_tab()` / `active_result_tab_mut()` accessors
- [ ] 3.6 Run `cargo build` and fix compile errors

## 4. Window Navigation Keybindings (`src/state.rs`)

- [ ] 4.1 Implement `navigate_window(direction: Direction)` — directional window focus change
- [ ] 4.2 Implement `cycle_windows()` — wrap-around window cycling
- [ ] 4.3 Add Ctrl+h/j/k/l handling in `handle_normal_key` BEFORE panel-specific dispatch
- [ ] 4.4 Add Ctrl+w w handling in window command dispatcher
- [ ] 4.5 Update `handle_window_command()` for new Ctrl+w h/j/k/l (navigate all windows + splits)
- [ ] 4.6 Add Ctrl+w o handler for close-other-splits
- [ ] 4.7 Ensure Ctrl+h/j/k/l do NOT interfere with editor cursor hjkl (only trigger with CONTROL modifier)
- [ ] 4.8 Run `cargo build` and verify key routing compiles

## 5. Tab Navigation Keybindings (`src/state.rs`)

- [ ] 5.1 Change Tab/Shift+Tab in Normal mode to navigate tabs (not panels) based on focused_window
- [ ] 5.2 Add Space+b handler for new tab in QueryEditor window
- [ ] 5.3 Add Space+x handler for close tab in QueryEditor and OutputResults windows
- [ ] 5.4 Add Ctrl+Shift+h/l handlers for tab reordering in QueryEditor window
- [ ] 5.5 Remove all panel-cycling Tab/BackTab logic from handle_explorer_key, handle_editor_normal_key, handle_result_normal_key, handle_output_key
- [ ] 5.6 Update Insert mode Tab (currently inserts spaces) — keep spaces insertion, tab navigation only in Normal mode
- [ ] 5.7 Run `cargo build` and verify key routing

## 6. UI: BufferLine Tab Bars (`src/ui.rs`)

- [ ] 6.1 Implement `render_editor_tab_bar(f, area, split, is_focused)` — renders tabs for one editor split
- [ ] 6.2 Active tab gets highlighted style; inactive tabs get dimmed style
- [ ] 6.3 Tab labels show first N chars of buffer content or "Query N" for empty buffers
- [ ] 6.4 Implement `render_output_tab_bar(f, area, output_results, is_focused)` — renders "Output" + result tabs
- [ ] 6.5 Update `render_single_editor()` to include tab bar above the editor content
- [ ] 6.6 Update `render_output_panel()` to use new OutputResultsState with tab bar
- [ ] 6.7 Update `render_main_area()` layout to accommodate tab bars (reduce editor/output area by 1 row each)
- [ ] 6.8 Run `cargo build` and verify rendering compiles

## 7. Refactor All Panel References to Window (`src/`)

- [ ] 7.1 Update `src/explorer.rs`: replace Panel references with Window
- [ ] 7.2 Update `src/result.rs`: add source_split/source_tab fields; adapt to Window
- [ ] 7.3 Update `src/editor/mod.rs`: adapt any Panel references (minimal changes expected)
- [ ] 7.4 Update `src/app.rs`: replace focused_panel with focused_window in session, startup
- [ ] 7.5 Update `src/keymap_help.rs`: change Panel to Window; add OutputResults bindings
- [ ] 7.6 Update `src/lua/` if it references Panel (minimal)
- [ ] 7.7 Update `src/events.rs` if it references Panel (minimal)
- [ ] 7.8 Run `cargo build` and fix all remaining compile errors across the project

## 8. Session Persistence Update (`src/state.rs`)

- [ ] 8.1 Update `BufferSnapshot` / `SessionData` to include per-split tab arrays
- [ ] 8.2 Update `to_session_data()` to serialize all splits' tabs
- [ ] 8.3 Update `apply_session_data()` to restore all splits' tabs
- [ ] 8.4 Update `SessionData` to use `focused_window` instead of `focused_panel`
- [ ] 8.5 Run `cargo build` and verify session code compiles

## 9. Verification

- [ ] 9.1 Run `cargo build` and verify no errors
- [ ] 9.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [ ] 9.3 Run `cargo fmt --check` and verify formatting
- [ ] 9.4 Run `cargo test` and verify all tests pass
- [ ] 9.5 Manually test Ctrl+h/j/k/l window navigation between 3 windows
- [ ] 9.6 Manually test Tab/Shift+Tab tab navigation within editor splits and output/results
- [ ] 9.7 Manually test Space+b (new tab), Space+x (close tab)
- [ ] 9.8 Manually test Ctrl+w v/s (split), Ctrl+w q (close split), Ctrl+w o (close others)
- [ ] 9.9 Manually test Ctrl+w w (cycle windows)
- [ ] 9.10 Manually test Ctrl+Shift+h/l (reorder tabs)
- [ ] 9.11 Manually test result-to-editor linkage: close editor tab, verify its results close
