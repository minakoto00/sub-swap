# Security

## Constraints and Invariants

| Constraint | Implementation | Verified By |
|------------|----------------|-------------|
| Inactive profiles encrypted with AES-256-GCM at rest | `src/crypto/mod.rs` encrypt/decrypt | Unit tests in `crypto::tests` |
| Encryption key never written to filesystem | `src/key.rs` resolves a native OS-keystore key by default, or derives a passphrase key without persisting the key bytes | `key::tests`, `crypto::passphrase::tests`, plus manual OS-keychain verification |
| All files under `~/.sub-swap/` are mode 0600 (Unix) | `#[cfg(unix)]` blocks in `config.rs`, `profile/store.rs`, `profile/switch.rs` | Manual — no automated permission tests |
| Zero network crates in dependency tree | `Cargo.toml` has no network crates | `tests/arch.rs` arch_03 test |
| Active profile stays plaintext in `~/.codex/` | Required by Codex to read credentials | N/A — design constraint |
| Profile names validated against path traversal | `validate_profile_name()` in `error.rs` | Unit tests in `error::tests` |

> **IMPORTANT:** The active profile's credentials (`~/.codex/auth.json`) are always plaintext.
> sub-swap protects *inactive* profiles only. This is a design constraint — Codex must be
> able to read its own config files without decryption.

## Encryption Model

**Algorithm:** AES-256-GCM (authenticated encryption with associated data)

**Nonce:** 96-bit (12 bytes), randomly generated per encryption call via OS CSPRNG (`rand::rng().fill_bytes`)

**Output format:** `[12-byte nonce][ciphertext][16-byte GCM authentication tag]` — concatenated into a single `Vec<u8>`

Each file (`auth.json`, `config.toml`) is encrypted independently. This means each file has its own random nonce and produces a separate ciphertext blob.

**Function signatures** (from `src/crypto/mod.rs`):

```rust
pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>
pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>
```

**Decryption validation:** The `decrypt` function validates a minimum length of 28 bytes (12 nonce + 16 tag). Data shorter than this is rejected before any decryption attempt.

**Source file:** `src/crypto/mod.rs`

## Key Management

**Key size:** 256-bit (32 bytes)

**Default backend:** Native OS key storage. `AppConfig::default()` sets `key_backend = native`, and legacy boolean-only encrypted configs are migrated on load to explicit native-backend metadata. This includes older macOS installs that previously only stored `encryption_enabled = true`.

**Native backend generation:** `generate_key()` using `rand::rng().fill_bytes` (OS CSPRNG). Called once when encryption is first enabled and native storage is available.

**Native backend storage:** Hex-encoded as a 64-character string in the OS keychain via the `keyring` crate.

**Keychain identifiers:**
- Service: `"sub-swap"`
- Account: `"encryption-key"`

**Platform backends:**

| Platform | Backend |
|----------|---------|
| macOS | Keychain Services |
| Linux | secret-service (GNOME Keyring / KWallet) |
| Windows | Credential Manager |

**Fallback policy:** On Linux and Windows, setup falls back to the passphrase backend when native key storage is unavailable. The passphrase backend derives the AES key locally with Argon2id instead of storing key bytes in the OS keychain.

**Passphrase metadata on disk:** Only Argon2id salt and parameters are stored in `config.json` for passphrase mode:
- `salt_b64`
- `memory_kib`
- `iterations`
- `parallelism`

The derived 32-byte encryption key itself is never written to disk.

**Key lifecycle:** Native keys are generated once, then retrieved from the OS keychain on each encrypt/decrypt operation. Passphrase keys are re-derived on demand from the provided passphrase plus the stored Argon2id salt/parameters. Keys are not cached between operations.

**Disabled encryption shortcut:** `get_or_default_key(keystore, needed: bool)` returns a zeroed 32-byte array when `needed` is false, bypassing the keychain call entirely. This supports the `encrypt = false` configuration without breaking the function signature.

**Key loss:** If the OS keychain entry is deleted or the key is otherwise lost, all natively encrypted profiles become permanently unrecoverable. For passphrase mode, losing the passphrase has the same result. There is no key backup or escrow mechanism.

