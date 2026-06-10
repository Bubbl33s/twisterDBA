## 1. Dependencies & Module Setup

- [x] 1.1 Add `mlua = { version = "0.10", features = ["luajit", "vendored", "serialize"] }` to `Cargo.toml`
- [x] 1.2 Create `src/lua/mod.rs` with `pub struct LuaRuntime { lua: mlua::Lua, keymaps: HashMap<String, mlua::Function>, commands: HashMap<String, mlua::Function>, hooks: HashMap<String, Vec<mlua::Function>>, extractors: HashMap<String, mlua::Function> }`
- [x] 1.3 Create `src/lua/api.rs` with `TwisterDBA` userdata struct implementing API methods
- [x] 1.4 Create `src/lua/hooks.rs` with event dispatch logic
- [x] 1.5 Add `pub mod lua;` to `src/main.rs`
- [x] 1.6 Run `cargo build` and verify mlua compiles (may take 1-2 minutes for LuaJIT vendored build)

## 2. `TwisterDBA` API Implementation (`src/lua/api.rs`)

- [x] 2.1 Implement `TwisterDBA::setup_theme(&self, theme_table: mlua::Table)` parsing partial theme overrides
- [x] 2.2 Implement `TwisterDBA::set_keymap(&self, mode: String, lhs: String, rhs: mlua::Function, opts: Option<mlua::Table>)` storing keymap in runtime
- [x] 2.3 Implement `TwisterDBA::register_command(&self, name: String, callback: mlua::Function)` storing command
- [x] 2.4 Implement `TwisterDBA::register_extractor(&self, name: String, callback: mlua::Function)` storing extractor
- [x] 2.5 Implement `TwisterDBA::on_event(&self, event: String, callback: mlua::Function)` registering hook
- [x] 2.6 Implement `TwisterDBA::execute_active_statement(&self)` sending DB command via stored channel reference
- [x] 2.7 Run `cargo build` and verify API compiles

## 3. Lua Runtime Initialization (`src/lua/mod.rs`)

- [x] 3.1 Implement `LuaRuntime::new() -> Result<Self>` creating Lua instance, removing dangerous globals (os.execute, io), and setting up `twisterDBA` global
- [x] 3.2 Implement `LuaRuntime::load_init()` reading and executing `~/.config/twisterDBA/init.lua`
- [x] 3.3 Implement `LuaRuntime::load_plugins()` scanning `lua/` and `after/plugin/` directories, executing `.lua` files in alphabetical order
- [x] 3.4 Handle Lua errors gracefully: log error with file path and line, continue execution
- [x] 3.5 Implement `LuaRuntime::execute(code: &str) -> Result<String>` for `:lua` command REPL
- [x] 3.6 Run `cargo build` and test init.lua loading

## 4. Sandbox Configuration

- [x] 4.1 Remove `os` global (`lua.globals().set("os", Nil)`)
- [x] 4.2 Remove `io` global
- [x] 4.3 Set `mlua` hook for instruction limit (e.g., 10M instructions) to prevent infinite loops
- [x] 4.4 Keep `string`, `table`, `math`, `coroutine` (safe standard libraries)
- [x] 4.5 Run `cargo build` and verify sandbox

## 5. Integration with AppState (`src/state.rs`, `src/app.rs`)

- [x] 5.1 Add `lua_runtime: Option<LuaRuntime>` to `AppState`
- [x] 5.2 In `App::new()`, create `LuaRuntime`, load init.lua and plugins, store on state
- [x] 5.3 In `AppState::handle_key`, before hardcoded keybindings, check `lua_runtime.keymaps` for a matching entry; if found, call the Lua function
- [x] 5.4 Add `:lua <code>` command to `execute_command` executing arbitrary Lua via `LuaRuntime::execute`
- [x] 5.5 Update `:help` command to list Lua-registered commands alongside built-in commands
- [x] 5.6 Update theme access: when `lua_runtime` is `Some`, theme values from Lua override defaults
- [x] 5.7 Run `cargo build` and `cargo clippy`

## 6. Event Hooks (`src/lua/hooks.rs`, `src/state.rs`)

- [x] 6.1 Implement `LuaRuntime::fire_event(event: &str, data: mlua::Table)` iterating registered hooks and calling each
- [x] 6.2 In `state.rs::apply_db_event`, fire `ConnectionOpened` on `DbEvent::Connected`, `ConnectionClosed` on `DbEvent::Disconnected`, `QueryExecuted` on `DbEvent::QueryCompleted`
- [x] 6.3 Build event data tables with relevant fields (dsn, sql, rows_affected, duration_ms)
- [x] 6.4 Handle hook errors: log, continue to next hook, don't crash
- [x] 6.5 Run `cargo build` and test hook firing

## 7. Custom Extractors (`src/state.rs`)

- [x] 7.1 Update `:export <extractor> <path>` command to check Lua extractors before built-in formats
- [x] 7.2 Implement `call_extractor(name, columns, rows) -> Result<String>` calling the Lua extractor function with result data as Lua tables
- [x] 7.3 Build Lua table from `result_grid.columns` and `result_grid.rows` to pass to extractor
- [x] 7.4 Write returned string to the specified file path
- [x] 7.5 Run `cargo build` and test custom extractor

## 8. Verification

- [x] 8.1 Run `cargo build` and verify no errors
- [x] 8.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 8.3 Run `cargo fmt --check` and verify formatting
- [x] 8.4 Run `cargo test` and verify all tests pass
- [ ] 8.5 Manually test init.lua with custom theme: create `~/.config/twisterDBA/init.lua`, verify colors applied
- [ ] 8.6 Manually test custom keybinding: `set_keymap('n', 'J', function() print('Hello') end)`, press J
- [ ] 8.7 Manually test custom command: `register_command('hello', callback)`, run `:hello`
- [ ] 8.8 Manually test event hooks: register `on_event('QueryExecuted', callback)`, run a query, verify callback fires
- [ ] 8.9 Manually test sandbox: attempt `os.execute('ls')` in init.lua, verify blocked
- [ ] 8.10 Manually test `:lua print('test')` REPL command
