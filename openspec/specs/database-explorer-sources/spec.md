# Database Explorer Sources

## Purpose

Defines how connected database sources are displayed and managed in the Database Explorer panel, including source nodes, schema loading, selection behavior, and engine-specific icons.

## Requirements

### Requirement: Database Explorer Shows Connection Sources
The explorer SHALL display all connected database sources as top-level expandable nodes. Each source shows the engine icon, connection name, and status indicator. The explorer title SHALL be "Database Explorer".

#### Scenario: Single connected source
- **WHEN** the user connects to "local-postgres"
- **THEN** the Database Explorer shows one root node: `🐘 local-postgres ●` (green status)
- **AND** the node is expandable; expanding reveals the schema tree

#### Scenario: Multiple connected sources
- **WHEN** the user has connections to "local-postgres" and "prod-mysql"
- **THEN** the explorer shows two root nodes, each with their engine icon
- **AND** each node can be expanded independently to show its schema

#### Scenario: Disconnected source
- **WHEN** a source's connection is lost or disconnected
- **THEN** the source node shows a gray/red status indicator
- **AND** the schema tree is cleared for that source

#### Scenario: Connecting source shows spinner
- **WHEN** a source is in the process of connecting
- **THEN** the source node shows a yellow status indicator with spinner animation
- **AND** the node is not yet expandable

### Requirement: Source Expansion Loads Schema
Expanding a source node SHALL trigger schema loading for that connection if not already loaded.

#### Scenario: Expand connected source with loaded schema
- **WHEN** the user expands a source that is connected and has a loaded schema
- **THEN** the schema tree is revealed immediately (no loading indicator)

#### Scenario: Expand connected source without loaded schema
- **WHEN** the user expands a source that is connected but schema not yet loaded
- **THEN** a "Loading..." indicator appears under the source
- **AND** `DbCommand::LoadSchema { connection_name }` is sent
- **AND** when schema loads, the indicator is replaced with the schema tree

#### Scenario: Expand disconnected source
- **WHEN** the user attempts to expand a disconnected source
- **THEN** nothing happens (or a tooltip/message indicates the source is not connected)

### Requirement: Source Selection Sets Active Connection
Selecting a source node (via Enter or navigation) SHALL set it as the active connection.

#### Scenario: Select source via Enter
- **WHEN** the user navigates to a source node and presses Enter
- **THEN** that source becomes the `active_connection`
- **AND** the explorer shows that source's schema tree
- **AND** subsequent queries target that connection

#### Scenario: Active source visual indicator
- **WHEN** a source is the active connection
- **THEN** the source node shows a visual indicator (bold, highlighted, or marker)
- **AND** the status bar shows the active connection name

### Requirement: Engine-Specific Icons
Each database engine SHALL have a distinct icon in the explorer.

#### Scenario: PostgreSQL source icon
- **WHEN** a PostgreSQL connection source is rendered
- **THEN** it shows the PostgreSQL database icon (teal/cyan color)

#### Scenario: MySQL source icon
- **WHEN** a MySQL connection source is rendered
- **THEN** it shows the MySQL cylinder icon (blue color)

#### Scenario: SQLite source icon
- **WHEN** a SQLite connection source is rendered
- **THEN** it shows the SQLite file icon (gray color)

#### Scenario: ASCII fallback when Nerd Fonts unavailable
- **WHEN** Nerd Fonts are not available
- **THEN** PostgreSQL shows `[PG]`, MySQL shows `[MY]`, SQLite shows `[SQ]`
