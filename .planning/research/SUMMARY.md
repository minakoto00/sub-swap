# Project Research Summary

**Project:** sub-swap — harness engineering infrastructure
**Domain:** Rust CLI/TUI — agent-first codebase hardening (CI/CD, lint enforcement, structural tests, documentation architecture)
**Researched:** 2026-04-02
**Confidence:** HIGH

## Executive Summary

sub-swap is an existing, functional Rust CLI/TUI (2,800 lines, 43+ passing tests) that manages
encrypted Codex profiles. This milestone is not a product feature — it is harness engineering:
making the codebase safe and legible for AI agents to operate in at high velocity. The OpenAI
harness engineering standard calls for a CLAUDE.md that is a map not a manual (under 80 lines),
a `docs/` knowledge base with progressive disclosure, mechanical enforcement of architectural
rules via CI and lint configuration, and structural tests that turn dependency violations into
deterministic `cargo test` failures. All four research streams converge on the same implementation
order: documentation foundation before CI, CI before structural tests, structural tests before
CLAUDE.md restructure.

The recommended approach requires no new runtime dependencies and minimal new dev-dependencies.
GitHub Actions CI uses `dtolnay/rust-toolchain` and `Swatinem/rust-cache` (both current community
standards; the `actions-rs` org is fully unmaintained since 2023). Lint enforcement lives in
`Cargo.toml [lints]` (stable since Rust 1.74, already satisfiable by this project). Architectural
boundary tests can be written as standard integration tests in `tests/arch.rs` without adding
`arch_test` as a dependency — the codebase is small enough for hand-written source-file assertions.
The entire harness adds approximately 400 lines across configuration files, CI YAML, and test code.

The dominant risks are operational, not architectural. Two pitfalls have caused project-day-level
regressions in comparable projects: (1) `rustfmt.toml` import options (`imports_granularity`,
`group_imports`) that silently produce non-idempotent output and permanently pollute `git blame`,
and (2) `keyring` crate failures on GitHub Actions Linux runners that break CI the first time it
runs. Both have clear preventions: run `cargo fmt --check` before committing any `rustfmt.toml`,
and ensure all tests use `MockKeyStore` rather than `OsKeyStore`. A third pitfall — `cargo-audit`
blocking CI on "unmaintained" transitive advisories — is avoided by using `cargo deny` with
`unmaintained = "warn"` rather than `deny`.

## Key Findings

### Recommended Stack

No new runtime dependencies are needed. All enforcement tooling ships with the standard Rust
toolchain (`rustfmt`, `clippy`, `cargo doc`) or is available as GitHub Actions with no local
installation required. The only new tools are CI actions and optional `cargo install` targets for
local developer use.

**Core technologies:**

- `dtolnay/rust-toolchain@stable`: Install Rust toolchain in CI — current community standard, replaces unmaintained `actions-rs/toolchain`
- `Swatinem/rust-cache@v2`: Cache Cargo registry and build artifacts in CI — purpose-built for Rust, smarter than raw `actions/cache`
- `actions-rust-lang/audit@v1`: Run `cargo audit` against RustSec advisory DB — official actions-rust-lang org action, actively maintained
- `EmbarkStudios/cargo-deny-action@v2`: License + supply chain checks — broader than `cargo-audit`, uses single `deny.toml` config
- `Cargo.toml [lints]` table: Native Rust 1.74+ lint enforcement — source-of-truth for `unsafe_code = "forbid"`, `clippy::all = warn`
- `rustfmt.toml`: Formatting baseline — stable options only; `edition = "2021"`, `max_width = 100`
- `clippy.toml`: Clippy tool config — msrv only (`1.74.0`); lint levels go in Cargo.toml, not here

**What NOT to use:** `actions-rs/*` — the entire org is unmaintained since 2023. `arch_test_core`
as a dev-dependency — hand-written structural tests in `tests/arch.rs` cover all needed rules for
a 14-file codebase with zero external dependency overhead.

### Expected Features

**Must have (table stakes — P1, v1.0 harness milestone):**

- CLAUDE.md restructured as a map (under 80 lines, pointer table to `docs/`) — every agent session loads this; every extra line costs token budget
- `docs/ARCHITECTURE.md` — agents need a stable component map before touching any code
- `docs/SECURITY.md` — security constraints (0600 permissions, offline-only, no secrets to disk) must be discoverable, not inferred
- `docs/TESTING.md` — test patterns (Paths injection, MockKeyStore, MockGuard) must be documented for TDD agents
- GitHub Actions CI: `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check` — automated regression gates on every push
- `rustfmt.toml` configuration — mechanical formatting baseline
- `Cargo.toml [lints]` table — `unsafe_code = "forbid"`, `clippy::all = "warn"` encoded in version control

