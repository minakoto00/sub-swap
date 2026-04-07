---
phase: 01-code-quality-foundation
reviewed: 2026-04-07T16:00:00Z
depth: standard
files_reviewed: 19
files_reviewed_list:
  - Cargo.toml
  - clippy.toml
  - justfile
  - rustfmt.toml
  - src/cli.rs
  - src/config.rs
  - src/crypto/keychain.rs
  - src/crypto/mod.rs
  - src/error.rs
  - src/guard.rs
  - src/lib.rs
  - src/main.rs
  - src/paths.rs
  - src/profile/mod.rs
  - src/profile/store.rs
  - src/profile/switch.rs
  - src/tui/mod.rs
  - src/tui/widgets.rs
  - src/tui/wizard.rs
findings:
  critical: 2
  warning: 6
  info: 5
  total: 13
status: issues_found
---

# Phase 01: Code Review Report

**Reviewed:** 2026-04-07T16:00:00Z
**Depth:** standard
**Files Reviewed:** 19
**Status:** issues_found

## Summary

The codebase is well-structured with clean separation of concerns, good test coverage, and solid security fundamentals (path traversal prevention, 0600 file permissions, encryption-at-rest via OS keychain). Phase 01 quality improvements (clippy pedantic lints, rustfmt config, justfile, lib.rs extraction) are correctly applied.

Key concerns: (1) The `encode_key` function silently discards `write!` errors, (2) the `switch_profile` function can leave the system in an inconsistent state if it fails mid-operation, (3) the TUI `handle_input_name` handler skips profile name validation (path traversal possible from TUI input), and (4) duplicated logic across CLI and TUI helpers could lead to drift.

## Critical Issues

### CR-01: TUI Input Name Handler Skips Profile Name Validation

**File:** `src/tui/mod.rs:320-327`
**Issue:** When a user types a profile name in the TUI `handle_input_name` handler for `Action::Add`, the entered name is passed directly to `switch::add_profile_from_codex` without calling `validate_profile_name()` first. While `add_profile_from_codex` does call `validate_profile_name` internally (line 103 of `switch.rs`), the `Action::Rename` path at line 367 calls `store.index.rename()` directly -- which does NOT validate the new name. This means a user can type `../evil` in the TUI rename flow and bypass the path traversal check. The CLI `cmd_rename` correctly validates both old and new names (cli.rs:203-204), but the TUI path does not.
**Fix:**
```rust
// In handle_input_name, at the start of the Enter handler (after trimming):
Some(Action::Rename) => {
    // Add validation before proceeding
    if let Err(e) = crate::error::validate_profile_name(&name) {
        state.message = Some(format!("Error: {e}"));
        state.screen = AppScreen::Main;
        state.pending_action = None;
        return Ok(());
    }
    let Some(n) = state.selected_name() else {
        // ... existing code
```

### CR-02: Non-Atomic Profile Switch Can Leave System in Inconsistent State

**File:** `src/profile/switch.rs:22-91`
**Issue:** The `switch_profile` function performs a multi-step operation (load target, encrypt old, write target to codex, update index) without any rollback mechanism. If the function fails after writing the old profile back to storage (step 7, line 72) but before writing the target to `~/.codex/` (step 8, line 76-77), the user loses their active codex config -- the old profile is now encrypted in storage, and the codex directory has stale or missing files. Similarly, if updating the index fails (line 89), the files have been swapped but the index still points to the old profile.
**Fix:** Implement a write-ahead pattern: write the new codex files to temporary paths first, then atomically rename them into place. At minimum, save a backup of the current codex files before overwriting:
```rust
// Before overwriting codex files, create backups
let auth_backup = paths.codex_dir.join("auth.json.bak");
let config_backup = paths.codex_dir.join("config.toml.bak");
fs::copy(paths.codex_auth(), &auth_backup)?;
if paths.codex_config().exists() {
    fs::copy(paths.codex_config(), &config_backup)?;
}

// Write target to codex dir
fs::write(paths.codex_auth(), &target_auth)?;
fs::write(paths.codex_config(), &target_config)?;

// ... update index ...

// Clean up backups on success
let _ = fs::remove_file(auth_backup);
let _ = fs::remove_file(config_backup);
```

