# Cross-Platform Key Storage Design

## Summary

`sub-swap` is documented as supporting macOS, Linux, and Windows key storage, but the current build only enables the `keyring` crate's `apple-native` feature. On Linux and Windows that causes `keyring` to fall back to its in-memory mock backend instead of a real OS credential store, so encryption keys do not persist across runs and encrypted profiles are not usable.

This design makes platform support real:

- Enable native key storage on macOS, Linux, and Windows.
- Keep macOS behavior intact for existing installs.
- Add a secure headless fallback for Linux and Windows using a passphrase-derived key.
- Store only backend metadata on disk, never the fallback key or passphrase.

## Goals

- Support persistent encrypted profile storage on macOS, Linux, and Windows.
- Use native OS credential storage by default on supported interactive environments.
- Provide a secure fallback for Linux and Windows systems where native key storage is unavailable.
- Preserve existing macOS encrypted installs and legacy configs without manual migration.
- Keep the AES-256-GCM data format unchanged so existing encrypted profile files remain readable with the correct key.

## Non-Goals

- Recover hypothetical Linux or Windows encrypted installs created by the current apple-only build.
- Introduce automatic silent backend switching after initial setup.
- Change the encryption algorithm or profile file layout.
- Add cloud sync, key escrow, or key export/import.

## Current State

The encryption key is stored through `src/crypto/keychain.rs` using `keyring::Entry`. The production code assumes that `keyring` is backed by a real OS store, and the docs already describe:

- macOS Keychain on macOS
- Secret Service / desktop keyring on Linux
- Credential Manager on Windows

The implementation problem is in dependency wiring:

- `Cargo.toml` enables `keyring = { version = "3.6.3", features = ["apple-native"] }`
- `keyring` has no default native backend features
- On Linux and Windows, `keyring` therefore uses its built-in mock backend

The result is:

- macOS works as designed
- Linux and Windows compile, but encryption keys are not persisted securely
- The wizard text and documentation overstate actual support

## Backend Model

Introduce an explicit key backend layer driven by config rather than assuming every platform uses a single storage mechanism.

### Backends

#### Native backend

Uses the `keyring` crate with real platform features enabled:

- macOS: `apple-native`
- Linux: `linux-native-sync-persistent`
- Windows: `windows-native`

This remains the preferred backend everywhere it is available.

#### Passphrase backend

Used as the secure fallback for Linux and Windows when native key storage is unavailable or fails during setup.

The encryption key is derived from a user passphrase using `Argon2id`. The app stores only:

- random salt
- Argon2id memory cost
- Argon2id time cost
- Argon2id parallelism

The app never stores:

- the derived 32-byte encryption key
- the passphrase

#### Auto-selection behavior

At setup time:

- Try native backend first.
- If native backend succeeds, persist `key_backend = native`.
- If native backend fails on Linux or Windows, offer passphrase fallback.
- If the user declines fallback, leave encryption disabled and return a clear error or cancellation path.

At runtime:

- Use the configured backend only.
- Do not silently change backend during normal encrypt/decrypt operations.

## Config Schema

Extend `config.json` from a single boolean into explicit encryption metadata.

### Proposed fields

```json
{
  "encryption_enabled": true,
  "key_backend": "native"
}
```

For passphrase mode:

```json
{
  "encryption_enabled": true,
  "key_backend": "passphrase",
  "passphrase_kdf": {
    "salt_b64": "base64-encoded-random-salt",
    "memory_kib": 65536,
    "iterations": 3,
    "parallelism": 1
  }
}
```

### Validation rules

- If `encryption_enabled` is `false`, `key_backend` and `passphrase_kdf` may be absent.
- If `key_backend = native`, `passphrase_kdf` must be absent.
- If `key_backend = passphrase`, `passphrase_kdf` must be present and complete.
- If `encryption_enabled = true` and required backend metadata is missing, fail with a configuration error.

## macOS Legacy Migration

Existing macOS installations must continue to work with no user action.

### Legacy config handling

Current configs may look like:

```json
{
  "encryption_enabled": true
}
```

Migration rule:

- If `key_backend` is missing and `encryption_enabled = true`, interpret the config as legacy native mode.
- On macOS, this means reading the existing key from the current Keychain entry and proceeding normally.
- The next successful config write should persist `key_backend = native` so the config becomes explicit.

If `encryption_enabled = false` and `key_backend` is missing:

- treat the config as a legacy unconfigured state
- keep encryption disabled
- do not synthesize passphrase metadata

This migration is intentionally conservative and only formalizes current macOS behavior.

## Key Derivation Design

The passphrase backend derives the same 32-byte key on demand from:

- passphrase bytes
- per-install random salt
- stored Argon2id parameters

