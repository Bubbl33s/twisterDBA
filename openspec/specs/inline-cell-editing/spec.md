## ADDED Requirements

### Requirement: Inline Cell Edit Mode
Pressing `e` on a selected cell in Normal mode SHALL enter inline edit mode. The cell value SHALL become editable in-place (not in a popup). Pressing `Enter` SHALL commit an `UPDATE` statement to the database. Pressing `Escape` SHALL cancel editing.

#### Scenario: Enter edit mode on a cell
- **WHEN** the result grid has rows, user navigates to row 3 column "name", and presses `e`
- **THEN** the cell highlights with a blinking cursor at the end of the value; the rest of the grid remains visible but unselectable

#### Scenario: Edit and commit a cell value
- **WHEN** in edit mode on cell "John", user backspaces to clear, types "Jane", and presses Enter
- **THEN** an `UPDATE <table> SET <column> = 'Jane' WHERE <pk_column> = <pk_value>` is sent to the DB; on success, the grid updates the cell value to "Jane"

#### Scenario: Cancel editing preserves original value
- **WHEN** in edit mode on cell "John", user types "Jane" but presses Escape
- **THEN** the cell reverts to "John", edit mode exits, and no query is sent

#### Scenario: Edit mode visual indicator
- **WHEN** a cell is in edit mode
- **THEN** the cell background changes to a distinct edit color (e.g., dark blue `#214283`) and the cursor blinks

#### Scenario: NULL cell editing
- **WHEN** user presses `e` on a cell displaying the NULL indicator (`∅`)
- **THEN** the edit field starts empty; typing a non-empty value and pressing Enter sets the cell to that value; leaving empty and pressing Enter sends `UPDATE ... SET col = NULL`

#### Scenario: Edit primary key column prevented
- **WHEN** user attempts to edit a primary key column cell
- **THEN** status bar shows "Cannot edit primary key column" and edit mode does not activate

### Requirement: Primary Key Detection for UPDATE
The application SHALL detect the primary key column(s) of the source table to construct valid `WHERE` clauses for `UPDATE` statements.

#### Scenario: Single-column primary key
- **WHEN** the result grid comes from `SELECT * FROM users` and `users` has PK `id`
- **THEN** editing cell in column "name" constructs `UPDATE users SET name = 'new_value' WHERE id = <row_id_value>`

#### Scenario: Composite primary key
- **WHEN** the result grid comes from a table with composite PK `(user_id, group_id)`
- **THEN** editing a cell constructs `UPDATE ... WHERE user_id = <val1> AND group_id = <val2>`

#### Scenario: No primary key detected
- **WHEN** the result grid comes from a query without identifiable source table (e.g., `SELECT 1+1`, or a join without PK metadata)
- **THEN** pressing `e` shows "Cannot edit: no primary key available for this result set" and edit mode does not activate

### Requirement: Edit Commit Feedback
After committing an edit, the application SHALL display the result (success with rows affected, or error message) in the output/services pane.

#### Scenario: Successful edit
- **WHEN** an `UPDATE` succeeds (1 row affected)
- **THEN** the output pane shows green "✓ 1 row updated in 3ms" and the grid updates immediately

#### Scenario: Edit fails with constraint violation
- **WHEN** an `UPDATE` fails (e.g., unique constraint)
- **THEN** the output pane shows red "✗ UPDATE failed: duplicate key value violates unique constraint..." and the grid retains the original value