## Warnings

### WR-01: Silent Error Suppression in encode_key

**File:** `src/crypto/keychain.rs:9`
**Issue:** The `let _ = write!(s, "{b:02x}");` line explicitly discards the `Result` from `write!`. While `write!` to a `String` cannot fail in practice (the `fmt::Write` impl for `String` is infallible), the `let _ =` pattern suppresses the compiler warning and makes it look intentional to ignore a potentially meaningful error. If the underlying type ever changes, this would silently produce a truncated hex key.
**Fix:** Use `write!(s, "{b:02x}").expect("writing to String cannot fail");` or simply use `format!("{b:02x}")` to make the intent clear:
```rust
fn encode_key(key: &[u8; 32]) -> String {
    key.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        write!(s, "{b:02x}").expect("writing to String is infallible");
        s
    })
}
```

### WR-02: Duplicated get_key Helper Between CLI and TUI

**File:** `src/cli.rs:331-336` and `src/tui/mod.rs:626-631`
**Issue:** The `get_or_warn_key` function in `cli.rs` and the `get_key` function in `tui/mod.rs` are identical in logic (return zeroed key when not needed, otherwise call keystore). This duplication means a bug fix or behavior change in one might not be applied to the other. The CLI version takes `&OsKeyStore` while both do the same thing.
**Fix:** Extract a shared helper into the `crypto::keychain` module or a shared utility module:
```rust
// In crypto/keychain.rs, add:
pub fn get_or_default_key(keystore: &impl KeyStore, needed: bool) -> Result<[u8; 32]> {
    if !needed {
        return Ok([0u8; 32]);
    }
    keystore.get_key()
}
```

### WR-03: Paths::new() Panics on Missing Home Directory

**File:** `src/paths.rs:17`
**Issue:** `Paths::new()` calls `dirs::home_dir().expect("Could not determine home directory")` which will panic if the home directory cannot be determined (e.g., in containers, CI environments, or unusual system configurations). Since this is called during normal program startup (cli.rs:74), an unhandled panic produces a poor user experience with a backtrace instead of a clean error message.
**Fix:** Return a `Result` instead of panicking:
```rust
pub fn new() -> Result<Self> {
    let home = dirs::home_dir()
        .ok_or_else(|| SubSwapError::Io(
            io::Error::new(io::ErrorKind::NotFound, "Could not determine home directory")
        ))?;
    Ok(Self {
        codex_dir: home.join(".codex"),
        sub_swap_dir: home.join(".sub-swap"),
    })
}
```

### WR-04: Missing Validation on --from Path in cmd_add

**File:** `src/cli.rs:164-169`
**Issue:** When the user passes `--from <path>` to the `add` command, the path is passed directly as `Path::new(source_path)` without any validation or canonicalization. While `add_profile_from_path` checks that `auth.json` exists at the source, there is no check that the path is a valid directory or that it does not point to a sensitive system location. A symlink or specially crafted path could read files from unexpected locations.
**Fix:** Add basic validation that the source path is a directory:
```rust
if let Some(source_path) = from {
    let source = Path::new(source_path);
    if !source.is_dir() {
        return Err(SubSwapError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("Source path '{}' is not a directory", source_path),
        )));
    }
    switch::add_profile_from_path(
        // ...
```

### WR-05: ProfileStore::load_profile_files Falls Through to Plaintext Without Checking Existence

**File:** `src/profile/store.rs:103-119`
**Issue:** In `load_profile_files`, if the `.enc` file does not exist, the function falls through to read the plaintext file without checking if that file exists either. If neither file exists, the user gets a raw `io::Error` ("No such file or directory") instead of a meaningful error like `ProfileNotFound` or a message indicating corrupted profile data. The auth and config files are also checked independently -- if `auth.json.enc` exists but `config.toml.enc` does not and `config.toml` also does not exist, the function will fail with an unhelpful OS error.
**Fix:** Check existence before reading and return a domain-specific error:
```rust
let auth_path = if profile_dir.join("auth.json.enc").exists() {
    profile_dir.join("auth.json.enc")
} else if profile_dir.join("auth.json").exists() {
    profile_dir.join("auth.json")
} else {
    return Err(SubSwapError::Crypto(format!(
        "Profile '{}' has no auth.json or auth.json.enc file — profile data may be corrupted",
        name
    )));
};
let auth_data = fs::read(&auth_path)?;
```

