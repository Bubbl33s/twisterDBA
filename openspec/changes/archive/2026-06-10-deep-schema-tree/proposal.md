## Why

The current schema explorer shows a flat 3-level hierarchy: Schema > Table > Column. DataGrip shows a much deeper hierarchy: Connection > Database > Schema > [Tables folder, Views folder] > Table > [Columns folder, Keys folder, Foreign Keys folder, Indexes folder] > individual items. This deeper hierarchy helps users navigate large databases by organizing objects into logical groups. Additionally, the current implementation only fetches columns with a primary key flag — it does not fetch indexes, foreign keys, or other constraints. Users need to see the full table structure including all constraints.

## What Changes

- **Database node**: Add `Database` variant to `SchemaNode` wrapping schemas under a database name (from the connection's default database)
- **Object type folders**: Add `ObjectFolder` variant for grouping: "tables", "views", "columns", "keys", "foreign_keys", "indexes"
- **Index, ForeignKey, Key leaf nodes**: Add `Index`, `ForeignKey`, `Key` variants to `SchemaNode`
- **LoadTableDetails command**: New `DbCommand::LoadTableDetails` that fetches indexes, foreign keys, and keys for a table
- **Per-engine introspection queries**: PostgreSQL uses `pg_indexes`, `pg_constraint`; MySQL uses `information_schema.STATISTICS`, `information_schema.REFERENTIAL_CONSTRAINTS`; SQLite uses `PRAGMA index_list()`, `PRAGMA foreign_key_list()`
- **Lazy loading**: Table details (indexes, FKs, keys) are loaded lazily when the user expands a table node, same pattern as current column loading
- **Explorer tree hierarchy**: Source > Database > Schema > [Tables folder, Views folder] > Table > [Columns, Keys, Foreign Keys, Indexes] > items

## Capabilities

### New Capabilities
- `deep-schema-hierarchy`: Full DataGrip-style tree with database, folders, and constraint nodes
- `table-details-introspection`: Fetch indexes, foreign keys, and keys for each table
- `object-folder-grouping`: Tables and views grouped into folders under schema

### Modified Capabilities
- `multi-db-support`: Schema loading now produces Database > Schema > Folders > Objects hierarchy
- `schema-explorer-rendering`: Explorer renders additional node types with appropriate icons

## Impact

- `src/explorer.rs`: Add `Database`, `ObjectFolder`, `Index`, `ForeignKey`, `Key` variants to `SchemaNode`; add corresponding `NodeKind` variants; update flattening
- `src/events.rs`: Add `IndexInfo`, `ForeignKeyInfo`, `KeyInfo` structs; add `TableDetailsLoaded` event; add `LoadTableDetails` to `DbCommand`
- `src/db/client.rs`: Add `handle_load_table_details` dispatching to per-engine functions
- `src/db/pg_schema.rs`: Add `load_pg_table_details` querying pg_indexes, pg_constraint
- `src/db/mysql.rs`: Add `load_mysql_table_details` querying information_schema
- `src/db/sqlite.rs`: Add `load_sqlite_table_details` using PRAGMA commands
- `src/state/events.rs`: Handle `TableDetailsLoaded` event, insert into explorer tree
- `src/state/handlers/explorer.rs`: Trigger `LoadTableDetails` when expanding a table
- `src/ui/explorer.rs`: Render new node types with icons (folder, key, foreign key, index)
- `src/theme.rs`: Add icons for folder, key, foreign_key, index
