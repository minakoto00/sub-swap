# Phase 3: Documentation Knowledge Base - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-08
**Phase:** 03-documentation-knowledge-base
**Areas discussed:** Documentation Depth, ADR Scope & Format, Docs Relationship to CLAUDE.md, Testing Doc Patterns
**Mode:** --auto (all decisions auto-selected)

---

## Documentation Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Agent-first with constraints leading | Docs lead with constraints/decisions in first 50 lines, implementation details follow | ✓ |
| Comprehensive narrative | Full human-readable documentation with extensive prose | |
| Minimal reference | Bare-bones reference cards with just the essentials | |

**User's choice:** Agent-first with constraints leading (auto-selected recommended default)
**Notes:** Aligns with harness engineering philosophy — agents need constraints and boundaries before implementation details. Consistent with Phase 2's clear remediation message approach.

---

## ADR Scope & Format

| Option | Description | Selected |
|--------|-------------|----------|
| Exactly 4 required ADRs in standard format | AES-256-GCM, OS keychain, path injection, offline-only. Context/Decision/Consequences format | ✓ |
| Extended ADRs with alternatives considered | Same 4 ADRs but with detailed alternatives analysis | |
| Lightweight decision records | Shorter format — just decision and rationale | |

**User's choice:** Exactly 4 required ADRs in standard format (auto-selected recommended default)
**Notes:** DOCS-04 requirement specifies exactly these 4 decisions. Standard ADR format is widely understood by agents and humans. Additional ADRs can be added in future milestones if needed.

---

## Docs Relationship to CLAUDE.md

| Option | Description | Selected |
|--------|-------------|----------|
| Self-contained docs, dedup in Phase 4 | Each doc stands alone; Phase 4 restructures CLAUDE.md to point to docs/ | ✓ |
| Minimal docs, CLAUDE.md remains primary | Docs add detail CLAUDE.md doesn't have; CLAUDE.md stays the entry point | |
| Immediate CLAUDE.md restructure | Restructure CLAUDE.md now as part of Phase 3 | |

**User's choice:** Self-contained docs, dedup in Phase 4 (auto-selected recommended default)
**Notes:** Phase 4 is specifically scoped for CLAUDE.md restructure (DOCS-05). Doing it now would violate phase boundaries. Some temporary overlap is acceptable.

---

## Testing Doc Patterns

| Option | Description | Selected |
|--------|-------------|----------|
| Template-based with copyable patterns | Show exact code patterns for Paths::from_temp, MockKeyStore, MockGuard with minimal examples | ✓ |
| Principle-based with guidelines | Explain testing philosophy and let agents figure out implementation | |
| Hybrid with both principles and templates | Principles section + template appendix | |

**User's choice:** Template-based with copyable patterns (auto-selected recommended default)
**Notes:** Agents work best with concrete, copyable patterns. A "How to Add a New Test" recipe gives step-by-step instructions. Consistent with the project's focus on agent-legibility.

---

## Claude's Discretion

- Exact wording and section ordering within each document
- Architecture visualization approach (Mermaid vs ASCII)
- Threat model depth
- ADR "Alternatives Considered" inclusion

## Deferred Ideas

None — discussion stayed within phase scope
