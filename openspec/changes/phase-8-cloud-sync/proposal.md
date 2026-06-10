## Why

The product spec defines an End-to-End Encrypted (E2EE) cloud sync feature so engineers can sync credentials and console state across machines: "Credentials and consoles sync across machines without ever being readable server-side." The sync protocol is a 3-phase Search→Reconcile→Apply loop with deterministic conflict resolution over TLS 1.3. Backends include AWS S3/MinIO and private Git repositories. The encryption uses Argon2id (key derivation from master passphrase) → AES-256-GCM (file encryption), compatible with SQLCipher for local databases.

## What Changes

- Add E2EE encryption module using `argon2` + `aes-gcm` crates
- Implement encrypted file format: `connections.enc` (encrypted connection profiles) and `consoles/` directory (encrypted per-file)
- Implement 3-phase sync protocol: Search (list remote files), Reconcile (compare timestamps/hashes), Apply (upload/download encrypted blobs)
- S3/MinIO sync backend via `aws-sdk-s3` (async, Tokio-compatible)
- Git sync backend via `git2` crate (auto-commit encrypted blobs)
- `:sync` command to trigger manual sync; configurable auto-sync on connect/disconnect
- Master passphrase prompt on first sync; stored in memory only (never persisted)

## Capabilities

### New Capabilities
- `e2ee-encryption`: Argon2id key derivation + AES-256-GCM file encryption, master passphrase protected
- `cloud-sync-s3`: S3/MinIO object storage sync backend
- `cloud-sync-git`: Private Git repository sync backend with auto-commit
- `sync-protocol`: 3-phase search→reconcile→apply with deterministic conflict resolution

### Modified Capabilities
- `credential-storage`: Encrypted connections file replaces/extends keychain for multi-machine portability

## Impact

- `Cargo.toml`: Add `argon2`, `aes-gcm`, `rand`, `sha2`, `aws-sdk-s3`, `aws-config`, `git2`, `chrono`
- `src/sync/`: New module tree: `mod.rs` (SyncEngine), `encrypt.rs` (AES-256-GCM), `s3.rs` (S3 backend), `git.rs` (Git backend), `protocol.rs` (3-phase sync)
- `src/state.rs`: Passphrase state, sync status
- `src/ui.rs`: Sync progress indicator in output pane
- `~/.local/share/twisterDBA/sync/`: Local encrypted blob store
