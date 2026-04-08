---
phase: 4
slug: agent-entry-point
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-08
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) |
| **Config file** | Cargo.toml |
| **Quick run command** | `cargo test --lib` |
| **Full suite command** | `cargo test && cargo fmt --check && cargo clippy -- -D warnings` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --lib`
- **After every plan wave:** Run `cargo test && cargo fmt --check && cargo clippy -- -D warnings`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Threat Ref | Secure Behavior | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|------------|-----------------|-----------|-------------------|-------------|--------|
| 04-01-01 | 01 | 1 | DOCS-05 | — | N/A | manual | `wc -l CLAUDE.md` (must be < 80) | ✅ | ⬜ pending |
| 04-01-02 | 01 | 1 | DOCS-05 | — | N/A | manual | `grep "Build & Test" CLAUDE.md` | ✅ | ⬜ pending |
| 04-01-03 | 01 | 1 | DOCS-05 | — | N/A | manual | `grep "docs/" CLAUDE.md` (pointer table) | ✅ | ⬜ pending |
| 04-02-01 | 02 | 1 | OBSV-03 | — | N/A | manual | `test -f HEALTH.md` | ❌ W0 | ⬜ pending |
| 04-02-02 | 02 | 1 | OBSV-03 | — | N/A | manual | `grep -c "crypto\|profile\|TUI\|docs\|enforcement" HEALTH.md` (must be 5) | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Existing infrastructure covers all phase requirements. No new test files needed — this phase creates documentation files only.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| CLAUDE.md under 80 lines | DOCS-05 | Line count check, not a test assertion | Run `wc -l CLAUDE.md` and verify < 80 |
| Build & Test Commands retained verbatim | DOCS-05 | Content comparison | Diff old and new Build & Test sections |
| Pointer table links to all docs/ files | DOCS-05 | Content verification | Verify each docs/ file appears in table |
| HEALTH.md has all 5 domains | OBSV-03 | Content verification | Grep for each domain name |
| Agent can find constraints via links | DOCS-05 | UX verification | Read CLAUDE.md and follow a link to verify |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
