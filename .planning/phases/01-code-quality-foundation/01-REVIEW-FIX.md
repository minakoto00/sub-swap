---
phase: 01-code-quality-foundation
fixed_at: 2026-04-07T16:30:00Z
review_path: .planning/phases/01-code-quality-foundation/01-REVIEW.md
iteration: 1
findings_in_scope: 8
fixed: 8
skipped: 0
status: all_fixed
---

# Phase 01: Code Review Fix Report

**Fixed at:** 2026-04-07T16:30:00Z
**Source review:** .planning/phases/01-code-quality-foundation/01-REVIEW.md
**Iteration:** 1

**Summary:**
- Findings in scope: 8
- Fixed: 8
- Skipped: 0

## Fixed Issues

### CR-01: TUI Input Name Handler Skips Profile Name Validation

**Files modified:** `src/tui/mod.rs`
**Commit:** 815d91e
**Applied fix:** Added `validate_profile_name()` call at the start of the `Action::Rename` branch in `handle_input_name`. If validation fails, an error message is displayed and the handler returns early without performing the rename. This closes the path traversal vector where a user could type `../evil` in the TUI rename flow.

### CR-02: Non-Atomic Profile Switch Can Leave System in Inconsistent State

**Files modified:** `src/profile/switch.rs`
**Commit:** 0103bb8
**Applied fix:** Added backup-and-restore pattern to `switch_profile`. Before overwriting codex files, `auth.json.bak` and `config.toml.bak` are created. The write-and-update operations are wrapped in a closure; on failure, backups are restored and then cleaned up. On success, backups are removed. This prevents data loss if the switch fails mid-operation.

### WR-01: Silent Error Suppression in encode_key

**Files modified:** `src/crypto/keychain.rs`
**Commit:** 2a673a6
**Applied fix:** Replaced `let _ = write!(s, "{b:02x}");` with `write!(s, "{b:02x}").expect("writing to String is infallible");`. This makes the intent explicit and would surface an error if the underlying type ever changed.

### WR-02: Duplicated get_key Helper Between CLI and TUI

**Files modified:** `src/crypto/keychain.rs`, `src/cli.rs`, `src/tui/mod.rs`
**Commit:** 1605821
**Applied fix:** Extracted a shared `pub fn get_or_default_key(keystore: &impl KeyStore, needed: bool)` into `crypto::keychain`. Removed the duplicate `get_or_warn_key` from `cli.rs` and `get_key` from `tui/mod.rs`. Both modules now import and call the shared function. Also cleaned up unused `KeyStore` trait imports that resulted from the refactor.

### WR-03: Paths::new() Panics on Missing Home Directory

**Files modified:** `src/paths.rs`, `src/cli.rs`
**Commit:** ca9c28d
**Applied fix:** Changed `Paths::new()` to return `Result<Self>` instead of panicking with `expect()`. Returns `SubSwapError::Io` with `ErrorKind::NotFound` when the home directory cannot be determined. Removed the `Default` impl (incompatible with fallible construction). Updated the single call site in `cli.rs` to propagate the error with `?`.

### WR-04: Missing Validation on --from Path in cmd_add

**Files modified:** `src/cli.rs`
**Commit:** d357e4f
**Applied fix:** Added an `is_dir()` check on the `--from` source path before passing it to `add_profile_from_path`. Returns a clear `SubSwapError::Io` with a message identifying the invalid path if the check fails.

### WR-05: ProfileStore::load_profile_files Falls Through to Plaintext Without Checking Existence

**Files modified:** `src/profile/store.rs`
**Commit:** ce661f8
**Applied fix:** Added existence checks before reading profile files. For `auth.json`, if neither `.enc` nor plaintext exists, returns a domain-specific `SubSwapError::Crypto` error indicating potential corruption. For `config.toml`, if neither variant exists, returns an empty `Vec` (config is optional throughout the codebase). Also added `SubSwapError` to the module imports.

### WR-06: Wizard Does Not Validate Profile Name Input

**Files modified:** `src/tui/wizard.rs`
**Commit:** 552541d
**Applied fix:** Replaced the single `prompt_string` call with a validation loop that calls `validate_profile_name()` on each input. Invalid names display the error message and re-prompt with "Please try again." until a valid name is entered. Added `validate_profile_name` to the imports.

---

_Fixed: 2026-04-07T16:30:00Z_
_Fixer: Claude (gsd-code-fixer)_
_Iteration: 1_
