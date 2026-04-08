# Phase 4: Agent Entry Point - Research

**Researched:** 2026-04-08
**Domain:** Documentation restructuring — CLAUDE.md navigation map, HEALTH.md quality scorecard
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Replace the detailed inline Architecture section (current lines 20-55) with a 1-2 sentence summary followed by a pointer table linking to docs/ files. Eliminates duplication with docs/ARCHITECTURE.md, docs/SECURITY.md, and docs/TESTING.md.
- **D-02:** Retain the `Build & Test Commands` section verbatim — required by DOCS-05 and essential for every agent session.
- **D-03:** Keep `Key constraints` inline as a brief checklist (4 bullet points). These are critical guardrails an agent needs BEFORE following any link — offline-only, 0600 permissions, decrypt is view-only, process guard.
- **D-04:** Use a markdown table with `File | Purpose` columns linking to every docs/ file. Clean, scannable, machine-readable format.
- **D-05:** Include a brief one-line purpose description for each linked document so an agent can decide which to read without clicking through.
- **D-06:** Grade 5 domains: crypto, profile, TUI, docs, enforcement. These cover all graded areas specified in OBSV-03.
- **D-07:** Use status emoji indicators in a markdown table: checkmark for passing, warning for partial, cross for failing. Each row has: Domain, Status indicator, one-line status description.
- **D-08:** Reflect Phase 1-3 outcomes in initial scores — lint config (Phase 1), structural tests (Phase 2), documentation (Phase 3) should all show as passing.
- **D-09:** HEALTH.md is manually updated at phase transitions. No automated script needed for v1.

### Claude's Discretion

- Exact wording of the architecture summary sentence in CLAUDE.md
- Section ordering within CLAUDE.md (as long as Build & Test Commands is prominent)
- Whether HEALTH.md includes a "last updated" timestamp or version reference
- Exact status descriptions for each domain in HEALTH.md
- Whether to add `justfile` commands to the Build & Test Commands section or keep them separate

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOCS-05 | CLAUDE.md restructured as map (<80 lines) with Build & Test Commands retained and pointers to docs/ | Current CLAUDE.md is 55 lines; restructured version will be ~45 lines with pointer table replacing 35 lines of inline architecture prose |
| OBSV-03 | HEALTH.md exists as machine-readable quality score grading each domain (crypto, profile, TUI, docs) with status indicators | All 5 domains have graded artifacts; Phase 1-3 all-green enables accurate initial scoring |
</phase_requirements>

---

## Summary

Phase 4 is a pure documentation restructuring phase — no code changes, no new build artifacts, no dependencies to install. It consists of exactly two file operations: rewriting `CLAUDE.md` and creating `HEALTH.md`. Both files already have all the content they need to point to; the work is selecting, condensing, and organizing that content correctly.

