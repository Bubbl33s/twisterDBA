# twisterDBA

> A DataGrip-class database TUI built in Rust. Zero mouse, zero bloat.

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.85+-orange.svg)](https://www.rust-lang.org)

**twisterDBA** is a keyboard-first, terminal-based SQL client for backend and data engineers who live in the terminal. Vim modal editing, Tree-sitter SQL parsing, schema introspection, multi-DB support, and Lua scripting — no JVM, no Electron, no context switch.

## Features

- **Multi-DB support** — PostgreSQL, MySQL/MariaDB, SQLite
- **Schema Explorer** — Introspect and navigate databases, schemas, tables, views, columns, indexes, foreign keys, and routines in a tree view
- **Schema-Aware SQL Editor** — Tree-sitter syntax highlighting, statement-under-cursor execution, Vim modal editing (Normal / Insert / Visual)
- **Results Grid** — Virtual scrolling, inline cell editing, auto-pagination, CSV export
- **Floating Modals** — Connection Manager, Quick Documentation, Keymap Help, Command Palette
- **Lua Scripting** — Custom themes, keybindings, commands, and hooks via LuaJIT
- **OS Keychain** — Credentials stored securely via Keychain Services / Credential Manager / Secret Service
- **Cloud Sync** (E2EE) — AES-256-GCM encrypted credential and console sync across machines
- **Nerd Font Icons** — Color-coded entity types with fallback ASCII

## Installation

### Prerequisites

- Rust 1.85+ (via [rustup](https://rustup.rs))
- A [Nerd Font](https://www.nerdfonts.com) for icons (optional, falls back to ASCII)
- TrueColor terminal (24-bit ANSI)

### From Source

```bash
git clone https://github.com/twisterDBA/twisterDBA.git
cd twisterDBA
cargo build --release
```

The binary will be at `target/release/twisterDBA`.

### Post-Install

Activate git hooks for development:

```bash
make githooks
```

## Quick Start

1. Launch twisterDBA:
   ```bash
   ./target/release/twisterDBA
   ```

2. Open the Connection Manager with `<leader>c` and enter your database credentials.

3. Navigate the schema tree (left panel) and the query editor (center) with Vim keys.

4. Write SQL and press `<leader>r` to execute the statement under the cursor.

5. Press `?` for context-sensitive keymap help.

## Configuration

Configuration lives at `~/.config/twisterDBA/config.toml`:

```toml
[connections.postgres_local]
host = "localhost"
port = 5432
user = "postgres"
password = "<keychain>"
database = "mydb"
```

Use `password = "<keychain>"` to store credentials in your OS keychain instead of plaintext.

## Key Bindings

| Key | Mode | Action |
|-----|------|--------|
| `h` / `j` / `k` / `l` | Normal | Navigate panels |
| `i` | Normal | Enter Insert mode |
| `v` | Normal | Enter Visual mode |
| `<Esc>` | Insert / Visual | Return to Normal mode |
| `<leader>r` | Normal | Execute statement under cursor |
| `<leader>c` | Normal | Open Connection Manager |
| `?` | Normal | Toggle keymap help |
| `:` | Normal | Command palette |
| `Ctrl+Q` / `K` | Normal | Quick documentation |

## Scripting

Customize twisterDBA with Lua (`~/.config/twisterDBA/init.lua`):

```lua
local twister = require("twisterDBA")

twister.setup_theme({
  background = "#1E1F22",
  keyword    = "#CC7832",
  accent     = "#9876AA",
})

twister.set_keymap("n", "<leader>q", function()
  twister.execute_active_statement()
end)
```

Plugins autoload from `~/.config/twisterDBA/lua/`.

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines on reporting bugs, proposing features, and submitting pull requests.

## Tech Stack

| Layer | Choice |
|-------|--------|
| Language | Rust |
| TUI | Ratatui + Crossterm |
| Async | Tokio |
| SQL Parsing | Tree-sitter (tree-sitter-sequel) |
| Database Driver | SQLx (PostgreSQL, MySQL, SQLite) |
| Scripting | mlua + LuaJIT |
| Credentials | keyring (OS keychain) |

## License

MIT — see [LICENSE](LICENSE).
