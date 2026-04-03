# Phase 1: Code Quality Foundation - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Establish a zero-violation mechanical quality baseline for the existing codebase. Configure formatting (rustfmt), linting (clippy + Cargo lints), add a library target (lib.rs), and create a justfile for developer ergonomics. No business logic changes — tooling and config only.

</domain>

<decisions>
## Implementation Decisions

### Clippy Pedantic Allow-List
- **D-01:** Enable `clippy::pedantic` as `warn` per QUAL-01, but suppress the following commonly noisy lints that don't add meaningful safety for a project this size:
  - `module_name_repetitions` — Rust modules naturally repeat parent names (e.g., `crypto::crypto_key`)
  - `must_use_candidate` — Over-annotates small utility functions
  - `missing_errors_doc` — Documentation is Phase 3's scope; don't force doc comments now
  - `missing_panics_doc` — Same as above
  - `doc_markdown` — Same as above
  - `return_self_not_must_use` — Builder patterns in clap derive don't need this
- **D-02:** Start with this allow-list, then let the first `cargo clippy` pass reveal if any additional suppression is needed for patterns already in the codebase. Fix real issues, suppress false positives.

### lib.rs Re-export Scope
- **D-03:** Re-export all top-level modules (`cli`, `config`, `crypto`, `error`, `guard`, `paths`, `profile`, `tui`) as `pub mod`. This is the simplest approach — gives Phase 2 structural tests full access to all modules for boundary checking, and `cargo doc` can document everything.
- **D-04:** Keep lib.rs thin — just `pub mod` declarations, no logic. main.rs switches to `use sub_swap::*` or explicit imports.

### Justfile Command Design
- **D-05:** Implement exactly the commands required by OBSV-01/02: `check`, `test`, `lint`, `fmt`, `validate`. No extras (no `watch`, `coverage`, `doc`) — keep it minimal and add commands when there's a real need.
- **D-06:** `just validate` runs in sequence: fmt-check → clippy → test. Stops on first failure. Use `&&` chaining for clear error propagation.

### Format Commit Strategy
- **D-07:** Two-commit approach: (1) commit the config files (`rustfmt.toml`, `clippy.toml`, Cargo.toml `[lints]` table, `lib.rs`, `justfile`), then (2) a separate commit that applies `cargo fmt` to reformat the existing codebase. This keeps the formatting diff isolated and makes git blame cleaner.

### Claude's Discretion
- Exact `clippy.toml` MSRV value and cognitive-complexity threshold — will use the current Rust stable version and a reasonable threshold (25, the clippy default)
- Specific rustfmt options beyond `edition` and `max_width` — keep minimal per QUAL-02 to avoid reformatting churn
- Whether `lib.rs` uses glob re-exports or explicit `pub mod` — will use `pub mod` for clarity

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Configuration
- `Cargo.toml` — Current dependency list and package config; needs `[lints]` table added
- `src/main.rs` — Current module declarations (lines 1-8); will change when lib.rs is added

### Requirements
- `.planning/REQUIREMENTS.md` §Code Quality — QUAL-01 through QUAL-04 exact specifications
- `.planning/REQUIREMENTS.md` §Observability — OBSV-01, OBSV-02 justfile specs

### Architecture Context
- `CLAUDE.md` §Architecture — Module layout and testability patterns that inform lib.rs design

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- None directly reusable — this phase creates new config files from scratch

### Established Patterns
- `main.rs` declares all 8 modules (`cli`, `config`, `crypto`, `error`, `guard`, `paths`, `profile`, `tui`) — lib.rs will mirror this structure
- No existing formatting config — codebase uses Rust defaults, so `rustfmt.toml` with `max_width = 100` may produce minimal diff
- No `[lints]` in Cargo.toml — enabling `clippy::pedantic` will likely surface warnings that need either fixing or allow-listing

### Integration Points
- `main.rs` module declarations become `lib.rs` re-exports; main.rs switches to importing from the library target
- Phase 2 depends on lib.rs existing to write structural tests against module boundaries

</code_context>

<specifics>
## Specific Ideas

No specific requirements — user deferred all decisions to Claude's judgment. Approach should favor simplicity and minimal config to avoid maintenance burden.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-code-quality-foundation*
*Context gathered: 2026-04-03*
