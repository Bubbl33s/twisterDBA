# Engine-Specific Fields

## Purpose

Defines the connection form fields shown in Step 2 of the connection dialog, varying by selected database engine type.

## Requirements

### Requirement: PostgreSQL Connection Fields
When PostgreSQL is selected, Step 2 SHALL show the following fields in order:

1. **Host** (default: `localhost`)
2. **Port** (default: `5432`)
3. **Database** (default: empty)
4. **User** (default: empty)
5. **Password** (default: empty, masked input)
6. **SSL Mode** (dropdown: `disable`, `allow`, `prefer`, `require`, `verify-ca`, `verify-full`; default: `prefer`)

#### Scenario: PostgreSQL DSN includes SSL mode
- **WHEN** SSL Mode is set to `require`
- **THEN** the DSN includes `?sslmode=require`

#### Scenario: PostgreSQL default port
- **WHEN** the Port field is empty
- **THEN** the DSN uses port 5432

### Requirement: MySQL Connection Fields
When MySQL is selected, Step 2 SHALL show the following fields in order:

1. **Host** (default: `localhost`)
2. **Port** (default: `3306`)
3. **Database** (default: empty)
4. **User** (default: empty)
5. **Password** (default: empty, masked input)

#### Scenario: MySQL default port
- **WHEN** the Port field is empty
- **THEN** the DSN uses port 3306

#### Scenario: MySQL DSN with empty user defaults to root
- **WHEN** User field is empty
- **THEN** the DSN uses `root` as the username

### Requirement: SQLite Connection Fields
When SQLite is selected, Step 2 SHALL show a single field:

1. **File Path** (default: empty)

#### Scenario: SQLite DSN is file path
- **WHEN** File Path is `/home/user/data.db`
- **THEN** the DSN is `sqlite:///home/user/data.db`

### Requirement: Connection Name Field
Step 2 SHALL include an editable **Connection Name** field at the top, above the engine-specific fields.

#### Scenario: Auto-generated name from type selection
- **WHEN** the user selects PostgreSQL type with host `localhost`
- **THEN** the connection name is auto-generated as `postgres-localhost`

#### Scenario: Profile name preserved
- **WHEN** the user selects a saved profile named "prod-primary"
- **THEN** the connection name is set to "prod-primary"

#### Scenario: Custom name override
- **WHEN** the user edits the connection name field to "my-db"
- **THEN** the connection is saved with name "my-db"

#### Scenario: Name conflict detection
- **WHEN** the user enters a connection name that already exists
- **THEN** the dialog shows a warning: "Connection 'X' already exists. Overwrite?"
- **AND** the user can confirm (overwrite) or cancel (edit name)

### Requirement: SSL Mode Dropdown for PostgreSQL
The SSL Mode field SHALL be a selectable dropdown in the PostgreSQL form.

#### Scenario: Navigate SSL mode options
- **WHEN** the SSL Mode field is active
- **THEN** Left/Right keys cycle through: disable → allow → prefer → require → verify-ca → verify-full → disable
- **AND** the current selection is highlighted

#### Scenario: SSL mode included in DSN
- **WHEN** SSL Mode is `require`
- **THEN** the built DSN appends `?sslmode=require`
- **WHEN** SSL Mode is `prefer` (default)
- **THEN** the built DSN appends `?sslmode=prefer`