### KDF choice

Use `Argon2id` because it is the standard password-based KDF choice for new systems with resistance to GPU and ASIC guessing attacks.

### Output

Derive exactly 32 bytes so the existing AES-256-GCM layer can remain unchanged.

### Recommended defaults

Initial defaults should be conservative and adjustable in code if platform testing requires tuning:

- memory: 64 MiB
- iterations: 3
- parallelism: 1

The exact constants belong in the implementation, but the design requirement is:

- choose values expensive enough to slow offline guessing
- avoid settings so aggressive that normal interactive use becomes unreliable on low-end systems

## UX and Prompting

### First-launch wizard

When the first-launch wizard enables encryption:

1. generate a random 32-byte key candidate only for native mode
2. try storing it in the native OS key store
3. if native storage succeeds, save `key_backend = native`
4. if native storage fails on Linux or Windows, explain the failure and offer passphrase fallback
5. if passphrase fallback is chosen, prompt twice, validate match, derive key, save KDF metadata, and proceed

Wizard copy should identify the backend actually chosen:

- macOS Keychain
- system keyring / Secret Service on Linux
- Windows Credential Manager
- passphrase-derived key fallback

### CLI

The current command surface is:

- `sub-swap config show`
- `sub-swap config set encryption true|false`

The implementation should preserve this surface for now and make `config set encryption true` interactive when backend setup is required.

Behavior:

- enabling encryption triggers backend setup
- disabling encryption decrypts inactive profiles to plaintext as it does today
- showing config should display backend metadata in a human-readable way

The existing wizard copy that references `sub-swap config --encrypt` should be corrected to match the actual CLI.

### TUI

The TUI currently performs encrypted operations without any key-entry UI. For passphrase mode it needs a masked prompt flow:

- prompt for passphrase only when an operation actually needs the key
- do not echo the passphrase
- keep the passphrase out of the persistent status area
- clear input buffer state after use

The TUI should continue to work unchanged for native backend users.

### Non-interactive support

For passphrase backend only, support an optional environment variable:

- `SUB_SWAP_PASSPHRASE`

Use it only when interactive prompting is unavailable or explicitly desired. This is less secure operationally than a typed prompt, but it is useful for headless automation and remote shells.

## Error Handling

### Native backend errors

If `key_backend = native` and retrieval fails:

- return a concrete key-storage error
- mention the configured native backend
- explain that encrypted profiles cannot be decrypted without the stored key

Do not silently switch to passphrase mode.

### Passphrase backend errors

If passphrase confirmation does not match during setup:

- fail the setup step and allow retry

If passphrase-derived decryption fails:

- return an error phrased as incorrect passphrase or corrupted encrypted profile data

If required passphrase metadata is missing:

- treat it as config corruption

## Documentation Updates

Update:

- `docs/SECURITY.md`
- `docs/ARCHITECTURE.md`
- any README or release-facing docs that claim Linux and Windows support

The docs must say explicitly:

- native backend is the default
- Linux and Windows have passphrase fallback when native key storage is unavailable
- macOS legacy configs migrate automatically to explicit `native`

## Testing Strategy

### Unit tests

- config parsing for legacy boolean-only macOS configs
- config parsing for explicit native backend
- config parsing for explicit passphrase backend
- config validation failures for incomplete metadata
- passphrase KDF determinism with same salt and passphrase
- passphrase KDF variance with different salt or passphrase
- backend selection logic for disabled, native, and passphrase modes

### Integration tests

- encrypt/decrypt roundtrip with passphrase backend metadata in temp directories
- enabling encryption with native backend when key store mock succeeds
- macOS legacy-config migration behavior at the config layer
- toggling encryption off still decrypts inactive profiles to plaintext

### Test constraints

- real OS keychain integration is still not part of automated tests
- production key store access remains behind a trait or wrapper so tests can use mocks
- passphrase prompting behavior should be tested at the smallest practical seam, not by brittle end-to-end terminal automation

## Implementation Impact

Primary files likely to change:

- `Cargo.toml`
- `src/config.rs`
- `src/crypto/keychain.rs`
- `src/cli.rs`
- `src/tui/wizard.rs`
- `src/tui/mod.rs`
- docs listed above

Potential new modules:

- `src/crypto/passphrase.rs` for Argon2id derivation and metadata handling

This split is preferred over overloading `src/crypto/keychain.rs` with both OS integration and passphrase KDF logic.

## Open Decisions Resolved by This Design

- Linux and Windows support should be real, not documentation-only.
- Headless fallback should prioritize security over convenience.
- Passphrase-derived fallback is preferred to an encrypted key file.
- Existing macOS configs must migrate transparently to explicit native backend metadata.
