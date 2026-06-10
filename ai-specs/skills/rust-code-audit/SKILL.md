---
name: rust-code-audit
description: Audit Rust code for unsafe blocks, unwrap usage, error handling patterns, and async correctness in twisterDBA.
author: twisterDBA
version: 1.0.0
---

# rust-code-audit Skill

Audit twisterDBA Rust code for safety, correctness, and TEA patterns.

## Instructions

1. **Search for unsafe blocks**
   ```bash
   rg "unsafe" src/
   ```
   Each unsafe block must have a safety comment explaining why it's necessary.

2. **Audit unwrap/expect usage**
   ```bash
   rg "\.unwrap\(\)" src/
   rg "\.expect\(" src/
   ```
   Prefer `?` with anyhow for error propagation. Audit each case.

3. **Check async patterns**
   - Verify all DB I/O happens via Tokio tasks, not in render loop
   - Verify mpsc channels are unbounded (prevent deadlocks)
   - Verify CancellationToken usage for query cancellation

4. **Check TEA architecture**
   - AppState must not be behind Arc<Mutex<>>
   - Render must be a pure function of state
   - All state mutation via event handlers, not during render

5. **Error handling**
   - All fallible operations use anyhow::Result or custom error types
   - Error paths log to tracing, not to terminal

6. **Report**
   Output findings grouped by severity: CRITICAL, WARNING, SUGGESTION.
