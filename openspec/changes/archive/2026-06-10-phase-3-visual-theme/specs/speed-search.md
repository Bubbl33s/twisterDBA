## ADDED Requirements

### Requirement: Real-Time Tree Filtering
When the schema explorer is focused, typing alphanumeric characters SHALL filter the tree view in real-time, showing only nodes whose names contain the typed substring (case-insensitive).

#### Scenario: Type to filter tables
- **WHEN** the explorer shows 5 tables: `users`, `orders`, `products`, `user_logs`, `order_items`
- **THEN** typing `user` immediately filters to show only `users` and `user_logs`

#### Scenario: Filter matches on child nodes
- **WHEN** the explorer tree is fully expanded and filter `id` is typed
- **THEN** columns named `id`, `user_id`, `order_id`, and tables named `idealists` are shown; parent nodes (schemas, tables) auto-expand to reveal matching children

#### Scenario: Clear filter restores full tree
- **WHEN** a search filter is active and the user presses `Escape`
- **THEN** the filter is cleared and the full tree is restored at the previously expanded state

#### Scenario: Filter with no matches shows placeholder
- **WHEN** the user types `xyz_not_found`
- **THEN** the explorer shows "(no matches)" text centered in the panel

#### Scenario: Navigation in filtered view
- **WHEN** a filter is active showing 3 nodes, and user presses `j`/`k`
- **THEN** selection moves only among the 3 visible filtered nodes, not hidden ones

### Requirement: Speed Search Key Routing
Typing in Normal mode while the schema explorer is focused SHALL enter Speed Search mode (accumulate query string) rather than triggering Normal mode shortcuts.

#### Scenario: Typing 't' starts filtering, not a vim command
- **WHEN** explorer is focused in Normal mode and user presses `t`
- **THEN** the search query becomes `"t"` and the tree filters, rather than executing any `t`-prefixed vim motion

#### Scenario: Backspace removes last character
- **WHEN** the search query is `"user"` and user presses `Backspace`
- **THEN** the search query becomes `"use"` and the filter updates

#### Scenario: After 1 second of inactivity, next key starts fresh search
- **WHEN** the search query is `"user"`, the user stops typing for 1 second, then presses `o`
- **THEN** the search query resets to `"o"` and a new filter begins (not append)
