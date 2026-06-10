## Why

The product spec mandates OS keychain credential storage ("Passwords stored via OS keychain using `keyring` crate") and session state persistence ("Open consoles and cursors saved to `~/.local/share/twisterDBA/`"). Currently, connection DSNs are built in-memory from the ConnectDialog form — passwords are ephemeral, and closing the app loses all open buffers and connection state. This phase adds secure credential storage via the OS-native keychain and session persistence so users can restore their workspace on restart.

## What Changes

- Integrate `keyring` crate to store/retrieve passwords per connection profile
- Passwords are never written to `config.toml` in plaintext — only stored in OS keychain
- Session persistence: on quit, save open buffers, cursor positions, scroll offsets, recent connection to `~/.local/share/twisterDBA/session.toml`
- On startup, restore the previous session if the file exists
- Connection profiles loaded from config.toml can auto-fill passwords from keychain

## Capabilities

### New Capabilities
- `credential-storage`: OS-native keychain integration for connection passwords via `keyring` crate
- `session-persistence`: Save and restore open buffers, cursor positions, connection state on app restart

### Modified Capabilities
- `config-management`: Connection profiles reference keychain entries, not plaintext passwords
- `connection-manager`: ConnectDialog auto-fills passwords from keychain for saved profiles

## Impact

- `Cargo.toml`: Add `keyring = "3"` dependency
- `src/state.rs`: `ConnectField` gains keychain integration; `AppState` gains `save_session()` / `load_session()` methods
- `src/config/`: ConnectionProfile gains `keychain_service` field; passwords loaded via `keyring::Entry`
- `src/main.rs`: Call `load_session()` on startup, `save_session()` on quit
- `~/.local/share/twisterDBA/`: Created at startup for session storage
