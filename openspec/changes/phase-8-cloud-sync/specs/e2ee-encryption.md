## ADDED Requirements

### Requirement: Master Passphrase Key Derivation
The application SHALL derive an AES-256-GCM key from a user-provided master passphrase using Argon2id (memory-hard KDF). The passphrase SHALL be prompted once per session and held in memory only (never persisted to disk). The derived key SHALL be used to encrypt/decrypt all sync blobs.

#### Scenario: First sync prompts for passphrase
- **WHEN** user runs `:sync` for the first time with no cached passphrase
- **THEN** a passphrase prompt appears (password-style, masked input); the passphrase is hashed via Argon2id to derive the encryption key

#### Scenario: Subsequent sync reuses cached key
- **WHEN** user runs `:sync` again within the same session
- **THEN** the cached encryption key is reused (no re-prompt); sync proceeds immediately

#### Scenario: Wrong passphrase on re-authenticate
- **WHEN** sync data exists from a previous session and user enters a wrong passphrase
- **THEN** decryption fails with authentication error (AES-GCM tag mismatch); output pane shows "Sync error: incorrect passphrase or corrupted data"

#### Scenario: Passphrase held in memory securely
- **WHEN** the encryption key is derived and stored in `AppState`
- **THEN** the key is wrapped in `secrecy::Secret<[u8; 32]>` to prevent accidental logging/display; zeroed on app exit

### Requirement: AES-256-GCM File Encryption
Sync blobs SHALL be encrypted using AES-256-GCM with a random 12-byte nonce per file. The encrypted file format SHALL be: `[4-byte version][12-byte nonce][ciphertext+tag]`. Files SHALL be unreadable without the derived key.

#### Scenario: Encrypt connection profile
- **WHEN** `connections.db` (or `.toml`) is encrypted for sync
- **THEN** the output file `connections.enc` is AES-256-GCM ciphertext; without the key, it reveals no metadata (all fields encrypted, not just passwords)

#### Scenario: Decrypt on another machine
- **WHEN** `connections.enc` is synced to another machine and decrypted with the same passphrase
- **THEN** the original connection profiles are recovered exactly (byte-for-byte match)

#### Scenario: Corrupted encrypted file detected
- **WHEN** `connections.enc` is truncated or modified in transit
- **THEN** AES-GCM authentication tag fails; decryption returns `Err`; output pane shows "Sync error: data integrity check failed"

### Requirement: 3-Phase Sync Protocol
The sync engine SHALL implement a 3-phase protocol: Phase 1 (Search) lists remote blobs and their metadata; Phase 2 (Reconcile) compares remote vs local timestamps/hashes to determine changes; Phase 3 (Apply) uploads local changes and downloads remote changes.

#### Scenario: No changes — skip sync
- **WHEN** local and remote blob hashes match exactly
- **THEN** Phase 2 detects no diffs; Phase 3 is skipped; output pane shows "Sync: up to date"

#### Scenario: Local changes uploaded
- **WHEN** a new connection profile was added locally since last sync
- **THEN** Phase 2 detects the new local blob; Phase 3 uploads the encrypted blob to remote; output pane shows "Sync: uploaded connections.enc"

#### Scenario: Remote changes downloaded
- **WHEN** another machine uploaded a new connection since last sync
- **THEN** Phase 2 detects the new remote blob; Phase 3 downloads and decrypts it; local profile list updates

#### Scenario: Conflict — local wins
- **WHEN** the same blob was modified on both machines since last sync
- **THEN** deterministic conflict resolution: the blob with the later modification timestamp wins; the losing blob is saved as `.conflict` for manual review; output pane shows "Sync: 1 conflict resolved (local wins)"

### Requirement: Deterministic Conflict Resolution
Conflicts SHALL be resolved by comparing modification timestamps. If timestamps are equal, the blob with the lexicographically larger hash SHA-256 wins. Losing blobs SHALL be saved locally with a `.conflict` suffix.

#### Scenario: Timestamp tiebreaker
- **WHEN** two blobs have the exact same modification timestamp (e.g., within same second)
- **THEN** the blob whose SHA-256 hash is lexicographically larger is chosen as the winner
