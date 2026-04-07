# Phase 2: Architectural Enforcement - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-07
**Phase:** 02-architectural-enforcement
**Areas discussed:** Module boundary rules, Crypto purity scope, Network crate enforcement, Remediation message style

---

## Gray Area Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Module boundary rules | Which cross-module imports to prohibit | ✓ |
| Crypto purity scope | How to reconcile ARCH-02 with keychain.rs side effects | ✓ |
| Network crate enforcement | How to detect forbidden crates for ARCH-03 | ✓ |
| Remediation message style | Format and verbosity of HOW TO FIX sections | ✓ |

**User's choice:** "You decide for me"
**Notes:** User deferred all gray area decisions to Claude's judgment, consistent with Phase 1 pattern.

---

## Module Boundary Rules

| Option | Description | Selected |
|--------|-------------|----------|
| Layered architecture | Foundation → Core → Business → Orchestration, lower layers can't import higher | ✓ |
| Flat deny-list | Enumerate specific prohibited imports without layer abstraction | |
| Per-module allowlist | Each module declares exactly what it can import | |

**User's choice:** Claude's discretion — chose layered architecture
**Notes:** Current codebase already follows this pattern naturally. No existing violations detected.

## Crypto Purity Scope

| Option | Description | Selected |
|--------|-------------|----------|
| mod.rs only | Apply ARCH-02 to crypto/mod.rs; exempt keychain.rs | ✓ |
| Whole module | Apply to entire crypto/ directory including keychain.rs | |
| Trait boundary | Test that all side effects go through KeyStore trait | |

**User's choice:** Claude's discretion — chose mod.rs only
**Notes:** keychain.rs is an OS abstraction by design; the KeyStore trait already isolates it for testing.

## Network Crate Enforcement

| Option | Description | Selected |
|--------|-------------|----------|
| Cargo.toml deny-list | Parse Cargo.toml and check against known network crate names | ✓ |
| cargo metadata | Use cargo metadata for transitive dependency checking | |
| cargo-deny integration | Use cargo-deny tool for comprehensive policy enforcement | |

**User's choice:** Claude's discretion — chose Cargo.toml deny-list
**Notes:** Simple source-level parsing is sufficient for this codebase size. Transitive checking deferred.

## Remediation Message Style

| Option | Description | Selected |
|--------|-------------|----------|
| 3-part structured | VIOLATION / FOUND / HOW TO FIX format | ✓ |
| Prose paragraph | Descriptive error message with fix suggestion | |
| Minimal | Just the rule name and offending item | |

**User's choice:** Claude's discretion — chose 3-part structured
**Notes:** Agent-readable format per OBSV-04. Messages name specific files and modules.

---

## Claude's Discretion

All four gray areas were deferred to Claude's judgment. Decisions favor simplicity, source-level parsing, and actionable remediation messages.

## Deferred Ideas

None — discussion stayed within phase scope.
