---
phase: 1
slug: code-quality-foundation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-03
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (built-in Rust test framework) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && cargo clippy -- -D warnings && cargo fmt --check` |
| **Estimated runtime** | ~15 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && cargo clippy -- -D warnings && cargo fmt --check`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 15 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | QUAL-02 | config | `test -f rustfmt.toml && cargo fmt --check` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | QUAL-03 | config | `test -f clippy.toml` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | QUAL-01 | lint | `cargo clippy -- -D warnings` | ✅ | ⬜ pending |
| 01-02-01 | 02 | 1 | QUAL-04 | build | `cargo build --lib` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 1 | OBSV-01 | config | `test -f justfile && just --list` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 1 | OBSV-02 | integration | `just validate` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements — no new test files needed.
- Phase 1 verification is via tool output (cargo fmt, cargo clippy, cargo build, just commands).

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Cargo.toml [lints] table format | QUAL-01 | Config structure check | Verify `[lints.rust]` and `[lints.clippy]` sections exist with correct values |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 15s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