**Should have (P2, after v1.0 baseline is stable):**

- GitHub Actions: `cargo deny` with `deny.toml` — supply chain audit with license and duplicate detection
- Structural tests (`tests/arch.rs`) — enforce CLI/TUI separation, crypto purity, no-network constraint mechanically
- `docs/adr/` — Architecture Decision Records for offline-only, AES-256-GCM, OS keychain, path injection choices

**Defer (v2+):**

- Coverage enforcement via `cargo tarpaulin --fail-under` — tarpaulin is Linux-only; adds CI complexity for a macOS-primary tool
- `clippy::pedantic` tuning — premature for v1 harness milestone; earn it incrementally
- TUI test infrastructure, property-based testing — explicitly out of scope per PROJECT.md

### Architecture Approach

The harness adds three orthogonal layers on top of the existing, unmodified code: a Documentation
Layer (CLAUDE.md map + `docs/` files), an Enforcement Layer (`rustfmt.toml`, `clippy.toml`,
`Cargo.toml [lints]`, `tests/arch.rs`), and a CI Layer (`.github/workflows/ci.yml` +
`.github/workflows/audit.yml`). None of these layers touch the existing Business Logic,
Abstraction, or Storage layers. The build order that respects dependencies is:
`lib.rs` (unblocks arch_test) → config files (unblock CI) → `tests/arch.rs` → `docs/` →
CLAUDE.md rewrite → GitHub Actions workflows.

**Major components added by this milestone:**

1. `CLAUDE.md` (rewritten) — map file: build commands + key constraints + pointer table to `docs/`
2. `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/TESTING.md` — progressive disclosure knowledge base
3. `docs/adr/` — architecture decision records (offline-only, AES-256-GCM, OS keychain, path injection)
4. `rustfmt.toml`, `clippy.toml`, `Cargo.toml [lints]` — mechanical code quality enforcement
5. `tests/arch.rs` — structural tests asserting module dependency direction rules
6. `.github/workflows/ci.yml` — parallel jobs: fmt, check, clippy, test
7. `.github/workflows/audit.yml` — weekly security advisory scan via `cargo audit`

**Key architectural constraint:** `arch_test` requires a `src/lib.rs` entry point. The current
project is binary-only (`src/main.rs`). A thin `src/lib.rs` that re-exports existing modules must
be created; Cargo auto-detects it alongside `main.rs` without changes to `Cargo.toml`.

### Critical Pitfalls

1. **rustfmt.toml import options produce non-idempotent output** — Do not set `imports_granularity` or `group_imports`. Restrict to `edition`, `max_width`, `trailing_comma`. Always run `cargo fmt --check` before committing any `rustfmt.toml`. A mass-reformat commit, if needed, must be a standalone commit so `git blame --ignore-rev` can skip it.

2. **keyring crate fails on GitHub Actions Linux (no D-Bus daemon)** — All tests must use `MockKeyStore`, never `OsKeyStore`. Verify before adding CI: run `cargo test --lib` locally and confirm no test instantiates `OsKeyStore` outside of `#[ignore]`-tagged tests. The existing test suite already follows this pattern.

3. **cargo-audit blocks CI on "unmaintained" transitive advisories** — Use `cargo deny` with `deny.toml` that sets `vulnerability = "deny"` and `unmaintained = "warn"`. Never use `--deny warnings` or `unmaintained = "deny"` in CI; unmaintained advisories on transitive deps are common and often not actionable.

4. **Cargo.toml `[lints]` priority interaction silently disables individual lints** — When mixing group-level and individual lint settings, always set `priority = -1` on group settings so individual overrides take effect. Example: `clippy::all = { level = "warn", priority = -1 }`.

5. **CLAUDE.md restructure losing build commands** — The `Build & Test Commands` section must be preserved verbatim in the new CLAUDE.md. It is the highest-ROI section for an agent first working in the repo. Never move it to a child docs file.

## Implications for Roadmap

Based on combined research, a four-phase structure is strongly indicated by the dependency graph
and pitfall-to-phase mapping. The ordering is not arbitrary — each phase is a prerequisite for
the next.

