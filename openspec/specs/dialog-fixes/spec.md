# Dialog Fixes

## Purpose

Visual and interaction improvements to the connection dialog, including backdrop removal, unified active row highlighting, and vim-style navigation.

## Requirements

### Requirement: No Backdrop Overlay
The connection dialog SHALL render without an opaque backdrop overlay. The underlying UI (schema explorer, SQL editor, output panel, status bar) SHALL remain fully visible when the dialog is open.

#### Scenario: Dialog opens without hiding underlying UI
- **WHEN** the user opens the connection dialog
- **THEN** the schema explorer, SQL editor, output panel, and status bar remain visible behind the dialog
- **AND** the dialog is visually separated by its border and background color

#### Scenario: Dialog closes without artifacts
- **WHEN** the user closes the connection dialog
- **THEN** the underlying UI is immediately visible without any visual artifacts
- **AND** no Clear widget residue remains

### Requirement: Unified Active Row Highlight
When a field is active in the Step 2 connection form, the entire row (label + input) SHALL have a unified background color to clearly indicate focus.

#### Scenario: Active field row has unified background
- **WHEN** a field is active in Step 2
- **THEN** the label and input area share the same background color (`theme.dialog_field_active_bg`)
- **AND** the label text is rendered in white for contrast against the dark background

#### Scenario: Inactive fields have distinct appearance
- **WHEN** a field is not active
- **THEN** the label uses `Color::DarkGray` text with no special background
- **AND** the field uses `theme.editor_bg` background with `Color::Gray` text

#### Scenario: Navigating between fields updates highlight
- **WHEN** the user navigates between fields with Tab/Up/Down
- **THEN** the previously active row returns to inactive styling
- **AND** the newly active row shows the unified highlight

### Requirement: j/k Navigation in Step 1
Step 1 of the connection dialog SHALL support `j` and `k` keys as aliases for Down and Up arrow keys respectively.

#### Scenario: j moves cursor down
- **WHEN** the user presses `j` in Step 1
- **THEN** the cursor moves down one position in the engine/profile list
- **AND** the behavior is identical to pressing Down arrow

#### Scenario: k moves cursor up
- **WHEN** the user presses `k` in Step 1
- **THEN** the cursor moves up one position in the engine/profile list
- **AND** the behavior is identical to pressing Up arrow

#### Scenario: Help text shows j/k keys
- **WHEN** Step 1 help footer is rendered
- **THEN** the navigation hint shows `↑↓/jk:navigate`
