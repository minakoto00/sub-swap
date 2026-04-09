# Testing

## Constraints and Invariants

| Rule | Mechanism | Notes |
|------|-----------|-------|
| Tests never touch real `~/.codex/` or `~/.sub-swap/` | `Paths::from_temp(tmp.path())` injects temp directory | Every test that does file I/O must use this |
| Tests never access real OS keychain | `MockKeyStore` backed by `RefCell<Option<[u8; 32]>>` | `get_key()` before `set_key()` returns Err |
| Tests never detect real processes | `MockGuard` with fixed PID list | `MockGuard::new(vec![])` for clean check, `MockGuard::new(vec![pid])` for blocked |
| `MockKeyStore`, `MockGuard`, `Paths::from_temp` are `#[cfg(test)]` only | Compile-time gating | Not available in production builds |
| Unit tests live in `#[cfg(test)] mod tests` inside the source file | Rust convention | Tests for `crypto/mod.rs` go in `src/crypto/mod.rs` |
| Integration tests use compiled binary via `std::process::Command` | `env!("CARGO_BIN_EXE_sub-swap")` | `cargo test --test integration` builds automatically |
| Structural tests parse source at runtime | `tests/arch.rs` reads `.rs` files with `std::fs::read_to_string` | See "Adding Architectural Rules" recipe below |
| Real OS keychain integration is manually verified | Automated tests use `MockKeyStore` instead of the platform keychain | Native backend smoke checks still require platform-specific manual runs |

> **IMPORTANT:** All three test abstractions (`Paths::from_temp`, `MockKeyStore`, `MockGuard`)
> are gated behind `#[cfg(test)]`. They are only available when running `cargo test`, not in
> production builds. Code examples below must appear inside `#[cfg(test)] mod tests { }` blocks.

---

## Test Infrastructure

### Paths Injection — Paths::from_temp

**Source:** `src/paths.rs` (constructor gated by `#[cfg(test)]`)

**What it does:** Maps `base/codex` to `~/.codex` equivalent and `base/sub-swap` to `~/.sub-swap` equivalent within a temp directory. Uses `TempDir` from the `tempfile` dev-dependency.

**Template (copy exactly):**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;
    use tempfile::TempDir;

    #[test]
    fn test_example() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        std::fs::create_dir_all(&paths.codex_dir).unwrap();
        // ... test using &paths
    }
}
```

**Note:** Callers must manually create required subdirectories. `Paths::from_temp` creates the mapping but not the directories themselves. If your test also needs `profiles_dir()`, create it explicitly:

```rust
std::fs::create_dir_all(paths.profiles_dir()).unwrap();
```

---

### Keychain Mock — MockKeyStore

**Source:** `src/crypto/keychain.rs` (gated by `#[cfg(test)]`)

**What it does:** In-memory keychain backed by `RefCell<Option<[u8; 32]>>`. Starts with no key stored. `get_key()` before `set_key()` returns `Err` (simulates a missing keychain entry).

**Template (copy exactly):**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_key, keychain::MockKeyStore};

    #[test]
    fn test_with_mock_keystore() {
        let store = MockKeyStore::new();
        let key = generate_key();
        store.set_key(&key).expect("set_key should succeed");
        let retrieved = store.get_key().expect("get_key should succeed");
        assert_eq!(retrieved, key);
    }
}
```

**Behavior:** `get_key()` before `set_key()` returns `Err` (simulates missing keychain entry). `MockKeyStore` implements the `KeyStore` trait, so it can be passed wherever a `&impl KeyStore` or `&dyn KeyStore` is expected.

---

### Process Guard Mock — MockGuard

**Source:** `src/guard.rs` (gated by `#[cfg(test)]`)

**What it does:** Takes a fixed `Vec<u32>` of PIDs at construction. `check()` returns `Ok(())` if the list is empty, `Err(SubSwapError::CodexRunning(pids))` if non-empty. Implements `CodexGuard` trait.

