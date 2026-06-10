## ADDED Requirements

### Requirement: OS Keychain Password Storage
The application SHALL store database passwords using the OS-native credential manager via the `keyring` crate. Passwords SHALL NOT be written to `config.toml` in plaintext.

#### Scenario: Save password to keychain
- **WHEN** a user successfully connects with a password, and the profile is saved
- **THEN** the password is stored in the OS keychain under service name `twisterDBA/<profile_name>` and account `<user>`

#### Scenario: Retrieve password from keychain on profile load
- **WHEN** a saved connection profile is loaded into the ConnectDialog
- **THEN** the Password field is populated from the OS keychain (if available); the user is not prompted to re-enter it

#### Scenario: Keychain unavailable on headless Linux
- **WHEN** the `keyring` crate cannot access a keychain backend (e.g., `dbus` not running on Linux server)
- **THEN** the application logs a warning and falls back to prompting the user for the password at connect time (no crash)

#### Scenario: Password stored per profile
- **WHEN** a user has two profiles "dev" and "prod" with the same username
- **THEN** passwords for each profile are stored independently under `twisterDBA/dev` and `twisterDBA/prod` service names

### Requirement: Connection Profile Password Management
The `ConnectionProfile` struct SHALL reference a keychain entry rather than storing a plaintext password. The ConnectDialog SHALL retrieve the password from the keychain only when needed (on connect, not on load).

#### Scenario: Profile saved without password in config
- **WHEN** a connection profile is saved to `config.toml`
- **THEN** the password field is written as `password = "<keychain>"` — a marker indicating keychain storage

#### Scenario: ConnectDialog shows password placeholder
- **WHEN** a profile with keychain-stored password is loaded into the ConnectDialog
- **THEN** the Password field shows `••••••` (masked placeholder) with a note `[from keychain]`

#### Scenario: Delete password from keychain
- **WHEN** user runs `:keychain delete <profile>` command
- **THEN** the stored password is removed from the OS keychain and the profile's password marker is cleared
