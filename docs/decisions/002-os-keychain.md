# ADR-002: OS Keychain for Encryption Key Storage

**Status:** Accepted
**Date:** 2026-04-02

## Context

The 256-bit encryption key must be stored somewhere the application can retrieve it, but NOT on the filesystem alongside the encrypted files. Storing the key in a dotfile would defeat the purpose of encryption — anyone with filesystem access could read both the key and the encrypted data.

## Decision

Store the encryption key in the operating system's native keychain via the `keyring` crate:
- Service name: `"sub-swap"`
- Account name: `"encryption-key"`
- Key format: hex-encoded (64 character string)
- Platform backends: macOS Keychain Services, Linux secret-service (GNOME Keyring / KWallet), Windows Credential Manager
- Abstracted behind `KeyStore` trait with `get_key()` and `set_key()` methods

## Consequences

**Easier:**
- Key is protected by OS-level access controls (user login required)
- Cross-platform via single crate abstraction
- No filesystem exposure of key material
- `KeyStore` trait enables `MockKeyStore` for testing without real keychain access

**Harder:**
- Key is machine-bound — migrating profiles between machines requires manual key transfer
- Linux requires a running secret-service daemon (GNOME Keyring or KWallet)

**Constrained:**
- Key loss = unrecoverable encrypted profiles (no escrow or backup mechanism)
- `keyring` crate is a required dependency

## Implementation

- `src/crypto/keychain.rs` — `KeyStore` trait, `OsKeyStore` (production), `MockKeyStore` (tests)
- `OsKeyStore` wraps `keyring::Entry::new("sub-swap", "encryption-key")`
