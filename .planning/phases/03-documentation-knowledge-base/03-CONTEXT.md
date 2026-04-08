# Phase 3: Documentation Knowledge Base - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Create the `docs/` knowledge base with ARCHITECTURE.md, SECURITY.md, TESTING.md, and design decision ADRs. Each document must be authoritative, agent-legible, and self-contained. No code changes — documentation only.

</domain>

<decisions>
## Implementation Decisions

### Documentation Depth & Style
- **D-01:** Agent-first documentation — each file leads with constraints, decisions, and boundaries in the first 50 lines. Implementation narrative follows for human readers. An agent should be able to understand what NOT to do before reading implementation details.
- **D-02:** Use structured sections with clear headers, not prose paragraphs. Tables for quick-reference data (module boundaries, file permissions, dependency rules). Prose only where rationale needs explaining.

### ADR Scope & Format
- **D-03:** Create exactly 4 ADRs as required by DOCS-04: AES-256-GCM encryption, OS keychain for key storage, path injection for testability, offline-only constraint.
- **D-04:** Use standard ADR format: Title, Status, Context, Decision, Consequences. Each ADR should be concise (under 100 lines) and reference the specific source files that implement the decision.
- **D-05:** ADR directory structure: `docs/decisions/` with files named `001-aes-256-gcm.md`, `002-os-keychain.md`, `003-path-injection.md`, `004-offline-only.md`.

### ARCHITECTURE.md Content
- **D-06:** Must include: module layout diagram (text-based), dependency graph matching the layer rules from Phase 2 (Foundation → Core → Business → Orchestration), and the specific boundary rules now enforced by `tests/arch.rs`.
- **D-07:** Reference the verified dependency map from Phase 2 context (02-CONTEXT.md §code_context) as the source of truth for the current module import relationships.

### SECURITY.md Content
- **D-08:** Must cover: encryption model (AES-256-GCM with nonce|ciphertext|tag format), key management (256-bit key via CSPRNG, hex-encoded in OS keychain), threat model (what's protected, what's not), and the 0600 file permission constraint with rationale.
- **D-09:** Include the profile switch lifecycle as a security-relevant workflow: decrypt target → read active → encrypt old → write target → update index. Note the atomic swap property.

### TESTING.md Content
- **D-10:** Template-based approach — show copyable code patterns for each testing abstraction: `Paths::from_temp(tempdir)`, `MockKeyStore`, `MockGuard`. Each pattern includes a minimal working example an agent can adapt.
- **D-11:** Include a "How to Add a New Test" recipe with step-by-step instructions covering: choosing test location (unit vs integration), setting up temp paths, mocking external deps, and asserting outcomes.
- **D-12:** Document the structural test approach (`tests/arch.rs`) and how to add new architectural boundary rules.

### Docs Relationship to CLAUDE.md
- **D-13:** Each docs/ file is self-contained — no dependency on reading CLAUDE.md first. Phase 4 will restructure CLAUDE.md to point to docs/ and remove duplication. Until then, some overlap with CLAUDE.md is acceptable and expected.

### Claude's Discretion
- Exact wording and section ordering within each document
- Whether to include Mermaid diagrams or ASCII art for architecture visualization
- Level of detail in threat model (pragmatic scope appropriate for a local-only CLI tool)
- Whether ADRs include "Alternatives Considered" sections or keep to the minimal format

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source Structure (to document)
- `src/lib.rs` — Module re-export list defining the public API surface
- `src/crypto/mod.rs` — Pure encrypt/decrypt/generate_key functions (SECURITY.md source)
- `src/crypto/keychain.rs` — OS keychain abstraction via `KeyStore` trait (SECURITY.md source)
- `src/profile/switch.rs` — Profile switch lifecycle (SECURITY.md atomic swap documentation)
- `src/paths.rs` — `Paths` struct with `from_temp` constructor (TESTING.md source)
- `src/guard.rs` — `CodexGuard` trait with `MockGuard` (TESTING.md source)
- `src/error.rs` — `validate_profile_name()` input validation (SECURITY.md source)

### Existing Tests (to document)
- `tests/arch.rs` — Structural tests enforcing module boundaries (TESTING.md + ARCHITECTURE.md source)
- `tests/integration.rs` — Integration test patterns (TESTING.md source)

### Prior Phase Artifacts
- `.planning/phases/02-architectural-enforcement/02-CONTEXT.md` §code_context — Verified dependency map with layer assignments
- `CLAUDE.md` §Architecture — Existing architecture description to align with (not contradict)

### Design Spec
- `docs/superpowers/specs/2026-04-02-sub-swap-design.md` — Original design spec with storage layout, encryption flow, and goals/non-goals

### Requirements
- `.planning/REQUIREMENTS.md` §Documentation — DOCS-01 through DOCS-04 specifications

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `docs/superpowers/specs/2026-04-02-sub-swap-design.md` — Original design spec; source material for SECURITY.md encryption model and ARCHITECTURE.md storage layout
- `CLAUDE.md` §Architecture — Concise architecture summary; ARCHITECTURE.md expands on this
- Phase 2 dependency map (02-CONTEXT.md) — Verified module import relationships; direct input to ARCHITECTURE.md

### Established Patterns
- Module structure: 8 top-level modules with sub-modules in `crypto/`, `profile/`, `tui/`
- Layer model: Foundation (`error`, `paths`) → Core (`crypto`, `config`, `guard`) → Business (`profile`) → Orchestration (`cli`, `tui`)
- Testing pattern: `Paths::from_temp`, `MockKeyStore`, `MockGuard` for isolation

### Integration Points
- `docs/` directory already exists with `superpowers/` subdirectory — new docs go alongside it
- Phase 4 will consume these docs to restructure CLAUDE.md as a pointer map
- `tests/arch.rs` enforces boundaries documented in ARCHITECTURE.md — keep them in sync

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. User has consistently deferred organizational and structural decisions to Claude's judgment across Phase 1 and Phase 2.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-documentation-knowledge-base*
*Context gathered: 2026-04-08*
