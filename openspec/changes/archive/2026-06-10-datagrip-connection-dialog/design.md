## ADR-001: Two-step ConnectForm with step enum

**Decision**: `ConnectForm` gains a `step: DialogStep` field where `DialogStep` is `SelectType` or `EnterDetails`. The form state machine:

```
SelectType → (Enter on type or profile) → EnterDetails
EnterDetails → (Esc) → SelectType
EnterDetails → (Enter) → Connect
SelectType → (Esc) → Close dialog
```

```rust
pub enum DialogStep {
    SelectType,
    EnterDetails,
}

pub struct ConnectForm {
    pub step: DialogStep,
    pub db_type: usize,           // 0=Postgres, 1=MySQL, 2=SQLite
    pub selected_profile: Option<usize>,  // index into filtered profiles for selected type
    pub connection_name: String,  // editable name for the connection
    pub fields: Vec<ConnectField>,
    pub active_field: usize,
    pub type_cursor: usize,       // cursor position in type grid (row, col)
}
```

**Rationale**:
- Clear state machine prevents invalid transitions
- `SelectType` step is read-only except for navigation and selection
- `EnterDetails` step is the familiar form editing
- Esc in EnterDetails goes back, not closes — matches DataGrip

## ADR-002: Type grid layout

**Decision**: Step 1 renders a 3-column grid (or single row of 3 items) showing each database type with its icon and name. Below the grid, saved profiles are listed, grouped by engine type.

```
┌─────────────────────────────────────┐
│         New Connection              │
│                                     │
│   🐘 PostgreSQL   🐬 MySQL   🪶 SQLite │  ← type grid
│                                     │
│   Saved Connections:                │
│   🐘 local-postgres  (localhost)    │  ← profile list
│   🐬 staging-mysql   (10.0.0.1)     │
│                                     │
│   Tab/↓↑:navigate  Enter:select     │
│   Esc:cancel                        │
└─────────────────────────────────────
```

**Rationale**:
- Visual grid is more discoverable than a dropdown
- Profiles listed below provide quick access to saved connections
- Engine icons next to profiles reinforce the type association

## ADR-003: Engine-specific field sets

**Decision**: Each engine type defines its own field set:

**PostgreSQL** (6 fields):
1. Host (default: localhost)
2. Port (default: 5432)
3. Database
4. User
5. Password
6. SSL Mode (dropdown: disable, allow, prefer, require, verify-ca, verify-full; default: prefer)

**MySQL** (5 fields):
1. Host (default: localhost)
2. Port (default: 3306)
3. Database
4. User
5. Password

**SQLite** (1 field):
1. File Path

**Rationale**:
- PostgreSQL SSL mode is important for production connections
- MySQL doesn't need SSL mode in the basic dialog (can be added via DSN params later)
- SQLite only needs a file path
- Field defaults match each engine's conventional defaults

## ADR-004: Connection name auto-generation and override

**Decision**: When entering Step 2 (either from type selection or profile selection), the connection name is auto-generated:
- From type selection: `{engine}-{host}` (e.g., `postgres-localhost`)
- From profile selection: use the profile's existing name

The name field is editable. On connect, if the name already exists, the user is prompted to confirm overwrite or enter a new name.

**Rationale**:
- Auto-generation provides sensible defaults
- Editability allows meaningful names like "prod-primary" or "analytics-db"
- Conflict prevention avoids accidental overwrites

## ADR-005: Profile selection auto-fills Step 2

**Decision**: Selecting a saved profile in Step 1 auto-fills all fields in Step 2, including the connection name (from the profile's name). The password is loaded from keychain if stored there.

**Rationale**:
- Quick reconnection to saved profiles
- Password from keychain means user doesn't re-enter it
- Connection name preserved from profile

## ADR-006: Dialog rendering with two layouts

**Decision**: `render_connect_dialog` switches between two layouts based on `form.step`:
- `SelectType`: Type grid + profile list + navigation help
- `EnterDetails`: Connection name field + engine-specific fields + connect/cancel help

The dialog size adjusts: Step 1 is wider (for the grid), Step 2 is narrower (for the form).

**Rationale**:
- Each step has optimal layout for its content
- Transition between steps feels natural
- Consistent with DataGrip's approach
