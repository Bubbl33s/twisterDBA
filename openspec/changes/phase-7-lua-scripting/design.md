## Context

The product spec prescribes a Lua scripting engine for extensibility: "Powered by mlua + LuaJIT: near-zero latency, minimal RAM, no recompile needed for customization." This is analogous to Neovim's Lua configuration model — users write `~/.config/twisterDBA/init.lua` to customize themes, keybindings, commands, and event hooks.

`mlua` is the de-facto Rust binding for Lua. With the `luajit` feature, it uses LuaJIT which is significantly faster than PUC-Lua and has a smaller memory footprint. The `vendored` feature compiles LuaJIT from source, avoiding system library dependencies.

## Goals / Non-Goals

**Goals:**
- Embedded LuaJIT runtime via `mlua`
- Load `init.lua`, `lua/` plugins, `after/plugin/` plugins on startup
- Expose `twisterDBA.*` API: `setup_theme`, `set_keymap`, `register_command`, `register_extractor`, `on_event`
- Fire event hooks on connection open/close and query completion
- Sandbox dangerous functions (`os.execute`, `io.popen`)
- Lua callbacks execute via message-passing to main thread (no shared state)

**Non-Goals:**
- Hot-reload of Lua files (reload on `:source` command only)
- Debug adapter protocol or Lua debugging support
- External Lua modules/libraries (only stdlib + twisterDBA API)
- Coroutine-based async Lua APIs

## Decisions

### ADR-001: mlua with vendored LuaJIT
**Choice:** `mlua = { version = "0.10", features = ["luajit", "vendored", "serialize"] }`.

**Rationale:** LuaJIT is 10-50x faster than PUC-Lua for typical config scripts. Vendoring avoids requiring LuaJIT development headers on the build machine. The `serialize` feature enables passing structured data (theme tables, result grids) between Rust and Lua.

**Alternative considered:** `rlua` — less maintained, no LuaJIT support, lacks serialize feature.

### ADR-002: API via Lua UserData and Global Table
**Choice:** Expose API functions as methods on a `TwisterDBA` Rust struct registered as a Lua userdata. Set it as a global `twisterDBA` in the Lua context. Each API method is a `mlua::Function` callback.

**Rationale:** UserData provides type-safe Rust↔Lua interop. The global table pattern (`twisterDBA.setup_theme(...)`) matches the spec's documented API and Neovim's convention.

**Alternative considered:** Flat global functions (`setup_theme(...)`) — namespace pollution; harder to document.

### ADR-003: Keymap Dispatch via Event Channel
**Choice:** `set_keymap` stores `(mode, key_sequence, Lua function_ref)` in a `HashMap` on `LuaRuntime`. On key press, `AppState::handle_key` checks the Lua keymap table before hardcoded defaults. If a match is found, the Lua function is called via `mlua::Function::call()`.

**Rationale:** The keymap check is O(1) per key press. Lua functions return immediately (they should be non-blocking). Long-running Lua code is guarded by mlua's instruction limit.

### ADR-004: Sandbox via `remove_global`
**Choice:** After creating the Lua instance, call `lua.globals().set("os", mlua::Value::Nil)?` and `lua.globals().set("io", mlua::Value::Nil)?` to remove dangerous modules.

**Rationale:** Simple, effective, and standard practice for embedded Lua. Malicious plugins cannot re-load these modules because `require` is also restricted.

**Alternative considered:** Custom `load` function override — more complex; `remove_global` is sufficient for MVP threat model.

### ADR-005: Event Hooks via Callback Registry
**Choice:** `LuaRuntime.hooks: HashMap<String, Vec<mlua::Function>>`. On event fire, iterate hooks for the event name and call each function with the event data as a Lua table.

**Rationale:** Multiple hooks per event are supported (Neovim-compatible). Hooks are called synchronously on the main thread but should be fast (no I/O). If a hook errors, it's logged and the next hook runs.

## Risks / Trade-offs

- **[Risk] LuaJIT compilation increases build time (~45s)** → Mitigation: `vendored` feature compiles LuaJIT once; cached in `target/`
- **[Risk] mlua panics on Rust panics in Lua callbacks** → Mitigation: wrap all API methods in `catch_unwind`; log the panic and return a Lua error
- **[Risk] Plugin loading order may cause non-deterministic behavior** → Mitigation: alphabetical ordering is predictable; document recommended loading strategy
- **[Risk] Lua memory leaks from leaked userdata references** → Mitigation: `mlua` uses Lua's garbage collector; Rust resources are freed via `Drop` trait

## Migration Plan

1. Add `mlua` dependency
2. Create `src/lua/mod.rs` with `LuaRuntime` struct
3. Implement `TwisterDBA` userdata with API methods
4. Add plugin loader scanning `lua/` and `after/plugin/`
5. Integrate into `App::new()` (load Lua after state creation)
6. Wire keymap dispatch in `AppState::handle_key`
7. Fire event hooks in `apply_db_event`
8. No data migration needed

## Open Questions

- Should Lua plugins have access to the filesystem for custom export paths? (Answer: limited — only via `:export` command; direct `io` is blocked)
- Should there be a `:lua` command to execute arbitrary Lua? (Answer: yes, useful for debugging; `:lua print('hello')` executes inline Lua)
