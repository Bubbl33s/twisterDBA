## Context

The product spec mandates Tree-sitter for SQL parsing: "SQL syntax highlighting via Tree-sitter (accurate AST parsing, not regex)." The current `highlight_sql()` in `src/editor.rs` uses byte-scanning with a keyword whitelist — it fails on dollar-quoted strings, nested CTEs, and compound statements. The spec also requires context-aware execution ("`<leader>r` runs only the statement under the cursor") and split windows.

Tree-sitter provides a C library compiled into the binary. Its Rust bindings expose `tree-sitter` (core) and `tree-sitter-sql` (grammar). The `cc` crate compiles the C grammar at build time via `build.rs`.

## Goals / Non-Goals

**Goals:**
- Replace `highlight_sql()` with `tree_sitter::Parser`-driven token iteration
- Add `extract_active_statement(cursor_byte: usize) -> Option<Range<usize>>` using `QueryCursor`
- Support split windows via `Vec<SqlEditor>` in AppState
- Zero `char boundary` panics — all slicing uses byte offsets

**Non-Goals:**
- SQL auto-completion / IntelliSense (out of scope for MVP)
- Multiple language support (only SQL needed)
- Abstract syntax tree visualization

## Decisions

### ADR-001: Tree-sitter Grammar Compiled at Build Time
**Choice:** `build.rs` uses `cc` crate to compile `tree-sitter-sql` sources.

**Rationale:** Tree-sitter grammars are C code. Pre-built binaries aren't available for all platforms. Compiling at build time ensures the grammar is always in sync with the crate version and works on all CI targets. `cc` is a standard Rust ecosystem dependency.

**Alternative considered:** `tree-sitter` with pre-compiled `.so` — requires platform-specific packaging, fragile across distros.

### ADR-002: Statement Extraction via Query
**Choice:** Use `Query::new(language, "(statement) @stmt")?` with `QueryCursor` to find the node containing the cursor.

**Rationale:** SQL has a well-defined `statement` node in tree-sitter-sql. A query matches all statements, then the cursor byte position is checked against each match's `node.start_byte()..node.end_byte()`. This is simpler and faster than a recursive AST walk.

**Alternative considered:** Recursive descent from root — same result, more code, slower.

### ADR-003: Highlighting via Token Classes
**Choice:** Map tree-sitter token types to Ratatui `Style` structs using a `HashMap<String, Style>`. Re-highlight on every render frame (memoize with a `Vec<Span>` cache keyed by content hash).

**Rationale:** Tree-sitter emits named node types like `keyword`, `string`, `comment`, `number`. A static map is fast and easy to theme. Caching avoids re-parsing unchanged lines.

**Alternative considered:** Lazy incremental re-highlighting — premature optimization for SQL (typical buffer < 500 lines).

### ADR-004: Split Windows as Vec<SqlEditor>
**Choice:** `AppState.editors: Vec<SqlEditor>` with `focused_editor: usize`. Layout splits the editor area proportionally.

**Rationale:** Each buffer is independent (own cursor, mode, result set). A `Vec` is simpler than a tree (no nested splits needed for MVP). Proportional layout avoids fixed pixel sizes.

**Alternative considered:** Tree-based split layout (like Vim) — over-engineered for MVP; 2-4 splits with `Vec` is sufficient.

### ADR-005: Buffer Content as String, Not Rope
**Choice:** Keep `Vec<String>` (lines) for buffer content. Tree-sitter needs contiguous byte slices for parsing, so `buffer.get_content()` joins lines with `\n` before parsing.

**Rationale:** Rope data structures add complexity with marginal benefit for SQL (typical file < 10K lines). Tree-sitter parses the full buffer on every change (O(n) but n is small).

**Alternative considered:** `xi-rope` or `ropey` — adds dependency, complicates byte-offset translation for Tree-sitter.

## Risks / Trade-offs

- **[Risk] Tree-sitter parse on every keystroke is O(n)** → Mitigation: typical SQL files are <1000 lines; parsing takes <1ms. For very large files (>50K lines), add debounce.
- **[Risk] `cc` crate requires a C compiler at build time** → Mitigation: documented in README; all CI targets have gcc/clang pre-installed.
- **[Risk] `tree-sitter-sql` grammar maturity** → Mitigation: grammar covers PostgreSQL, MySQL, SQLite syntax well enough for highlighting; edge cases (e.g., PL/pgSQL) degrade gracefully to plain text.
- **[Risk] Split windows increase AppState memory** → Mitigation: limit to 8 buffers max; each buffer reuses the same Tree-sitter parser (single `Parser` instance shared).

## Migration Plan

1. Add `tree-sitter` and `tree-sitter-sql` deps, create `build.rs`
2. Implement `src/editor/tree.rs` with `Parser` wrapper, `highlight_line()`, `extract_statement()`
3. Replace `highlight_sql()` call in `ui.rs` with tree-sitter-based highlighting
4. Replace `editor.execute()` with context-aware extraction
5. Add split window management to `AppState` and `ui.rs`
6. Remove old `highlight_sql()` function and keyword list
7. No data migration needed

## Open Questions

- Should the Tree-sitter `Parser` be `thread_local!` or per-buffer? (Answer: per-buffer, since each buffer has its own source text)
- Should we support `language` injection for PL/pgSQL blocks? (Answer: out of scope for MVP; treat them as plain SQL)
