# Phase 4: Agent Entry Point - Context

**Gathered:** 2026-04-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Restructure CLAUDE.md as a concise navigation map (under 80 lines) pointing to all docs/ files, and create HEALTH.md as a machine-readable quality score grading all domains. No code changes — documentation restructuring only.

</domain>

<decisions>
## Implementation Decisions

### CLAUDE.md Content Strategy
- **D-01:** Replace the detailed inline Architecture section (current lines 20-55) with a 1-2 sentence summary followed by a pointer table linking to docs/ files. This eliminates duplication with docs/ARCHITECTURE.md, docs/SECURITY.md, and docs/TESTING.md created in Phase 3.
- **D-02:** Retain the `Build & Test Commands` section verbatim — required by DOCS-05 and essential for every agent session.
- **D-03:** Keep `Key constraints` inline as a brief checklist (4 bullet points). These are critical guardrails an agent needs BEFORE following any link — offline-only, 0600 permissions, decrypt is view-only, process guard. Negligible line count.

### Pointer Table Format
- **D-04:** Use a markdown table with `File | Purpose` columns linking to every docs/ file (ARCHITECTURE.md, SECURITY.md, TESTING.md, decisions/). Clean, scannable, machine-readable format.
- **D-05:** Include a brief one-line purpose description for each linked document so an agent can decide which to read without clicking through.

### HEALTH.md Domains & Scoring
- **D-06:** Grade 5 domains: crypto, profile, TUI, docs, enforcement. These cover all the graded areas specified in OBSV-03.
- **D-07:** Use status emoji indicators in a markdown table: checkmark for passing, warning for partial, cross for failing. Each row has: Domain, Status indicator, one-line status description.
- **D-08:** Reflect Phase 1-3 outcomes in initial scores — lint config (Phase 1), structural tests (Phase 2), documentation (Phase 3) should all show as passing since those phases are complete.

### HEALTH.md Update Mechanism
- **D-09:** HEALTH.md is manually updated at phase transitions. No automated script needed for v1 — the file is small enough to update by hand, and automation would add complexity without proportional value at this codebase size.

### Claude's Discretion
- Exact wording of the architecture summary sentence in CLAUDE.md
- Section ordering within CLAUDE.md (as long as Build & Test Commands is prominent)
- Whether HEALTH.md includes a "last updated" timestamp or version reference
- Exact status descriptions for each domain in HEALTH.md
- Whether to add `justfile` commands to the Build & Test Commands section or keep them separate

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Current CLAUDE.md
- `CLAUDE.md` — Current 55-line file to be restructured; Build & Test Commands section (lines 7-18) retained verbatim

### Docs to Link
- `docs/ARCHITECTURE.md` — Module layout, dependency graph, layer boundaries (Phase 3 output)
- `docs/SECURITY.md` — Encryption model, key management, threat model, file permissions (Phase 3 output)
- `docs/TESTING.md` — Test strategy, patterns, how to add tests (Phase 3 output)
- `docs/decisions/` — 4 ADRs: AES-256-GCM, OS keychain, path injection, offline-only (Phase 3 output)

### Requirements
- `.planning/REQUIREMENTS.md` §Documentation — DOCS-05 specification (CLAUDE.md as map, <80 lines)
- `.planning/REQUIREMENTS.md` §Observability — OBSV-03 specification (HEALTH.md quality score)

### Prior Phase Context
- `.planning/phases/03-documentation-knowledge-base/03-CONTEXT.md` — D-13: each docs/ file is self-contained; Phase 4 restructures CLAUDE.md to point to docs/ and remove duplication

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `CLAUDE.md` — Current file to restructure; 55 lines, already under the 80-line target
- `docs/ARCHITECTURE.md` — Exists; CLAUDE.md pointer table will link here
- `docs/SECURITY.md` — Exists; CLAUDE.md pointer table will link here
- `docs/TESTING.md` — Exists; CLAUDE.md pointer table will link here
- `docs/decisions/` — Exists with 4 ADRs; CLAUDE.md pointer table will link here

### Established Patterns
- Agent-first documentation style from Phase 3 (D-01): lead with constraints/decisions in first 50 lines
- Structured sections with headers and tables (Phase 3 pattern)
- CLAUDE.md already uses clear section headers and code blocks

### Integration Points
- `justfile` — Created in Phase 1; CLAUDE.md may reference `just validate` as the primary quality check
- `tests/arch.rs` — Created in Phase 2; HEALTH.md enforcement domain grades this
- All docs/ files — Phase 3 outputs that CLAUDE.md will point to

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. User has consistently deferred organizational and structural decisions to Claude's judgment across all prior phases.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-agent-entry-point*
*Context gathered: 2026-04-08*
