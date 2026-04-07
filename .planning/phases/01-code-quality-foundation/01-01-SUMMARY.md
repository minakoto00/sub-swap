---
phase: 01-code-quality-foundation
plan: 01
subsystem: tooling/lint
tags: [clippy, rustfmt, cargo-lints, dead-code, pedantic]
dependency_graph:
  requires: []
  provides: [zero-warning-clippy-baseline, rustfmt-config, clippy-config, cargo-lints-table]
  affects: [src/cli.rs, src/crypto/keychain.rs, src/error.rs, src/guard.rs, src/profile/mod.rs, src/tui/mod.rs, src/tui/widgets.rs, src/tui/wizard.rs, Cargo.toml, rustfmt.toml, clippy.toml]
tech_stack:
  added: [rustfmt.toml, clippy.toml, Cargo.toml [lints] table]
  patterns: [cfg(test)-gated mock types, let-else for early returns, clone_from for assigning_clones]
key_files:
  created: [rustfmt.toml, clippy.toml]
  modified: [Cargo.toml, src/cli.rs, src/crypto/keychain.rs, src/error.rs, src/guard.rs, src/profile/mod.rs, src/tui/mod.rs, src/tui/widgets.rs, src/tui/wizard.rs]
decisions:
  - "Removed has_key from KeyStore trait: it was dead in production code (never called through trait or anywhere); removing it is cleaner than suppressing with #[allow]"
  - "Used priority=-1 for all/pedantic lint groups in [lints.clippy] to let individual allow entries take precedence (required by Cargo 1.94 lint_groups_priority lint)"
  - "Used let-else pattern for selected_name() early-return cases rather than if let with negation — more idiomatic Rust 2021"
metrics:
  duration: "~25 minutes"
  completed: "2026-04-07"
  tasks_completed: 2
  files_modified: 9
  files_created: 2
---

# Phase 1 Plan 1: Fix Clippy Violations and Add Lint Config Summary

**One-liner:** Zero-warning clippy baseline under pedantic lints via source fixes and Cargo.toml `[lints]` table with rustfmt.toml and clippy.toml config files.

## What Was Built

Established mechanical code quality enforcement for the sub-swap codebase by:

1. **Fixing all clippy violations** in 8 source files (6 dead-code warnings + ~14 pedantic violations)
2. **Creating `rustfmt.toml`** with `edition = "2021"` and `max_width = 100`
3. **Creating `clippy.toml`** with `msrv = "1.94.1"` and `cognitive-complexity-threshold = 25`
4. **Adding `[lints]` tables to `Cargo.toml`** with `unsafe_code = "forbid"` and `clippy::pedantic = "warn"` plus D-01 allow-list

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Fix all clippy violations | 08e3600 | src/cli.rs, src/crypto/keychain.rs, src/error.rs, src/guard.rs, src/profile/mod.rs, src/tui/mod.rs, src/tui/widgets.rs, src/tui/wizard.rs |
| 2 | Create lint and format config files | fd3852f | rustfmt.toml, clippy.toml, Cargo.toml |

## Verification Results

- `cargo clippy -- -D warnings`: exits 0, zero warnings
- `cargo test`: 43 unit tests + 2 integration tests = 45 passed, 0 failed
- `rustfmt.toml` contains `edition = "2021"` and `max_width = 100`
- `clippy.toml` contains `msrv = "1.94.1"` and `cognitive-complexity-threshold = 25`
- `Cargo.toml` has `[lints.rust]` with `unsafe_code = "forbid"`
- `Cargo.toml` has `[lints.clippy]` with `pedantic` at `priority = -1` and full D-01 allow-list

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Removed `has_key` from `KeyStore` trait (not just `MockKeyStore`)**
- **Found during:** Task 1 — after gating `MockKeyStore` with `#[cfg(test)]`, the trait method `has_key` still triggered `dead_code` warning
- **Issue:** `has_key` is defined on the `KeyStore` trait and both `OsKeyStore` and `MockKeyStore` implement it, but it is never called anywhere in the codebase (the plan incorrectly stated it was called in `cli.rs`)
- **Fix:** Removed `has_key` from the `KeyStore` trait definition and both implementations (`OsKeyStore` and `MockKeyStore`)
- **Files modified:** `src/crypto/keychain.rs`
- **Commit:** 08e3600

**2. [Rule 1 - Bug] Used `priority = -1` for lint group entries in `[lints.clippy]`**
- **Found during:** Task 2 — `cargo clippy -- -D warnings` failed with `lint_groups_priority` error
- **Issue:** Cargo 1.94 enforces that lint groups (`all`, `pedantic`) must have lower priority than individual lint overrides, otherwise the table ordering is ambiguous
- **Fix:** Changed `all = "warn"` and `pedantic = "warn"` to `all = { level = "warn", priority = -1 }` and `pedantic = { level = "warn", priority = -1 }`
- **Files modified:** `Cargo.toml`
- **Commit:** fd3852f

## Known Stubs

None — this plan adds no UI or data rendering code.

## Threat Flags

None — this plan modifies only tooling configuration and dead-code cleanup. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- [x] `rustfmt.toml` exists: `/Users/brainco/security/sub-swap/rustfmt.toml`
- [x] `clippy.toml` exists: `/Users/brainco/security/sub-swap/clippy.toml`
- [x] Commit 08e3600 exists (Task 1)
- [x] Commit fd3852f exists (Task 2)
- [x] `cargo clippy -- -D warnings` exits 0
- [x] All 45 tests pass
