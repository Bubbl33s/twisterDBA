## ADDED Requirements

### Requirement: Dialog Backdrop Dimming
The connection dialog backdrop SHALL dim the underlying UI without fully obscuring it, creating a "curtain" effect rather than a solid wall.

#### Scenario: Backdrop shows dimmed underlying UI
- **WHEN** the connection dialog is open
- **THEN** the schema explorer, SQL editor, output panel, and status bar are faintly visible behind the backdrop
- **AND** the backdrop color is a very dark shade (`Color::Rgb(10, 10, 10)`) that blends with the terminal background

#### Scenario: Backdrop covers full terminal except dialog
- **WHEN** the connection dialog is rendered
- **THEN** the backdrop covers the entire terminal area
- **AND** the dialog popup area is rendered on top of the backdrop without being dimmed

#### Scenario: Closing dialog restores underlying UI
- **WHEN** the user closes the dialog (Esc or successful connection)
- **THEN** the underlying UI is immediately and fully restored to its pre-dialog appearance
- **AND** no visual artifacts remain from the backdrop

#### Scenario: Backdrop renders without performance impact
- **WHEN** the dialog is opened or closed
- **THEN** there is no noticeable lag or frame drop
- **AND** no additional heap allocations occur in the render loop

### Requirement: Dialog Cursor Visibility
The cursor position within dialog input fields SHALL be clearly and unambiguously visible at all times using a reverse-video rendering style.

#### Scenario: Cursor visible on non-empty field
- **WHEN** an input field is active and contains text
- **THEN** the character at the cursor position is rendered with inverted foreground and background colors
- **AND** the inverted colors provide strong contrast against the field background

#### Scenario: Cursor visible on empty field
- **WHEN** an input field is active and empty
- **THEN** a block cursor character (`` U+258C) is rendered at position 0
- **AND** the block cursor uses inverted foreground and background colors

#### Scenario: Cursor visible when navigating between fields
- **WHEN** the user navigates between fields using Tab, Up, or Down
- **THEN** the cursor immediately appears in the newly active field
- **AND** the previous field's cursor is removed

#### Scenario: Cursor distinguishable without color
- **WHEN** the terminal has limited color support (e.g., 16 colors)
- **THEN** the cursor is still distinguishable through brightness inversion (light on dark vs dark on light)
- **AND** the cursor does not rely solely on color hue for visibility

### Requirement: Dialog Content-Proportional Sizing
The connection dialog SHALL size itself proportionally to its content with minimum and maximum bounds, rather than using fixed terminal percentages.

#### Scenario: Step 1 dialog fits content
- **WHEN** Step 1 is rendered with 3 engines and 1 saved profile
- **THEN** the dialog height accommodates exactly: 3 engine rows + 1 separator + 1 label + 1 profile + 1 help row + 2 padding rows
- **AND** the dialog width is sufficient for the longest profile name

#### Scenario: Step 1 dialog scales with profiles
- **WHEN** the number of saved profiles increases
- **THEN** the dialog height grows to accommodate all profiles
- **AND** the dialog height does not exceed 70% of terminal height

#### Scenario: Step 2 dialog fits form content
- **WHEN** Step 2 is rendered for PostgreSQL (6 fields + name + DSN + help)
- **THEN** the dialog height accommodates all fields without scrolling
- **AND** the dialog width accommodates the longest label + input comfortably

#### Scenario: Dialog never exceeds terminal bounds
- **WHEN** the terminal is small (e.g., 80x24)
- **THEN** the dialog width does not exceed 80% of terminal width
- **AND** the dialog height does not exceed 70% of terminal height
- **AND** the dialog remains centered
