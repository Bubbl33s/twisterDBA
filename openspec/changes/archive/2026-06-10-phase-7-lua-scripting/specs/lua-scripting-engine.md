## ADDED Requirements

### Requirement: Lua Runtime Initialization
The application SHALL embed a LuaJIT runtime via `mlua`, loading and executing `~/.config/twisterDBA/init.lua` at startup if the file exists. Errors in user Lua scripts SHALL be logged via `tracing` and SHALL NOT crash the application.

#### Scenario: init.lua loads successfully
- **WHEN** the application starts and `~/.config/twisterDBA/init.lua` contains valid Lua code calling `twisterDBA.setup_theme({...})`
- **THEN** the theme is applied before the first render frame; the status bar shows "Lua: init.lua loaded"

#### Scenario: init.lua has syntax error
- **WHEN** `init.lua` contains `twisterDBA.setup_theme({ background = )` (malformed)
- **THEN** the error is logged with file path and line number; the application continues with default settings; output pane shows "Lua error in init.lua:5: unexpected symbol near ')'"

#### Scenario: No init.lua exists
- **WHEN** `~/.config/twisterDBA/init.lua` does not exist
- **THEN** the application starts normally with no Lua overhead; no error messages

### Requirement: Plugin Autoload
The application SHALL scan `~/.config/twisterDBA/lua/` and `~/.config/twisterDBA/after/plugin/` for `.lua` files and execute them in order. Plugins in `after/plugin/` SHALL load after all other plugins (Neovim-compatible ordering).

#### Scenario: Plugin directory scanned
- **WHEN** `~/.config/twisterDBA/lua/` contains `my_theme.lua` and `custom_keys.lua`
- **THEN** both files are loaded in alphabetical order before the first render frame

#### Scenario: Plugin error isolated
- **WHEN** `my_theme.lua` loads successfully but `custom_keys.lua` raises an error
- **THEN** the error is logged for `custom_keys.lua`; `my_theme.lua`'s settings remain applied; the application continues

#### Scenario: Plugin requires a module
- **WHEN** `~/.config/twisterDBA/lua/myplugin/init.lua` calls `local twisterDBA = require('twisterDBA')`
- **THEN** the `twisterDBA` global table is accessible (exposed as a Lua module)

### Requirement: Exposed API — setup_theme
The Lua API SHALL expose `twisterDBA.setup_theme(config)` accepting a table with optional fields: `background`, `editor_bg`, `keyword`, `string`, `number`, `comment`, `accent`, and `icons` (sub-table with per-entity icon/color overrides).

#### Scenario: Partial theme override
- **WHEN** `setup_theme({ keyword = "#FF0000", background = "#000000" })` is called in init.lua
- **THEN** keyword color changes to red and background to black; all other colors remain Darcula defaults

#### Scenario: Icon override
- **WHEN** `setup_theme({ icons = { table = { icon = "∏", color = "#FF00FF" } } })` is called
- **THEN** table nodes in the explorer render with the custom icon (`∏`) in magenta; other entity icons unchanged

#### Scenario: Invalid theme value logged
- **WHEN** `setup_theme({ background = "not a color" })` is called
- **THEN** the error is logged; the background keeps its previous value; no crash

### Requirement: Exposed API — set_keymap
The Lua API SHALL expose `twisterDBA.set_keymap(mode, lhs, rhs, opts?)` where mode is `"n"`, `"i"`, `"v"`, `"c"` (Normal, Insert, Visual, Command), lhs is a key sequence string, rhs is a Lua function, and opts is an optional table.

#### Scenario: Custom keybinding in Normal mode
- **WHEN** `set_keymap('n', '<leader>r', function() twisterDBA.execute_active_statement() end)` is called in init.lua
- **THEN** pressing `<leader>r` (e.g., `\r`) in Normal mode calls the Lua function; the Lua function calls back into Rust to execute the active statement

#### Scenario: Keybinding overrides default
- **WHEN** `set_keymap('n', 'j', function() print('custom j') end)` is called
- **THEN** pressing `j` in Normal mode executes the Lua function instead of the default `move_down` behavior

#### Scenario: Invalid mode string
- **WHEN** `set_keymap('x', 'a', function() end)` is called with unrecognized mode `"x"`
- **THEN** the error is logged and the keybinding is not registered

### Requirement: Exposed API — register_command
The Lua API SHALL expose `twisterDBA.register_command(name, callback)` where name is a string and callback is a Lua function. Registered commands SHALL appear in the `:help` command list and SHALL be invocable via `:name`.

#### Scenario: Register and invoke a custom command
- **WHEN** `register_command('hello', function() print('Hello from Lua!') end)` is called in init.lua
- **THEN** running `:hello` in command mode invokes the Lua function; "Hello from Lua!" appears in the output pane

#### Scenario: Duplicate command name
- **WHEN** `register_command('quit', function() end)` overrides the built-in `:quit`
- **THEN** the custom function replaces the built-in quit behavior for this session

### Requirement: Exposed API — on_event
The Lua API SHALL expose `twisterDBA.on_event(event_name, callback)` for reactive hooks. Supported events: `"QueryExecuted"` (fired after each query completes), `"ConnectionOpened"`, `"ConnectionClosed"`.

#### Scenario: QueryExecuted hook
- **WHEN** a query completes and a Lua hook is registered for `"QueryExecuted"`
- **THEN** the callback is invoked with a table `{ sql = "...", rows_affected = 42, duration_ms = 12 }`

#### Scenario: ConnectionOpened hook
- **WHEN** a database connection is established and a hook is registered for `"ConnectionOpened"`
- **THEN** the callback is invoked with `{ dsn = "postgresql://user@host/db" }` (masked)

#### Scenario: Hook error does not break application
- **WHEN** a `QueryExecuted` hook callback raises a Lua error
- **THEN** the error is logged; the application continues normally; hook is NOT removed (retries next time)
