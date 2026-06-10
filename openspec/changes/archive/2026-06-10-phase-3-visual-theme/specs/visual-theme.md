## ADDED Requirements

### Requirement: Darcula TrueColor Palette
The application SHALL render all UI elements using the JetBrains Darcula-inspired TrueColor palette with 24-bit ANSI colors. Colors SHALL be defined in one central location (`src/theme.rs`).

#### Scenario: Background color applied globally
- **WHEN** the application starts
- **THEN** the terminal background renders as `#2B2B2B` and the editor area renders as `#1E1F22`

#### Scenario: SQL syntax colors match Darcula spec
- **WHEN** a SQL query `SELECT 'hello' FROM users WHERE active = true` is displayed
- **THEN** `SELECT`, `FROM`, `WHERE` render in amber/orange `#CC7832`, `'hello'` in olive green `#6A8759`, `true` in light blue `#6897BB`, and any comments in grey `#808080`

#### Scenario: Theme accessible from Lua (Phase 7 compatible)
- **WHEN** a Lua script calls `twisterDBA.setup_theme({...})`
- **THEN** the theme struct in Rust is updated and the next render frame reflects the new colors

#### Scenario: Terminal without TrueColor falls back gracefully
- **WHEN** the terminal does not support 24-bit color (e.g., `TERM=xterm-256color`)
- **THEN** colors degrade to the nearest 256-color palette equivalent without crashing

### Requirement: Borderless Panel Rendering
All panels (schema explorer, query editor, result grid, status bar) SHALL render without `Borders::ALL` or any box-drawing characters. Panel separation SHALL be achieved through background color contrast and blank space.

#### Scenario: No box-drawing characters visible
- **WHEN** the main area renders with three panels
- **THEN** zero `─`, `│`, `┌`, `┐`, `└`, `┘` characters appear in the rendered output

#### Scenario: Panel separation by background color
- **WHEN** the schema explorer (left panel) and SQL editor (center panel) render side by side
- **THEN** the visual boundary between them is a 2-column gap of the global background color (`#2B2B2B`)

#### Scenario: Panel titles rendered inline
- **WHEN** each panel needs a title (e.g., "Schema Explorer", "SQL Editor")
- **THEN** the title is rendered as a text label at the top-left of the panel area, using the panel's background color (not a bordered block)

### Requirement: Statusline Focus Communication
Panel focus SHALL be communicated exclusively through the statusline background color, not via border color changes.

#### Scenario: Active panel has vibrant statusline
- **WHEN** the SQL editor is focused
- **THEN** the statusline mode indicator (mode + connection + keybindings) renders with a vibrant blue background (`#3A6EA5`) and black text

#### Scenario: Inactive panels have muted statusline
- **WHEN** the schema explorer is NOT focused
- **THEN** its statusline area renders in a muted dark grey background with grey text, clearly distinct from the active panel

#### Scenario: Mode change updates statusline instantly
- **WHEN** user switches from Normal to Insert mode
- **THEN** the mode indicator in the statusline changes from "NORMAL" (grey bg) to "INSERT" (green bg) within the same render frame
