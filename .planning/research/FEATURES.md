# Feature Research

**Domain:** Harness engineering infrastructure for a Rust CLI/TUI project (sub-swap)
**Researched:** 2026-04-02
**Confidence:** HIGH (primary sources: OpenAI harness engineering article, HumanLayer blog, official Rust tooling docs, agents.md spec)

---

## Feature Landscape

### Table Stakes (Infrastructure That Must Exist)

Features that any serious, agent-first Rust project is expected to have. Missing these = the
project does not meet the baseline OpenAI harness engineering standard.

| Feature | Why Expected | Complexity | Dependency on Existing Code |
|---------|--------------|------------|----------------------------|
| CLAUDE.md restructured as map (table of contents) | OpenAI principle: "treat AGENTS.md as a table of contents, not an encyclopedia." Current CLAUDE.md is a 56-line monolith mixing commands, architecture narrative, and constraints — agents load all of it on every session | LOW | Replaces existing `CLAUDE.md`. No code changes. |
| `docs/` knowledge base with structured files | Progressive disclosure: agents only load what they need. HumanLayer finding: root file under 60 lines, task-specific docs in separate files | LOW | New directory. References existing logic already described in CLAUDE.md. |
| GitHub Actions CI: `cargo test` | Every Rust project is expected to run tests on every push. 43 unit + 2 integration tests exist; none are gated in CI | LOW | Exercises existing test suite. No new code. |
| GitHub Actions CI: `cargo clippy -- -D warnings` | Clippy-as-errors is the Rust community standard for enforcing lint hygiene. Without it, lint regressions accumulate silently | LOW | No new code. May surface existing warnings to fix. |
| GitHub Actions CI: `cargo fmt --check` | rustfmt in CI prevents formatting drift. Standard practice across all serious Rust projects | LOW | No new code. Requires `rustfmt.toml` for project-specific config. |
| `rustfmt.toml` configuration file | Mechanical formatting enforcement. Without it, `cargo fmt` applies defaults but agents may produce inconsistent style | LOW | No new code. New config file at repo root. |
| `[lints]` table in `Cargo.toml` | Native Rust 2021 mechanism (RFC 3389, stable since 1.73) to deny `unsafe_code`, set `clippy::all = warn`. More authoritative than CI flags alone | LOW | Modifies existing `Cargo.toml`. No code changes unless lints surface violations. |
| `docs/ARCHITECTURE.md` | Agents need a stable map of module boundaries, data flow, and component responsibilities. Currently buried in CLAUDE.md narrative prose | LOW–MEDIUM | Extracts and expands existing CLAUDE.md architecture section. |
| `docs/SECURITY.md` | Security constraints (0600 permissions, offline-only, no secrets to disk) need a dedicated, authoritative doc agents can check during security-sensitive tasks | LOW | Extracts existing constraints from CLAUDE.md. Adds rationale. |
| `docs/TESTING.md` | Test patterns, what is tested, what is not, how to add tests. Currently undocumented outside CLAUDE.md command listing | LOW | No code changes. Documents existing `Paths::from_temp`, trait mock patterns. |

### Differentiators (Agent-First Value Beyond Baseline)

Features that go beyond baseline CI hygiene to actively improve agent execution quality and
encode architectural intent mechanically.

| Feature | Value Proposition | Complexity | Dependency on Existing Code |
|---------|-------------------|------------|----------------------------|
| GitHub Actions CI: `cargo audit` / `cargo deny` | Supply chain security enforcement in CI. `cargo deny` is preferred (advisory checks + license policy + duplicate detection in one pass). Offline-only constraint makes dependency surface important to audit | LOW–MEDIUM | New `deny.toml` config file. No code changes. Verifies existing 11 crate deps. |
| Structural tests for architectural boundaries | Mechanically enforces that the CLI layer (`cli.rs`) does not import from `tui/`, that crypto functions remain pure, that no new network crates enter the dep tree. Catches boundary violations before review. Tool: `arch_test` crate or hand-rolled `#[test]` assertions against `Cargo.lock` / module paths | MEDIUM | New test file. References existing module structure. Depends on `Cargo.lock` being committed (it is). |
| Design decision log (`docs/decisions/`) | ADR-style records for choices already made (AES-256-GCM, offline-only, `pub(crate)` for internals, ratatui for TUI). Agents consulting these avoid re-litigating settled questions. Low-cost to create; high value for long-running projects | LOW | No code changes. New docs directory. |
| `clippy.toml` with project-specific lint config | Allows setting MSRV, disabling false-positive lints, enabling pedantic groups selectively. More targeted than `-D warnings` alone | LOW | New config file. No code changes unless lints surface new violations. |
| Coverage measurement in CI (cargo-tarpaulin) | `--fail-under 70` in CI enforces that test deletion does not silently reduce coverage. Tarpaulin supports Linux CI natively; macOS runners require Docker or llvm-cov fallback | MEDIUM | No code changes. Measures existing 43 tests. Note: tarpaulin is Linux-only; use `cargo llvm-cov` for cross-platform. |
| Agent legibility score tracked in `docs/HEALTH.md` | Single file that captures: test count, coverage %, CI status, lint grade, last-audited date. Updated by agents or CI. Provides a quality snapshot without digging through multiple sources | LOW | No code changes. New doc file, optionally auto-updated via CI step. |

