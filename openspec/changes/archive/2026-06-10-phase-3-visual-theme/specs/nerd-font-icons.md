## ADDED Requirements

### Requirement: Nerd Font Entity Icons
Schema explorer nodes SHALL display Nerd Font icons color-coded by entity type, as specified in the product spec icon table.

#### Scenario: Database icon renders correctly
- **WHEN** the explorer shows a database/connection node
- **THEN** the node is prefixed with `󰆼 ` (Nerd Font codepoint `U+F06FC`) in turquoise `#4DB6AC`

#### Scenario: Table icon renders correctly
- **WHEN** the explorer shows a table node
- **THEN** the node is prefixed with ` ` (Nerd Font codepoint `U+F021A`) in ocean blue `#548AF7`

#### Scenario: View icon renders correctly
- **WHEN** the explorer shows a view node
- **THEN** the node is prefixed with `󰈙 ` (Nerd Font codepoint `U+F0219`) in purple `#BA68C8`

#### Scenario: Routine icon renders correctly
- **WHEN** the explorer shows a routine/stored procedure node
- **THEN** the node is prefixed with ` ` (Nerd Font codepoint `U+F0B21`) in soft red `#E57373`

#### Scenario: Missing Nerd Font falls back to ASCII
- **WHEN** the terminal font does not include Nerd Font glyphs
- **THEN** icons render as their fallback ASCII equivalents: `[DB]` for database, `[T]` for table, `[V]` for view, `[R]` for routine, preserving color coding but without Nerd Font glyphs

### Requirement: Icon Color Mapping
Each entity type SHALL have a fixed color from the product spec, applied to both the icon and the node name text.

#### Scenario: Schema/folder node in gold
- **WHEN** the explorer shows a schema node
- **THEN** the node name and icon render in gold `#FFCB6B`; child nodes (tables, views) use their respective colors

#### Scenario: Expanded schema shows children with own colors
- **WHEN** a schema node is expanded showing tables and views beneath
- **THEN** child table nodes use ocean blue (`#548AF7`) icons and child view nodes use purple (`#BA68C8`) icons, not inheriting the parent's gold color

#### Scenario: Column nodes use muted style
- **WHEN** a table node is expanded showing column children
- **THEN** column nodes render without an icon, in a muted grey color, with type information in parentheses
