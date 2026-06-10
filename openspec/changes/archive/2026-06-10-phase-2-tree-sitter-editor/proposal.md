## Why

The current SQL syntax highlighting in `src/editor.rs` uses hand-rolled byte scanning with a keyword list — it breaks on complex SQL (nested CTEs, dollar-quoted strings, compound statements). The product spec mandates **Tree-sitter** for accurate AST parsing, context-aware statement execution (run only the statement under cursor), and split-window editing. Tree-sitter provides production-grade SQL parsing without regex fragility, and its byte-range API guarantees UTF-8 safety.

## What Changes

- Replace hand-rolled `highlight_sql()` with `tree-sitter-sql` grammar and token-based styling
- Add `src/editor/tree.rs` for Tree-sitter init, parsing, and statement extraction
- Implement context-aware execution: `<leader>r` finds the `statement` node containing the cursor and sends only that SQL slice
- Handle CTEs (WITH clauses), semicolons inside string literals, and multi-statement files correctly
- Add split-window support: `AppState` manages a `Vec<SqlEditor>` with a `focused_buffer` index
- All buffer slices use `node.start_byte()` / `node.end_byte()` — no `char boundary` panics

## Capabilities

### New Capabilities
- `tree-sitter-syntax`: Accurate SQL syntax highlighting using `tree-sitter-sql` grammar with AST-token color mapping
- `context-aware-execution`: Statement-under-cursor extraction and execution, handling CTEs, strings, and multi-statement files
- `split-window-editing`: Multiple editor buffers side-by-side with vim-style window navigation

### Modified Capabilities
- `sql-editor`: Highlighting engine replaced from regex to Tree-sitter; buffer management extended for split windows

## Impact

- `Cargo.toml`: Add `tree-sitter = "0.24"`, `tree-sitter-sql = "0.1"`, `cc = "1"` for grammar compilation
- `src/editor.rs`: `highlight_sql` replaced; `execute()` updated for context-aware extraction
- `src/editor/tree.rs`: New module for Tree-sitter init, tree parsing, statement node extraction
- `src/state.rs`: `AppState.editor` becomes `Vec<SqlEditor>`; new `focused_buffer: usize` field
- `src/ui.rs`: `render_sql_editor` handles multiple buffers with split layout and separator
- `src/app.rs`: `handle_key` routes to focused buffer; new split/create/close buffer bindings
- `build.rs`: Compile `tree-sitter-sql` grammar at build time
