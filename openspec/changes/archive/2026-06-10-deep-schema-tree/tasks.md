## 1. SchemaNode Extensions (`src/explorer.rs`)

- [x] 1.1 Add `Database { name: String, expanded: bool, children: Vec<SchemaNode> }` variant to `SchemaNode`
- [x] 1.2 Add `ObjectFolder { kind: FolderKind, expanded: bool, loaded: bool, children: Vec<SchemaNode> }` variant
- [x] 1.3 Add `Index { name: String, columns: Vec<String>, is_unique: bool, is_primary: bool }` variant
- [x] 1.4 Add `ForeignKey { name: String, columns: Vec<String>, ref_table: String, ref_columns: Vec<String> }` variant
- [x] 1.5 Add `Key { name: String, columns: Vec<String> }` variant
- [x] 1.6 Add `FolderKind` enum: `Tables`, `Views`, `Columns`, `Keys`, `ForeignKeys`, `Indexes`
- [x] 1.7 Add `NodeKind::Database`, `NodeKind::Folder`, `NodeKind::Index`, `NodeKind::ForeignKey`, `NodeKind::Key` variants
- [x] 1.8 Update `FlatNode` with new fields: `folder_kind: Option<FolderKind>`, `columns: Option<Vec<String>>`, `ref_table: Option<String>`, `ref_columns: Option<Vec<String>>`, `is_unique: bool`, `is_primary: bool`
- [x] 1.9 Update `flatten()` to handle all new node types
- [x] 1.10 Update `expand_node`/`collapse_node` for Database and ObjectFolder nodes
- [x] 1.11 Update `path_to_node`/`build_path` for new node types
- [x] 1.12 Add `insert_table_details(&mut self, schema: &str, table: &str, details: TableDetails)` method
- [x] 1.13 Run `cargo build` and fix compile errors

## 2. TableDetails Structs and Events (`src/events.rs`)

- [x] 2.1 Add `KeyInfo { name: String, columns: Vec<String> }` struct
- [x] 2.2 Add `ForeignKeyInfo { name: String, columns: Vec<String>, ref_table: String, ref_columns: Vec<String> }` struct
- [x] 2.3 Add `IndexInfo { name: String, columns: Vec<String>, is_unique: bool, is_primary: bool }` struct
- [x] 2.4 Add `TableDetails { columns: Vec<ColumnInfo>, keys: Vec<KeyInfo>, foreign_keys: Vec<ForeignKeyInfo>, indexes: Vec<IndexInfo> }` struct
- [x] 2.5 Add `LoadTableDetails { connection_name: String, schema: String, table: String }` to `DbCommand`
- [x] 2.6 Add `TableDetailsLoaded { connection_name: String, schema: String, table: String, details: TableDetails }` to `DbEvent`
- [x] 2.7 Run `cargo build` and fix compile errors

## 3. DbClient Table Details Handler (`src/db/client.rs`)

- [x] 3.1 Add `handle_load_table_details` method to `DbClient`
- [x] 3.2 Add match arm in `run()` for `DbCommand::LoadTableDetails`
- [x] 3.3 Dispatch to `load_pg_table_details`, `load_mysql_table_details`, or `load_sqlite_table_details`
- [x] 3.4 Send `DbEvent::TableDetailsLoaded` with results
- [x] 3.5 Run `cargo build` and fix compile errors

## 4. PostgreSQL Table Details (`src/db/pg_schema.rs`)

- [x] 4.1 Add `load_pg_table_details(pool, schema, table) -> Result<TableDetails, String>` function
- [x] 4.2 Fetch columns (reuse existing query from `load_pg_columns`)
- [x] 4.3 Fetch primary keys from `information_schema.table_constraints` + `key_column_usage`
- [x] 4.4 Fetch foreign keys from `information_schema.table_constraints` + `key_column_usage` + `referential_constraints`
- [x] 4.5 Fetch indexes from `pg_indexes` joined with `pg_index` for uniqueness
- [x] 4.6 Assemble into `TableDetails` struct and return
- [x] 4.7 Run `cargo build` and fix compile errors

## 5. MySQL Table Details (`src/db/mysql.rs`)

- [x] 5.1 Add `load_mysql_table_details(pool, schema, table) -> Result<TableDetails, String>` function
- [x] 5.2 Fetch columns (reuse existing query)
- [x] 5.3 Fetch primary keys from `information_schema.STATISTICS` WHERE `INDEX_NAME = 'PRIMARY'`
- [x] 5.4 Fetch foreign keys from `information_schema.REFERENTIAL_CONSTRAINTS` + `KEY_COLUMN_USAGE`
- [x] 5.5 Fetch indexes from `information_schema.STATISTICS` WHERE `INDEX_NAME != 'PRIMARY'`, grouped by name
- [x] 5.6 Assemble into `TableDetails` struct and return
- [x] 5.7 Run `cargo build` and fix compile errors

