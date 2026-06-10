## Context

The product spec defines cloud sync as an E2EE feature: "Credentials and consoles sync across machines without ever being readable server-side." The architecture is: master passphrase → Argon2id → AES-256-GCM key → encrypted blobs → sync over TLS 1.3 to S3/Git backends. This is a significant feature with cryptographic and networking complexity.

The design must ensure: keys never touch disk, all data at rest in cloud storage is opaque ciphertext, the sync protocol handles conflicts deterministically, and the implementation is async (Tokio-compatible) to avoid blocking the UI.

## Goals / Non-Goals

**Goals:**
- Argon2id KDF from master passphrase → 256-bit key
- AES-256-GCM per-file encryption with random nonces
- 3-phase sync protocol (Search → Reconcile → Apply)
- S3/MinIO backend via `aws-sdk-s3`
- Git backend via `git2`
- Deterministic conflict resolution (timestamp + hash tiebreaker)
- Non-blocking sync (runs in Tokio task, updates UI via events)

**Non-Goals:**
- Public key cryptography or key sharing (single-user model)
- Multi-device simultaneous sync (single session at a time)
- File versioning or snapshot history
- WebDAV, FTP, or other backends

## Decisions

### ADR-001: Argon2id + AES-256-GCM
**Choice:** `argon2` crate for KDF with recommended params (19MB memory, 2 iterations, 1 parallelism). `aes-gcm` crate for authenticated encryption. Random 12-byte nonces via `rand::rngs::OsRng`.

**Rationale:** Argon2id is the OWASP-recommended password KDF (won the Password Hashing Competition). AES-256-GCM provides both confidentiality and integrity (authenticated encryption). Both crates are well-audited and widely used.

**Alternative considered:** `chacha20poly1305` — also good but AES-GCM has hardware acceleration on modern CPUs (AES-NI).

### ADR-002: Per-File Encryption
**Choice:** Each file is encrypted independently with its own random nonce. File format is a binary blob: `[version: u32 LE][nonce: [u8; 12]][ciphertext]`.

**Rationale:** Per-file encryption allows granular sync (only changed files are uploaded). Independent nonces prevent nonce reuse across files. The version byte allows future format upgrades.

**Alternative considered:** Encrypted tar/zip archive — requires downloading the entire archive to decrypt one file; inefficient for incremental sync.

### ADR-003: Trait-Based Backend
**Choice:** `trait SyncBackend` with async methods: `list_blobs() -> Vec<BlobMeta>`, `download_blob(key) -> Vec<u8>`, `upload_blob(key, data)`, `delete_blob(key)`. S3 and Git implement this trait.

**Rationale:** Trait-based backends allow adding new sync targets without changing the protocol. The trait is `async_trait` (Tokio-compatible). Each backend is isolated in its own module.

**Alternative considered:** Enum-based dispatch — less extensible; adding a new backend requires modifying the enum and all match arms.

### ADR-004: Sync Runs in Dedicated Tokio Task
**Choice:** `:sync` sends a `SyncCommand::Start` to a `SyncEngine` running in a `tokio::spawn` task. The engine sends progress events via `mpsc` to the UI.

**Rationale:** Sync operations (network I/O, encryption, decryption) can take seconds. Running them in a Tokio task prevents UI blocking. Progress events update the output pane in real-time.

### ADR-005: Conflict Resolution as Pure Function
**Choice:** `fn resolve(local: BlobMeta, remote: BlobMeta) -> BlobMeta` with deterministic logic: if `local.modified > remote.modified`, pick local; if `remote.modified > local.modified`, pick remote; if equal, pick the one with larger SHA-256 hash.

**Rationale:** Deterministic resolution ensures both machines converge to the same state without additional network round-trips. The hash tiebreaker ensures a decision even for simultaneous writes.

## Risks / Trade-offs

- **[Risk] AES-GCM nonce reuse would catastrophically break encryption** → Mitigation: random 12-byte nonces from `/dev/urandom`; 96 bits provides astronomically low collision probability
- **[Risk] Lost passphrase means lost sync data** → Mitigation: documented prominently; users should use a password manager
- **[Risk] Git backend requires SSH key setup** → Mitigation: document SSH key configuration; support HTTPS with token auth as alternative
- **[Risk] S3 costs for frequent sync** → Mitigation: sync is manual by default; auto-sync is opt-in
- **[Risk] `git2` linking against system libgit2 can fail** → Mitigation: use `git2/vendored` feature to compile libgit2 from source

## Migration Plan

1. Add crypto and networking dependencies
2. Implement encryption module (`encrypt.rs`)
3. Implement sync protocol (`protocol.rs`)
4. Implement S3 backend (`s3.rs`)
5. Implement Git backend (`git.rs`)
6. Implement `SyncEngine` and UI integration
7. Add `[sync]` config parsing
8. No data migration needed (new feature)

## Open Questions

- Should we support file-level selective sync (e.g., sync only connections, not consoles)? (Answer: yes, via `[sync.files]` config section)
- Should sync use compression before encryption? (Answer: no for MVP; encrypted data is already incompressible)
