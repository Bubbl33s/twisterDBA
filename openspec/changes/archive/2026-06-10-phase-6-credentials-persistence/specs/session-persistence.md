## ADDED Requirements

### Requirement: Session Persistence on Quit
When the application exits (via `:quit`, `q`, or terminal close), the current workspace state SHALL be serialized to `~/.local/share/twisterDBA/session.toml`. State includes: open editor buffers with content, cursor positions, scroll offsets, active connection profile, and focused panel.

#### Scenario: Quit and restart restores workspace
- **WHEN** the user has 2 split buffers with SQL content, cursor at specific positions, and a PostgreSQL connection active, then quits and restarts
- **THEN** the app reconnects to the same database, restores both buffers with content and cursor positions, and restores the focused panel

#### Scenario: Session file created on first quit
- **WHEN** the app quits for the first time
- **THEN** `~/.local/share/twisterDBA/` directory is created and `session.toml` is written

#### Scenario: Session restore on startup when file exists
- **WHEN** the app starts and `session.toml` exists from a previous session
- **THEN** the session is loaded, and the user sees their previous workspace

#### Scenario: Session restore skipped when file absent
- **WHEN** the app starts and `session.toml` does not exist (first run or deleted)
- **THEN** the app starts fresh with one empty editor buffer, no connection

#### Scenario: Corrupt session file handled gracefully
- **WHEN** `session.toml` exists but contains invalid TOML or missing fields
- **THEN** the app logs a warning, deletes the corrupt file, and starts fresh (no crash)

### Requirement: Session Data Structure
The session file SHALL serialize: connection profile name (not password — retrieved from keychain), Vec of buffer contents, cursor positions, scroll offsets, focused buffer index, focused panel, and split layout direction.

#### Scenario: Session includes buffer contents
- **WHEN** the user quits with a buffer containing `SELECT * FROM users WHERE id = 1;`
- **THEN** the session file stores the full buffer content as a string; on restore, the buffer has exactly that content

#### Scenario: Session excludes passwords
- **WHEN** the session file is read
- **THEN** no password values are present — only the profile name reference used to retrieve from keychain

### Requirement: Auto-Save During Session
The application SHALL periodically auto-save the session (every 60 seconds) to minimize data loss on crash. This is separate from the quit save.

#### Scenario: Auto-save writes session file
- **WHEN** the app has been running for 60 seconds
- **THEN** the session state is written to `session.toml` — content is identical to quit save

#### Scenario: Crash recovery uses auto-save
- **WHEN** the app crashes and is restarted within 60 seconds of the last auto-save
- **THEN** the restored session reflects the state at the time of the last auto-save
