---
phase: 02-architectural-enforcement
plan: "01"
subsystem: test-infrastructure
tags: [architecture, enforcement, structural-tests, cargo-test]
dependency_graph:
  requires: []
  provides: [arch-tests, layer-boundary-enforcement, crypto-purity-enforcement, network-free-enforcement]
  affects: [tests/arch.rs]
tech_stack:
  added: []
  patterns: [source-file-parsing-at-test-runtime, three-part-remediation-messages]
key_files:
  created:
    - tests/arch.rs
  modified: []
decisions:
  - "Use toml::Table (not toml::Value) for Cargo.toml parsing — toml v1.1.2 only implements FromStr for Table, not Value"
  - "ARCH-02 scoped to crypto/mod.rs only — crypto/keychain.rs explicitly exempted per D-04 (side effects are its purpose)"
  - "assert_no_crate_import uses prefix matching (use crate::{forbidden}:: and use crate::{forbidden};) to avoid false positives from substring matches"
metrics:
  duration: "~8 minutes"
  completed: "2026-04-08T03:34:06Z"
  tasks_completed: 2
  tasks_total: 2
  files_created: 1
  files_modified: 0
requirements_addressed: [ARCH-01, ARCH-02, ARCH-03, OBSV-04]
---

# Phase 2 Plan 1: Architectural Boundary Enforcement Tests Summary

Structural tests in `tests/arch.rs` enforce module layer boundaries, crypto purity, and the offline-only dependency constraint as deterministic `cargo test` failures with agent-readable three-part remediation messages.

## What Was Built

`tests/arch.rs` — 232 lines, 11 test functions covering four requirements:

- **9 ARCH-01 tests** — Layer boundary enforcement for Foundation/Core/Business modules. Each reads a source file at test-runtime and calls `assert_no_crate_import` with a forbidden module list derived from the layer rule table.
- **1 ARCH-02 test** — Crypto purity: `crypto/mod.rs` must not use `std::fs`, `std::io::Write`, `std::net`, or `std::process`. `crypto/keychain.rs` is explicitly exempted.
- **1 ARCH-03 test** — Network-free constraint: no crate from the 13-entry deny-list (`reqwest`, `hyper`, `surf`, `ureq`, `attohttpc`, `isahc`, `curl`, `tungstenite`, `websocket`, `tokio`, `async-std`, `async_std`, `smol`) in `[dependencies]` or `[dev-dependencies]`.

Helper functions: `read_source`, `assert_no_crate_import`, `dep_names`.

## Verification

| Check | Result |
|-------|--------|
| `cargo test --test arch arch_01` | 9 passed |
| `cargo test --test arch` | 11 passed |
| `cargo test` (full suite) | 56 passed (43 unit + 11 arch + 2 integration) |
| Arch test runtime | <1 second |
| VIOLATION/FOUND/HOW TO FIX in every panic | Confirmed |
| min_lines >= 150 | 232 lines |
| No new Cargo.toml dependencies | Confirmed (toml already in [dependencies]) |

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| Task 1 | `0b04cda` | Add arch_01 layer boundary tests |
| Task 2 | `73a68fb` | Add arch_02 crypto purity and arch_03 network prohibition tests |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed toml::Value parse incompatibility**
- **Found during:** Task 2 verification
- **Issue:** `content.parse::<toml::Value>()` panics with "unexpected content, expected nothing" in toml v1.1.2 — `FromStr` is only implemented for `toml::Table`, not `toml::Value`
- **Fix:** Changed `toml::Value` to `toml::Table` in both `dep_names` signature and `arch_03` test body
- **Files modified:** tests/arch.rs
- **Commit:** 73a68fb

## Known Stubs

None — this plan adds test infrastructure only. No UI rendering, no data stubs.

## Threat Flags

None — this plan adds test infrastructure only (no new production endpoints, auth paths, file access patterns, or schema changes).

## Self-Check: PASSED

- `tests/arch.rs` exists: FOUND
- Commit `0b04cda` exists: FOUND
- Commit `73a68fb` exists: FOUND
- 11 tests pass: CONFIRMED
- Full suite (56 tests) passes: CONFIRMED
