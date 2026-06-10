## Why

The product spec defines a Lua scripting engine as a core extensibility mechanism: "Powered by mlua + LuaJIT: near-zero latency, minimal RAM, no recompile needed for customization." Users should be able to customize themes, keybindings, and behavior via `~/.config/twisterDBA/init.lua` — similar to how Neovim users configure their editor. The exposed API surface includes `set_keymap`, `register_command`, `register_extractor`, and `on_event` hooks.

## What Changes

- Integrate `mlua` with LuaJIT backend for embedded Lua scripting
- Create `src/lua/` module with `LuaRuntime` wrapping an `mlua::Lua` instance
- Expose `twisterDBA` global table with API functions: `setup_theme`, `set_keymap`, `register_command`, `register_extractor`, `on_event`
- Implement plugin autoload from `~/.config/twisterDBA/lua/` and `~/.config/twisterDBA/after/plugin/` (Neovim-compatible layout)
- Fire event hooks: `QueryExecuted`, `ConnectionOpened`, `ConnectionClosed`
- Keymap callback dispatch: user-defined Lua functions called on key presses
- Command palette integration: `register_command` entries appear in `:help` and are executable

## Capabilities

### New Capabilities
- `lua-scripting-engine`: Embedded LuaJIT runtime with `mlua`, executing user scripts on startup
- `lua-api-surface`: `twisterDBA.*` global table with theme, keymap, command, extractor, and event hook APIs
- `plugin-autoload`: Automatic loading of Lua plugins from config directories, mirroring Neovim's runtimepath

### Modified Capabilities
- `visual-theme`: Theme configurable via `twisterDBA.setup_theme()`
- `keybindings`: Dynamic keybindings via `twisterDBA.set_keymap()` override compile-time defaults

## Impact

- `Cargo.toml`: Add `mlua = { version = "0.10", features = ["luajit", "vendored", "serialize"] }`
- `src/lua/`: New module tree: `mod.rs` (LuaRuntime), `api.rs` (API surface), `hooks.rs` (event dispatch)
- `src/main.rs`: `mod lua;` declaration
- `src/app.rs`: Initialize `LuaRuntime` after state creation; fire hooks on connection/query events
- `src/state.rs`: Theme and keymap fields become Lua-updatable; `handle_key` checks Lua keymap overrides
- `src/config/`: No changes (Lua is separate from TOML config)
