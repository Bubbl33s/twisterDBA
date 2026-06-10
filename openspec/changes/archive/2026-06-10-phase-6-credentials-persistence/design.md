## Context

The product spec defines credential storage and state persistence as architectural requirements: "Passwords stored via OS keychain using `keyring` crate" and "Open consoles and cursors saved to `~/.local/share/twisterDBA/`." These are orthogonal features — credential storage secures passwords, persistence saves workspace state — but both involve file I/O and startup/shutdown hooks.

The `keyring` crate provides a cross-platform abstraction over Keychain Services (macOS), Credential Manager (Windows), and Secret Service/Keyutils (Linux). Session persistence uses serde + TOML serialization to a known XDG data directory.

## Goals / Non-Goals

**Goals:**
- Passwords stored exclusively in OS keychain, never in plaintext files
- Session state (buffers, cursors, connection ref) serialized to TOML on quit
- Session restored on restart with automatic reconnection
- Auto-save every 60 seconds for crash recovery
- Cross-platform keychain support (macOS, Linux, Windows)

**Non-Goals:**
- Encrypted session files (Phase 8 covers E2EE cloud sync)
- Multi-profile session management
- Undo/redo persistence across sessions

## Decisions

### ADR-001: Keychain Entry Per Profile
**Choice:** Create a `keyring::Entry::new("twisterDBA/<profile_name>", "<username>")` for each connection profile.

**Rationale:** The `keyring` crate identifies entries by (service, account). Using the profile name as part of the service ensures uniqueness. The username as account allows the same username across different profiles.

**Alternative considered:** Single keychain entry with a JSON blob of all passwords — violates the principle of least privilege; you can't query individual entries.

### ADR-002: Session Serialization with Serde + TOML
**Choice:** `SessionData` struct with `#[derive(Serialize, Deserialize)]` serialized to `~/.local/share/twisterDBA/session.toml`.

**Rationale:** TOML is human-readable, which helps debugging. Serde derives are zero-cost. The session file is small (<10KB for typical use). No need for SQLite or binary formats.

**Alternative considered:** SQLite database for session — overkill for a single session record; adds dependency and complexity.

### ADR-003: Auto-Save via Tokio Interval
**Choice:** Spawn a `tokio::spawn(async { loop { interval.tick().await; save_session(); } })` task in `App::run()`.

**Rationale:** Tokio intervals are non-blocking and precise. The save function reads `AppState` immutably (it's owned by the main thread), so it must be called from the main event loop via a `Tick` event or a dedicated message.

**Alternative considered:** Saving in the render loop — violates the pure-render principle; rendering must have no side effects.

### ADR-004: Password Prompt Fallback
**Choice:** When keychain is unavailable (detected by `keyring::Entry::new(...).get_password()` returning an error), prompt the user for password in the ConnectDialog. Add a `[keychain_unavailable]` boolean to the connect flow.

**Rationale:** Headless Linux servers often lack `gnome-keyring` or `kwallet`. A fallback ensures the app works everywhere. The password is held in memory only and never persisted.

### ADR-005: Session Data Directory
**Choice:** `dirs::data_local_dir()/twisterDBA/` for session files, `dirs::config_dir()/twisterDBA/` for config files.

**Rationale:** Follows XDG spec: data is for application state (sessions), config is for user configuration (connection profiles, keybindings). `data_local_dir()` maps to `~/.local/share` on Linux, `~/Library/Application Support` on macOS.

## Risks / Trade-offs

- **[Risk] Keychain access may trigger OS prompts (e.g., macOS "twisterDBA wants to use your keychain")** → Mitigation: documented in README; only accessed on explicit connect
- **[Risk] Auto-save may briefly block the main thread on slow I/O** → Mitigation: use `tokio::task::spawn_blocking` for file writes; session data is small
- **[Risk] Session restore may attempt invalid connection after profile changes** → Mitigation: wrap in error handling; if reconnection fails, show error in output pane and allow user to reconnect manually

## Migration Plan

1. Add `keyring` dependency
2. Implement keychain store/retrieve in config module
3. Implement `SessionData` struct with serde
4. Add save/load methods to `AppState`
5. Hook save on quit, load on startup, auto-save timer
6. No data migration needed (new features)

## Open Questions

- Should session restore prompt the user before reconnecting? (Answer: yes, show "Restoring session..." in output pane; reconnect automatically if profile found)
- Should the `keyring` fallback cache the password in memory for the session? (Answer: yes, store in `Secrecy<SecretString>` after first prompt)
