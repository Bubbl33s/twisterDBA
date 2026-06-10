## ADR-001: Extended SchemaNode hierarchy

**Decision**: Add new variants to `SchemaNode`:

```rust
pub enum SchemaNode {
    // Existing
    Schema { name: String, expanded: bool, children: Vec<SchemaNode> },
    Table { schema: String, name: String, expanded: bool, loaded: bool, children: Vec<SchemaNode> },
    View { schema: String, name: String },
    Column { name: String, data_type: String, nullable: bool, is_primary_key: bool },
    Loading { schema: String, table: String },
    // New
    Database { name: String, expanded: bool, children: Vec<SchemaNode> },
    ObjectFolder { kind: FolderKind, expanded: bool, loaded: bool, children: Vec<SchemaNode> },
    Index { name: String, columns: Vec<String>, is_unique: bool, is_primary: bool },
    ForeignKey { name: String, columns: Vec<String>, ref_table: String, ref_columns: Vec<String> },
    Key { name: String, columns: Vec<String> },
}

pub enum FolderKind {
    Tables,
    Views,
    Columns,
    Keys,
    ForeignKeys,
    Indexes,
}
```

**Rationale**:
- `Database` wraps schemas for engines that have a database concept (PostgreSQL, MySQL)
- `ObjectFolder` provides the folder grouping (Tables, Views, Columns, Keys, Foreign Keys, Indexes)
- `Index`, `ForeignKey`, `Key` are leaf nodes with their metadata
- SQLite skips the Database level (single "main" database)

## ADR-002: Folder-based schema organization

**Decision**: Schema loading produces this hierarchy:

```
Schema "public"
├── ObjectFolder "tables" (Tables)
│   ├── Table "album"
│   ├── Table "artist"
│   └── Table "customer"
├── ObjectFolder "views" (Views)
│   └── View "active_users"
```

For engines with database-level hierarchy:
```
Database "chinook"
├── Schema "public"
│   ├── ObjectFolder "tables"
│   │   └── Table "album"
│   └── ObjectFolder "views"
```

**Rationale**:
- Folders make large schemas navigable
- Tables and views are separated for clarity
- Matches DataGrip's organization exactly

## ADR-003: Table details lazy loading

**Decision**: When a user expands a Table node, the explorer sends `DbCommand::LoadTableDetails { connection_name, schema, table }`. The response includes columns, keys, foreign keys, and indexes, organized into folder children:

```
Table "album" (expanded, loaded)
├── ObjectFolder "columns" (Columns, loaded)
│   ├── Column "album_id" integer NOT NULL [PK]
│   ├── Column "title" varchar(160)
│   ── Column "artist_id" integer
├── ObjectFolder "keys" (Keys, loaded)
│   └── Key "album_pkey" (album_id)
├── ObjectFolder "foreign_keys" (Foreign Keys, loaded)
│   └── ForeignKey "album_artist_id_fkey" (artist_id) → artist(artist_id)
└── ObjectFolder "indexes" (Indexes, loaded)
    ├── Index "album_pkey" (album_id) UNIQUE
    └── Index "album_artist_id_idx" (artist_id)
```

**Rationale**:
- Lazy loading prevents fetching unnecessary metadata for tables the user never inspects
- Same pattern as current column loading — proven to work well
- All details loaded in a single query round-trip per table

## ADR-004: Per-engine table details queries

**Decision**: Each engine has its own `load_*_table_details` function:

**PostgreSQL** (`load_pg_table_details`):
- Columns: same existing query from `information_schema.columns`
- Keys: `information_schema.table_constraints` WHERE `constraint_type = 'PRIMARY KEY'` + `key_column_usage`
- Foreign Keys: `information_schema.table_constraints` WHERE `constraint_type = 'FOREIGN KEY'` + `key_column_usage` + `referential_constraints`
- Indexes: `pg_indexes` joined with `pg_index` for uniqueness info

**MySQL** (`load_mysql_table_details`):
- Columns: same existing query
- Keys: `information_schema.STATISTICS` WHERE `INDEX_NAME = 'PRIMARY'`
- Foreign Keys: `information_schema.REFERENTIAL_CONSTRAINTS` + `KEY_COLUMN_USAGE`
- Indexes: `information_schema.STATISTICS` WHERE `INDEX_NAME != 'PRIMARY'`

**SQLite** (`load_sqlite_table_details`):
- Columns: `PRAGMA table_info(table)`
- Keys: `PRAGMA table_info(table)` WHERE `pk > 0`
- Foreign Keys: `PRAGMA foreign_key_list(table)`
- Indexes: `PRAGMA index_list(table)` + `PRAGMA index_info(index_name)` for each

**Rationale**:
- Each engine has different system catalogs
- Queries are optimized per engine's strengths
- Results normalized into the same `TableDetails` struct

## ADR-005: TableDetails event and struct

**Decision**: New event and struct for table details:

```rust
pub struct TableDetails {
    pub columns: Vec<ColumnInfo>,
    pub keys: Vec<KeyInfo>,
    pub foreign_keys: Vec<ForeignKeyInfo>,
    pub indexes: Vec<IndexInfo>,
}

pub struct KeyInfo {
    pub name: String,
    pub columns: Vec<String>,
}

pub struct ForeignKeyInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub ref_table: String,
    pub ref_columns: Vec<String>,
}

pub struct IndexInfo {
    pub name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
}

// In DbEvent:
TableDetailsLoaded {
    connection_name: String,
    schema: String,
    table: String,
    details: TableDetails,
}
```

**Rationale**:
- Single event for all table details avoids multiple round-trips
- Structured data allows the explorer to organize into folders
- Connection name for routing (from multi-connection architecture)

## ADR-006: Icons for new node types

**Decision**: Add to `IconMap`:
- `folder: (char, Color)` — folder icon (gold/yellow)
- `key: (char, Color)` — key icon (gold)
- `foreign_key: (char, Color)` — linked/chain icon (orange)
- `index: (char, Color)` — index/list icon (blue)

ASCII fallbacks: `[F]` folder, `[K]` key, `[FK]` foreign key, `[I]` index

**Rationale**:
- Visual differentiation between constraint types
- Matches DataGrip's icon conventions
- Nerd Font icons when available
