## 1. Dependencies

- [ ] 1.1 Add crypto crates to `Cargo.toml`: `argon2 = "0.5"`, `aes-gcm = "0.10"`, `rand = "0.8"`, `sha2 = "0.10"`
- [ ] 1.2 Add sync backend crates: `aws-sdk-s3 = "1"`, `aws-config = "1"`, `git2 = { version = "0.19", features = ["vendored"] }`
- [ ] 1.3 Add `chrono = "0.4"` for timestamp handling in sync metadata
- [ ] 1.4 Add `async-trait = "0.1"` for `SyncBackend` trait
- [ ] 1.5 Run `cargo build` and verify all deps compile

## 2. Encryption Module (`src/sync/encrypt.rs`)

- [ ] 2.1 Create `src/sync/mod.rs` and `src/sync/encrypt.rs`
- [ ] 2.2 Implement `derive_key(passphrase: &str, salt: &[u8]) -> [u8; 32]` using Argon2id with recommended params
- [ ] 2.3 Implement `encrypt_blob(key: &[u8; 32], plaintext: &[u8]) -> Vec<u8>` producing `[version][nonce][ciphertext]` with AES-256-GCM
- [ ] 2.4 Implement `decrypt_blob(key: &[u8; 32], encrypted: &[u8]) -> Result<Vec<u8>>` verifying GCM tag
- [ ] 2.5 Generate random salt (32 bytes) stored alongside the encrypted data (in manifest, not per file)
- [ ] 2.6 Add unit tests: encrypt-then-decrypt roundtrip, wrong key fails, corrupted data fails
- [ ] 2.7 Run `cargo test` and verify encryption tests pass

## 3. Sync Protocol (`src/sync/protocol.rs`)

- [ ] 3.1 Create `src/sync/protocol.rs` with `SyncProtocol` struct
- [ ] 3.2 Define `BlobMeta { key: String, modified: DateTime<Utc>, sha256: String }` struct with serde derives
- [ ] 3.3 Implement Phase 1 `search(backend) -> Vec<BlobMeta>` listing remote blobs
- [ ] 3.4 Implement Phase 2 `reconcile(local: HashMap, remote: Vec) -> SyncPlan { upload: Vec, download: Vec, conflicts: Vec }`
- [ ] 3.5 Implement Phase 3 `apply(backend, plan, local_blobs)` uploading, downloading, resolving conflicts
- [ ] 3.6 Implement deterministic conflict resolution: timestamp comparison, SHA-256 tiebreaker, `.conflict` file save
- [ ] 3.7 Run `cargo build` and `cargo test`

## 4. Sync Backend Trait (`src/sync/mod.rs`)

- [ ] 4.1 Define `#[async_trait] pub trait SyncBackend` with methods: `list_blobs`, `download_blob`, `upload_blob`, `delete_blob`
- [ ] 4.2 Define `SyncBackend::name() -> &str` for display
- [ ] 4.3 Add `pub mod s3;` and `pub mod git;` to `src/sync/mod.rs`
- [ ] 4.4 Run `cargo build` and verify trait compiles

## 5. S3 Backend (`src/sync/s3.rs`)

- [ ] 5.1 Create `src/sync/s3.rs` with `S3Backend` struct implementing `SyncBackend`
- [ ] 5.2 Implement `S3Backend::new(bucket, region, endpoint_override) -> Self` creating AWS S3 client
- [ ] 5.3 Implement `list_blobs` using `ListObjectsV2` API
- [ ] 5.4 Implement `download_blob` using `GetObject` API
- [ ] 5.5 Implement `upload_blob` using `PutObject` API
- [ ] 5.6 Implement `delete_blob` using `DeleteObject` API
- [ ] 5.7 Handle S3 errors: not found, access denied, network timeout → convert to `SyncError` enum
- [ ] 5.8 Run `cargo build` and verify S3 backend compiles

## 6. Git Backend (`src/sync/git.rs`)

- [ ] 6.1 Create `src/sync/git.rs` with `GitBackend` struct implementing `SyncBackend`
- [ ] 6.2 Implement `GitBackend::new(remote_url, local_path) -> Result<Self>` cloning or opening the repo
- [ ] 6.3 Implement `list_blobs` by reading the file tree from the latest commit
- [ ] 6.4 Implement `download_blob` by reading a blob from the repo (already cloned locally)
- [ ] 6.5 Implement `upload_blob` by writing file, staging, committing with timestamp message, and pushing
- [ ] 6.6 Handle Git errors: auth failure, merge conflict, network error → convert to `SyncError`
- [ ] 6.7 Implement rebase logic for non-fast-forward pushes
- [ ] 6.8 Run `cargo build` and verify Git backend compiles

## 7. Sync Engine & Config (`src/sync/mod.rs`, `src/config/`)

- [ ] 7.1 Create `SyncEngine` struct owning the backend, encryption key, and local manifest
- [ ] 7.2 Implement `SyncEngine::new(backend: Box<dyn SyncBackend>, key: [u8; 32]) -> Self`
- [ ] 7.3 Implement `SyncEngine::run(&mut self) -> SyncResult` executing the full 3-phase sync and returning stats
- [ ] 7.4 Add `SyncConfig` to `Config` struct: `backend: String`, `auto_sync: bool`, and backend-specific sections
- [ ] 7.5 Implement `SyncConfig::create_backend(&self) -> Result<Box<dyn SyncBackend>>` factory method
- [ ] 7.6 Implement passphrase prompt in `AppState` (using ConnectDialog-style password input)
- [ ] 7.7 Run `cargo build` and `cargo clippy`

## 8. UI Integration (`src/state.rs`, `src/ui.rs`, `src/events.rs`)

- [ ] 8.1 Add `sync_state: Option<SyncState>` to `AppState` tracking sync status (idle, running, error, complete)
- [ ] 8.2 Add `SyncCommand` messages: `Start`, `Cancel` sent to sync engine via channel
- [ ] 8.3 Add `SyncEvent` messages: `Progress { phase, message }`, `Complete { stats }`, `Error { message }` sent back to UI
- [ ] 8.4 Implement `:sync` command handler: check config, prompt passphrase if needed, spawn sync task
- [ ] 8.5 Handle `SyncEvent` in `AppState::apply_sync_event` updating `sync_state` and output pane
- [ ] 8.6 Render sync progress in output pane during sync operations
- [ ] 8.7 Wire auto-sync on `DbEvent::Connected` and `DbEvent::Disconnected` when `auto_sync = true`
- [ ] 8.8 Run `cargo build` and manually test sync command

## 9. Verification

- [ ] 9.1 Run `cargo build` and verify no errors
- [ ] 9.2 Run `cargo clippy -- -D warnings` and fix all warnings
- [ ] 9.3 Run `cargo fmt --check` and verify formatting
- [ ] 9.4 Run `cargo test` and verify all tests pass (encryption roundtrip tests)
- [ ] 9.5 Manually test encryption: derive key, encrypt file, decrypt, verify byte-for-byte match
- [ ] 9.6 Manually test S3 sync with MinIO docker container
- [ ] 9.7 Manually test Git sync with local bare repo
- [ ] 9.8 Manually test conflict resolution: modify same file on two "machines", sync both, verify deterministic result
- [ ] 9.9 Manually test wrong passphrase detection
- [ ] 9.10 Manually test sync progress shown in output pane
