## ADDED Requirements

### Requirement: Context-Sensitive Keymap Help
Pressing `?` SHALL display a floating popup listing all valid keybindings for the currently focused panel and active mode. The help text SHALL update if the user switches panels or modes while the popup is open.

#### Scenario: Help in Normal mode with Schema Explorer focused
- **WHEN** Schema Explorer is focused, mode is Normal, and user presses `?`
- **THEN** the popup shows Normal mode keybindings for the explorer: `j/k` navigate, `l` expand, `h` collapse, `R` reload, `Tab` switch to editor, `K` quick doc, `?` help

#### Scenario: Help in Insert mode with Query Editor focused
- **WHEN** Query Editor is focused, mode is Insert, and user presses `?`
- **THEN** the popup shows Insert mode keybindings: `Escape` to Normal, `Ctrl+E` execute, `Ctrl+C` cancel, and standard text input behavior

#### Scenario: Help updates on panel change while popup is open
- **WHEN** Keymap Help popup is open showing Explorer bindings, and user presses `Tab` switching focus to Query Editor
- **THEN** the popup contents update to show Query Editor bindings without closing and reopening

#### Scenario: Close help popup
- **WHEN** Keymap Help popup is open and user presses `Escape`, `?`, or `q`
- **THEN** the popup closes and focus returns to the previously focused panel

#### Scenario: Help respects terminal size
- **WHEN** the terminal is smaller than the help content (e.g., 80x24)
- **THEN** the popup is scrollable with `j`/`k` within the popup

### Requirement: Visual Format
The Keymap Help SHALL render as a two-column table: left column shows the key sequence, right column shows the action description. The popup SHALL have a title showing the active panel name and mode.

#### Scenario: Two-column layout
- **WHEN** Keymap Help is displayed for Query Editor Normal mode
- **THEN** rows show `Ctrl+E` | `Execute statement`, `Ctrl+C` | `Cancel query`, `Ctrl+P/N` | `History prev/next`, `i` | `Enter Insert mode`, etc.
