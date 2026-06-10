## ADDED Requirements

### Requirement: Command Palette with Tab Completion
The `:` command mode SHALL support Tab completion for known commands, a command history navigable with Up/Down arrows, and an extended command set.

#### Scenario: Tab completion for known commands
- **WHEN** user types `:con` and presses Tab
- **THEN** the command auto-completes to `:connect`; pressing Tab again cycles through `:connect` variants

#### Scenario: Command history navigation
- **WHEN** user presses `:` to enter command mode and presses Up arrow
- **THEN** the last executed command is recalled; pressing Up again shows the command before that

#### Scenario: Execute command from history
- **WHEN** a previous command `:export csv /tmp/dump.csv` is recalled and user presses Enter
- **THEN** the command executes as if typed fresh

### Requirement: Extended Command Set
The command palette SHALL support: toggle keyword case (`:upper` / `:lower`), run formatters (`:format`), invoke Lua commands (Phase 7), and list available commands (`:help`).

#### Scenario: Toggle keyword case to uppercase
- **WHEN** the SQL buffer contains `select * from users` and user runs `:upper`
- **THEN** SQL keywords are uppercased in place: `SELECT * FROM users`; identifiers remain unchanged

#### Scenario: Toggle keyword case to lowercase
- **WHEN** the SQL buffer contains `SELECT * FROM users` and user runs `:lower`
- **THEN** SQL keywords are lowercased: `select * from users`

#### Scenario: List available commands
- **WHEN** user runs `:help` or `:h` in command mode
- **THEN** a list of all available commands with brief descriptions appears in the output pane

#### Scenario: Unknown command
- **WHEN** user runs `:nonexistent_command`
- **THEN** the output pane shows red "Unknown command: nonexistent_command. Type :help for available commands."

### Requirement: SQL Keyword Case Toggle
The `:upper` and `:lower` commands SHALL use Tree-sitter to identify SQL keywords in the buffer and toggle their case without changing identifiers, strings, or comments.

#### Scenario: Only keywords toggled
- **WHEN** buffer contains `select name, 'select' as literal from "select_table"` and user runs `:upper`
- **THEN** the buffer becomes `SELECT name, 'select' as literal FROM "select_table"` — the string `'select'` and quoted identifier `"select_table"` are unchanged

#### Scenario: Undo after case toggle
- **WHEN** user runs `:upper` and then `:lower` on the same buffer
- **THEN** the buffer returns to its original case (or close to it, depending on original casing)
