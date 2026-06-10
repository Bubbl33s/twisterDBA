## ADDED Requirements

### Requirement: Connection Sources Persist in Session
The session SHALL persist the list of connected source names. On application restart, saved sources SHALL be automatically reconnected and restored in the explorer.

#### Scenario: Session saves connected sources
- **WHEN** the user has "local-postgres" and "prod-mysql" connected and quits
- **THEN** `session.toml` contains `saved_sources = ["local-postgres", "prod-mysql"]`
- **AND** `active_connection = "local-postgres"` (or whichever was active)

#### Scenario: Auto-reconnect on startup
- **WHEN** the app starts with saved sources in session
- **THEN** the app attempts to reconnect to each saved source using its profile
- **AND** successfully reconnected sources appear in the explorer
- **AND** failed reconnections show as "error" status

#### Scenario: Auto-reconnect preserves source order
- **WHEN** sources are saved in order ["local-postgres", "prod-mysql"]
- **THEN** after auto-reconnect, sources appear in the same order in the explorer

#### Scenario: Manual disconnect removes from saved sources
- **WHEN** the user manually disconnects a source via `:disconnect`
- **THEN** the source is removed from the saved sources list in session
- **AND** it will not auto-reconnect on next startup

### Requirement: Source Removal from Explorer
When a connection is fully disconnected, its source SHALL be removed from the explorer.

#### Scenario: Disconnect removes source
- **WHEN** the user disconnects from "prod-mysql"
- **THEN** the "prod-mysql" source node is removed from the explorer
- **AND** if it was the active connection, `active_connection` is cleared

#### Scenario: Last source disconnected
- **WHEN** the user disconnects from the last remaining source
- **THEN** the explorer shows "(no connections)"
- **AND** `active_connection` is `None`
