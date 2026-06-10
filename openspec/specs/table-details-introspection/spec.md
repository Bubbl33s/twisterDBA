# Table Details Introspection

## Purpose

Defines how table details (columns, primary keys, foreign keys, and indexes) are fetched from different database engines (PostgreSQL, MySQL, SQLite) and communicated back to the application via the `TableDetailsLoaded` event.

## Requirements

### Requirement: PostgreSQL Table Details Introspection
The application SHALL fetch table details from PostgreSQL system catalogs using optimized queries.

#### Scenario: Load PostgreSQL table columns
- **WHEN** `LoadTableDetails` is sent for a PostgreSQL table
- **THEN** columns are fetched from `information_schema.columns` with PK detection via `table_constraints` + `key_column_usage`

#### Scenario: Load PostgreSQL primary keys
- **WHEN** `LoadTableDetails` is sent for a PostgreSQL table
- **THEN** primary keys are fetched from `information_schema.table_constraints` WHERE `constraint_type = 'PRIMARY KEY'`
- **AND** key columns from `information_schema.key_column_usage`
- **AND** each key includes its name and ordered column list

#### Scenario: Load PostgreSQL foreign keys
- **WHEN** `LoadTableDetails` is sent for a PostgreSQL table
- **THEN** foreign keys are fetched from `information_schema.table_constraints` WHERE `constraint_type = 'FOREIGN KEY'`
- **AND** FK columns from `key_column_usage`
- **AND** reference table/columns from `referential_constraints` + `key_column_usage` on the referenced side

#### Scenario: Load PostgreSQL indexes
- **WHEN** `LoadTableDetails` is sent for a PostgreSQL table
- **THEN** indexes are fetched from `pg_indexes` joined with `pg_index` for uniqueness
- **AND** each index includes name, columns, and is_unique flag
- **AND** primary key indexes are included (they appear in pg_indexes too)

### Requirement: MySQL Table Details Introspection
The application SHALL fetch table details from MySQL information_schema.

#### Scenario: Load MySQL table columns
- **WHEN** `LoadTableDetails` is sent for a MySQL table
- **THEN** columns are fetched from `information_schema.COLUMNS` with PK detection

#### Scenario: Load MySQL primary keys
- **WHEN** `LoadTableDetails` is sent for a MySQL table
- **THEN** primary keys are fetched from `information_schema.STATISTICS` WHERE `INDEX_NAME = 'PRIMARY'`
- **AND** columns ordered by `SEQ_IN_INDEX`

#### Scenario: Load MySQL foreign keys
- **WHEN** `LoadTableDetails` is sent for a MySQL table
- **THEN** foreign keys are fetched from `information_schema.REFERENTIAL_CONSTRAINTS`
- **AND** FK columns from `KEY_COLUMN_USAGE`
- **AND** reference table/columns from `REFERENTIAL_CONSTRAINTS.REFERENCED_TABLE_NAME` + `KEY_COLUMN_USAGE.REFERENCED_COLUMN_NAME`

#### Scenario: Load MySQL indexes
- **WHEN** `LoadTableDetails` is sent for a MySQL table
- **THEN** indexes are fetched from `information_schema.STATISTICS` WHERE `INDEX_NAME != 'PRIMARY'`
- **AND** grouped by `INDEX_NAME` with columns ordered by `SEQ_IN_INDEX`
- **AND** uniqueness determined by `NON_UNIQUE` column

### Requirement: SQLite Table Details Introspection
The application SHALL fetch table details from SQLite using PRAGMA commands.

#### Scenario: Load SQLite table columns
- **WHEN** `LoadTableDetails` is sent for a SQLite table
- **THEN** columns are fetched via `PRAGMA table_info(table)`
- **AND** PK columns identified by `pk > 0` field

#### Scenario: Load SQLite primary keys
- **WHEN** `LoadTableDetails` is sent for a SQLite table
- **THEN** primary keys are derived from `PRAGMA table_info(table)` WHERE `pk > 0`
- **AND** key name is "primary" (SQLite doesn't name PK constraints)

#### Scenario: Load SQLite foreign keys
- **WHEN** `LoadTableDetails` is sent for a SQLite table
- **THEN** foreign keys are fetched via `PRAGMA foreign_key_list(table)`
- **AND** each FK includes columns, reference table, and reference columns

#### Scenario: Load SQLite indexes
- **WHEN** `LoadTableDetails` is sent for a SQLite table
- **THEN** indexes are fetched via `PRAGMA index_list(table)`
- **AND** for each index, columns are fetched via `PRAGMA index_info(index_name)`
- **AND** uniqueness from `PRAGMA index_list`'s `unique` field

### Requirement: TableDetailsLoaded Event
After fetching table details, the DbClient SHALL send a `TableDetailsLoaded` event with all details in a single event.

#### Scenario: TableDetailsLoaded contains all details
- **WHEN** table details are fully loaded
- **THEN** `DbEvent::TableDetailsLoaded` contains columns, keys, foreign_keys, and indexes
- **AND** the event includes `connection_name`, `schema`, and `table` for routing

#### Scenario: TableDetailsLoaded with empty details
- **WHEN** a table has no foreign keys or indexes
- **THEN** `TableDetailsLoaded` is still sent
- **AND** `foreign_keys` and `indexes` are empty vectors
- **AND** the explorer hides the empty folders