The current `CLAUDE.md` (55 lines) contains a detailed Architecture section (lines 20-55, ~35 lines) that duplicates what now lives in `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, and `docs/TESTING.md`. The restructured `CLAUDE.md` replaces that prose with a pointer table — dropping to roughly 40-45 lines, comfortably under the 80-line limit. The `Build & Test Commands` section (lines 5-18) stays verbatim. The `Key constraints` section (lines 50-55) stays inline.

`HEALTH.md` requires accurate status for all five domains based on verified Phase 1-3 outcomes. Research has confirmed: all 56 tests pass (`cargo test`), clippy is clean, but `cargo fmt --check` currently fails on three files (`src/cli.rs`, `src/tui/mod.rs`, `tests/arch.rs`) with trailing-newline and line-length issues. This means `just validate` is currently red. The planner must decide whether to fix the formatting as a Wave 0 prerequisite (recommended) or score the `enforcement` domain as partial.

**Primary recommendation:** Run `cargo fmt` as a Wave 0 step before writing HEALTH.md so all scores reflect a fully-green baseline. Then write CLAUDE.md and HEALTH.md.

---

## Standard Stack

This phase has no library dependencies. It is markdown authoring only.

### Tooling In Use (Verified Present)
[VERIFIED: codebase inspection]

| Tool | Version | Purpose |
|------|---------|---------|
| `cargo fmt` | (bundled with rustup) | Formatting gate in `just validate` |
| `cargo clippy` | (bundled with rustup) | Lint gate in `just validate` |
| `just` | present | `justfile` with `check`, `test`, `lint`, `fmt`, `validate` recipes |

### Files to Create or Modify

| File | Operation | Current State |
|------|-----------|---------------|
| `CLAUDE.md` | Rewrite (restructure) | 55 lines; architecture prose in lines 20-55 |
| `HEALTH.md` | Create (new file) | Does not exist |

---

## Architecture Patterns

### Recommended CLAUDE.md Structure

The restructured CLAUDE.md follows the "constraints-first, links-second" pattern established by Phase 3 docs — critical information before navigational information.

```
CLAUDE.md (~42 lines target)
├── H1 title + one-line project description     (3 lines)
├── ## Build & Test Commands [VERBATIM]         (14 lines — lines 5-18 of current file)
├── ## Key constraints [INLINE, KEPT]           (7 lines — lines 50-55 of current file)
├── ## Architecture                             (3 lines: 1-2 sentence summary)
└── ## Documentation                            (10 lines: pointer table, 6 rows)
```

This ordering ensures an agent starting a session sees:
1. How to build and run tests immediately (Build & Test Commands)
2. Hard stops before any implementation (Key constraints)
3. Where to go for deep dives (Architecture summary + pointer table)

### Pointer Table Pattern

The `docs/` directory now contains six linkable targets:

```markdown
| File | Purpose |
|------|---------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Module layout, layer rules, dependency graph, boundary enforcement |
| [docs/SECURITY.md](docs/SECURITY.md) | Encryption model, key management, threat model, file permissions |
| [docs/TESTING.md](docs/TESTING.md) | Test strategy, infrastructure (MockKeyStore, MockGuard, Paths::from_temp), recipes |
| [docs/decisions/001-aes-256-gcm.md](docs/decisions/001-aes-256-gcm.md) | ADR: why AES-256-GCM |
| [docs/decisions/002-os-keychain.md](docs/decisions/002-os-keychain.md) | ADR: why OS keychain |
| [docs/decisions/003-path-injection.md](docs/decisions/003-path-injection.md) | ADR: why path injection |
| [docs/decisions/004-offline-only.md](docs/decisions/004-offline-only.md) | ADR: why strictly offline |
```

An alternative is a single row linking `docs/decisions/` as a directory rather than four individual ADR rows. This saves 3 lines. The tradeoff: an agent cannot determine which ADR is relevant without opening the directory. Individual rows are more agent-friendly and stay within the 80-line budget.

### HEALTH.md Structure Pattern

HEALTH.md is a single markdown table graded against verified artifacts, plus a timestamp.

```markdown
# Health

**Updated:** YYYY-MM-DD (after Phase N)

| Domain | Status | Detail |
|--------|--------|--------|
| crypto | ✅ | AES-256-GCM encrypt/decrypt + key management; 9 unit tests passing |
| profile | ✅ | Switch lifecycle, store, path-injected tests; 25 unit tests passing |
| TUI | ⚠️ | No automated widget tests (known gap; ratatui testing deferred) |
| docs | ✅ | ARCHITECTURE.md, SECURITY.md, TESTING.md, 4 ADRs — all created Phase 3 |
| enforcement | ✅ | 11 arch tests + clippy + fmt all green; just validate passes |
```

Note: The TUI domain is the only expected partial (warning) — this is a documented known gap from the project requirements (TUI test infrastructure is explicitly out of scope per REQUIREMENTS.md).

---

## Don't Hand-Roll

This phase has no implementation code. Nothing to hand-roll.

| Problem | Don't Build | Use Instead |
|---------|-------------|-------------|
| Line counting to verify <80 | Manual counting | `wc -l CLAUDE.md` after writing |
| HEALTH.md status automation | Script to parse test output | Manual update at phase transitions (D-09) |

---

## Common Pitfalls

### Pitfall 1: Line Count Creep

**What goes wrong:** CLAUDE.md grows past 80 lines because the pointer table rows are verbose or extra sections get added.
**Why it happens:** Each docs/ link with a purpose description takes a full line; 7 links + table header/separator = 9 lines. Adding optional sections (justfile commands, contributing notes) pushes over the limit.
**How to avoid:** Commit to the structure above first, count lines before adding anything optional. Run `wc -l CLAUDE.md` before finalizing.
**Warning signs:** Draft exceeds 65 lines — no room for additions.

### Pitfall 2: Inaccurate HEALTH.md Scores

**What goes wrong:** HEALTH.md shows enforcement as fully passing when `just validate` currently fails due to `cargo fmt --check` errors in three files.
**Why it happens:** Assuming Phase 1-3 are "done" means all quality gates pass — but `cargo fmt` was not re-run after Phase 2 added `tests/arch.rs`.
**How to avoid:** Run `cargo fmt` (not `--check`) as a Wave 0 step, then verify `just validate` passes before writing HEALTH.md scores.
**Warning signs:** `just validate` exits non-zero. Confirmed: trailing-newline issue in `src/cli.rs:331`, `src/tui/mod.rs:626`; line-length issues in `tests/arch.rs`.

### Pitfall 3: Duplicating Key constraints

**What goes wrong:** The pointer table links to docs/SECURITY.md, but the Key constraints section is also kept inline — an agent might see both and be confused about which is authoritative.
**Why it happens:** The two serve different purposes that aren't labeled clearly.
**How to avoid:** Add a one-line note in the Architecture/Documentation section clarifying that Key constraints is the "stop first" summary and docs/SECURITY.md is the full specification.

### Pitfall 4: Omitting ADRs from Pointer Table

**What goes wrong:** Pointer table links to ARCHITECTURE.md, SECURITY.md, TESTING.md but treats `docs/decisions/` as a directory entry rather than navigable content.
**Why it happens:** Directory-level links are shorter but less usable — an agent cannot infer which ADR is relevant without reading the directory.
**How to avoid:** Link each ADR individually (4 rows). The line cost is 4 lines; well within budget.

---

## Current State Assessment

### CLAUDE.md Dissection
[VERIFIED: file read]

| Lines | Content | Action |
|-------|---------|--------|
| 1-4 | Title + introductory sentence | Keep; minimal |
| 5-18 | `## Build & Test Commands` with code block | Keep verbatim (D-02) |
| 20-48 | `## Architecture` prose (CLI, TUI, wizard; testability patterns; encryption flow; switch lifecycle; input validation) | Replace with 1-2 sentence summary + pointer table |
| 50-55 | `## Key constraints` (4 bullets) | Keep verbatim (D-03) |