## 6. SQLite Table Details (`src/db/sqlite.rs`)

- [x] 6.1 Add `load_sqlite_table_details(pool, schema, table) -> Result<TableDetails, String>` function
- [x] 6.2 Fetch columns via `PRAGMA table_info(table)`
- [x] 6.3 Derive primary keys from `PRAGMA table_info` WHERE `pk > 0`
- [x] 6.4 Fetch foreign keys via `PRAGMA foreign_key_list(table)`
- [x] 6.5 Fetch indexes via `PRAGMA index_list(table)` + `PRAGMA index_info(name)` for each
- [x] 6.6 Assemble into `TableDetails` struct and return
- [x] 6.7 Run `cargo build` and fix compile errors

## 7. Schema Loading with Folders (`src/db/pg_schema.rs`, `src/db/mysql.rs`, `src/db/sqlite.rs`)

- [x] 7.1 Update `load_pg_schema` to wrap tables/views in ObjectFolder nodes under each schema
- [x] 7.2 Update `load_mysql_schema` to include Database nodes wrapping schemas, with ObjectFolder nodes
- [x] 7.3 Update `load_sqlite_schema` to wrap tables/views in ObjectFolder nodes (no Database level)
- [x] 7.4 Run `cargo build` and fix compile errors

## 8. Explorer Event Handler (`src/state/events.rs`)

- [x] 8.1 Add handler for `DbEvent::TableDetailsLoaded`
- [x] 8.2 Convert `TableDetails` into `SchemaNode` children (folders with items)
- [x] 8.3 Call `explorer.insert_table_details(schema, table, details)` to update the tree
- [x] 8.4 Run `cargo build` and fix compile errors

## 9. Explorer Key Handler Update (`src/state/handlers/explorer.rs`)

- [x] 9.1 Update table expand logic: if table not loaded, send `LoadTableDetails` instead of `LoadColumns`
- [x] 9.2 Handle ObjectFolder expansion (just toggle expanded, no async load needed — children already present)
- [x] 9.3 Handle Database node expansion (toggle expanded)
- [x] 9.4 Run `cargo build` and fix compile errors

## 10. Theme Icons for New Node Types (`src/theme.rs`)

- [x] 10.1 Add `folder: (char, Color)` to `IconMap` (folder icon, gold)
- [x] 10.2 Add `key: (char, Color)` to `IconMap` (key icon, gold)
- [x] 10.3 Add `foreign_key: (char, Color)` to `IconMap` (chain/link icon, orange)
- [x] 10.4 Add `index: (char, Color)` to `IconMap` (index icon, blue)
- [x] 10.5 Add `database_icon: (char, Color)` to `IconMap` (database icon, teal) — if not already present
- [x] 10.6 Run `cargo build` and fix compile errors

## 11. Explorer Rendering Update (`src/ui/explorer.rs`)

- [x] 11.1 Render `NodeKind::Database` with database icon and name
- [x] 11.2 Render `NodeKind::Folder` with folder icon, kind label, and item count (e.g., "tables 11")
- [x] 11.3 Render `NodeKind::Index` with index icon, name, columns, and UNIQUE indicator
- [x] 11.4 Render `NodeKind::ForeignKey` with FK icon, name, columns, and reference (e.g., "→ artist(artist_id)")
- [x] 11.5 Render `NodeKind::Key` with key icon, name, and columns
- [x] 11.6 Update ASCII fallbacks: `[DB]` database, `[F]` folder, `[K]` key, `[FK]` foreign key, `[I]` index
- [x] 11.7 Update Column rendering to show PK indicator (e.g., key icon or [PK] suffix)
- [x] 11.8 Run `cargo build` and fix compile errors

## 12. Verification

- [x] 12.1 Run `cargo build` and verify no errors
- [x] 12.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 12.3 Run `cargo fmt --check` and verify formatting
- [x] 12.4 Run `cargo test` and verify all tests pass
- [ ] 12.5 Manually test: connect to PostgreSQL, verify Database > Schema > Folders > Tables hierarchy
- [ ] 12.6 Manually test: expand table, verify columns/keys/FKs/indexes folders appear
- [ ] 12.7 Manually test: expand each folder, verify items render correctly
- [ ] 12.8 Manually test: connect to MySQL, verify same hierarchy works
- [ ] 12.9 Manually test: connect to SQLite, verify hierarchy without Database level
- [ ] 12.10 Manually test: table with no foreign keys — verify FK folder is hidden
- [ ] 12.11 Manually test: index with UNIQUE flag — verify indicator shown