**Source files:** `src/crypto/keychain.rs`, `src/crypto/passphrase.rs`, `src/key.rs`, `src/config.rs`

## Threat Model

### What Is Protected

- **Inactive credentials encrypted at rest** with AES-256-GCM (confidentiality + integrity). Authentication tags detect tampering.
- **Key material never written to disk.** Native mode stores the key in the OS keychain; passphrase mode stores only Argon2id salt/parameters in `config.json`, not the derived key bytes.
- **All files 0600 on Unix** — owner-only read/write access. Applied via `fs::Permissions::from_mode(0o600)` inside `#[cfg(unix)]` blocks; this is a no-op on Windows.
- **Zero network attack surface.** No network crates in the dependency tree; enforced mechanically by `tests/arch.rs` arch_03 test.
- **Path traversal blocked** by `validate_profile_name()` (rejects `/`, `\`, `..`, leading `.`; allows only alphanumeric + `-` + `_`).

### What Is NOT Protected

- **Active profile credentials in `~/.codex/auth.json` are always plaintext.** Codex must be able to read its own config files; encryption of the active profile is not possible under the current design.
- **No memory zeroization.** Keys and plaintext exist in heap memory until Rust drops them. They are not explicitly zeroed before deallocation.
- **No protection against root-level or same-user access.** The OS keychain provides user-level isolation only, and a malicious same-user process can still access plaintext active credentials or prompt-captured passphrases.
- **Key loss = unrecoverable encrypted profiles.** There is no key backup, escrow, or recovery mechanism.
- **No protection against malicious code running as the same user.** A compromised process with the same UID can read `~/.codex/auth.json` and call keychain APIs.
- **`profiles.json` is not encrypted** (contains metadata only — profile names and active flag). It is 0600 to restrict access, but the contents themselves are not confidential.

## Profile Switch Lifecycle

The full implementation in `src/profile/switch.rs` performs these 14 steps in order:

1. Load `ProfileStore` from `profiles.json`
2. If target is already active → return Ok (no-op)
3. If target doesn't exist in index → return `ProfileNotFound` error
4. Load target profile files from `~/.sub-swap/profiles/<target>/`
5. Detect if encrypted (`.enc` suffix present on files)
6. Decrypt target files in memory if encrypted
7. Read current active profile from `~/.codex/`
8. Encrypt old active profile files (if `encrypt=true`), write to `~/.sub-swap/profiles/<old>/`
9. Create backups: `~/.codex/auth.json.bak` and `~/.codex/config.toml.bak`
10. Write target plaintext to `~/.codex/auth.json` and `~/.codex/config.toml`
11. Set 0600 permissions on codex files (Unix, via `#[cfg(unix)]`)
12. Update `profiles.json` index: set_active(target), save
13. On success: remove backup files
14. On any failure in steps 10-12: restore from backups, remove backups, return error

### Atomicity Note

The backup-and-restore mechanism (steps 9, 13, 14) provides the atomic-swap property. There is a brief window between steps 9 and 10 where both backup and active codex files exist simultaneously. If the process is killed in this window, backup files (`auth.json.bak`, `config.toml.bak`) will remain in `~/.codex/` and must be cleaned up manually before the next switch attempt.

The design spec described the switch as 5 steps; the actual implementation adds the backup/restore mechanism which the spec omitted. SECURITY.md reflects the actual code.

## File Permissions

| File | Location | Encrypted | Mode (Unix) | Notes |
|------|----------|-----------|-------------|-------|
| `profiles.json` | `~/.sub-swap/` | No | 0600 | Metadata only (profile names, active flag) |
| `config.json` | `~/.sub-swap/` | No | 0600 | App configuration |
| `auth.json(.enc)` | `~/.sub-swap/profiles/<name>/` | Yes (inactive) | 0600 | Codex API credentials |
| `config.toml(.enc)` | `~/.sub-swap/profiles/<name>/` | Yes (inactive) | 0600 | Codex configuration |
| `auth.json` | `~/.codex/` | No (active) | 0600 | Active profile — plaintext for Codex |
| `config.toml` | `~/.codex/` | No (active) | 0600 | Active profile — plaintext for Codex |

The `.enc` suffix is added to profile files in `~/.sub-swap/profiles/<name>/` when they are stored encrypted. The absence of `.enc` indicates plaintext storage (used when `encrypt = false` in app config).

Mode 0600 is applied via `fs::Permissions::from_mode(0o600)` inside `#[cfg(unix)]` blocks. On Windows, these blocks are compiled out and file permissions are managed by the OS via ACLs instead.

## Input Validation

`validate_profile_name()` in `src/error.rs` enforces these rules before any profile operation:

- **Non-empty:** blank names are rejected
- **No path separators:** `/` and `\` are forbidden
- **No traversal sequences:** `..` is forbidden anywhere in the name
- **No leading dot:** names cannot start with `.`
- **Character allowlist:** only alphanumeric characters (`a-z`, `A-Z`, `0-9`), hyphens (`-`), and underscores (`_`) are permitted

**Called at:** CLI dispatch (before any operation reaches business logic) and inside `add_profile_from_codex` and `add_profile_from_path` functions in `src/profile/switch.rs`.

**Prevents:** Directory traversal attacks where a crafted profile name like `../../etc/passwd` could escape the `~/.sub-swap/profiles/` directory and overwrite arbitrary files.

## Testing Security Properties

Security properties are verified by the existing test suite. Key areas covered:

**Encryption correctness** (`src/crypto/mod.rs` test module):
- `test_encrypt_then_decrypt_roundtrip` — round-trip produces identical plaintext
- `test_decrypt_with_wrong_key_fails` — wrong key returns an error (no silent success)
- `test_decrypt_tampered_data_fails` — modified ciphertext/tag is detected by GCM authentication
- `test_decrypt_too_short_data_fails` — data shorter than 28 bytes is rejected before decryption
- `test_two_encryptions_produce_different_output` — same plaintext encrypts to different ciphertext (random nonce)

**Passphrase derivation** (`src/crypto/passphrase.rs` test module):
- `test_derive_key_matches_fixed_output` — Argon2id derivation stays stable for fixed inputs
- `test_derive_key_changes_when_salt_changes` — changing the salt changes the derived key
- `test_decode_salt_b64_roundtrip` — stored salt metadata round-trips through base64 encoding
- `test_decode_salt_b64_rejects_malformed_input` / `test_decode_salt_b64_rejects_wrong_length` — malformed or truncated salt metadata is rejected
- `test_derive_key_rejects_invalid_params` — invalid Argon2id parameters fail closed

**Key management** (`src/crypto/keychain.rs` test module via `MockKeyStore`):
- `test_mock_keystore_roundtrip` — set_key then get_key returns the same key
- `test_mock_keystore_get_without_set_fails` — calling get_key before set_key returns an error

**Backend resolution** (`src/key.rs` test module):
- `test_resolve_key_reads_native_key_from_store` — native backend returns the stored OS-keychain key
- `test_resolve_key_derives_passphrase_backend` — passphrase backend derives the same key for the same passphrase + KDF metadata
- `test_resolve_key_passphrase_backend_requires_passphrase` — passphrase mode rejects missing runtime passphrase input
- `test_initialize_native_backend_is_idempotent` — native initialization reuses an existing stored key
- `test_initialize_native_backend_propagates_non_missing_key_error` — unexpected native keystore failures surface instead of silently falling back
- `test_initialize_passphrase_backend_returns_config_and_key` — passphrase initialization returns the KDF metadata plus derived key

**Profile switch atomicity** (`src/profile/switch.rs` test module):
- `test_switch_updates_codex_files` — target profile appears in `~/.codex/` after switch
- `test_switch_encrypts_old_profile` — old profile files are encrypted when encrypt=true
- `test_switch_to_nonexistent_fails` — switching to a missing profile returns `ProfileNotFound`

**Note on real keychain verification:** Automated tests use `MockKeyStore`; real OS keychain integration remains manually verified on the target platform.

**Note on permission testing:** The 0600 file permission constraint (`#[cfg(unix)]`) is not covered by automated tests. Verifying file permissions in a cross-platform test suite requires platform-specific test code. This is a known gap — the implementation is correct but the enforcement is manual.
