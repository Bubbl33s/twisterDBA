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

## 7. Mandatory OpenSpec Artifact Updates

When a new fix/change request appears after implementation and before archive, update artifacts first:

1. Update the affected OpenSpec change artifacts (scenarios, requirements, tasks)
2. If artifact regeneration is needed, run `/opsx-continue` or `/opsx-ff` before coding
3. Implement code only after artifacts reflect the new request
4. Re-run `/opsx-verify` against updated artifacts before archiving

Do not apply direct code-only fixes without updating OpenSpec artifacts.
