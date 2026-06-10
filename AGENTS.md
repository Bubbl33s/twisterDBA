## 1. Core Principles

- **Small tasks, one at a time**: Always work in baby steps, one at a time. Never go forward more than one step.
- **Test-Driven Development**: Run `cargo test` before and after any code change.
- **Type Safety**: All Rust code must compile without errors or warnings.
- **Clear Naming**: Use descriptive names for variables, functions, enums, and modules.
- **Incremental Changes**: Prefer incremental, focused changes over large modifications.
- **Question Assumptions**: Always question assumptions and inferences.
- **Pattern Detection**: Detect and highlight repeated code patterns for refactoring.

## 2. Language Standards

- **English Only**: All technical artifacts must always use English, including:
    - Code (variables, functions, enums, comments, error messages, log messages)
    - Documentation (README, specs, design docs)
    - OpenSpec artifacts (proposals, specs, design, tasks)
    - Configuration files and scripts
    - Git commit messages
    - Test names and descriptions

## 3. Project Skills

- Skills live in `ai-specs/skills`.
- When a request matches a skill, load and follow the corresponding `SKILL.md` automatically before continuing.
- Also load any referenced files in the skill folder when the skill requires them.

## 4. OpenSpec Workflow

This project uses OpenSpec for spec-driven development. The workflow:

```
/opsx-propose ──► /opsx-ff ──► /opsx-apply ──► /opsx-verify ──► /opsx-archive
```

Custom commands:
- `/derive <doc.md>` — Parse project document and generate all phase changes
- `/opsx-ff <name>` — Fast-forward: create all artifacts for a change
- `/opsx-continue` — Step-by-step artifact creation
- `/opsx-verify` — Validate implementation against specs

## 5. Rust Build Hygiene

Before marking any task complete, verify:
- [ ] `cargo build` — no errors
- [ ] `cargo clippy -- -D warnings` — no warnings
- [ ] `cargo fmt --check` — properly formatted
- [ ] `cargo test` — all tests green

Use `bash ai-specs/scripts/cargo-verify.sh` for the full pipeline.

## 6. Architecture Constraints

- **TEA Pattern**: AppState is the single source of truth, owned exclusively by the main thread. Never use `Arc<Mutex<AppState>>`.
- **Async Isolation**: All PostgreSQL I/O runs in Tokio tasks, never in the render loop.
- **Pure Render**: `fn render(f: &mut Frame, state: &AppState)` — immutable borrow, no side effects.
- **Message-Driven DB**: DB layer communicates via `mpsc::unbounded_channel`. No shared state.
- **FSM Modal**: Key routing must go through the Mode enum (Normal, Insert, Command, ConnectDialog, Visual).

## 7. Module Structure

The codebase is organized into focused modules to maintain clarity and single responsibility:

### `state/` - Application State (from original 2712-line state.rs)

| File | Responsibility |
|------|----------------|
| `mod.rs` | AppState struct, EditorSplit, core navigation methods |
| `mode.rs` | Mode enum, SplitDirection, Window |
| `connection.rs` | ConnectionStatus, ConnectForm, ConnectField, DSN builders |
| `output.rs` | OutputPaneState, OutputResultsState, ResultTab, CellPopupState |
| `popup.rs` | PopupState enum |
| `session.rs` | SessionData, BufferSnapshot, save/load functions |
| `events.rs` | apply_db_event implementation |
| `handlers/` | Key handling dispatch |
| `handlers/mod.rs` | handle_key dispatch, utility functions |
| `handlers/normal.rs` | handle_normal_key, handle_editor_normal_key, handle_window_command |
| `handlers/insert.rs` | handle_insert_key |
| `handlers/command.rs` | handle_command_key, execute_command, mask_raw_dsn |
| `handlers/visual.rs` | handle_visual_key |
| `handlers/connect.rs` | handle_connect_dialog_key |
| `handlers/popup.rs` | handle_popup_key, open_quick_doc |
| `handlers/result.rs` | handle_result_normal_key, handle_edit_mode, commit_cell_edit |
| `handlers/explorer.rs` | handle_explorer_key, explorer_toggle_expand |

### `ui/` - Rendering (from original 1040-line ui.rs)

| File | Responsibility |
|------|----------------|
| `mod.rs` | Main render(), render_main_area layout composition |
| `explorer.rs` | render_schema_explorer |
| `editor.rs` | render_sql_editor, render_single_editor, render_editor_tab_bar |
| `output.rs` | render_output_panel, render_output_pane, render_output_tab_bar |
| `result.rs` | render_result_grid |
| `status.rs` | render_status_bar, render_connection_status |
| `dialog.rs` | render_connect_dialog |
| `popup.rs` | render_cell_popup, render_quick_doc, render_keymap_help |
| `utils.rs` | centered_rect, format_count, render_dialog_backdrop, render_help_footer |

### `editor/` - SQL Editor (from original 829-line editor/mod.rs)

| File | Responsibility |
|------|----------------|
| `mod.rs` | SqlEditor struct, Direction enum, query execution |
| `buffer.rs` | TextBuffer - text manipulation, cursor movement |
| `history.rs` | QueryHistory - SQL history navigation |
| `render.rs` | render_line_with_cursor |
| `tree.rs` | Tree-sitter parsing and syntax highlighting |

### Design Principles

- **Max file size**: ~400-500 lines (previously 2712 lines in state.rs)
- **Single responsibility**: Each file has one clear purpose
- **Cohesive grouping**: Related functionality stays together
- **Clear boundaries**: Public APIs are explicit via `pub use` re-exports

## 8. Mandatory OpenSpec Artifact Updates

When a new fix/change request appears after implementation and before archive, update artifacts first:

1. Update the affected OpenSpec change artifacts (scenarios, requirements, tasks)
2. If artifact regeneration is needed, run `/opsx-continue` or `/opsx-ff` before coding
3. Implement code only after artifacts reflect the new request
4. Re-run `/opsx-verify` against updated artifacts before archiving

Do not apply direct code-only fixes without updating OpenSpec artifacts.
