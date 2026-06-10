## ADDED Requirements

### Requirement: AST-Based SQL Syntax Highlighting
The SQL editor SHALL highlight keywords, identifiers, strings, numbers, comments, and operators using `tree-sitter-sql` AST token types. Highlighting SHALL update on every keystroke in Insert mode and every buffer change.

#### Scenario: Keyword highlighted in SELECT statement
- **WHEN** the buffer contains `SELECT * FROM users WHERE id = 1;`
- **THEN** `SELECT`, `FROM`, `WHERE` render in the keyword color (`#CC7832`), `*` renders as operator, `users` as identifier, `1` as number, and `;` as punctuation

#### Scenario: String literal with escaped quotes rendered correctly
- **WHEN** the buffer contains `'it''s a string'`
- **THEN** the entire string `'it''s a string'` renders in string color (`#6A8759`), not splitting at the doubled quote

#### Scenario: Multi-line CTE highlighted correctly
- **WHEN** the buffer contains:
  ```sql
  WITH regional_sales AS (
    SELECT region, SUM(amount) AS total_sales
    FROM orders
    GROUP BY region
  )
  SELECT * FROM regional_sales;
  ```
- **THEN** `WITH` and `AS` render as keywords, `regional_sales` as CTE name, `SUM` as function, and the highlighting spans all lines consistently

#### Scenario: Comment line dimmed
- **WHEN** the buffer contains `-- this is a comment\nSELECT 1;`
- **THEN** the line starting with `--` renders in grey comment color (`#808080`), and `SELECT 1;` on the next line highlights normally

#### Scenario: Dollar-quoted string (PostgreSQL) handled
- **WHEN** the buffer contains `$$function body with 'quotes'$$`
- **THEN** the entire dollar-quoted block renders as a single string token, no syntax confusion from internal quotes

### Requirement: Context-Aware Statement Execution
Pressing execute (`<leader>r` / `Ctrl+E`) SHALL extract only the SQL statement that contains the cursor, using Tree-sitter's AST to find the enclosing `statement` node. Semicolons inside string literals or dollar-quoted blocks SHALL NOT cause false statement boundaries.

#### Scenario: Execute statement under cursor in multi-statement file
- **WHEN** the cursor is on line 5 of a buffer containing three `SELECT` statements separated by `;`, and user presses `Ctrl+E`
- **THEN** only the statement containing the cursor is sent to the DB; the other two statements are ignored

#### Scenario: Semicolon inside string does not split statement
- **WHEN** the buffer contains `SELECT 'hello;world' AS greeting; SELECT 2;` and cursor is on the first line
- **THEN** pressing `Ctrl+E` sends `SELECT 'hello;world' AS greeting;` — the semicolon inside the string is not treated as a statement boundary

#### Scenario: CTE with multiple chained statements
- **WHEN** the buffer contains:
  ```sql
  WITH a AS (SELECT 1), b AS (SELECT * FROM a)
  SELECT * FROM b;
  ```
  and the cursor is inside the CTE body
- **THEN** pressing `Ctrl+E` sends the entire statement from `WITH` to the final `;` — the CTE is treated as part of the enclosing statement

#### Scenario: Empty statement skipped
- **WHEN** the cursor is on a blank line between two statements
- **THEN** pressing `Ctrl+E` shows a status bar message "No statement under cursor" and does not send a query

#### Scenario: Cursor inside a comment
- **WHEN** the cursor is on a comment line `-- TODO: optimize`
- **THEN** pressing `Ctrl+E` shows a status bar message "No statement under cursor" and does not send a query

### Requirement: UTF-8 Safe Byte-Range Slicing
All SQL extraction from the buffer SHALL use `node.start_byte()` and `node.end_byte()` (byte offsets from Tree-sitter) to slice the source text. No `char` boundary indexing.

#### Scenario: Emoji in SQL string does not cause panic
- **WHEN** the buffer contains `SELECT 'hello 🚀 world' AS emoji_test;`
- **THEN** pressing `Ctrl+E` extracts and executes the statement without a `char boundary` panic

#### Scenario: Accented identifiers handled correctly
- **WHEN** the buffer contains `SELECT * FROM "café";`
- **THEN** the identifier `café` is highlighted correctly and statement extraction uses byte offsets without panic

#### Scenario: CJK characters in comments
- **WHEN** the buffer contains `-- こんにちは世界\nSELECT 1;`
- **THEN** the comment renders in grey and the `SELECT 1` highlights normally; no panics on extraction

### Requirement: Split Window Buffer Management
The application SHALL support multiple editor buffers displayed side-by-side with a vertical `│` separator. The user SHALL create, close, and navigate between split windows via keybindings.

#### Scenario: Create horizontal split
- **WHEN** user presses `Ctrl+W s` in Normal mode with one editor buffer open
- **THEN** a new empty buffer opens below the current one, the editor area splits vertically, and focus moves to the new buffer

#### Scenario: Create vertical split
- **WHEN** user presses `Ctrl+W v` in Normal mode with one editor buffer open
- **THEN** a new empty buffer opens to the right, the editor area splits horizontally, and focus moves to the new buffer

#### Scenario: Navigate between splits
- **WHEN** two splits are open and user presses `Ctrl+W h` or `Ctrl+W l`
- **THEN** focus moves to the adjacent split; the focused split shows the cursor, inactive splits show no cursor

#### Scenario: Close focused split
- **WHEN** two splits are open and user presses `Ctrl+W q` on the focused split
- **THEN** the focused buffer closes, the remaining buffer expands to fill the editor area

#### Scenario: Terminal resize redistributes split sizes
- **WHEN** the terminal is resized while two splits are open
- **THEN** both splits scale proportionally (50/50 or as set), and content within each buffer remains scrolled correctly

### Requirement: Tree-Sitter Grammar Compilation
The `tree-sitter-sql` grammar SHALL be compiled at build time via a `build.rs` script, linking against the system C compiler. The grammar SHALL not require pre-installed tree-sitter tooling.

#### Scenario: Clean build compiles grammar
- **WHEN** a developer runs `cargo build` from a clean checkout
- **THEN** `build.rs` compiles `tree-sitter-sql` C sources into the binary; no separate build step required

#### Scenario: Recompilation on grammar update
- **WHEN** `tree-sitter-sql` crate version is bumped
- **THEN** cargo rebuilds the grammar automatically (cc crate tracks changes)