**Template (copy exactly):**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_blocks_when_codex_running() {
        let guard = MockGuard::new(vec![99999]);
        let result = guard.check();
        assert!(result.is_err());
        match result.unwrap_err() {
            SubSwapError::CodexRunning(pids) => assert_eq!(pids, vec![99999]),
            other => panic!("Expected CodexRunning, got: {:?}", other),
        }
    }
}
```

**For a clean check (no Codex running):** `MockGuard::new(vec![])` — `check()` returns `Ok(())`.

---

## Current Coverage Highlights

### Passphrase derivation tests

**Source:** `src/crypto/passphrase.rs`

These tests cover the pure Argon2id derivation path without touching the filesystem or real keychain:

- `test_derive_key_matches_fixed_output` locks the Argon2id output for a fixed passphrase, salt, and parameter set.
- `test_derive_key_changes_when_salt_changes` confirms salt uniqueness changes the derived key.
- `test_decode_salt_b64_roundtrip` verifies the salt metadata format written to config.
- `test_decode_salt_b64_rejects_malformed_input` and `test_decode_salt_b64_rejects_wrong_length` reject invalid on-disk salt metadata.
- `test_derive_key_rejects_invalid_params` ensures invalid Argon2id parameters fail cleanly.

### Backend resolution tests

**Source:** `src/key.rs`

These tests cover the orchestration layer that chooses the key backend:

- `test_resolve_key_returns_zero_key_when_encryption_disabled`
- `test_resolve_key_reads_native_key_from_store`
- `test_resolve_key_derives_passphrase_backend`
- `test_resolve_key_passphrase_backend_requires_passphrase`
- `test_initialize_native_backend_is_idempotent`
- `test_initialize_native_backend_propagates_non_missing_key_error`
- `test_initialize_passphrase_backend_returns_config_and_key`

They verify backend selection and metadata handling in `src/key.rs` while keeping the actual crypto and keychain details isolated in their own modules.

---

## How to Add a New Test

1. **Choose test location:**
   - Testing a single function → unit test in the source file's `#[cfg(test)] mod tests` block
   - Testing CLI behavior end-to-end → integration test in `tests/integration.rs`
   - Enforcing a structural rule → structural test in `tests/arch.rs`

2. **Set up temp paths** (if test does file I/O):
   ```rust
   let tmp = TempDir::new().unwrap();
   let paths = Paths::from_temp(tmp.path());
   std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
   std::fs::create_dir_all(&paths.codex_dir).unwrap();
   std::fs::create_dir_all(paths.profiles_dir()).unwrap();
   ```

3. **Mock external dependencies:**
   - Keychain access → `MockKeyStore::new()` (implements `KeyStore` trait)
   - Process detection → `MockGuard::new(vec![])` (implements `CodexGuard` trait)

4. **Write assertions** — use `assert_eq!`, `assert!(result.is_ok())`, or pattern match on error variants

5. **Run:** `cargo test --lib module::tests::test_name` for a single unit test, `cargo test --test integration` for integration tests, `cargo test --test arch` for structural tests

---

## Adding Architectural Rules

To add a new layer boundary rule (e.g., "`profile/store.rs` must not import `tui`"):

1. Open `tests/arch.rs`
2. Add a new `#[test]` function:
   ```rust
   #[test]
   fn arch_01_profile_store_does_not_import_tui() {
       let source = read_source("src/profile/store.rs");
       assert_no_crate_import("src/profile/store.rs", &source, &["tui"]);
   }
   ```
3. Run `cargo test --test arch` to verify it passes on the current codebase
4. The test uses prefix matching: checks for `use crate::{module}::` and `use crate::{module};`
5. Failure messages follow the 3-part format: VIOLATION / FOUND / HOW TO FIX

To add a new purity check (e.g., forbid `std::net` in a module):

1. Add a `#[test]` function that reads the source file and scans for the forbidden string pattern
2. Follow the `arch_02_crypto_mod_has_no_filesystem_io` pattern in `tests/arch.rs`

To add a new network crate to the deny-list:

1. Add the crate name to the `NETWORK_CRATES` constant at the top of `tests/arch.rs`

**Limitation:** `assert_no_crate_import` assumes all `use crate::` imports are single-line. Multi-line grouped imports (e.g., `use crate::{mod1, mod2}` spanning lines) may not be detected. Do not introduce multi-line grouped `use crate::` imports if boundary enforcement matters for that file.

---

## Test Location Map

| Test Type | Location | Run Command |
|-----------|----------|-------------|
| Unit tests (all) | `src/**/*.rs` inside `#[cfg(test)] mod tests` | `cargo test --lib` |
| Unit tests (module) | e.g., `src/crypto/mod.rs` | `cargo test --lib crypto::tests` |
| Unit tests (passphrase derivation) | `src/crypto/passphrase.rs` | `cargo test --lib crypto::passphrase::tests` |
| Unit tests (backend resolution) | `src/key.rs` | `cargo test --lib key::tests` |
| Unit tests (single) | e.g., `src/crypto/mod.rs` | `cargo test --lib crypto::tests::test_encrypt_then_decrypt_roundtrip` |
| Integration tests | `tests/integration.rs` | `cargo test --test integration` |
| Structural tests | `tests/arch.rs` | `cargo test --test arch` |
| All tests | All of the above | `cargo test` |
| Validate (fmt + lint + test) | justfile recipe | `just validate` |
