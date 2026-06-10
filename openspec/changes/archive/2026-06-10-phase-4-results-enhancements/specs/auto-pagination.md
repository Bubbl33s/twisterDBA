## ADDED Requirements

### Requirement: Transparent Auto-Pagination
The application SHALL automatically append `LIMIT <page_size> OFFSET <offset>` to user-submitted `SELECT` queries. When the user scrolls to the last visible row, the next page SHALL load asynchronously without blocking the UI.

#### Scenario: LIMIT injected into simple SELECT
- **WHEN** user executes `SELECT * FROM users` with page_size=200
- **THEN** the query sent to the DB is `SELECT * FROM users LIMIT 201 OFFSET 0` (page_size+1 to detect more rows)

#### Scenario: Next page loads on scroll to bottom
- **WHEN** results grid shows 200 rows from page 1, and user scrolls to row 200 (last visible row)
- **THEN** the next page (`LIMIT 201 OFFSET 200`) is fetched automatically and appended to the grid, spinner shows briefly, and user can continue scrolling

#### Scenario: Pagination respects original ORDER BY
- **WHEN** user executes `SELECT * FROM users ORDER BY created_at DESC`
- **THEN** the injected pagination wraps the query: `SELECT * FROM (SELECT * FROM users ORDER BY created_at DESC) AS _twister_page LIMIT 201 OFFSET 0`

#### Scenario: Non-SELECT queries skip pagination
- **WHEN** user executes `UPDATE users SET active = true` or `CREATE TABLE test (...)` or `EXPLAIN SELECT *`
- **THEN** no LIMIT/OFFSET is injected; the query runs as-is

#### Scenario: Query with existing LIMIT preserved
- **WHEN** user executes `SELECT * FROM users LIMIT 50`
- **THEN** the user's `LIMIT 50` is respected (no injection); auto-pagination is disabled for this query

#### Scenario: Pagination toggle via keybinding
- **WHEN** user presses `Ctrl+P` in Normal mode (editor focused)
- **THEN** auto-pagination toggles on/off; status bar shows "Auto-pagination: on" or "Auto-pagination: off"

### Requirement: Async Page Loading
Page fetches SHALL run in the Tokio DB task and SHALL NOT block the render loop. The result grid SHALL show a spinner in the status bar while fetching, and rows SHALL appear as they stream in.

#### Scenario: Spinner during page fetch
- **WHEN** a new page is being fetched after scrolling to bottom
- **THEN** the status bar shows the spinner with "Loading page 3..." and the user can still navigate other panels

#### Scenario: No more rows — pagination stops
- **WHEN** page N returns fewer rows than page_size
- **THEN** the grid marks "end of results" and no further page fetches are triggered

#### Scenario: Query cancellation during page load
- **WHEN** a page fetch is in progress and user presses Ctrl+C
- **THEN** the fetch is cancelled, the grid remains with previously loaded rows, and editor exits executing state
