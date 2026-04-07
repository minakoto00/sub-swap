# Phase 2: Architectural Enforcement - Context

**Gathered:** 2026-04-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Add structural tests (`tests/arch.rs`) that enforce module boundary rules, crypto purity, and dependency constraints as deterministic `cargo test` failures. Every failure message includes agent-readable remediation. No business logic changes — test infrastructure only.

</domain>

<decisions>
## Implementation Decisions

### Module Boundary Rules
- **D-01:** Enforce a layered dependency direction. Lower layers must never import higher layers:
  - **Foundation** (`error`, `paths`): Can only import `std` and external crates. `paths` may import `error`. No other internal imports.
  - **Core** (`crypto`, `config`, `guard`): Can import foundation modules only. `crypto/mod.rs` imports `error`. `config` imports `error`, `paths`. `guard` imports `error`.
  - **Business** (`profile`): Can import core and foundation (`crypto`, `error`, `paths`, `config`). Cannot import `cli`, `tui`, or `guard`.
  - **Orchestration** (`cli`, `tui`): Can import anything — these are the top-level entry points.
- **D-02:** Structural tests parse source files (`use crate::` statements) to detect prohibited imports. No proc-macro or build-time enforcement — keep it simple and readable.

### Crypto Purity Scope
- **D-03:** ARCH-02 ("crypto/ has no filesystem I/O and no side effects") applies to `crypto/mod.rs` only — the encrypt/decrypt/generate_key pure functions. Verify it has no `std::fs`, `std::io::Write`, `std::net`, or `std::process` imports.
- **D-04:** `crypto/keychain.rs` is explicitly exempted from the purity constraint. It's the OS keychain abstraction layer — side effects are its purpose. The `KeyStore` trait boundary already isolates it for testing.

### Network Crate Enforcement
- **D-05:** ARCH-03 test parses `Cargo.toml` `[dependencies]` section and checks crate names against a deny-list of known network crates: `reqwest`, `hyper`, `tokio`, `async-std`, `surf`, `ureq`, `attohttpc`, `isahc`, `curl`, `tungstenite`, `websocket`.
- **D-06:** Also deny async runtimes (`tokio`, `async-std`, `smol`) since they imply network capability and violate the offline-only constraint.
- **D-07:** Check `[dependencies]` and `[dev-dependencies]` sections. Transitive dependency checking via `cargo metadata` is not needed for v1 — the direct deny-list is sufficient for this codebase size.

### Remediation Message Format
- **D-08:** Every structural test failure message uses a 3-part format:
  ```
  VIOLATION: [Rule name — what boundary was crossed]
  FOUND: [Specific file, line, or crate that violates]
  HOW TO FIX: [1-2 actionable steps an agent can follow without additional context]
  ```
- **D-09:** Messages must name specific files and modules, not abstract descriptions. An agent should be able to act on the message without reading ARCHITECTURE.md.

### Claude's Discretion
- Exact deny-list contents for network crates — will use common Rust HTTP/async ecosystem crates
- Whether to use string parsing or a TOML parsing crate for Cargo.toml checks — will choose based on what's available in dev-dependencies
- Test function naming conventions in `tests/arch.rs`
- Whether to group tests by requirement (ARCH-01, ARCH-02, ARCH-03) or by concern

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Source Structure
- `src/lib.rs` — Module re-export list; defines what's available for structural tests
- `src/crypto/mod.rs` — Pure encrypt/decrypt functions; target of ARCH-02 purity check
- `src/crypto/keychain.rs` — OS keychain abstraction; exempted from purity (D-04)
- `Cargo.toml` — Dependency list; target of ARCH-03 network crate check

### Existing Tests
- `tests/integration.rs` — Existing integration test file; `tests/arch.rs` will be a peer

### Requirements
- `.planning/REQUIREMENTS.md` §Architectural Enforcement — ARCH-01, ARCH-02, ARCH-03 specifications
- `.planning/REQUIREMENTS.md` §Observability — OBSV-04 remediation message requirement

### Prior Context
- `.planning/phases/01-code-quality-foundation/01-CONTEXT.md` — Phase 1 decisions on lib.rs structure (D-03, D-04)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `src/lib.rs` — Thin `pub mod` re-exports; structural tests import `sub_swap::*` to access all modules
- `tests/integration.rs` — Establishes the `tests/` directory pattern; `arch.rs` follows the same convention

### Established Patterns
- All internal imports use `crate::` prefix — structural tests can grep for `use crate::module` patterns
- Module structure is flat at top level (8 modules) with sub-modules in `crypto/`, `profile/`, `tui/`
- Current dependency flow is clean — no existing violations of the proposed layer rules

### Integration Points
- `just validate` already runs `cargo test` — structural tests will be included automatically
- `just test` will also pick up `tests/arch.rs` without any configuration change

### Current Dependency Map (verified by codebase scan)
| Module | Imports From | Layer |
|--------|-------------|-------|
| `error` | (nothing internal) | Foundation |
| `paths` | `error` | Foundation |
| `crypto/mod.rs` | `error` | Core |
| `crypto/keychain.rs` | `error` | Core |
| `config` | `error`, `paths` | Core |
| `guard` | `error` | Core |
| `profile/mod.rs` | `error` | Business |
| `profile/store.rs` | `error`, `paths`, `profile` | Business |
| `profile/switch.rs` | `crypto`, `error`, `paths`, `profile` | Business |
| `cli` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/mod.rs` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/wizard.rs` | `config`, `crypto`, `error`, `paths`, `profile` | Orchestration |
| `tui/widgets.rs` | `profile` | Orchestration |

</code_context>

<specifics>
## Specific Ideas

No specific requirements — user deferred all decisions to Claude's judgment. Approach should favor simplicity, source-level parsing over complex tooling, and clear remediation messages.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-architectural-enforcement*
*Context gathered: 2026-04-07*
