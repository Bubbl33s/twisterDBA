# Contributing to twisterDBA

Thanks for your interest in contributing.

## Getting Started

1. Fork the repository and clone it locally.
2. Install Rust via [rustup](https://rustup.rs) (1.85+).
3. Install DX tools and activate git hooks:
   ```bash
   make install-tools
   make githooks
   ```

## Development Workflow

- **Build:** `make build` or `cargo build`
- **Full check:** `make check` (fmt + clippy + build + test)
- **Auto-fix:** `make fix` (fmt + clippy --fix)
- **Tests:** `make test` or `bash ai-specs/scripts/cargo-verify.sh`

The full verification pipeline runs: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo build`, and `cargo test`.

## Code Conventions

- **TEA Pattern** — `AppState` is the single source of truth, owned exclusively by the main thread. Never use `Arc<Mutex<AppState>>`.
- **Async Isolation** — All PostgreSQL I/O runs in Tokio tasks, never in the render loop.
- **Pure Render** — `fn render(f: &mut Frame, state: &AppState)` is an immutable borrow with no side effects.
- **Message-Driven DB** — DB layer communicates via `mpsc::unbounded_channel`. No shared state.
- **FSM Modal** — Key routing goes through the `Mode` enum (Normal, Insert, Command, ConnectDialog, Visual).
- **No unsafe code** — `unsafe_code = "forbid"` in Cargo.toml.
- **No unwrap in production** — Use proper error handling; `unwrap_used` and `expect_used` are warned against.

## Commit Messages

Use [Conventional Commits](https://www.conventionalcommits.org/):

```
type(scope): description
```

Types: `feat`, `fix`, `refactor`, `chore`, `docs`, `test`, `style`, `perf`
Scope: the `src/` module name (e.g., `editor`, `state`, `events`, `db`, `result`, `explorer`)

All messages in English.

## Pull Requests

- Open an issue first for major changes to discuss the approach.
- Keep PRs focused on a single feature or fix.
- Ensure `make check` passes before submitting.
- Add or update tests as appropriate.
- Update OpenSpec artifacts if the change touches spec-driven features.

## Spec-Driven Development

This project uses OpenSpec for spec-driven development. See `AGENTS.md` and the `openspec/` directory for the workflow. When a change affects spec-tracked features, update the corresponding artifacts in `openspec/changes/`.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
