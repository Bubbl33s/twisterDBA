## ADDED Requirements

### Requirement: Vertical Engine List in Step 1
Step 1 of the connection dialog SHALL display database engine types as a vertical list rather than a horizontal grid, with each entry showing an icon and engine name.

#### Scenario: Engines displayed as vertical list
- **WHEN** Step 1 is rendered
- **THEN** PostgreSQL, MySQL, and SQLite are shown as three consecutive rows
- **AND** each row shows the engine icon followed by the engine name
- **AND** the currently selected engine is highlighted with `theme.dialog_type_selected_bg`

#### Scenario: Engine icons use theme colors
- **WHEN** engine entries are rendered
- **THEN** PostgreSQL icon uses `theme.icons.postgres` color
- **AND** MySQL icon uses `theme.icons.mysql` color
- **AND** SQLite icon uses `theme.icons.sqlite` color

### Requirement: Combined Engine and Profile Navigation
Step 1 SHALL use a single cursor that navigates through both engine types and saved profiles as one combined list, with a visual separator between the two sections.

#### Scenario: Single cursor moves through combined list
- **WHEN** the user presses Down on the last engine (SQLite)
- **THEN** the cursor moves to the first saved profile
- **AND** the engine selection highlight is removed

#### Scenario: Cursor moves from profiles back to engines
- **WHEN** the user presses Up on the first saved profile
- **THEN** the cursor moves to the last engine (SQLite)
- **AND** the profile selection highlight is removed

#### Scenario: Visual separator between sections
- **WHEN** Step 1 is rendered with both engines and profiles
- **THEN** a horizontal separator line (e.g., `───`) appears between the engine list and the profile list
- **AND** the label "Saved Connections:" appears above the profile entries

#### Scenario: No profiles shows only engines
- **WHEN** no saved profiles exist
- **THEN** only the 3 engine entries are shown
- **AND** no separator or "Saved Connections" label is displayed
- **AND** the cursor can only navigate between the 3 engines

#### Scenario: Enter on engine proceeds to Step 2
- **WHEN** the cursor is on an engine entry and the user presses Enter
- **THEN** the dialog transitions to Step 2 with engine-specific fields
- **AND** the connection name is auto-generated

#### Scenario: Enter on profile proceeds to Step 2 with pre-filled data
- **WHEN** the cursor is on a saved profile and the user presses Enter
- **THEN** the dialog transitions to Step 2 with all fields pre-filled from the profile
- **AND** the connection name is set to the profile name

### Requirement: Theme-Aware Dialog Styling
The connection dialog SHALL use colors from the application `Theme` for all visual elements instead of hardcoded color values.

#### Scenario: Dialog border uses theme color
- **WHEN** the dialog is rendered
- **THEN** the dialog border color matches `theme.identifier`
- **AND** the dialog background matches `theme.editor_bg`

#### Scenario: Selected item uses theme highlight
- **WHEN** an engine or profile is selected
- **THEN** the highlight background uses `theme.dialog_type_selected_bg`
- **AND** the highlight is consistent across both Step 1 and Step 2

#### Scenario: Help text uses theme colors
- **WHEN** the help footer is rendered
- **THEN** key names use `theme.statusline_active_bg` for highlighting
- **AND** the help text background matches the dialog background

#### Scenario: Active field highlight uses theme
- **WHEN** a field is active in Step 2
- **THEN** the field background uses a theme-derived active color
- **AND** the cursor uses inverted theme colors (fg=`theme.editor_bg`, bg=`theme.identifier`)