### Anti-Features (Seem Useful, Create Problems)

| Feature | Why Requested | Why Problematic | Alternative |
|---------|---------------|-----------------|-------------|
| Auto-generated CLAUDE.md from code | Tempting to keep docs current automatically | ETH Zurich study found LLM-generated AGENTS.md *hurts* agent performance while costing 20%+ more tokens. Auto-gen produces verbose, stale, unfocused content | Manually curate. Keep under 80 lines. Every sentence earns its place. |
| Exhaustive CLAUDE.md with all constraints inline | Feels thorough | Every token in CLAUDE.md loads on every session. A 500-line CLAUDE.md burns token budget on irrelevant context. Agent performance degrades with instruction count past ~150-200 | Progressive disclosure: root map points to `docs/`. Load on demand. |
| Property-based testing (proptest/quickcheck) | Good engineering practice | Out of scope for this milestone (noted in PROJECT.md). High complexity for limited harness benefit compared to structural tests | Defer. Add after core harness is stable. |
| TUI test infrastructure (ratatui widget testing) | Completeness | Complex, brittle, requires terminal emulation. Out of scope for this milestone | Defer. The CLI path has complete coverage. |
| Codecov / external coverage service | Dashboards look professional | Adds external service dependency, token/auth config, network requirement. Contradicts offline-first philosophy of the project. Coverage threshold can be enforced locally via `--fail-under` | Use `cargo tarpaulin --fail-under 70` in CI. No external service needed. |
| Workspace restructuring | Future-proofing | sub-swap is a single-crate project with no workspace. Adding workspace plumbing now adds complexity with zero benefit | Stay single-crate. Revisit if the project grows. |

---

## Feature Dependencies

```
[Restructured CLAUDE.md as map]
    └──requires──> [docs/ knowledge base exists]
                       ├──requires──> [docs/ARCHITECTURE.md]
                       ├──requires──> [docs/SECURITY.md]
                       └──requires──> [docs/TESTING.md]

[GitHub Actions CI: cargo test]
    └──enhances──> [Structural tests for architectural boundaries]
                       └──requires──> [Cargo.lock committed] (already true)

[GitHub Actions CI: cargo clippy]
    └──enhances──> [Cargo.toml [lints] table]
                       └──may surface violations in──> [existing source files]

[GitHub Actions CI: cargo deny]
    └──requires──> [deny.toml config file]

[Coverage measurement (tarpaulin)]
    └──requires──> [GitHub Actions CI: cargo test] (runs in same environment)

[docs/HEALTH.md quality score]
    └──enhanced by──> [Coverage measurement]
    └──enhanced by──> [GitHub Actions CI status]
```

### Dependency Notes

- **CLAUDE.md restructure requires docs/ to exist first:** The map file must point to real destinations. Create docs files before shrinking CLAUDE.md.
- **Cargo.toml `[lints]` may require source fixes:** Setting `clippy::all = "warn"` or `unsafe_code = "forbid"` may surface violations in existing code. These are LOW complexity fixes but must be sequenced: fix first, then enforce.
- **Structural tests require architectural understanding:** Writing accurate `arch_test` or hand-rolled assertions requires reading the existing module structure. Depends on `docs/ARCHITECTURE.md` being accurate.
- **Coverage CI step is Linux-only for tarpaulin:** macOS GitHub Actions runners require `cargo llvm-cov` as alternative. Can conditionally run per OS.

---

## MVP Definition

### Launch With (v1.0 Harness Milestone)

Minimum viable harness — what makes this repo agent-first compliant with the OpenAI standard.

- [ ] CLAUDE.md restructured as map (under 80 lines, pointers to docs/) — foundational, all other docs flow from this
- [ ] `docs/ARCHITECTURE.md` — agents need component map before touching code
- [ ] `docs/SECURITY.md` — security constraints must be discoverable, not inferred
- [ ] `docs/TESTING.md` — test patterns must be documented for agents adding tests
- [ ] GitHub Actions: `cargo test` — automated test gate on every push
- [ ] GitHub Actions: `cargo clippy -- -D warnings` — lint regressions blocked
- [ ] GitHub Actions: `cargo fmt --check` — formatting drift blocked
- [ ] `rustfmt.toml` — mechanical formatting baseline
- [ ] `Cargo.toml [lints]` table — `unsafe_code = "forbid"`, `clippy::all = "warn"` encoded in source of truth

### Add After Baseline is Stable (v1.x)

- [ ] GitHub Actions: `cargo deny` — supply chain audit. Trigger: after deny.toml is configured and tested locally.
- [ ] Structural tests for architectural boundaries — enforce CLI/TUI separation, no-network constraint mechanically. Trigger: after ARCHITECTURE.md captures layer model.
- [ ] `docs/decisions/` ADR log — Trigger: any time a non-obvious decision needs justification.
- [ ] `docs/HEALTH.md` quality score file — Trigger: after CI is running and coverage is measurable.

