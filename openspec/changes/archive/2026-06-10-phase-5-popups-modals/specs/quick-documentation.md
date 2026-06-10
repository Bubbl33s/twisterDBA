## ADDED Requirements

### Requirement: Quick Documentation Popup
Pressing `K` or `Ctrl+Q` while a table or view is selected in the Schema Explorer SHALL display a floating popup showing the entity's DDL, row count, and estimated table size.

#### Scenario: Show table documentation
- **WHEN** user selects the `users` table in the schema explorer and presses `K`
- **THEN** a floating popup appears anchored near the selected node, showing the `CREATE TABLE users (...)` DDL statement, row count (e.g., "10,234 rows"), and table size (e.g., "2.3 MB")

#### Scenario: Close documentation popup
- **WHEN** the Quick Documentation popup is open and user presses `Escape`, `K`, `Ctrl+Q`, or `q`
- **THEN** the popup closes and focus returns to the schema explorer

#### Scenario: Popup respects terminal bounds
- **WHEN** the selected table is near the bottom or right edge of the terminal
- **THEN** the popup repositions to stay within the terminal boundaries (flips to above or left of the anchor point)

#### Scenario: Documentation for a view
- **WHEN** user presses `K` on a view node
- **THEN** the popup shows the view's `CREATE VIEW ... AS ...` definition and row count (if available from statistics)

#### Scenario: Documentation for unsupported node type
- **WHEN** user presses `K` on a schema/folder, column, or loading node
- **THEN** no popup appears (or shows "No documentation available for this entity type")

#### Scenario: Documentation fetched asynchronously
- **WHEN** table info is not yet loaded and user presses `K`
- **THEN** the popup shows "Loading..." with a spinner until `DbEvent::TableInfoLoaded` arrives, then updates with the data

### Requirement: DDL and Statistics Querying
The DB layer SHALL fetch DDL, row count, and table size using engine-specific queries (e.g., `pg_get_tabledef`, `information_schema`, `sqlite_master`).

#### Scenario: PostgreSQL table info
- **WHEN** `LoadTableInfo` is sent for a PostgreSQL table
- **THEN** DDL is fetched via `pg_get_tabledef()` or reconstructed from `information_schema.columns`, row count via `pg_stat_user_tables.n_live_tup`, size via `pg_total_relation_size()`

#### Scenario: MySQL table info
- **WHEN** `LoadTableInfo` is sent for a MySQL table
- **THEN** DDL is reconstructed from `information_schema.COLUMNS`, row count via `information_schema.TABLES.TABLE_ROWS`, size via `data_length + index_length`

#### Scenario: SQLite table info
- **WHEN** `LoadTableInfo` is sent for a SQLite table
- **THEN** DDL is fetched from `sqlite_master.sql`, row count via `SELECT COUNT(*)`, size via file stat (approximate)

#### Scenario: Query error handled gracefully
- **WHEN** `LoadTableInfo` fails (e.g., insufficient permissions on `pg_stat_user_tables`)
- **THEN** the popup shows available data (DDL if possible) with "Row count: unavailable" and "Size: unavailable"
