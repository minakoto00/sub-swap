# sub-swap

## What This Is

A Rust CLI/TUI tool for managing multiple `~/.codex/` profiles (auth.json + config.toml) with AES-256-GCM encryption at rest. Lets developers switch between Codex accounts seamlessly, keeping inactive profiles encrypted and the active profile plaintext for Codex to read.

## Core Value

Secure, frictionless switching between multiple Codex profiles — inactive credentials are always encrypted, switching is atomic, and no network access is ever required.

## Current State

Shipped v1.0 Harness Engineering Alignment. The codebase now has mechanical quality enforcement, architectural boundary tests, agent-legible documentation, and a concise CLAUDE.md navigation map. Ready for next milestone.

## Requirements

### Validated

- ✓ CLI with 8 subcommands (add, list, use, rename, delete, decrypt, config, tui) — v0.1
- ✓ TUI with 7-screen state machine and ratatui event loop — v0.1
- ✓ AES-256-GCM encryption at rest for inactive profiles — v0.1
- ✓ OS keychain integration for 256-bit key storage — v0.1
- ✓ Atomic profile switch lifecycle (decrypt target → encrypt old → update index) — v0.1
- ✓ First-launch wizard for initial profile setup — v0.1
- ✓ Process guard blocking switch when Codex is running — v0.1
- ✓ Path injection (`&Paths`) for testable filesystem operations — v0.1
- ✓ Trait abstractions (KeyStore, CodexGuard) for mockable external deps — v0.1
- ✓ Input validation rejecting path traversal in profile names — v0.1
- ✓ Unix file permissions (0600) on all sub-swap files — v0.1
- ✓ Mechanical code quality enforcement (rustfmt, clippy, Cargo lints) — v1.0
- ✓ Architectural boundary enforcement via structural tests — v1.0
- ✓ Structured docs/ knowledge base (ARCHITECTURE.md, SECURITY.md, TESTING.md) — v1.0
- ✓ Design decisions documentation (4 ADRs) — v1.0
- ✓ CLAUDE.md restructured as navigation map — v1.0
- ✓ HEALTH.md quality scorecard — v1.0

### Active

(None — next milestone not yet defined)

### Out of Scope

- GitHub Actions CI/CD — deferred to v2.0 milestone
- Remote sync or multi-machine sharing — not part of core product
- TUI test infrastructure — complex ratatui testing is a separate concern
- Property-based testing — good practice but not a harness engineering requirement
- Feature additions (new CLI commands, new TUI screens) — infrastructure milestones only

## Context

- **Codebase size:** 3,118 lines of Rust across 14 source files
- **Test coverage:** 56 tests (43 unit + 11 arch + 2 integration), all passing
- **Quality gate:** `just validate` runs fmt + clippy + all tests; currently green
- **Dependencies:** 11 external crates, all mainstream Rust ecosystem
- **Documentation:** CLAUDE.md (45 lines), docs/ with 3 reference docs + 4 ADRs, HEALTH.md

## Constraints

- **Offline-only**: No network crates in dependency tree, no async runtime
- **Security**: All files under ~/.sub-swap/ must be 0600 on Unix
- **Rust edition**: 2021 (per Cargo.toml)
- **No breaking changes**: All existing tests must continue to pass

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Brownfield milestone (improve existing, don't rewrite) | Codebase is already well-structured; needs enforcement, not restructuring | ✓ Good — all 56 tests pass, no business logic changed |
| CLAUDE.md as map, not manual | Harness engineering principle: progressive disclosure reduces context waste | ✓ Good — 45 lines with pointer table to 7 docs |
| GitHub Actions for CI | Industry standard; agents can reason about it; well-documented | Deferred to v2.0 |
| Structural tests for architecture | Mechanical enforcement > documentation-only rules | ✓ Good — 11 arch tests enforce layer boundaries |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-08 after v1.0 milestone completion*