### Phase 1: Code Quality Enforcement

**Rationale:** Mechanical enforcement (rustfmt, clippy, Cargo.toml lints) must be in place before
CI is wired up. If lints produce violations in the existing code, they must be fixed first — CI
should go green on its first run. This phase also creates `src/lib.rs`, which is a prerequisite
for structural tests in Phase 3.

**Delivers:** Zero-violation lint baseline. `cargo fmt`, `cargo clippy`, and `cargo test` all pass
cleanly with the new configuration in place.

**Addresses features:** `rustfmt.toml`, `clippy.toml`, `Cargo.toml [lints]` table, `src/lib.rs`

**Avoids pitfalls:** rustfmt import option non-idempotence (Pitfall 1), `[lints]` priority
interaction (Pitfall 5), clippy pedantic noise in crypto code (Pitfall 4)

### Phase 2: CI/CD Pipeline

**Rationale:** CI depends on Phase 1 producing a clean baseline. Wiring CI before fixing lint
violations means CI is broken from day one. After Phase 1, CI should go green immediately.

**Delivers:** Automated test gate, lint gate, and format gate on every push and PR. Weekly security
advisory scan.

**Uses stack:** `dtolnay/rust-toolchain@stable`, `Swatinem/rust-cache@v2`, `actions-rust-lang/audit@v1`

**Addresses features:** GitHub Actions `cargo test`, `cargo clippy -- -D warnings`, `cargo fmt --check`, `cargo audit`

**Avoids pitfalls:** keyring CI failures on Linux (Pitfall 2), cargo-audit unmaintained advisory
noise (Pitfall 3). Requires `cargo deny` with `deny.toml` configured before the audit step goes live.

### Phase 3: Structural Tests

**Rationale:** Structural tests require `src/lib.rs` from Phase 1. They also benefit from having
the architecture documented in Phase 4's docs, but since documentation can be written after tests
are passing (tests define the rules; docs explain them), this phase can proceed with the existing
CLAUDE.md as context.

**Delivers:** `tests/arch.rs` with assertions that `crypto/` does not import `profile/`, that
`error.rs` and `paths.rs` have no domain imports, and that no network crates appear in source.
Violations surface as `cargo test` failures with descriptive messages.

**Addresses features:** Structural tests for architectural boundaries

**Avoids pitfalls:** Over-brittle file-path assertions (use module-path assertions, not file names).
Each test must be verified to fail when the boundary is intentionally violated.

### Phase 4: Documentation and CLAUDE.md Restructure

**Rationale:** Documentation comes last because the docs must reflect the actual, enforced
architecture (Phases 1-3). Writing docs before enforcement is done risks describing a target state
that doesn't match the enforcement layer. CLAUDE.md restructure must happen after `docs/` files
exist, because the map must point to real destinations.

**Delivers:** `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/TESTING.md`, `docs/adr/` (4+
decision records), and a rewritten `CLAUDE.md` under 80 lines that maps to all of them.

**Addresses features:** CLAUDE.md as map, all docs/ files, ADR log

**Avoids pitfalls:** CLAUDE.md losing build commands (Pitfall — keep verbatim), docs/ files
describing "what" not "why" (docs must lead with constraints and decisions, not code walkthrough),
dead links in CLAUDE.md (stub files must exist before pointers are added)

### Phase Ordering Rationale

- **Enforcement before CI** (Phase 1 before Phase 2): CI should be green from its first run.
  Enabling CI on a codebase that has unresolved lint violations creates a broken-by-default state
  that erodes trust in the CI signal.
- **lib.rs early** (in Phase 1): Required by `arch_test` in Phase 3. It is also the cleanest change
  (pure re-export, zero behavior change) and is best done while the change surface is minimal.
- **Tests before docs** (Phase 3 before Phase 4): Structural tests define the architectural rules
  mechanically. Documenting those rules in `docs/ARCHITECTURE.md` after the tests pass ensures
  docs and enforcement agree.
- **CLAUDE.md last** (end of Phase 4): The map must point to documents that exist. Creating stubs
  is acceptable but the full rewrite should happen after docs/ content is substantive.

### Research Flags

Phases with well-documented patterns (skip additional research-phase):

- **Phase 1 (Code Quality):** Fully documented by official Rust tooling references. `rustfmt`,
  `clippy`, and `Cargo.toml [lints]` have authoritative docs. No open questions.
- **Phase 2 (CI/CD):** GitHub Actions YAML structure is well-established. The keyring workaround
  is documented. The `cargo deny` configuration pattern is clear.

