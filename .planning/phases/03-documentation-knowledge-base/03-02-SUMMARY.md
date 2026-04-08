---
phase: 03-documentation-knowledge-base
plan: "02"
subsystem: docs
tags: [documentation, testing, adr, decision-records]
dependency_graph:
  requires: []
  provides: [docs/TESTING.md, docs/decisions/001-aes-256-gcm.md, docs/decisions/002-os-keychain.md, docs/decisions/003-path-injection.md, docs/decisions/004-offline-only.md]
  affects: []
tech_stack:
  added: []
  patterns: [agent-first-docs, constraints-first-structure, adr-format]
key_files:
  created:
    - docs/TESTING.md
    - docs/decisions/001-aes-256-gcm.md
    - docs/decisions/002-os-keychain.md
    - docs/decisions/003-path-injection.md
    - docs/decisions/004-offline-only.md
  modified: []
decisions:
  - "docs/TESTING.md leads with constraints table per agent-first D-01 principle"
  - "ADRs use minimal format (Status, Date, Context, Decision, Consequences, Implementation) without Alternatives Considered sections"
  - "All three test abstractions (Paths::from_temp, MockKeyStore, MockGuard) documented with explicit #[cfg(test)] warning"
  - "TESTING.md arch.rs single-line import limitation documented to prevent future confusion"
metrics:
  duration_minutes: 2
  completed_date: "2026-04-08"
  tasks_completed: 2
  tasks_total: 2
  files_created: 5
  files_modified: 0
---

# Phase 03 Plan 02: TESTING.md and ADRs Summary

**One-liner:** Test guide with three copyable templates (Paths::from_temp, MockKeyStore, MockGuard) plus four ADRs documenting AES-256-GCM, OS keychain, path injection, and offline-only decisions.

## Tasks Completed

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create docs/TESTING.md | 490f3d5 | docs/TESTING.md (182 lines) |
| 2 | Create 4 ADRs in docs/decisions/ | bea24e0 | 4 ADR files (37-39 lines each) |

## What Was Built

### docs/TESTING.md (182 lines)

Agent-first test guide structured as:
1. **Constraints and Invariants** — 7-row table of what must always be true in tests (first 50 lines)
2. **Test Infrastructure** — Three subsections with copyable `#[cfg(test)]` templates for `Paths::from_temp`, `MockKeyStore`, and `MockGuard`
3. **How to Add a New Test** — 5-step recipe for location choice, temp path setup, mock injection, assertions, and run commands
4. **Adding Architectural Rules** — Recipe for extending `tests/arch.rs` with new boundary rules, purity checks, and network crate deny-list entries
5. **Test Location Map** — Table mapping test types to locations and `cargo test` run commands

Notable accuracy points:
- Explicitly warns that all three abstractions are `#[cfg(test)]` gated (not available in production)
- Documents the `arch.rs` single-line import assumption limitation
- `Paths::from_temp` note: creates mapping only, not directories — callers must `create_dir_all` manually

### docs/decisions/001-aes-256-gcm.md (37 lines)

ADR for AES-256-GCM encryption decision. Documents nonce format (`[12-byte nonce][ciphertext][16-byte GCM tag]`), pure function design, tamper detection property, and 28-byte minimum ciphertext constraint. References `src/crypto/mod.rs`.

### docs/decisions/002-os-keychain.md (38 lines)

ADR for OS keychain key storage. Documents hex-encoded key format, service/account names (`"sub-swap"/"encryption-key"`), cross-platform backends, machine-binding consequence, and `KeyStore` trait enabling `MockKeyStore` in tests. References `src/crypto/keychain.rs`.

### docs/decisions/003-path-injection.md (37 lines)

ADR for path injection testability pattern. Documents `Paths` struct with `codex_dir`/`sub_swap_dir` fields, `#[cfg(test)]` gating of `from_temp`, the "create mapping not directories" constraint, and files that use the pattern. References `src/paths.rs`.

### docs/decisions/004-offline-only.md (39 lines)

ADR for offline-only constraint. Documents the 13-crate deny-list in `NETWORK_CRATES`, mechanical enforcement via `arch_03_no_network_crates_in_dependencies`, and cross-machine sharing impossibility. References `Cargo.toml` and `tests/arch.rs`.

## Verification

- `cargo test` — all 11 structural tests + 2 integration tests pass (no regressions)
- `docs/TESTING.md` — 182 lines, all 12 acceptance criteria met
- `docs/decisions/` — exactly 4 files, all under 100 lines, none reference CLAUDE.md

## Deviations from Plan

None — plan executed exactly as written.

## Known Stubs

None — all documentation is complete and self-contained.

## Threat Flags

None — documentation files only, no new executable code or attack surface.

## Self-Check: PASSED

- docs/TESTING.md: FOUND
- docs/decisions/001-aes-256-gcm.md: FOUND
- docs/decisions/002-os-keychain.md: FOUND
- docs/decisions/003-path-injection.md: FOUND
- docs/decisions/004-offline-only.md: FOUND
- Commit 490f3d5: FOUND
- Commit bea24e0: FOUND
- cargo test: 13 tests passed, 0 failed
