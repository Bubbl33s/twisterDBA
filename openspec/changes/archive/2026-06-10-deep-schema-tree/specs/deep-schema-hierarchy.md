## ADDED Requirements

### Requirement: Database Node in Schema Tree
The schema tree SHALL include a `Database` node level for engines that support multiple databases (PostgreSQL, MySQL). The database name comes from the connection's target database.

#### Scenario: PostgreSQL schema includes database node
- **WHEN** connected to PostgreSQL with database "chinook"
- **THEN** the schema tree has a `Database "chinook"` root node
- **AND** schemas (public, etc.) are children of the database node

#### Scenario: MySQL schema includes database node
- **WHEN** connected to MySQL
- **THEN** each database from `information_schema.SCHEMATA` appears as a `Database` node
- **AND** schemas/tables are children of the respective database node

#### Scenario: SQLite skips database node
- **WHEN** connected to SQLite
- **THEN** the schema tree does NOT include a Database node
- **AND** the "main" schema appears directly under the source

### Requirement: Object Folder Grouping
Schemas SHALL organize tables and views into `ObjectFolder` nodes: "tables" and "views".

#### Scenario: Tables grouped in folder
- **WHEN** a schema contains tables
- **THEN** tables appear under an `ObjectFolder "tables"` node
- **AND** the folder shows a count: "tables 11"

#### Scenario: Views grouped in folder
- **WHEN** a schema contains views
- **THEN** views appear under an `ObjectFolder "views"` node
- **AND** the folder shows a count: "views 3"

#### Scenario: Empty folders hidden
- **WHEN** a schema has no views
- **THEN** the "views" folder is NOT shown
- **AND** only the "tables" folder appears

### Requirement: Table Details Folders
Each table SHALL organize its children into `ObjectFolder` nodes: "columns", "keys", "foreign_keys", "indexes".

#### Scenario: Table shows columns folder
- **WHEN** a table is expanded and details are loaded
- **THEN** columns appear under `ObjectFolder "columns"` with count
- **AND** each column shows name, data type, nullable status, and PK indicator

#### Scenario: Table shows keys folder
- **WHEN** a table has primary key constraints
- **THEN** keys appear under `ObjectFolder "keys"` with count
- **AND** each key shows name and column list

#### Scenario: Table shows foreign keys folder
- **WHEN** a table has foreign key constraints
- **THEN** foreign keys appear under `ObjectFolder "foreign_keys"` with count
- **AND** each FK shows name, columns, and reference table/columns

#### Scenario: Table shows indexes folder
- **WHEN** a table has indexes
- **THEN** indexes appear under `ObjectFolder "indexes"` with count
- **AND** each index shows name, columns, and UNIQUE indicator if applicable

#### Scenario: Empty detail folders hidden
- **WHEN** a table has no foreign keys
- **THEN** the "foreign_keys" folder is NOT shown
- **AND** only folders with items are displayed

### Requirement: Full Tree Hierarchy
The complete explorer tree SHALL follow this hierarchy:

```
Source "local-postgres" (connected)
└── Database "chinook"
    └── Schema "public"
        ├── ObjectFolder "tables" (11)
        │   ├── Table "album"
        │   │   ├── ObjectFolder "columns" (3)
        │   │   │   ├── Column "album_id" integer NOT NULL [PK]
        │   │   │   ├── Column "title" varchar(160)
        │   │   │   └── Column "artist_id" integer
        │   │   ├── ObjectFolder "keys" (1)
        │   │   │   ── Key "album_pkey" (album_id)
        │   │   ├── ObjectFolder "foreign_keys" (1)
        │   │   │   └── ForeignKey "album_artist_id_fkey" (artist_id) → artist(artist_id)
        │   │   └── ObjectFolder "indexes" (2)
        │   │       ├── Index "album_pkey" (album_id) UNIQUE
        │   │       └── Index "album_artist_id_idx" (artist_id)
        │   └── Table "artist"
        │       └── ...
        └── ObjectFolder "views" (2)
            └── View "active_albums"
```

#### Scenario: Navigate full hierarchy
- **WHEN** the user expands Source > Database > Schema > tables folder > table
- **THEN** all four detail folders appear (columns, keys, foreign_keys, indexes)
- **AND** expanding each folder reveals its items

#### Scenario: Lazy loading of table details
- **WHEN** the user expands a table for the first time
- **THEN** a "Loading..." indicator appears
- **AND** `DbCommand::LoadTableDetails` is sent
- **AND** when details arrive, folders are populated with items
