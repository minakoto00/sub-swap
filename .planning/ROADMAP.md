# Roadmap: sub-swap — v1.0 Harness Engineering Alignment

## Overview

Starting from a functional, well-structured Rust CLI/TUI (8.1/10 agent legibility), this milestone
adds the mechanical enforcement layer that makes the codebase safe for AI agents to operate in at
high velocity. Phase 1 establishes the lint and formatting baseline alongside the developer
ergonomics tooling. Phase 2 uses that baseline to add structural tests that make architectural
violations into `cargo test` failures. Phase 3 builds the docs/ knowledge base that agents will
use for architectural context. Phase 4 restructures CLAUDE.md as a map pointing to all of it,
and adds HEALTH.md to grade ongoing quality. Each phase is a complete, verifiable step; none
touches existing business logic.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [ ] **Phase 1: Code Quality Foundation** - Establish lint/format baseline and developer ergonomics tooling (lib.rs, justfile, rustfmt/clippy/Cargo lints)
- [ ] **Phase 2: Architectural Enforcement** - Add structural tests that enforce module boundary rules as `cargo test` failures with agent-readable remediation
- [ ] **Phase 3: Documentation Knowledge Base** - Create docs/ with ARCHITECTURE.md, SECURITY.md, TESTING.md, and design decision ADRs
- [ ] **Phase 4: Agent Entry Point** - Restructure CLAUDE.md as a map under 80 lines and add HEALTH.md quality score grading all domains

## Phase Details

### Phase 1: Code Quality Foundation
**Goal**: The codebase has a zero-violation mechanical quality baseline — formatting, lints, and ergonomics tooling are all configured, passing, and usable from a single command
**Depends on**: Nothing (first phase)
**Requirements**: QUAL-01, QUAL-02, QUAL-03, QUAL-04, OBSV-01, OBSV-02
**Success Criteria** (what must be TRUE):
  1. `cargo fmt --check` passes with no diff after `rustfmt.toml` is committed
  2. `cargo clippy -- -D warnings` passes on the existing codebase with the new Cargo.toml `[lints]` table active
  3. `cargo build --lib` succeeds after `src/lib.rs` is added alongside `src/main.rs`
  4. `just validate` runs fmt-check, clippy, and test in sequence and stops on first failure with clear output
  5. `just check`, `just test`, `just lint`, and `just fmt` each work as standalone commands
**Plans:** 2 plans

Plans:
- [x] 01-01-PLAN.md — Fix clippy violations and create lint/format config files (rustfmt.toml, clippy.toml, Cargo.toml [lints])
- [x] 01-02-PLAN.md — Create lib.rs library target, justfile command runner, and apply cargo fmt

### Phase 2: Architectural Enforcement
**Goal**: Module boundary violations surface as deterministic `cargo test` failures with messages that tell an agent exactly what to change and why
**Depends on**: Phase 1
**Requirements**: ARCH-01, ARCH-02, ARCH-03, OBSV-04
**Success Criteria** (what must be TRUE):
  1. `cargo test` includes `tests/arch.rs` and all structural tests pass on the current codebase
  2. Introducing a prohibited import in `crypto/` (e.g., importing from `profile/`) causes a specific `tests/arch.rs` test to fail
  3. Adding a network crate to `Cargo.toml` causes a structural test to fail with a message naming the forbidden crate
  4. Every structural test failure message includes a "HOW TO FIX" section an agent can act on without reading additional context
**Plans:** 1 plan

Plans:
- [x] 02-01-PLAN.md — Create tests/arch.rs with structural tests for layer boundaries, crypto purity, and network-free constraint

### Phase 3: Documentation Knowledge Base
**Goal**: The docs/ directory contains authoritative, agent-legible documentation for architecture, security, testing, and key design decisions
**Depends on**: Phase 1
**Requirements**: DOCS-01, DOCS-02, DOCS-03, DOCS-04
**Success Criteria** (what must be TRUE):
  1. `docs/ARCHITECTURE.md` describes the module layout, dependency graph, and layer boundaries in a form an agent can use before touching code
  2. `docs/SECURITY.md` documents the encryption model, key management, threat model, and the 0600 file-permission constraint with rationale
  3. `docs/TESTING.md` documents the Paths injection pattern, MockKeyStore, and MockGuard patterns with enough detail for an agent to add a new test
  4. `docs/decisions/` contains ADRs for the four settled choices: AES-256-GCM, OS keychain, path injection, offline-only constraint
**Plans:** 2 plans

Plans:
- [x] 03-01-PLAN.md — Create docs/ARCHITECTURE.md and docs/SECURITY.md with agent-first structure
- [x] 03-02-PLAN.md — Create docs/TESTING.md with copyable templates and 4 ADRs in docs/decisions/

### Phase 4: Agent Entry Point
**Goal**: CLAUDE.md is a concise map that points to all docs/, and HEALTH.md gives a machine-readable quality score across all graded domains
**Depends on**: Phase 3
**Requirements**: DOCS-05, OBSV-03
**Success Criteria** (what must be TRUE):
  1. CLAUDE.md is under 80 lines, retains the `Build & Test Commands` section verbatim, and contains a pointer table linking to every docs/ file
  2. A new agent session can find any architectural constraint by following a CLAUDE.md link — no scrolling through implementation details required
  3. HEALTH.md exists with status indicators for each domain (crypto, profile, TUI, docs, enforcement) and is updated to reflect Phase 1-3 outcomes
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 → 2 → 3 → 4

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Code Quality Foundation | 0/2 | Planning complete | - |
| 2. Architectural Enforcement | 0/1 | Planning complete | - |
| 3. Documentation Knowledge Base | 0/2 | Planning complete | - |
| 4. Agent Entry Point | 0/TBD | Not started | - |
