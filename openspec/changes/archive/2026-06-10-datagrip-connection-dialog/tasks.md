## 1. ConnectForm Two-Step Refactor (`src/state/connection.rs`)

- [x] 1.1 Add `DialogStep` enum: `SelectType`, `EnterDetails`
- [x] 1.2 Add `step: DialogStep` field to `ConnectForm`
- [x] 1.3 Add `connection_name: String` field to `ConnectForm`
- [x] 1.4 Add `ssl_mode: usize` field to `ConnectForm` (index into SSL mode options)
- [x] 1.5 Add `type_cursor: usize` field for type grid navigation
- [x] 1.6 Update `ConnectForm::default()` to start in `SelectType` step
- [x] 1.7 Add `ConnectForm::from_profile_with_name(profile, name)` for profile pre-fill with custom name
- [x] 1.8 Add SSL mode options constant: `["disable", "allow", "prefer", "require", "verify-ca", "verify-full"]`
- [x] 1.9 Update `build_pg_dsn()` to append `?sslmode=<mode>`
- [x] 1.10 Update `masked_pg_dsn()` to append `?sslmode=<mode>`
- [x] 1.11 Add `auto_generate_name(&self) -> String` method
- [x] 1.12 Run `cargo build` and fix compile errors

## 2. Connect Dialog Key Handler Rewrite (`src/state/handlers/connect.rs`)

- [x] 2.1 Rewrite `handle_connect_dialog_key` with step-based dispatch
- [x] 2.2 Implement Step 1 key handling: Left/Right for type grid, Down to profiles, Up from profiles to type grid
- [x] 2.3 Implement Step 1 Enter: if on type → go to Step 2 with auto-generated name; if on profile → go to Step 2 with profile data
- [x] 2.4 Implement Step 1 Esc: close dialog
- [x] 2.5 Implement Step 2 key handling: Tab/Down/Up for field navigation (including connection name and SSL mode)
- [x] 2.6 Implement Step 2 Enter: validate, check name conflict, initiate connection
- [x] 2.7 Implement Step 2 Esc: return to Step 1
- [x] 2.8 Implement SSL mode Left/Right cycling when SSL mode field is active
- [x] 2.9 Implement connection name field editing (Backspace, Delete, Char, Home, End)
- [x] 2.10 Run `cargo build` and fix compile errors

## 3. Connect Dialog Rendering Rewrite (`src/ui/dialog.rs`)

- [x] 3.1 Rewrite `render_connect_dialog` with step-based layout
- [x] 3.2 Implement Step 1 rendering: type grid with engine icons, profile list below
- [x] 3.3 Implement Step 1 type grid: 3 columns with icons, highlight selected type
- [x] 3.4 Implement Step 1 profile list: engine icon + name + host, highlight selected profile
- [x] 3.5 Implement Step 1 help footer: "Tab/↓↑:navigate  Enter:select  Esc:cancel"
- [x] 3.6 Implement Step 2 rendering: connection name field + engine-specific fields
- [x] 3.7 Implement Step 2 connection name field with cursor rendering
- [x] 3.8 Implement Step 2 SSL mode dropdown for PostgreSQL (show current value with ←→ hint)
- [x] 3.9 Implement Step 2 help footer: "Tab/↓↑:next  Esc:back  Enter:connect  ←→:ssl-mode"
- [x] 3.10 Adjust dialog size: Step 1 wider (~70% width), Step 2 narrower (~60% width)
- [x] 3.11 Run `cargo build` and fix compile errors

## 4. Theme Engine Icons for Dialog (`src/theme.rs`)

- [x] 4.1 Add large engine icons for dialog type grid (or reuse existing icons at larger display)
- [x] 4.2 Add `dialog_type_selected_bg: Color` for highlighted type in grid
- [x] 4.3 Add `dialog_profile_bg: Color` for profile list items
- [x] 4.4 Run `cargo build` and fix compile errors

## 5. Connection Name Conflict Handling (`src/state/handlers/connect.rs`, `src/config/mod.rs`)

- [x] 5.1 Add name conflict check before connect: search `config.connections` for existing name
- [x] 5.2 If conflict: show inline warning in dialog, require confirmation or name change
- [x] 5.3 Add `ConnectForm::name_conflict: bool` field for rendering warning
- [x] 5.4 Add confirm key (e.g., Enter again or 'y') to overwrite existing profile
- [x] 5.5 Run `cargo build` and fix compile errors

## 6. Command Handler Update (`src/state/handlers/command.rs`)

- [x] 6.1 Update `:connect` command to create form in `SelectType` step
- [x] 6.2 Update `:connect <profile_name>` to create form pre-filled from profile, starting in `EnterDetails` step
- [x] 6.3 Run `cargo build` and fix compile errors

## 7. Verification

- [x] 7.1 Run `cargo build` and verify no errors
- [x] 7.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 7.3 Run `cargo fmt --check` and verify formatting
- [x] 7.4 Run `cargo test` and verify all tests pass
- [ ] 7.5 Manually test: open dialog, verify Step 1 shows type grid and profiles
- [ ] 7.6 Manually test: navigate type grid with arrow keys
- [ ] 7.7 Manually test: select type, verify Step 2 shows correct fields
- [ ] 7.8 Manually test: select profile, verify Step 2 is pre-filled
- [ ] 7.9 Manually test: Esc in Step 2 returns to Step 1
- [ ] 7.10 Manually test: connection name editing and auto-generation
- [ ] 7.11 Manually test: SSL mode cycling for PostgreSQL
- [ ] 7.12 Manually test: name conflict warning and overwrite confirmation
