## ADDED Requirements

### Requirement: Multiple Named Database Backends
The `DbClient` SHALL manage multiple database backends concurrently using a `HashMap<String, DbBackend>` keyed by connection name. Each backend operates independently with its own connection pool.

#### Scenario: Connect to second database while first remains connected
- **WHEN** the user is connected to "local-postgres" and connects to "prod-mysql"
- **THEN** both backends exist in the HashMap simultaneously
- **AND** the "local-postgres" pool remains open and usable

#### Scenario: Reconnect to existing connection name replaces backend
- **WHEN** the user connects with name "local-postgres" and a backend with that name already exists
- **THEN** the old backend pool is closed first
- **AND** the new backend is registered under "local-postgres"

#### Scenario: Disconnect removes backend from HashMap
- **WHEN** the user disconnects from "prod-mysql"
- **THEN** the "prod-mysql" backend is removed from the HashMap
- **AND** other backends remain unaffected

### Requirement: Connection-Named DbCommand Variants
All `DbCommand` variants that target a specific backend SHALL carry a `connection_name: String` field. The DbClient SHALL dispatch each command to the backend matching the connection_name.

#### Scenario: LoadSchema targets specific connection
- **WHEN** `DbCommand::LoadSchema { connection_name: "local-postgres" }` is received
- **THEN** the schema is loaded from the "local-postgres" backend only
- **AND** the result event carries `connection_name: "local-postgres"`

#### Scenario: ExecuteQuery targets specific connection
- **WHEN** `DbCommand::ExecuteQuery { connection_name: "prod-mysql", sql: "..." }` is received
- **THEN** the query runs against the "prod-mysql" backend only
- **AND** all result events (ResultColumns, QueryRow, QueryCompleted) carry `connection_name: "prod-mysql"`

#### Scenario: Command for unknown connection name
- **WHEN** a command references a connection_name not in the HashMap
- **THEN** the DbClient sends a `ConnectionFailed` event with message "Unknown connection: <name>"
- **AND** no crash or panic occurs

### Requirement: Connection-Named DbEvent Variants
All `DbEvent` variants SHALL carry a `connection_name: String` field so the UI layer can route events to the correct connection entry.

#### Scenario: Connected event identifies source connection
- **WHEN** a PostgreSQL connection succeeds for "local-postgres"
- **THEN** `DbEvent::Connected { connection_name: "local-postgres" }` is emitted

#### Scenario: SchemaLoaded event identifies source connection
- **WHEN** schema loading completes for "prod-mysql"
- **THEN** `DbEvent::SchemaLoaded { connection_name: "prod-mysql", nodes: [...] }` is emitted

#### Scenario: Query events are routed by connection name
- **WHEN** a query on "local-postgres" returns rows
- **THEN** `DbEvent::QueryRow { connection_name: "local-postgres", cells: [...] }` is emitted for each row
- **AND** `DbEvent::QueryCompleted { connection_name: "local-postgres", ... }` is emitted at the end
