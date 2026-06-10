## ADDED Requirements

### Requirement: Statement Extraction via AST Walk
The editor SHALL use `tree-sitter::QueryCursor` with a query matching `statement` nodes to find the one containing or nearest to the cursor byte position.

#### Scenario: Cursor at start of statement
- **WHEN** the cursor is at the first character of `SELECT * FROM users;` (byte 0 of the statement)
- **THEN** `extract_active_statement()` returns the byte range spanning the entire `SELECT * FROM users;`

#### Scenario: Cursor in middle of statement
- **WHEN** the cursor is at the `F` in `FROM` within `SELECT * FROM users WHERE id = 1;`
- **THEN** `extract_active_statement()` returns the byte range of the entire statement from `SELECT` to `;`

#### Scenario: Cursor at end of statement after semicolon
- **WHEN** the cursor is on the blank line after `SELECT 1;`
- **THEN** `extract_active_statement()` returns `None` — there is no statement at the cursor position

#### Scenario: Multiple statements on one line
- **WHEN** the buffer contains `SELECT 1; SELECT 2; SELECT 3;` and cursor is on `2`
- **THEN** `extract_active_statement()` returns only `SELECT 2;`

### Requirement: Statement-Aware Execute Keybinding
In both Normal and Insert modes, a configurable execute binding SHALL call `extract_active_statement()` and send the result to the DB layer. The editor SHALL show a visual indicator (brief highlight or flash) of the executed statement range.

#### Scenario: Normal mode execution
- **WHEN** editor is in Normal mode and user presses `Ctrl+E`
- **THEN** the active statement is extracted, submitted to the DB, and the editor enters a "running" state (spinner, disabled editing)

#### Scenario: Insert mode execution
- **WHEN** editor is in Insert mode and user presses `Ctrl+E`
- **THEN** the active statement is extracted and submitted; editor stays in Insert mode but shows running indicator

#### Scenario: Query in progress blocks re-execution
- **WHEN** a query is already running and user presses `Ctrl+E` again
- **THEN** the second execute is ignored (no double-submit); status bar shows "Query already running"
