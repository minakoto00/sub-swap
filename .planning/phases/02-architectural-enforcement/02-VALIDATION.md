---
phase: 2
slug: architectural-enforcement
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml (existing) |
| **Quick run command** | `cargo test --test arch` |
| **Full suite command** | `cargo test` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --test arch`
- **After every plan wave:** Run `cargo test`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | ARCH-01 | — | N/A | integration | `cargo test --test arch test_module_boundary` | ❌ W0 | ⬜ pending |
| 02-01-02 | 01 | 1 | ARCH-02 | — | N/A | integration | `cargo test --test arch test_crypto_purity` | ❌ W0 | ⬜ pending |
| 02-01-03 | 01 | 1 | ARCH-03 | — | N/A | integration | `cargo test --test arch test_no_network_crates` | ❌ W0 | ⬜ pending |
| 02-01-04 | 01 | 1 | OBSV-04 | — | N/A | integration | `cargo test --test arch` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `tests/arch.rs` — structural test file with all ARCH-01, ARCH-02, ARCH-03, OBSV-04 tests

*Existing test infrastructure (cargo test) covers framework needs. Only the test file itself is new.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| HOW TO FIX messages are actionable | OBSV-04 | Readability is subjective | Review each failure message for clarity and completeness |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
