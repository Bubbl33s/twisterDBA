## ADDED Requirements

### Requirement: Multiple Connection Entries in AppState
AppState SHALL track multiple connections via `connections: Vec<ConnectionEntry>` instead of a single `connection_status`. Each entry independently tracks its connection lifecycle.

#### Scenario: Two connections with different statuses
- **WHEN** "local-postgres" is connected and "prod-mysql" is connecting
- **THEN** `connections[0].status` is `Connected { ... }`
- **AND** `connections[1].status` is `Connecting { ... }`

#### Scenario: Connection entry contains engine type
- **WHEN** a PostgreSQL connection is established
- **THEN** the ConnectionEntry has `engine_type: EngineType::Postgres`
- **AND** the entry's `masked_dsn` shows `postgresql://user@host:5432/db`

### Requirement: Active Connection Tracking
AppState SHALL track `active_connection: Option<String>` to identify which connection the user is currently working with. The explorer displays the schema of the active connection.

#### Scenario: Active connection set on connect
- **WHEN** the user successfully connects to "local-postgres"
- **THEN** `active_connection` is set to `Some("local-postgres")`

#### Scenario: Active connection cleared on disconnect
- **WHEN** the user disconnects from the active connection
- **THEN** `active_connection` is set to `None`
- **AND** the explorer shows "(no schema)"

#### Scenario: Active connection switches when selecting different source
- **WHEN** the user selects a different connection source in the explorer
- **THEN** `active_connection` updates to the selected connection name
- **AND** the explorer rebuilds to show that connection's schema

### Requirement: Event Routing by Connection Name
`apply_db_event` SHALL route each DbEvent to the correct ConnectionEntry by matching `connection_name`. Schema and column events update the explorer only if they match the active connection.

#### Scenario: SchemaLoaded for active connection updates explorer
- **WHEN** `SchemaLoaded { connection_name: "local-postgres", nodes }` arrives
- **AND** `active_connection` is `Some("local-postgres")`
- **THEN** the explorer tree is updated with the new nodes

#### Scenario: SchemaLoaded for non-active connection does not update explorer
- **WHEN** `SchemaLoaded { connection_name: "prod-mysql", nodes }` arrives
- **AND** `active_connection` is `Some("local-postgres")`
- **THEN** the explorer tree is NOT updated
- **AND** the "prod-mysql" entry's status is updated to Connected

#### Scenario: Query events always route to active result grid
- **WHEN** query events arrive for any connection
- **THEN** they are processed into the active result grid (queries only run on active connection)

### Requirement: Connection Entry Persistence in Session
The session SHALL persist the `active_connection` name. On restart, the app SHALL attempt to reconnect all saved connections and restore the active connection.

#### Scenario: Session saves active connection name
- **WHEN** the session is saved with "local-postgres" as active
- **THEN** `session.toml` contains `active_connection = "local-postgres"`

#### Scenario: Session restores and reconnects
- **WHEN** the app starts with a saved session containing `active_connection = "local-postgres"`
- **THEN** the app attempts to reconnect to "local-postgres" using the saved profile
- **AND** `active_connection` is restored after successful reconnect
