## ADDED Requirements

### Requirement: Two-Step Connection Dialog Flow
The connection dialog SHALL operate in two steps: Step 1 (SelectType) for choosing a database type or saved profile, and Step 2 (EnterDetails) for entering connection parameters.

#### Scenario: Open dialog enters Step 1
- **WHEN** the user triggers `:connect`
- **THEN** the dialog opens in Step 1 (SelectType)
- **AND** the type grid shows PostgreSQL, MySQL, SQLite with icons
- **AND** saved profiles are listed below the grid

#### Scenario: Select type moves to Step 2
- **WHEN** the user navigates to "PostgreSQL" in the type grid and presses Enter
- **THEN** the dialog transitions to Step 2 (EnterDetails)
- **AND** PostgreSQL-specific fields are shown (Host, Port, Database, User, Password, SSL Mode)
- **AND** the connection name is auto-generated as "postgres-localhost"

#### Scenario: Select profile moves to Step 2 with pre-filled fields
- **WHEN** the user navigates to a saved profile "local-postgres" and presses Enter
- **THEN** the dialog transitions to Step 2
- **AND** all fields are pre-filled from the profile
- **AND** the connection name is set to "local-postgres"
- **AND** the password is loaded from keychain if stored there

#### Scenario: Esc in Step 2 returns to Step 1
- **WHEN** the user is in Step 2 and presses Esc
- **THEN** the dialog returns to Step 1
- **AND** the previous type/profile selection is preserved

#### Scenario: Esc in Step 1 closes dialog
- **WHEN** the user is in Step 1 and presses Esc
- **THEN** the dialog closes and mode returns to Normal

#### Scenario: Enter in Step 2 initiates connection
- **WHEN** the user fills in Step 2 fields and presses Enter
- **THEN** the connection is initiated with the provided parameters
- **AND** a ConnectionProfile is saved with the connection name
- **AND** the dialog closes

### Requirement: Database Type Grid
Step 1 SHALL display a visual grid of database types with engine-specific icons.

#### Scenario: Type grid shows three engines
- **WHEN** Step 1 is rendered
- **THEN** three type options are shown: PostgreSQL, MySQL, SQLite
- **AND** each has its engine icon (🐘 PostgreSQL, 🐬 MySQL, 🪶 SQLite)
- **AND** the currently selected type is highlighted

#### Scenario: Navigate type grid with arrow keys
- **WHEN** the user presses Left/Right in the type grid
- **THEN** the selection moves between PostgreSQL, MySQL, SQLite
- **AND** the selection wraps around at edges

#### Scenario: Navigate to profile list with Down arrow
- **WHEN** the user presses Down while on the last row of the type grid
- **THEN** focus moves to the saved profiles list
- **AND** the first profile is selected

### Requirement: Saved Profiles in Step 1
Step 1 SHALL list saved connection profiles below the type grid, grouped by engine type.

#### Scenario: Profiles listed with engine icons
- **WHEN** saved profiles exist
- **THEN** each profile is shown with its engine icon, name, and host
- **AND** profiles are grouped by engine type (PostgreSQL profiles together, etc.)

#### Scenario: No saved profiles
- **WHEN** no profiles are saved
- **THEN** the profile section shows "(no saved connections)"
- **AND** only the type grid is interactive

#### Scenario: Select profile auto-fills Step 2
- **WHEN** the user selects a saved profile and presses Enter
- **THEN** Step 2 is shown with all fields pre-filled from the profile
- **AND** the connection name matches the profile name
