---
phase: 01-code-quality-foundation
plan: 02
subsystem: tooling/build-structure
tags: [lib-target, justfile, cargo-fmt, refactor, build]
dependency_graph:
  requires: [01-01-zero-warning-clippy-baseline]
  provides: [lib-target, justfile-recipes, fmt-baseline]
  affects: [src/cli.rs, src/lib.rs, src/main.rs, src/paths.rs, justfile]
tech_stack:
  added: [just 1.49.0, src/lib.rs library target]
  patterns: [thin-entry-point main.rs, lib re-export module, Default impl delegation]
key_files:
  created: [src/lib.rs, justfile]
  modified: [src/cli.rs, src/main.rs, src/paths.rs, src/config.rs, src/crypto/keychain.rs, src/crypto/mod.rs, src/error.rs, src/profile/store.rs, src/profile/switch.rs, src/tui/mod.rs, src/tui/widgets.rs, src/tui/wizard.rs]
decisions:
  - "Moved Cli/Commands/ConfigAction from main.rs to cli.rs so the library target owns all public API types"
  - "Added Default impl for Paths delegating to new() — required by clippy::new_without_default triggered by library target visibility"
  - "Formatted 10 files mechanically in a separate commit (D-07) from source/config changes"
metrics:
  duration: "~3 minutes"
  completed: "2026-04-07"
  tasks_completed: 3
  files_modified: 12
  files_created: 2
---

# Phase 1 Plan 2: Library Target, justfile, and Formatting Baseline Summary

**One-liner:** Library target (lib.rs) added with thin main.rs entry point, justfile providing 5 standardized dev recipes, and cargo fmt applied as a clean zero-diff formatting baseline.

## What Was Built

Completed the Phase 1 quality foundation by:

1. **Creating `src/lib.rs`** — thin re-export module declaring 8 `pub mod` entries, enabling `cargo build --lib`, `cargo doc`, and Phase 2 structural tests
2. **Refactoring `src/main.rs`** — reduced to 9 lines importing from `sub_swap::cli`; all `Cli`/`Commands`/`ConfigAction` types moved into `src/cli.rs`
3. **Creating `justfile`** — 5 recipes: `check`, `test`, `lint`, `fmt`, `validate`; the `validate` recipe chains with `&&` to stop on first failure
4. **Applying `cargo fmt`** — 10 files reformatted per `rustfmt.toml` (edition 2021, max_width 100); `cargo fmt --check` now exits 0
5. **Full pipeline verified** — `just validate` (fmt-check + clippy + all 45 tests) exits 0 end-to-end

## Tasks Completed

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | Move types to cli.rs, create lib.rs, update main.rs | 2fb4f0a | src/lib.rs (new), src/cli.rs, src/main.rs, src/paths.rs |
| 2 | Install just and create justfile | 9dfa186 | justfile (new) |
| 3 | Apply cargo fmt and verify full validation | a5c94df | 10 source files reformatted |

## Verification Results

- `cargo build --lib`: exits 0 (library target works)
- `cargo build`: exits 0 (binary target works)
- `cargo fmt --check`: exits 0 (no formatting diff)
- `cargo clippy -- -D warnings`: exits 0 (zero violations)
- `cargo test`: 43 unit + 2 integration = 45 passed, 0 failed
- `just validate`: exits 0 (full pipeline green)
- `just check`, `just test`, `just lint`, `just fmt`: each exit 0

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Added `Default` impl for `Paths` to fix `new_without_default` clippy lint**
- **Found during:** Task 1 — after adding lib.rs, `Paths` became part of the public API surface. The existing `new()` constructor without a corresponding `Default` impl triggered `clippy::new_without_default` under `-D warnings`
- **Issue:** This lint was not present in the pre-lib.rs codebase because `Paths` was only used internally (binary crate context). Adding `src/lib.rs` exposed it publicly, making the lint fire
- **Fix:** Added `impl Default for Paths { fn default() -> Self { Self::new() } }` in `src/paths.rs`
- **Files modified:** `src/paths.rs`
- **Commit:** 2fb4f0a

## Known Stubs

None — this plan adds no UI or data rendering code.

## Threat Flags

None — this plan modifies only build structure and formatting. No new network endpoints, auth paths, file access patterns, or schema changes introduced.

## Self-Check: PASSED

- [x] `src/lib.rs` exists: `/Users/brainco/security/sub-swap/.claude/worktrees/agent-a7d50a16/src/lib.rs`
- [x] `justfile` exists: `/Users/brainco/security/sub-swap/.claude/worktrees/agent-a7d50a16/justfile`
- [x] Commit 2fb4f0a exists (Task 1)
- [x] Commit 9dfa186 exists (Task 2)
- [x] Commit a5c94df exists (Task 3)
- [x] `cargo build --lib` exits 0
- [x] `cargo fmt --check` exits 0
- [x] `just validate` exits 0
- [x] All 45 tests pass
