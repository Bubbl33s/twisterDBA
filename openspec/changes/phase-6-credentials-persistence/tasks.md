## 1. Keychain Integration (`src/config/`, `Cargo.toml`)

- [x] 1.1 Add `keyring = "3"` to `Cargo.toml`
- [x] 1.2 Add `keychain_service: Option<String>` field to `ConnectionProfile` struct in `src/config/mod.rs`
- [x] 1.3 Implement `ConnectionProfile::store_password(&self, password: &str) -> Result<()>` using `keyring::Entry::new("twisterDBA/<name>", &user)`
- [x] 1.4 Implement `ConnectionProfile::get_password(&self) -> Result<String>` retrieving from keychain
- [x] 1.5 Implement `ConnectionProfile::delete_password(&self) -> Result<()>` removing from keychain
- [x] 1.6 Handle keychain errors gracefully: log warning, return `Err` so caller can prompt user
- [x] 1.7 Run `cargo build` and verify keyring compiles

## 2. Config TOML Password Markers (`src/config/mod.rs`)

- [x] 2.1 When saving a profile, if password was used, store it in keychain and write `password = "<keychain>"` marker to TOML
- [x] 2.2 When loading a profile with `password = "<keychain>"`, attempt to retrieve from keychain
- [x] 2.3 If keychain retrieval fails, leave password field empty (user prompted in ConnectDialog)
- [x] 2.4 Ensure plaintext passwords are never written to `config.toml`
- [x] 2.5 Run `cargo build` and `cargo clippy`

## 3. ConnectDialog Keychain Integration (`src/state.rs`, `src/ui.rs`)

- [x] 3.1 When loading a saved profile into ConnectDialog, call `profile.get_password()` and pre-fill if available
- [x] 3.2 Show `•••••• [from keychain]` in the Password field when password is loaded from keychain
- [x] 3.3 After successful connection, if profile is saved, call `profile.store_password(&password)` in a spawned task
- [x] 3.4 Add `:keychain delete <profile>` command to `execute_command` in state.rs
- [x] 3.5 Run `cargo build` and manually test keychain save/retrieve

## 4. Session Data Model (`src/state.rs`)

- [x] 4.1 Define `SessionData` struct: `connection_profile: Option<String>`, `buffers: Vec<BufferSnapshot>`, `focused_buffer: usize`, `focused_panel: String`, `split_layout: String`
- [x] 4.2 Define `BufferSnapshot` struct: `content: String`, `cursor_row: usize`, `cursor_col: usize`, `scroll_offset: usize`
- [x] 4.3 Derive `Serialize, Deserialize` on both structs
- [x] 4.4 Implement `AppState::to_session_data(&self) -> SessionData` collecting current state
- [x] 4.5 Implement `AppState::apply_session_data(&mut self, data: SessionData)` restoring state
- [x] 4.6 Run `cargo build` and verify serde derives compile

## 5. Session Persistence I/O (`src/main.rs`, `src/state.rs`)

- [x] 5.1 Implement `save_session_to_disk(data: &SessionData) -> Result<()>` writing to `~/.local/share/twisterDBA/session.toml`
- [x] 5.2 Implement `load_session_from_disk() -> Result<Option<SessionData>>` reading from session file
- [x] 5.3 Use `dirs::data_local_dir()` to resolve the path; create directory if missing
- [x] 5.4 Handle corrupt file: log warning, return `None` (fresh start)
- [x] 5.5 In `main.rs`, call `load_session_from_disk()` on startup; if `Some`, apply to state and auto-reconnect
- [x] 5.6 In `main.rs`, after the event loop exits, call `save_session_to_disk()` with final state
- [x] 5.7 Run `cargo build` and test session save/restore

## 6. Auto-Save Timer (`src/app.rs`)

- [x] 6.1 Add `auto_save_tx: mpsc::UnboundedSender<AppEvent>` for auto-save tick events
- [x] 6.2 Spawn a Tokio task with `tokio::time::interval(Duration::from_secs(60))` sending `AppEvent::AutoSave` ticks
- [x] 6.3 In `App::run()`, handle `AppEvent::AutoSave` to call `save_session_to_disk()`
- [x] 6.4 Ensure auto-save does not block the UI: use `std::thread::spawn` or spawn_blocking for the file write
- [x] 6.5 Run `cargo build` and `cargo clippy`

## 7. Verification

- [x] 7.1 Run `cargo build` and verify no errors
- [x] 7.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [x] 7.3 Run `cargo fmt --check` and verify formatting
- [x] 7.4 Run `cargo test` and verify all tests pass
- [ ] 7.5 Manually test keychain password storage: connect, quit, restart, verify password pre-filled
- [ ] 7.6 Manually test session persistence: open buffers, quit, restart, verify state restored
- [ ] 7.7 Manually test auto-save: wait >60s, kill app, restart, verify recent changes saved
- [ ] 7.8 Manually test corrupt session file: hand-edit session.toml, restart, verify clean start
- [ ] 7.9 Manually test keychain fallback on headless Linux (unset DBUS_SESSION_BUS_ADDRESS)
