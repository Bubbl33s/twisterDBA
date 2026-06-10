## Context

The product spec defines three results-related features: auto-pagination ("LIMIT/OFFSET injected transparently"), inline cell editing ("in-place mutation commits on save"), and an Output/Services pane ("connection progress, query latency, rows affected, error traces"). These are independent capabilities but share the same layout space (bottom half). The current code has a basic ResultGrid with manual page scrolling and a cell popup viewer — no editing, no auto-pagination, no output pane.

## Goals / Non-Goals

**Goals:**
- Wrap SELECT queries in a subquery with LIMIT/OFFSET injected automatically
- Fetch next page on scroll-to-bottom without blocking UI
- Inline cell editing with UPDATE generation using detected primary keys
- Independent Output pane per buffer with scrollback
- Tab cycling through 4 panels: Explorer → Editor → Results → Output

**Non-Goals:**
- Server-side cursor pagination (keyset pagination) — OFFSET is sufficient for MVP
- Multi-cell or row-level editing in a single commit
- Export directly from Output pane

## Decisions

### ADR-001: LIMIT/OFFSET Injection via Subquery Wrapping
**Choice:** When auto-pagination is enabled and the query is a `SELECT`, wrap it as `SELECT * FROM (<original_query>) AS _twister_page LIMIT <N> OFFSET <M>`.

**Rationale:** The user may already have `ORDER BY`, `WHERE`, `JOIN` clauses. Wrapping preserves the original semantics. Detecting `SELECT` vs non-SELECT is done by checking if the trimmed SQL starts with `SELECT` (case-insensitive). Queries already containing `LIMIT` are left untouched.

**Alternative considered:** Appending `LIMIT ... OFFSET ...` to the end — fails if the query has `UNION`, `INTERSECT`, or a trailing `FOR UPDATE`.

### ADR-002: Primary Key Detection for Edits
**Choice:** When `LoadColumns` is processed for a table, also query primary key columns from system catalogs (`pg_constraint` for PG, `information_schema.KEY_COLUMN_USAGE` for MySQL, `PRAGMA table_info` for SQLite). Store PK column names in `ColumnMeta`.

**Rationale:** UPDATE statements need a reliable `WHERE` clause. Without PK detection, inline editing would be impossible. The PK metadata is fetched once when the table node is expanded and cached.

**Alternative considered:** Heuristic detection (look for `id` column) — fragile and wrong for composite keys.

### ADR-003: Inline Edit as Overlay, Not Popup
**Choice:** When `e` is pressed on a cell, the cell enters edit mode in-place (same grid position). A `CellEditState` enum on `ResultGrid` tracks: `None | Editing { row, col, value, cursor }`.

**Rationale:** The current `CellPopupState` is a separate overlay. Replacing it with inline editing is simpler, more intuitive, and avoids the popup Z-index concern. The cell's background changes to indicate edit mode.

**Alternative considered:** Separate edit popup — adds visual clutter, harder to reference neighboring cells.

### ADR-004: Output Pane as Ring Buffer
**Choice:** `OutputPaneState` holds a `VecDeque<String>` with a maximum of 500 lines (ring buffer). Each buffer (split window) has its own `OutputPaneState`.

**Rationale:** Prevents unbounded memory growth from long-running sessions. 500 lines is enough for ~30 query executions with their metadata.

### ADR-005: Auto-Pagination Flag Per Query
**Choice:** `DbCommand::ExecuteQuery` gains `auto_paginate: bool` and `page_size: u32`. The DB client only applies wrapping when both are set.

**Rationale:** Some queries (DDL, EXPLAIN, SET) should never be paginated. The UI layer decides based on query type and user toggle.

## Risks / Trade-offs

- **[Risk] Subquery wrapping may cause performance issues for complex queries** → Mitigation: only applies when auto-paginate is on; user can disable with Ctrl+P
- **[Risk] UPDATE without explicit transaction may surprise users** → Mitigation: display "Are you sure?" confirmation for UPDATEs in a future iteration; for MVP, treat as feature
- **[Risk] PK detection requires an extra query per table** → Mitigation: query is cheap (<5ms) and cached; loaded lazily when user expands the table node
- **[Risk] Output ring buffer discard loses older entries** → Mitigation: log everything to `tracing` file (`/tmp/dbterm.log`) so full history is available

## Migration Plan

1. Add auto-pagination fields to `DbCommand::ExecuteQuery`
2. Implement LIMIT/OFFSET injection in `editor.rs::execute()`
3. Add PK detection to `db/client.rs` schema loading methods
4. Implement inline edit mode in `result.rs` and `state.rs`
5. Add `OutputPaneState` to `AppState` and render in `ui.rs`
6. Update Tab cycling to include Output pane
7. No data migration needed

## Open Questions

- Should auto-pagination be on by default? (Answer: yes, with page_size=200)
- Should inline editing support multi-line text cells? (Answer: basic single-line MVP; multi-line deferred)