Net change: remove ~28 lines of inline architecture prose, add ~12 lines for summary + pointer table = approximately 42-line final file.

### Phase 1-3 Outcomes
[VERIFIED: cargo test run 2026-04-08]

| Domain | Artifact | Status | Evidence |
|--------|----------|--------|----------|
| crypto | `src/crypto/mod.rs`, `keychain.rs` | PASSING | 9 unit tests, all green |
| profile | `src/profile/` | PASSING | 25 unit tests (approx), all green; integration tests pass |
| TUI | `src/tui/` | PARTIAL | No automated widget tests; 0 TUI-specific tests exist |
| docs | `docs/` directory | PASSING | 3 core docs + 4 ADRs all created in Phase 3 |
| enforcement | `tests/arch.rs` + lint config | PARTIAL | 11 arch tests pass, clippy clean, but `cargo fmt --check` fails on 3 files |

`just validate` currently exits with code 1 (fmt check failure). Fix is `cargo fmt` — one command, no code logic changes.

---

## Code Examples

### Verified Pointer Table Format
[ASSUMED — standard markdown; no external verification needed]

```markdown
## Documentation

sub-swap's architecture and design decisions are captured in docs/. Start here for any
non-trivial investigation:

| File | Purpose |
|------|---------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Module layout, layer rules, dependency graph, enforcement tests |
| [docs/SECURITY.md](docs/SECURITY.md) | Encryption model, key management, threat model, file permissions |
| [docs/TESTING.md](docs/TESTING.md) | Test infrastructure, mocks, recipes for adding tests |
| [docs/decisions/001-aes-256-gcm.md](docs/decisions/001-aes-256-gcm.md) | ADR: AES-256-GCM chosen for authenticated encryption |
| [docs/decisions/002-os-keychain.md](docs/decisions/002-os-keychain.md) | ADR: OS keychain for encryption key storage |
| [docs/decisions/003-path-injection.md](docs/decisions/003-path-injection.md) | ADR: path injection for test isolation |
| [docs/decisions/004-offline-only.md](docs/decisions/004-offline-only.md) | ADR: strictly offline constraint |
```

### Verified HEALTH.md Format
[VERIFIED: REQUIREMENTS.md OBSV-03; domain list from CONTEXT.md D-06, D-07]

```markdown
# Health

**Updated:** 2026-04-08 (after Phase 4)

| Domain | Status | Detail |
|--------|--------|--------|
| crypto | ✅ | AES-256-GCM, key management via OS keychain; 9 unit tests passing |
| profile | ✅ | Switch lifecycle, store, path-injected isolation; all unit + integration tests passing |
| TUI | ⚠️ | No automated widget tests — ratatui testing infrastructure deferred (out of scope) |
| docs | ✅ | ARCHITECTURE.md, SECURITY.md, TESTING.md, 4 ADRs — created Phase 3 |
| enforcement | ✅ | 11 arch tests, clippy clean, fmt clean; `just validate` passes |
```

