## Spec: Window-Tab Navigation

### Requirement: Three spatial windows replace four panels

The application SHALL present exactly 3 navigable windows: SchemaExplorer (left, full height), QueryEditor (right-top, ~60%), and OutputResults (right-bottom, ~40%). Output and ResultGrid are tabs within OutputResults, not separate panels.

#### Scenario: Tab cycles within OutputResults window
- GIVEN focus is on the OutputResults window
- WHEN user presses Tab
- THEN focus moves from Output tab to Result tab 1
- WHEN user presses Tab again
- THEN focus moves to Result tab 2 (if exists)
- WHEN user presses Shift+Tab
- THEN focus moves back to previous tab

#### Scenario: Output and Results share the same screen area
- GIVEN OutputResults window is rendered
- WHEN active tab is 0 (Output)
- THEN the output log with its scroll buffer is displayed
- WHEN active tab is 1 (a ResultGrid)
- THEN the result grid table is displayed
- AND both share the same spatial region
- AND a tab bar at the top shows "Output | Result 1 | Result 2"

### Requirement: Directional window navigation via Ctrl+h/j/k/l

Pressing Ctrl+h/j/k/l in Normal mode SHALL move focus to the adjacent window spatially.

#### Scenario: Navigate left from QueryEditor to SchemaExplorer
- GIVEN focus is on QueryEditor window
- WHEN user presses Ctrl+h
- THEN focus moves to SchemaExplorer window

#### Scenario: Navigate down from QueryEditor to OutputResults
- GIVEN focus is on QueryEditor window
- WHEN user presses Ctrl+j
- THEN focus moves to OutputResults window

#### Scenario: Navigate up from OutputResults to QueryEditor
- GIVEN focus is on OutputResults window
- WHEN user presses Ctrl+k
- THEN focus moves to QueryEditor window

#### Scenario: Navigate right from SchemaExplorer to QueryEditor
- GIVEN focus is on SchemaExplorer window
- WHEN user presses Ctrl+l
- THEN focus moves to QueryEditor window

#### Scenario: Ctrl+h/j/k/l do not interfere with editor cursor movement
- GIVEN focus is on QueryEditor window in Normal mode
- WHEN user presses `h` (without Ctrl)
- THEN the editor cursor moves left (vim behavior preserved)

### Requirement: Editor splits contain tabs

Each editor split SHALL have its own tab bar (BufferLine) containing one or more SQL editor buffers.

#### Scenario: Create a new buffer in current split
- GIVEN focus is on QueryEditor window, split 0, tab 0
- WHEN user presses Space+b
- THEN a new empty SqlEditor tab is created in split 0
- AND the new tab becomes active
- AND tab bar shows both tabs (e.g., "Query 1 | Query 2")

#### Scenario: Close current buffer via Space+x
- GIVEN focus is on QueryEditor window, split 0 with 2 tabs
- WHEN user presses Space+x
- THEN the active tab is closed
- AND remaining tab becomes active
- WHEN only 1 tab remains and user presses Space+x
- THEN the tab is NOT closed (last buffer prevention)

#### Scenario: Navigate between tabs
- GIVEN focus is on QueryEditor window, split 0 with 3 tabs
- WHEN user presses Tab
- THEN active tab advances to next
- WHEN user presses Shift+Tab
- THEN active tab goes to previous

### Requirement: Reorder tabs with Ctrl+Shift+h/l

Pressing Ctrl+Shift+h or Ctrl+Shift+l in Normal mode SHALL move the current tab left or right in the tab bar.

#### Scenario: Move tab right
- GIVEN a split with tabs [A, B, C] and active tab is A
- WHEN user presses Ctrl+Shift+l
- THEN tab order becomes [B, A, C] and A remains active

#### Scenario: Move tab left
- GIVEN a split with tabs [A, B, C] and active tab is C
- WHEN user presses Ctrl+Shift+h
- THEN tab order becomes [A, C, B] and C remains active

### Requirement: Ctrl+w commands operate on windows and splits

Ctrl+w prefix SHALL open the window command sub-mode. Available commands:

#### Scenario: Create vertical split
- GIVEN focus is on QueryEditor window
- WHEN user presses Ctrl+w v
- THEN a new editor split is created with one tab
- AND layout adjusts to accommodate the new split proportionally

#### Scenario: Create horizontal split
- GIVEN focus is on QueryEditor window
- WHEN user presses Ctrl+w s
- THEN a new editor split is created with one tab
- AND split_direction is set to Horizontal

#### Scenario: Close current split
- GIVEN QueryEditor window has 2 splits
- WHEN user presses Ctrl+w q
- THEN the active split is closed
- AND its tabs (and linked results) are removed
- WHEN only 1 split remains and user presses Ctrl+w q
- THEN the split is NOT closed (last split prevention)

#### Scenario: Close other splits
- GIVEN QueryEditor window has 3 splits
- WHEN user presses Ctrl+w o
- THEN all splits except the active one are closed
- AND their tabs and linked results are removed

#### Scenario: Navigate between splits
- GIVEN QueryEditor window has 2 splits (Vertical)
- WHEN user presses Ctrl+w l
- THEN focus moves to the next split (right)
- WHEN user presses Ctrl+w h
- THEN focus moves to the previous split (left)

#### Scenario: Cycle all windows
- GIVEN focus is on SchemaExplorer
- WHEN user presses Ctrl+w w
- THEN focus moves to QueryEditor
- WHEN user presses Ctrl+w w again
- THEN focus moves to OutputResults
- WHEN user presses Ctrl+w w again
- THEN focus wraps back to SchemaExplorer

### Requirement: Results are linked to their source editor

Each ResultTab SHALL record which editor (split + tab) generated it. Closing an editor tab or split SHALL close its linked results.

#### Scenario: Close editor tab closes its results
- GIVEN QueryEditor split 0, tab 1 executed a query producing Result tab 3
- WHEN user closes QueryEditor split 0, tab 1 via Space+x
- THEN Result tab 3 is automatically closed
- AND remaining result tabs are re-indexed

#### Scenario: Close editor split closes all its tabs' results
- GIVEN QueryEditor split 1, tab 0 produced Result tab 2 and split 1, tab 1 produced Result tab 4
- WHEN user closes QueryEditor split 1 via Ctrl+w q
- THEN Result tabs 2 and 4 are closed
- AND remaining result tabs re-indexed
