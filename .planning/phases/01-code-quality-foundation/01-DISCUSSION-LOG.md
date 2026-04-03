# Phase 1: Code Quality Foundation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 01-code-quality-foundation
**Areas discussed:** Clippy pedantic allows, lib.rs re-export scope, Justfile extras, Format commit strategy

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Clippy pedantic allows | Which pedantic lints to suppress — too strict fights the linter, too loose makes pedantic meaningless | |
| lib.rs re-export scope | Which modules get pub re-exports — everything, or only what Phase 2 structural tests need | |
| Justfile extras | Should there be additional convenience commands beyond the required set | |
| Format commit strategy | One standalone formatting commit first, or fold the reformat into the config commit | |

**User's choice:** "You decide for me, I know nothing about rust, so don't get too technical"
**Notes:** User deferred all four areas to Claude's discretion. All decisions made as reasonable defaults favoring simplicity.

---

## Clippy Pedantic Allows

**User's choice:** Claude's discretion
**Notes:** Selected a conservative allow-list of 6 commonly noisy pedantic lints. Doc-related lints deferred since documentation is Phase 3's scope.

## lib.rs Re-export Scope

**User's choice:** Claude's discretion
**Notes:** Re-export all modules — simplest approach, gives Phase 2 full access for structural tests.

## Justfile Command Design

**User's choice:** Claude's discretion
**Notes:** Exactly the required commands only (check, test, lint, fmt, validate). No extras.

## Format Commit Strategy

**User's choice:** Claude's discretion
**Notes:** Two-commit approach — config first, then formatting application. Keeps git blame clean.

---

## Claude's Discretion

All four gray areas were deferred to Claude. Decisions documented in CONTEXT.md.

## Deferred Ideas

None — discussion stayed within phase scope.
