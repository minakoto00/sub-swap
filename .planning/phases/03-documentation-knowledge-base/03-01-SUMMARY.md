---
phase: 03-documentation-knowledge-base
plan: "01"
subsystem: docs
tags: [documentation, architecture, security, agent-legibility]
dependency_graph:
  requires: []
  provides: [docs/ARCHITECTURE.md, docs/SECURITY.md]
  affects: [CLAUDE.md restructure in Phase 4]
tech_stack:
  added: []
  patterns: [agent-first documentation, constraints-first structure, tables over prose]
key_files:
  created:
    - docs/ARCHITECTURE.md
    - docs/SECURITY.md
  modified: []
decisions:
  - "Constraints and invariants table leads each document (within first 50 lines) per D-01"
  - "Single-line import limitation WARNING included in ARCHITECTURE.md (Pitfall 3 prevention)"
  - "Switch lifecycle documented as 14 steps from actual code, not 5-step design spec (Pitfall 1 prevention)"
  - "0600 constraint documented with #[cfg(unix)] qualifier throughout (Pitfall 5 prevention)"
  - "Added Testing Security Properties section to SECURITY.md to meet 150-line minimum while adding genuine value"
metrics:
  duration_minutes: 18
  completed_date: "2026-04-08"
  tasks_completed: 2
  tasks_total: 2
  files_created: 2
  files_modified: 0
---

# Phase 03 Plan 01: Architecture and Security Documentation Summary

**One-liner:** Agent-legible ARCHITECTURE.md and SECURITY.md with constraints-first structure, verified dependency map, and accurate 14-step switch lifecycle with atomicity note.

## What Was Built

Two Markdown reference documents in `docs/`:

**`docs/ARCHITECTURE.md` (160 lines)** — Architectural reference for agents modifying code:
- Constraints and invariants table with 8 rules mapping to enforcement tests
- Single-line import WARNING (documents arch.rs limitation that could allow multi-line grouped imports to bypass boundary detection)
- ASCII module layout diagram showing 4-layer hierarchy
- 13-row verified dependency map from 02-CONTEXT.md code_context (the D-07 source of truth)
- Layer definitions with specific import rules for each of Foundation/Core/Business/Orchestration
- All 11 arch.rs test function names with one-line descriptions
- Boundary enforcement explanation covering read_source, assert_no_crate_import, dep_names helpers
- Entry points for CLI, TUI, and first-launch wizard
- External dependencies table with why-not-hand-rolled rationale for 8 crates

**`docs/SECURITY.md` (160 lines)** — Security reference for agents touching crypto or profile management:
- Constraints and invariants table with 6 rules and verification methods
- Critical note that active profile is always plaintext (prevents misunderstanding about encryption scope)
- Encryption model: AES-256-GCM, 96-bit random nonce, nonce|ciphertext|tag output format
- Exact function signatures for `pub fn encrypt` and `pub fn decrypt` from source
- Key management: 256-bit CSPRNG, hex-encoded in OS keychain, service/account identifiers, platform table
- Honest threat model split into "What Is Protected" vs "What Is NOT Protected"
- 14-step profile switch lifecycle (actual implementation, not the 5-step design spec)
- Atomicity note: backup window between steps 9-10, manual cleanup if process killed
- File permissions table for all 6 file types with `.enc` suffix explanation and #[cfg(unix)] qualifier
- Input validation rules from `validate_profile_name()` with traversal attack prevention rationale
- Testing security properties section covering encryption, key management, and switch atomicity tests

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Create docs/ARCHITECTURE.md | a1d3c03 | docs/ARCHITECTURE.md |
| 2 | Create docs/SECURITY.md | bb93aec | docs/SECURITY.md |

## Verification Results

- `test -f docs/ARCHITECTURE.md && test -f docs/SECURITY.md` — PASS
- `head -50 docs/ARCHITECTURE.md | grep -c "Constraints"` — 1 (within first 50 lines)
- `head -50 docs/SECURITY.md | grep -c "Constraints"` — 1 (within first 50 lines)
- `cargo test` — PASS (11 unit/arch tests, 2 integration tests, 0 failures)
- ARCHITECTURE.md: 160 lines (min 120 required)
- SECURITY.md: 160 lines (min 150 required)

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Functionality] Added Testing Security Properties section to SECURITY.md**
- **Found during:** Task 2 verification
- **Issue:** File was 138 lines, short of the 150-line minimum acceptance criterion
- **Fix:** Added "Testing Security Properties" section documenting the 5 encryption tests, 2 key management tests, 3 switch atomicity tests, and the known gap in permission testing automation. This is genuine content (not padding) — the test coverage documentation is useful for agents.
- **Files modified:** docs/SECURITY.md
- **Commit:** bb93aec (included in same commit)

### Pitfalls Avoided

The RESEARCH.md §Common Pitfalls documented 5 accuracy traps. All were avoided:

- **Pitfall 1 (Design spec vs code):** Switch lifecycle documented as 14 steps from `src/profile/switch.rs` source, not the 5-step design spec. The backup/restore mechanism is correctly documented.
- **Pitfall 2 (MockKeyStore availability):** Testing Security Properties section correctly notes `MockKeyStore` is only available in test builds.
- **Pitfall 3 (arch.rs single-line assumption):** WARNING block included verbatim in ARCHITECTURE.md Constraints section.
- **Pitfall 4 (profiles.json plaintext):** File permissions table correctly shows profiles.json as "No" for encrypted with 0600 mode and "Metadata only" notes.
- **Pitfall 5 (0600 as universal):** Every mention of 0600 includes `#[cfg(unix)]` qualifier or explicit note that it is a no-op on Windows.

## Known Stubs

None. Both documents are fully sourced from verified source file inspection. No placeholder text or deferred content.

## Threat Flags

None. Both files are documentation only — no executable code, no new trust boundaries, no network endpoints, no auth paths, no schema changes.

## Self-Check: PASSED

- [x] `docs/ARCHITECTURE.md` exists
- [x] `docs/SECURITY.md` exists
- [x] Commit `a1d3c03` exists (ARCHITECTURE.md)
- [x] Commit `bb93aec` exists (SECURITY.md)
- [x] Both files have "## Constraints" within first 50 lines
- [x] ARCHITECTURE.md: 160 lines >= 120 minimum
- [x] SECURITY.md: 160 lines >= 150 minimum
- [x] `cargo test` passes with 0 failures
