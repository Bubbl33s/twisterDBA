## Why

The product spec defines a results grid with auto-pagination ("LIMIT/OFFSET injected transparently; scrolling down loads the next page asynchronously"), inline cell editing ("in-place mutation commits on save"), and an independent Output/Services pane for metadata. The current implementation has basic results display, a cell popup viewer, and manual pagination via Ctrl+D/U. This phase brings the results experience to full MVP spec: transparent auto-pagination, editable cells that commit UPDATE statements, and a bottom-left output pane for connection progress, query latency, row counts, and error traces.

## What Changes

- **Auto-pagination**: Inject `LIMIT/OFFSET` into user queries transparently; detect scroll-to-bottom and fetch next page asynchronously
- **Inline cell editing**: Press `e` on a cell in Normal mode to enter an edit field; commit sends an `UPDATE` statement with the primary key
- **Output/Services pane**: New bottom-left panel showing connection progress, query execution time, rows affected, and error traces
- Per-query-console: each buffer (split window) gets independent result and output state
- Remove or rework the cell popup into the inline editor

## Capabilities

### New Capabilities
- `auto-pagination`: Transparent LIMIT/OFFSET injection with async page loading on scroll
- `inline-cell-editing`: In-place cell value editing with UPDATE commit to database
- `output-services-pane`: Metadata panel for connection progress, query timing, errors

### Modified Capabilities
- `results-grid`: Enhanced with auto-pagination logic, edit mode, and per-buffer independence
- `db-client`: Accept auto-pagination parameters in ExecuteQuery command

## Impact

- `src/result.rs`: Add pagination state (`page_size`, `current_page`), edit mode state, per-buffer ownership
- `src/db/client.rs`: `DbCommand::ExecuteQuery` gains `auto_paginate: bool` and `page_size: u32` fields
- `src/state.rs`: New `OutputPaneState` struct for output/services data; AppState gets `output_pane` field
- `src/ui.rs`: New fourth panel layout (Output/Services bottom-left), updated result grid rendering with edit mode
- `src/editor.rs`: `execute()` injects LIMIT/OFFSET when auto-pagination is enabled