### WR-06: Wizard Does Not Validate Profile Name Input

**File:** `src/tui/wizard.rs:32`
**Issue:** In `run_first_launch`, the profile name collected via `prompt_string("Profile name", Some("default"))` is passed directly to `switch::add_profile_from_codex` without calling `validate_profile_name` first. While `add_profile_from_codex` does call the validation internally (switch.rs:103), the error message from `add_profile_from_codex` is a generic `SubSwapError::InvalidProfileName` that would propagate up as a program error. It would be better UX to validate early and re-prompt the user in the wizard flow.
**Fix:** Add validation with a retry loop in the wizard:
```rust
let name = loop {
    let input = prompt_string("Profile name", Some("default"))?;
    match crate::error::validate_profile_name(&input) {
        Ok(()) => break input,
        Err(e) => {
            println!("{e}");
            println!("Please try again.");
        }
    }
};
```

## Info

### IN-01: Unused Import of use_std::path::Path in cli.rs

**File:** `src/cli.rs:1`
**Issue:** `use std::path::Path` is imported at the top of the file but it is only used inside `cmd_add` where it wraps `source_path` with `Path::new()`. This is technically used, but the import could be scoped to the function for clarity. This is a minor style observation.
**Fix:** No action needed -- the import is used. Consider moving it into the function body if preferred:
```rust
fn cmd_add(...) -> Result<()> {
    use std::path::Path;
    // ...
}
```

### IN-02: Magic Constant for Cognitive Complexity Threshold

**File:** `clippy.toml:2`
**Issue:** `cognitive-complexity-threshold = 25` is set without comment explaining why 25 was chosen over the default of 25 (which is actually the default). If this was intentionally set to match the default, a comment would clarify intent. If it was intended to be different, it should be adjusted.
**Fix:** Add a comment or adjust the value:
```toml
# Default is 25; kept explicit for visibility in code review
cognitive-complexity-threshold = 25
```

### IN-03: ProfileStore Reloaded on Every TUI Loop Iteration

**File:** `src/tui/mod.rs:53`
**Issue:** Inside the TUI event loop, `ProfileStore::load(paths)?` is called on every iteration (every keypress). This means the profiles.json file is re-read and re-parsed from disk on every single key event, including cursor movements. While this ensures freshness, it is unnecessary overhead for a single-user tool where the store only changes when the TUI itself makes modifications.
**Fix:** Consider reloading only after mutation operations (switch, add, rename, delete, note). This is a minor observation and not a correctness issue.

### IN-04: Inconsistent Error Variant Usage for Non-Crypto Errors

**File:** `src/cli.rs:258-259` and `src/cli.rs:275-276`
**Issue:** The `cmd_config` function uses `SubSwapError::Crypto(...)` for what are really configuration validation errors (invalid value for encryption, unknown config key). This conflates crypto errors with config errors, which could confuse users or make error handling less precise in the future.
**Fix:** Consider adding a `SubSwapError::Config(String)` variant for configuration-related errors, or use a more generic variant name.

### IN-05: codex_found Check in Wizard Only Requires auth.json

**File:** `src/tui/wizard.rs:21`
**Issue:** The wizard checks `paths.codex_auth().exists() && paths.codex_config().exists()` to determine if Codex is configured, requiring both files. However, elsewhere in the codebase (e.g., `add_profile_from_codex` at switch.rs:110), only `auth.json` is required -- `config.toml` is optional (an empty Vec is used if missing). This means the wizard would report "No Codex configuration found" even when a valid auth.json exists but config.toml does not.
**Fix:** Align the check with the rest of the codebase:
```rust
let codex_found = paths.codex_auth().exists();
```

---

_Reviewed: 2026-04-07T16:00:00Z_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
