## ADDED Requirements

### Requirement: TOML Configuration File
The application SHALL read connection profiles, keybindings, and preferences from `~/.config/twisterDBA/config.toml` at startup and SHALL create a default config file if none exists.

#### Scenario: Config file found at startup
- **WHEN** `twisterDBA` starts and `~/.config/twisterDBA/config.toml` exists with valid TOML
- **THEN** connection profiles are loaded into the connection manager list, and the dropdown in ConnectDialog shows saved profiles

#### Scenario: No config file — auto-create defaults
- **WHEN** `twisterDBA` starts and no config file exists
- **THEN** the application creates `~/.config/twisterDBA/config.toml` with a commented-out example connection profile and default keybindings

#### Scenario: Invalid TOML syntax
- **WHEN** the config file contains malformed TOML
- **THEN** the application logs a `tracing::error!` with the parse error location and continues with hardcoded defaults (no crash)

### Requirement: Connection Profile Schema
The config file SHALL support named connection profiles with fields: type (postgres/mysql/sqlite), host, port, database, user, and an optional name label.

#### Scenario: Save connection from dialog to profile
- **WHEN** the user successfully connects via the ConnectDialog
- **THEN** the connection details (except password) are appended to `~/.config/twisterDBA/config.toml` as a named profile

#### Scenario: Load profile into connect dialog
- **WHEN** the ConnectDialog opens, the user presses Tab to the profile list, and selects a saved profile
- **THEN** all form fields (host, port, database, user, db type) are populated from the profile

#### Scenario: Password never written to config
- **WHEN** a connection profile is saved
- **THEN** the password field is NOT written to `config.toml` — it SHALL be stored via the OS keychain (Phase 6) or prompted at connect time

### Requirement: Keybinding Overrides
The application SHALL allow users to override default keybindings in `config.toml` under a `[keybindings]` section, mapping action names to key sequences.

#### Scenario: User overrides execute-query binding
- **WHEN** `config.toml` contains `[keybindings]` with `execute_query = "F5"` or `execute_query = "Ctrl+Enter"`
- **THEN** pressing the bound key in Normal or Insert mode in the editor triggers query execution

#### Scenario: Invalid keybinding syntax
- **WHEN** config contains an unrecognized key sequence (e.g., `execute_query = "Banana"`)
- **THEN** the application logs a warning and falls back to the default keybinding