Phases that may benefit from a brief implementation check before coding:

- **Phase 3 (Structural Tests):** The decision to use hand-written `tests/arch.rs` vs `arch_test`
  crate is settled (hand-written wins for this codebase size), but verifying that `src/lib.rs`
  auto-detection by Cargo works cleanly alongside `main.rs` should be done as the first step.
- **Phase 4 (Documentation):** No technical uncertainty. The content question — what belongs in
  each doc — is answered by the research. The risk is quality, not technical implementation.

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All recommended tools have official documentation. `dtolnay/rust-toolchain`, `Swatinem/rust-cache`, `actions-rust-lang/audit`, and Cargo.toml `[lints]` are primary-source verified. |
| Features | HIGH | OpenAI harness engineering article, HumanLayer blog, and official Rust tooling docs are primary sources. Feature prioritization is grounded in the OpenAI harness engineering standard. |
| Architecture | HIGH | All architectural patterns are verified against official Rust and GitHub Actions docs. The `src/lib.rs` alongside `src/main.rs` Cargo behavior is a known, documented pattern. |
| Pitfalls | HIGH | Each critical pitfall is linked to a confirmed upstream issue (rustfmt non-idempotence tracked at rust-lang/rustfmt#6195, keyring Linux CI at actions/runner-images#6683, Cargo.toml lints priority at rust-clippy#11237). |

**Overall confidence:** HIGH

### Gaps to Address

- **`src/lib.rs` module visibility:** When adding `src/lib.rs` alongside `main.rs`, some modules
  currently declared `pub(crate)` in `main.rs` context may need visibility adjustments for the
  library target. Verify with `cargo build` immediately after creating `lib.rs`. Expect this to be
  a minor fix if it surfaces at all.
- **OsKeyStore test audit:** Before wiring CI, run `grep -r "OsKeyStore" tests/` to confirm no
  integration test instantiates the OS keychain. The existing architecture strongly suggests all
  tests use `MockKeyStore`, but this should be verified explicitly before the CI step goes live.
- **cargo deny initial scan:** Run `cargo deny check` locally before adding the audit CI job.
  The existing 11 dependencies may already have advisories. The initial scan output determines
  whether any `ignore` entries are needed in `deny.toml` before CI is enabled.

## Sources

### Primary (HIGH confidence)

- [dtolnay/rust-toolchain GitHub](https://github.com/dtolnay/rust-toolchain) — toolchain action, version specifiers
- [Swatinem/rust-cache GitHub](https://github.com/Swatinem/rust-cache) — cache strategy and key structure
- [actions-rust-lang/audit GitHub](https://github.com/actions-rust-lang/audit) — security audit action
- [Clippy configuration docs](https://doc.rust-lang.org/clippy/configuration.html) — clippy.toml options, MSRV
- [rustfmt Configurations.md](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md) — stable option reference
- [Cargo lints RFC 3389](https://rust-lang.github.io/rfcs/3389-manifest-lint.html) — `[lints]` table syntax and priority
- [cargo-deny advisories config](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html) — unmaintained vs vulnerability
- [Clippy GitHub Actions — Official Rust Docs](https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html) — CI integration patterns

### Secondary (MEDIUM confidence)

- [HumanLayer: Writing a good CLAUDE.md](https://www.humanlayer.dev/blog/writing-a-good-claude-md) — progressive disclosure patterns, map vs manual
- [HumanLayer: Skill Issue — Harness Engineering](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents) — OpenAI harness standard summary
- [OpenAI Harness Engineering (secondary summary)](https://alexlavaee.me/blog/openai-agent-first-codebase-learnings/) — primary article behind 403; secondary summary used
- [GitHub Actions for Rust — shift.click](https://shift.click/blog/github-actions-rust/) — CI structure patterns

### Tertiary (LOW confidence — specific issue tracking)

- [rustfmt non-idempotent imports issue #6195](https://github.com/rust-lang/rustfmt/issues/6195) — imports_granularity + group_imports bug confirmation
- [Cargo.toml lints not obeyed — clippy#11237](https://github.com/rust-lang/rust-clippy/issues/11237) — priority interaction bug confirmation
- [keyring-rs Linux headless CI](https://github.com/open-source-cooperative/keyring-rs) — gnome-keyring unlock workaround pattern

---
*Research completed: 2026-04-02*
*Ready for roadmap: yes*
