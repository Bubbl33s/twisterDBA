## ADDED Requirements

### Requirement: S3/MinIO Sync Backend
The application SHALL support S3-compatible object storage (AWS S3, MinIO, DigitalOcean Spaces) as a sync backend, configured via `config.toml` under `[sync.s3]`.

#### Scenario: S3 backend configured and synced
- **WHEN** `config.toml` contains `[sync]` with `backend = "s3"`, `bucket = "my-twisterdba-sync"`, `region = "us-east-1"`, and AWS credentials are available via environment or `~/.aws/credentials`
- **THEN** `:sync` uploads/downloads encrypted blobs to/from the specified S3 bucket

#### Scenario: S3 connection error
- **WHEN** S3 bucket is unreachable (network error, wrong region, credentials expired)
- **THEN** Phase 1 (Search) fails; output pane shows "Sync error: S3 connection failed: <error details>"

#### Scenario: MinIO self-hosted endpoint
- **WHEN** `config.toml` specifies `endpoint = "https://minio.example.com"` and `force_path_style = true`
- **THEN** the sync backend connects to the custom MinIO endpoint (not AWS)

### Requirement: Git Sync Backend
The application SHALL support a private Git repository as a sync backend, configured via `config.toml` under `[sync.git]`. Encrypted blobs SHALL be auto-committed and pushed on sync.

#### Scenario: Git backend configured and synced
- **WHEN** `config.toml` contains `[sync]` with `backend = "git"`, `remote = "git@github.com:user/twisterdba-sync.git"`, and SSH keys are configured
- **THEN** `:sync` clones/pulls the repo, commits changed encrypted blobs, and pushes

#### Scenario: Git merge conflict
- **WHEN** the Git remote has diverged (non-fast-forward)
- **THEN** the sync engine performs a rebase (local changes on top of remote); if rebase conflicts, sync fails with "Sync error: Git rebase conflict — resolve manually"

### Requirement: Sync Configuration
The sync backend and parameters SHALL be configured in `[sync]` section of `config.toml`. Sync SHALL be triggerable manually via `:sync` command and automatically on connection/disconnection events.

#### Scenario: Auto-sync on connect
- **WHEN** `[sync]` has `auto_sync = true` and user connects to a database
- **THEN** sync runs automatically after the connection is established; output pane shows sync progress

#### Scenario: Sync status in output pane
- **WHEN** a sync operation is in progress
- **THEN** the output pane shows "Syncing... Phase 1/3: Searching remote" with a spinner; on completion, shows "Sync complete: 2 uploaded, 1 downloaded, 0 conflicts"

#### Scenario: Sync not configured
- **WHEN** no `[sync]` section exists in `config.toml` and user runs `:sync`
- **THEN** output pane shows "Sync not configured. Add [sync] section to ~/.config/twisterDBA/config.toml"