### Future Consideration (v2+)

- [ ] Coverage enforcement via `cargo tarpaulin --fail-under` in CI — useful but tarpaulin is Linux-only; adds CI complexity for a macOS-primary tool. Defer until cross-platform strategy is clear.
- [ ] `clippy.toml` for pedantic lint tuning — useful for mature codebase; premature for v1 harness milestone.
- [ ] Property-based testing, TUI test infrastructure — explicitly out of scope per PROJECT.md.

---

## Feature Prioritization Matrix

| Feature | Agent Value | Implementation Cost | Priority |
|---------|-------------|---------------------|----------|
| CLAUDE.md as map | HIGH — every session benefits | LOW — editing only | P1 |
| docs/ARCHITECTURE.md | HIGH — required before agents touch architecture | LOW — extract + expand | P1 |
| docs/SECURITY.md | HIGH — security constraints must be explicit | LOW — extract + expand | P1 |
| docs/TESTING.md | HIGH — test patterns required for TDD agents | LOW — new prose | P1 |
| CI: cargo test | HIGH — blocks regressions | LOW — new YAML | P1 |
| CI: cargo clippy | HIGH — blocks lint drift | LOW — new YAML | P1 |
| CI: cargo fmt | MEDIUM — style consistency | LOW — new YAML | P1 |
| Cargo.toml lints | HIGH — source-of-truth enforcement | LOW — 3-line change | P1 |
| rustfmt.toml | MEDIUM — formatting baseline | LOW — new config | P1 |
| CI: cargo deny | MEDIUM — supply chain safety | LOW–MEDIUM — new deny.toml | P2 |
| Structural tests | HIGH — mechanical boundary enforcement | MEDIUM — new test file | P2 |
| docs/decisions/ ADR log | MEDIUM — prevents re-litigating settled choices | LOW — new markdown | P2 |
| docs/HEALTH.md quality score | LOW–MEDIUM — useful dashboard | LOW — new markdown | P2 |
| Coverage CI enforcement | MEDIUM — prevents test deletion regressions | MEDIUM — Linux-only complexity | P3 |
| clippy.toml pedantic config | LOW — premature optimization | LOW | P3 |

**Priority key:**
- P1: Must have for v1.0 harness milestone
- P2: Should have, add when v1.0 is stable
- P3: Nice to have, future consideration

---

## Reference Architecture: What "Map Not Manual" Looks Like

### Before (Current CLAUDE.md — 56-line monolith)

```
CLAUDE.md
  - Build commands (10 lines)
  - Architecture narrative (25 lines)
  - Encryption flow detail (8 lines)
  - Profile switch lifecycle steps (8 lines)
  - Input validation (3 lines)
  - Key constraints (5 lines)
```

Every session loads all 56 lines. Architecture narrative is buried next to build commands.

### After (Map + docs/ — target structure)

```
CLAUDE.md                          ~60 lines — table of contents only
docs/
  ARCHITECTURE.md                  Module map, component responsibilities, data flow
  SECURITY.md                      Security constraints, rationale, verification points
  TESTING.md                       Test patterns, what to test, how to add tests
  HEALTH.md                        Quality snapshot: test count, coverage, CI status
  decisions/
    001-aes-256-gcm.md             Why AES-256-GCM, not ChaCha20
    002-offline-only.md            Why no network crates
    003-ratatui-tui.md             Why ratatui for interactive mode
```

CLAUDE.md becomes: project description + build commands + "for X, read docs/Y.md".
Agents load docs/ files on demand when the task requires that context.

---

## Sources

- [OpenAI: Harness engineering — leveraging Codex in an agent-first world](https://openai.com/index/harness-engineering/)
- [HumanLayer: Skill Issue — Harness Engineering for Coding Agents](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents)
- [HumanLayer: Writing a good CLAUDE.md](https://www.humanlayer.dev/blog/writing-a-good-claude-md)
- [AI Hero: A Complete Guide to AGENTS.md](https://www.aihero.dev/a-complete-guide-to-agents-md)
- [OpenAI Developers: Custom instructions with AGENTS.md](https://developers.openai.com/codex/guides/agents-md)
- [Clippy documentation: GitHub Actions CI integration](https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html)
- [cargo-tarpaulin: --fail-under coverage threshold](https://github.com/xd009642/tarpaulin)
- [arch_test: Rule-based architecture tests for Rust](https://github.com/tdymel/arch_test)
- [Rust RFC 3389: `[lints]` table in Cargo.toml](https://rust-lang.github.io/rfcs/3389-manifest-lint.html)
- [InfoQ: OpenAI Introduces Harness Engineering](https://www.infoq.com/news/2026/02/openai-harness-engineering-codex/)
- [EmbarkStudios/cargo-deny-action](https://github.com/marketplace/actions/cargo-deny-action)
- [Rust Auditing Tools 2025](https://markaicode.com/rust-auditing-tools-2025-automated-security-scanning-for-production-code/)

---
*Feature research for: Harness engineering infrastructure — sub-swap (Rust CLI/TUI)*
*Researched: 2026-04-02*
