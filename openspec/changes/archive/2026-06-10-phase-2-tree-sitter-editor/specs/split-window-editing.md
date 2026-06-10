## ADDED Requirements

### Requirement: Split Window Layout
The editor area SHALL support splitting into multiple panes, arranged in a row (`│` separator) or column (`─` separator). Each pane contains an independent `SqlEditor` with its own buffer, cursor, and mode.

#### Scenario: Equal-width vertical split
- **WHEN** two buffers are open in a vertical split (left/right)
- **THEN** each buffer occupies 50% of the editor area width, separated by a single `│` character column

#### Scenario: Focus indicator
- **WHEN** two splits are open and the right split is focused
- **THEN** the right split's status line / border renders in active color (blue bg), the left split in inactive grey

#### Scenario: Buffer-independent modes
- **WHEN** split A is in Insert mode and split B is in Normal mode
- **THEN** typing inserts text in split A only; navigation keys move cursor in split B when focused

### Requirement: Window Navigation Keybindings
The application SHALL support `Ctrl+W` prefix for window commands: `h/j/k/l` for directional navigation, `s` for horizontal split, `v` for vertical split, `q` for close, `=` for equalize sizes.

#### Scenario: Focus moves clockwise
- **WHEN** two vertical splits are open, focus is on left, user presses `Ctrl+W l`
- **THEN** focus moves to the right split; pressing `Ctrl+W h` moves back to left

#### Scenario: Equalize after resize
- **WHEN** user manually resized a split and presses `Ctrl+W =`
- **THEN** all splits resize to equal proportions

#### Scenario: Close last buffer prevented
- **WHEN** only one buffer remains and user presses `Ctrl+W q`
- **THEN** the buffer is NOT closed (at least one editor buffer must exist); status bar shows "Cannot close last buffer"

### Requirement: Per-Buffer Result Sets
Each editor buffer SHALL have its own independent `ResultGrid`, displayed below or beside its editor pane. Executing a query in buffer A SHALL NOT affect buffer B's results.

#### Scenario: Independent query results
- **WHEN** two splits are open, split A runs `SELECT 1` and split B runs `SELECT 2`
- **THEN** split A's result grid shows `1`, split B's result grid shows `2` — no cross-contamination

#### Scenario: Buffer close discards results
- **WHEN** a buffer with query results is closed (`Ctrl+W q`)
- **THEN** the result grid is freed and memory is reclaimed