---

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | cargo test (built-in) |
| Config file | none (uses Cargo.toml) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |
| Validate command | `just validate` |

### Phase Requirements — Test Map

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| DOCS-05 | CLAUDE.md under 80 lines | manual | `wc -l CLAUDE.md` | Not automated |
| DOCS-05 | Build & Test Commands section present verbatim | manual | Visual inspection | Not automated |
| DOCS-05 | Pointer table links all docs/ files | manual | Visual inspection | Not automated |
| OBSV-03 | HEALTH.md exists with domain status indicators | manual | `test -f HEALTH.md` | Not automated |
| OBSV-03 | All 5 domains graded | manual | Visual inspection | Not automated |

This phase has no automated test coverage — it is documentation-only. Verification is manual inspection of file contents and line counts.

### Wave 0 Gaps

- [ ] Run `cargo fmt` to fix formatting in `src/cli.rs`, `src/tui/mod.rs`, `tests/arch.rs` — prerequisite for enforcement domain to score passing in HEALTH.md, and for `just validate` to pass

---

## Environment Availability

| Dependency | Required By | Available | Notes |
|------------|------------|-----------|-------|
| `cargo fmt` | Wave 0 fix + validation | Yes | Part of standard Rust toolchain |
| `just` | validate command | Yes | justfile present and working |
| `wc` | line count verification | Yes | Standard Unix utility |

No missing dependencies.

---

## Security Domain

This phase makes no code changes, introduces no new inputs or outputs, and does not affect the attack surface. The only security-relevant consideration is that CLAUDE.md links to docs/SECURITY.md — confirming the threat model is accessible rather than hidden.

No ASVS categories apply to a documentation restructuring phase.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | Individual ADR rows are more agent-friendly than a single `docs/decisions/` directory row | Architecture Patterns | Low — either format satisfies DOCS-05; individual rows are the safer choice and stay within line budget |
| A2 | TUI test coverage gap warrants ⚠️ (warning) not ❌ (fail) in HEALTH.md | Common Pitfalls / Code Examples | Low — REQUIREMENTS.md explicitly excludes TUI test infrastructure as out of scope |
| A3 | `cargo fmt` (one command) fully resolves the current fmt check failures | Common Pitfalls | Low — confirmed failures are formatting style only (trailing newlines, line wrapping), not logic |

---

## Open Questions

1. **Should `just validate` commands appear in the Build & Test Commands section?**
   - What we know: The `justfile` exists with `check`, `test`, `lint`, `fmt`, `validate` recipes. The current `Build & Test Commands` section has cargo commands only.
   - What's unclear: D-09 (Claude's discretion) leaves this open.
   - Recommendation: Add 2-3 just commands as a second code block after the cargo block. Costs 5 lines but makes `just validate` discoverable from the entry point. Final call is Claude's discretion per CONTEXT.md.

2. **Should HEALTH.md include a version or phase reference in the "Updated" field?**
   - What we know: D-09 (Claude's discretion) — "whether to include a timestamp or version reference."
   - What's unclear: Pure date vs. "date (after Phase N)" vs. both.
   - Recommendation: Include both date and phase reference: `2026-04-08 (after Phase 4)`. Phase reference makes it immediately clear which work is reflected.

---

## Sources

### Primary (HIGH confidence)
- `CLAUDE.md` (current, 55 lines) — verified by file read; dissected line-by-line
- `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/TESTING.md` — verified by file read; all exist with full content
- `docs/decisions/` — verified by directory listing; 4 ADRs confirmed present
- `cargo test` output — verified by running 2026-04-08; all 56 tests pass
- `cargo fmt --check` output — verified by running 2026-04-08; 3 files have formatting violations
- `cargo clippy -- -D warnings` — verified by running 2026-04-08; no warnings
- `.planning/REQUIREMENTS.md` — DOCS-05 and OBSV-03 specifications verified
- `.planning/phases/04-agent-entry-point/04-CONTEXT.md` — all decisions D-01 through D-09 read

### Secondary (MEDIUM confidence)
None — all claims derive from direct codebase inspection.

### Tertiary (LOW confidence)
None — no web research was needed for this documentation phase.

---

## Metadata

**Confidence breakdown:**
- File structure and line counts: HIGH — verified by direct inspection
- Phase 1-3 outcomes: HIGH — verified by running test suite
- Formatting issues: HIGH — verified by running `cargo fmt --check`
- Pointer table format: HIGH — standard markdown, all target files confirmed present
- HEALTH.md scores: HIGH — derived from verified test outcomes and explicit requirements scope

**Research date:** 2026-04-08
**Valid until:** Indefinite — this is a static documentation phase with no external dependencies
