## ADDED Requirements

### Requirement: Output/Services Pane Layout
The application SHALL display a fourth panel (bottom-left) titled "Output" that shows connection progress, query execution metadata, and error traces. This pane is independent of the status bar.

#### Scenario: Pane position in layout
- **WHEN** the main area renders
- **THEN** the Output pane occupies the bottom-left quadrant, to the left of the Results grid (or below the Schema Explorer in narrow layouts)

#### Scenario: Pane shows connection progress
- **WHEN** a database connection is being established
- **THEN** the Output pane shows "Connecting to postgresql://user@host:5432/db ..." with timestamp

#### Scenario: Pane shows query execution metadata
- **WHEN** a query completes successfully
- **THEN** the Output pane appends a line: `[14:32:05] SELECT completed: 42 rows, 12ms` — and scrolls to the bottom

#### Scenario: Pane shows error traces
- **WHEN** a query fails with an error
- **THEN** the Output pane appends the full error message in red, including the SQL state code if available (e.g., `[14:32:08] ERROR: relation "nonexistent" does not exist (42P01)`)

#### Scenario: Pane scrollback
- **WHEN** the Output pane has more lines than its viewport
- **THEN** the user can scroll up/down within the output pane using `j`/`k` when the pane is focused via Tab

### Requirement: Per-Query-Console Independence
Each editor buffer (split window) SHALL have its own independent output state and result set.

#### Scenario: Two buffers, two independent outputs
- **WHEN** buffer A executes `SELECT 1` and buffer B executes `SELECT x FROM bad_table` (error)
- **THEN** buffer A's output shows success, buffer B's output shows error; switching focus between buffers switches the displayed output pane content

#### Scenario: Buffer close clears output
- **WHEN** a buffer is closed via `Ctrl+W q`
- **THEN** its associated output and result data is freed

### Requirement: Output Pane Focus and Interaction
The Output pane SHALL be focusable via `Tab` cycling. When focused, `j`/`k` scroll, `y` copies selected line, and `g`/`G` navigate to top/bottom.

#### Scenario: Tab cycles to Output pane
- **WHEN** user presses Tab from Result Grid
- **THEN** focus moves to Output pane (after Schema Explorer → Query Editor → Result Grid → Output cycle)

#### Scenario: Copy output line
- **WHEN** Output pane is focused, a line is selected, and user presses `y`
- **THEN** the line's text is copied to the system clipboard
