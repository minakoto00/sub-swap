# sub-swap

## What This Is

A Rust CLI/TUI tool for managing multiple `~/.codex/` profiles (auth.json + config.toml) with AES-256-GCM encryption at rest. Lets developers switch between Codex accounts seamlessly, keeping inactive profiles encrypted and the active profile plaintext for Codex to read.

## Core Value

Secure, frictionless switching between multiple Codex profiles — inactive credentials are always encrypted, switching is atomic, and no network access is ever required.

## Current Milestone: v1.0 Harness Engineering Alignment

**Goal:** Align the repository's structure, CI/CD, documentation, and mechanical enforcement with OpenAI's harness engineering philosophy for agent-first development.

**Target features:**
- Restructure CLAUDE.md as a map (table of contents) with progressive disclosure into structured docs/
- Add CI/CD with GitHub Actions (test, lint, security audit)
- Mechanical code quality enforcement (rustfmt.toml, clippy.toml, Cargo.toml lints)
- Architectural boundary enforcement via structural tests
- Structured documentation (ARCHITECTURE.md, SECURITY.md, TESTING.md, design decisions)
- Quality tracking and agent-legibility improvements

## Requirements

### Validated

<!-- Shipped and confirmed valuable. Inferred from existing codebase. -->

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

### Active

<!-- Current scope: Harness engineering alignment. -->

- [x] Restructure CLAUDE.md as map with progressive disclosure — Validated in Phase 4: agent-entry-point
- [x] Structured docs/ knowledge base (ARCHITECTURE.md, SECURITY.md, TESTING.md) — Validated in Phase 3: documentation-knowledge-base
- [ ] GitHub Actions CI/CD (test, lint, security audit)
- [x] Mechanical code quality enforcement (rustfmt, clippy, Cargo lints) — Validated in Phase 1: code-quality-foundation
- [x] Architectural boundary enforcement via structural tests — Validated in Phase 2: architectural-enforcement
- [x] Design decisions documentation — Validated in Phase 3: documentation-knowledge-base

### Out of Scope

<!-- Explicit boundaries for this milestone. -->

- Feature additions (new CLI commands, new TUI screens) — this milestone is infrastructure/quality only
- Remote sync or multi-machine sharing — not part of core product
- TUI test infrastructure — complex ratatui testing is a separate concern
- Property-based testing — good practice but not a harness engineering requirement

## Context

- **Codebase size:** ~2,800 lines of Rust across 14 source files
- **Agent legibility baseline:** 8.1/10 — excellent inline docs and CLAUDE.md, but no mechanical enforcement
- **Dependencies:** 11 external crates, all mainstream Rust ecosystem
- **Existing docs:** CLAUDE.md (56 lines), design spec, implementation plan in docs/superpowers/
- **Gap analysis source:** OpenAI "Harness Engineering" article (Feb 2026) by Ryan Lopopolo

## Constraints

- **Offline-only**: No network crates in dependency tree, no async runtime
- **Security**: All files under ~/.sub-swap/ must be 0600 on Unix
- **Rust edition**: 2021 (per Cargo.toml)
- **No breaking changes**: All existing tests must continue to pass

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Brownfield milestone (improve existing, don't rewrite) | Codebase is already well-structured; needs enforcement, not restructuring | — Pending |
| CLAUDE.md as map, not manual | Harness engineering principle: progressive disclosure reduces context waste | — Pending |
| GitHub Actions for CI | Industry standard; agents can reason about it; well-documented | — Pending |
| Structural tests for architecture | Mechanical enforcement > documentation-only rules | — Pending |

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
*Last updated: 2026-04-08 after Phase 4 (agent-entry-point) completion*
