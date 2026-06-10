## ADDED Requirements

### Requirement: Lua to Rust Callback Safety
All Lua API functions that call into Rust (e.g., `execute_active_statement`) SHALL execute via `mlua::Function::call()` on the Lua thread. Rust functions SHALL be `Send + 'static` and use message-passing to communicate with the main thread (never block the Lua runtime).

#### Scenario: Lua callback sends DB command
- **WHEN** a Lua function calls `twisterDBA.execute_active_statement()`
- **THEN** the Rust API handler sends a `DbCommand::ExecuteQuery` via the existing `mpsc` channel; the Lua function returns immediately (non-blocking)

#### Scenario: Long-running Lua code does not freeze UI
- **WHEN** a Lua `on_event` callback contains an infinite loop
- **THEN** the Lua runtime is sandboxed (mlua has a built-in instruction limit); after exceeding the limit, the callback is terminated with an error logged

### Requirement: API — register_extractor
The Lua API SHALL expose `twisterDBA.register_extractor(name, callback)` where callback receives the result grid data as a Lua table and returns a formatted string for custom export formats.

#### Scenario: Register JSON extractor
- **WHEN** `register_extractor('json', function(columns, rows) ... end)` is called in init.lua
- **THEN** user can run `:export json /tmp/data.json` and the callback serializes the current result grid to JSON

#### Scenario: Extract called on empty results
- **WHEN** `:export json /tmp/empty.json` is run with no results in the grid
- **THEN** the extractor callback is invoked with empty rows; the output file contains valid empty JSON (`[]`)

### Requirement: API Sandboxing
The Lua runtime SHALL restrict access to `os.execute`, `io.popen`, and other dangerous standard library functions to prevent malicious plugins from executing arbitrary shell commands.

#### Scenario: os.execute blocked
- **WHEN** a Lua plugin calls `os.execute('rm -rf /')`
- **THEN** the call returns `nil, "os.execute is not available"` — the function is removed from the Lua global table

#### Scenario: Safe functions preserved
- **WHEN** a Lua plugin calls `string.upper('hello')` or `table.concat({...})`
- **THEN** these standard library functions work normally (no sandbox restriction on safe operations)
