# Architecture Research

**Domain:** Harness engineering alignment for a Rust CLI/TUI project
**Researched:** 2026-04-02
**Confidence:** HIGH

## Standard Architecture

### System Overview

The existing system is well-layered. Harness engineering adds three orthogonal concerns that
wrap the existing layers without modifying them:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                     DOCUMENTATION LAYER (new)                           │
│  CLAUDE.md (map)  →  docs/ARCHITECTURE.md  →  docs/SECURITY.md         │
│                   →  docs/TESTING.md       →  docs/adr/                 │
├─────────────────────────────────────────────────────────────────────────┤
│                     ENFORCEMENT LAYER (new)                             │
│  rustfmt.toml   clippy.toml   Cargo.toml[lints]   tests/arch.rs        │
├─────────────────────────────────────────────────────────────────────────┤
│                     CI LAYER (new)                                      │
│  .github/workflows/ci.yml    .github/workflows/audit.yml               │
├─────────────────────────────────────────────────────────────────────────┤
│                     ENTRY POINTS (existing)                             │
│       main.rs ──→ cli.rs (8 subcommands via clap derive)               │
│               └─→ tui/mod.rs (7-screen state machine)                  │
│               └─→ tui/wizard.rs (first-launch stdin prompts)           │
├─────────────────────────────────────────────────────────────────────────┤
│                     BUSINESS LOGIC (existing)                           │
│  profile/switch.rs   profile/mod.rs (ProfileIndex)   config.rs         │
├─────────────────────────────────────────────────────────────────────────┤
│                     ABSTRACTION LAYER (existing)                        │
│  crypto/mod.rs (pure fns)    paths.rs (Paths struct)                   │
│  crypto/keychain.rs (KeyStore trait + OsKeyStore + MockKeyStore)        │
│  guard.rs (CodexGuard trait + OsGuard + MockGuard)                     │
├─────────────────────────────────────────────────────────────────────────┤
│                     STORAGE / OS LAYER (existing)                       │
│  profile/store.rs (file I/O, 0600 perms)    OS keychain                │
│  ~/.sub-swap/profiles/   ~/.codex/   ~/.sub-swap/profiles.json         │
└─────────────────────────────────────────────────────────────────────────┘
```

### Component Responsibilities

#### Existing (unmodified by this milestone)

| Component | Responsibility | Module |
|-----------|---------------|--------|
| cli.rs | Dispatch clap commands to business logic | entry point |
| tui/mod.rs | Event loop, key handlers, render | entry point |
| tui/widgets.rs | AppState, AppScreen, Action types | state |
| tui/wizard.rs | First-launch stdin/stdout prompts | entry point |
| profile/mod.rs | ProfileIndex, Profile types and mutations | domain model |
| profile/store.rs | Filesystem I/O for profiles, 0600 enforcement | storage |
| profile/switch.rs | Switch lifecycle, add, decrypt-to-stdout | business logic |
| crypto/mod.rs | Pure AES-256-GCM encrypt/decrypt, generate_key | crypto primitives |
| crypto/keychain.rs | KeyStore trait + OsKeyStore + MockKeyStore | OS abstraction |
| guard.rs | CodexGuard trait + OsGuard + MockGuard | OS abstraction |
| paths.rs | Paths struct with from_temp for test isolation | path injection |
| config.rs | AppConfig load/save with 0600 enforcement | configuration |
| error.rs | SubSwapError enum, validate_profile_name | error types |

#### New (added by this milestone)

| Component | Responsibility | File |
|-----------|---------------|------|
| CLAUDE.md (rewritten) | Map to docs/, build commands, key constraints | CLAUDE.md |
| docs/ARCHITECTURE.md | Full layered architecture, dependency directions | docs/ |
| docs/SECURITY.md | Encryption model, key storage, threat model | docs/ |
| docs/TESTING.md | Test strategy, mock usage, how to write new tests | docs/ |
| docs/adr/001-*.md | Architecture decision records for key choices | docs/adr/ |
| rustfmt.toml | Formatting enforcement configuration | repo root |
| clippy.toml | Clippy configuration (msrv, disallowed names) | repo root |
| Cargo.toml [lints] section | Deny unsafe_code, warn on missing_docs pattern | Cargo.toml |
| tests/arch.rs | Structural tests enforcing module dependency rules | tests/ |
| .github/workflows/ci.yml | Test + lint + format check on push/PR | .github/ |
| .github/workflows/audit.yml | cargo-audit security scan, scheduled daily | .github/ |

## Recommended Project Structure

### docs/ Directory (progressive disclosure)

```
docs/
├── ARCHITECTURE.md     # Layer diagram, module map, dependency rules
├── SECURITY.md         # Encryption model, key storage, threat surface
├── TESTING.md          # Test strategy, mock patterns, how to add tests
└── adr/                # Architecture decision records
    ├── 001-offline-only.md
    ├── 002-aes-256-gcm.md
    ├── 003-os-keychain.md
    └── 004-path-injection.md
```

Rationale for this structure: OpenAI's harness engineering guidance explicitly organizes knowledge
into discoverable layers. The agent entry point (CLAUDE.md) is a map; detailed knowledge lives
in docs/. Each doc is self-contained so agents load only what is relevant to the current task.
The adr/ subdirectory captures the "why" behind constraints that would otherwise surprise an agent.

Do NOT create a `docs/superpowers/` path for new harness docs. The existing
`docs/superpowers/plans/` and `docs/superpowers/specs/` stay as-is (they are historical). New
harness docs live directly under `docs/`.

### .github/ Directory

```
.github/
└── workflows/
    ├── ci.yml          # test + clippy + rustfmt on push and PR
    └── audit.yml       # cargo-audit on dependency changes + daily cron
```

### Config Files at Repo Root

```
rustfmt.toml            # Formatting rules
clippy.toml             # Clippy tool configuration (msrv)
```

Lint levels go in `Cargo.toml [lints]`, not in clippy.toml — this is the Rust 1.74+ standard.

### Structural Tests

```
tests/
├── integration.rs      # Existing binary smoke tests (unchanged)
└── arch.rs             # New: module dependency direction enforcement
```

`tests/arch.rs` uses the `arch_test` crate. It is compiled only for `cargo test` (standard
integration test convention), adds no runtime dependency.

### Structure Rationale

- **docs/ flat top-level:** Agents read specific files by name. Deep nesting adds friction.
  Three topic files plus an adr/ subdirectory is the right granularity for this project size.
- **adr/ subdirectory:** Decisions are stable reference material, not active context. Separating
  them prevents topic docs from becoming bloated with rationale prose.
- **rustfmt.toml at root:** Cargo fmt respects it automatically; no CI configuration needed to
  point to it.
- **[lints] in Cargo.toml:** Rust 1.74+ standard. Keeps all lint levels in one place, version
  controlled, no toolchain-specific env var needed.
- **tests/arch.rs:** Standard Rust integration test file. `cargo test --test arch` runs only
  structural tests; `cargo test` includes them automatically.

## Architectural Patterns

### Pattern 1: CLAUDE.md as Map, Not Manual

**What:** CLAUDE.md contains only: (1) build/test commands, (2) key constraints as one-liners,
(3) a table pointing to docs/ files with one-sentence descriptions. No implementation detail.

**When to use:** Always. Every session loads CLAUDE.md. Content not relevant to the current task
still burns context. Under 100 lines is the target.

**Trade-offs:** Requires docs/ to be authoritative and well-maintained. The map is only useful if
the linked documents exist. Reward: agents that start any task will immediately know where to look
for deep context rather than trying to load everything.

**Current CLAUDE.md issues to fix:**
- Architecture section repeats module-level detail available by reading source
- "Key constraints" is good — keep it
- Add a pointer table to docs/

### Pattern 2: Mechanical Enforcement Over Documentation-Only Rules

**What:** Rules that can be violated silently must be made mechanically unviolable. For this
project: (a) formatting enforced by `cargo fmt --check` in CI so code never merges misformatted;
(b) unsafe code denied in Cargo.toml `[lints]` so it never compiles; (c) module dependency
directions enforced in `tests/arch.rs` so they fail `cargo test` if violated.

**When to use:** For any rule that has a clear binary correct/incorrect outcome. "Don't use unsafe"
is binary. "Write good variable names" is not.

**Trade-offs:** Small upfront configuration cost. CI jobs add ~30-90 seconds per run. The payoff
is that agents never introduce silent violations; they get deterministic error output to correct.

### Pattern 3: Structural Tests for Module Dependency Directions

**What:** A test file (`tests/arch.rs`) that uses `arch_test` to assert that business logic
modules do not import entry points, that crypto is not imported by entry points directly, and
that store.rs is not imported by crypto/. These are compile-time-visible but not compile-time-
enforced relationships; the structural test makes them fail at `cargo test` time.

**When to use:** For the dependency directions that matter most: entry points should call
business logic, not be imported by it. The abstraction layer should not reach up to the CLI layer.

**Trade-offs:** `arch_test` adds a dev-dependency. It works via static analysis of `use`
statements. It cannot enforce runtime behavior, only which modules import which. False positives
are possible if the rule is specified too broadly.

**Target rules for this project:**

```
Rule 1: profile/switch.rs and profile/mod.rs do NOT import cli.rs or tui/
Rule 2: crypto/mod.rs does NOT import profile/, config, guard, cli, or tui/
Rule 3: error.rs does NOT import any domain module (it is a pure types module)
Rule 4: paths.rs does NOT import any domain module (it is a pure data module)
```

**Example arch_test code (HIGH confidence — arch_test API confirmed):**

```rust
// tests/arch.rs
use arch_test::core::{Architecture, ModuleTree};
use std::collections::HashSet;

#[test]
fn crypto_does_not_import_profile() {
    // crypto/ is a pure primitives layer; it must not know about profiles
    let architecture = Architecture::new(
        ["crypto".to_owned(), "profile".to_owned()].into()
    )
    .with_access_rule(arch_test::core::access_rules::MayNotAccess::new(
        "crypto".to_owned(),
        ["profile".to_owned()].into(),
        true,
    ));
    let module_tree = ModuleTree::new("src/lib.rs");
    assert!(architecture.check_access_rules(&module_tree).is_ok());
}

#[test]
fn error_module_has_no_domain_imports() {
    // error.rs is a pure types module; domain modules import it, not vice versa
    let architecture = Architecture::new(
        ["error".to_owned(), "profile".to_owned()].into()
    )
    .with_access_rule(arch_test::core::access_rules::MayNotAccess::new(
        "error".to_owned(),
        ["profile".to_owned()].into(),
        true,
    ));
    let module_tree = ModuleTree::new("src/lib.rs");
    assert!(architecture.check_access_rules(&module_tree).is_ok());
}
```

Note: arch_test requires a `lib.rs` entry point. The current project uses `main.rs` only. This
requires adding a thin `src/lib.rs` that re-exports the crate modules for arch_test to analyze.
See Integration Points section below.

### Pattern 4: CI Pipeline Layered by Speed

**What:** Organize CI jobs by execution time, fastest first. Gate expensive jobs on cheaper ones
passing. For a small Rust CLI: (1) `cargo fmt --check` (~2s), (2) `cargo check` (~5s),
(3) `cargo clippy` (~10s), (4) `cargo test` (~20s). Audit runs separately on a cron schedule.

**When to use:** Always. Agents get feedback from failing CI jobs. Fastest-failing jobs minimize
the context burned before the relevant error is surfaced.

**Trade-offs:** Separate jobs have per-job overhead (~20s GitHub runner startup). For this
project size, parallel jobs are better than sequential since the overhead is dominated by runner
startup, not execution time.

## Data Flow

### Enforcement Pipeline (new)

```
Developer/Agent push or PR
    ↓
GitHub Actions: ci.yml
    ├── job: fmt       → cargo fmt --check (fail = formatting error)
    ├── job: check     → cargo check (fail = compile error)
    ├── job: clippy    → cargo clippy -D warnings (fail = lint violation)
    └── job: test      → cargo test (fail = test failure or arch violation)

GitHub Actions: audit.yml (daily cron + dependency file changes)
    └── job: audit     → cargo audit (fail = known CVE in dependency tree)
```

### Documentation Navigation Flow (new)

```
Agent starts task
    ↓
reads CLAUDE.md (~60 lines)
    ├── build/test commands → immediate actionability
    ├── key constraints table → guards the invariants
    └── docs/ pointer table → knows where to go for deep context

Agent needs architecture detail → reads docs/ARCHITECTURE.md
Agent needs security invariants → reads docs/SECURITY.md
Agent needs test guidance → reads docs/TESTING.md
Agent needs "why" for a constraint → reads docs/adr/00N-*.md
```

### Structural Test Data Flow (new)

```
cargo test
    ↓
tests/arch.rs compiled as integration test
    ↓
arch_test reads src/lib.rs → builds module tree via static analysis
    ↓
checks access rules against module tree
    ↓
PASS: dependency directions are clean
FAIL: error message names the violating module pair → agent fixes import
```

## Scaling Considerations

This is a local CLI tool. Scaling is not applicable to user count. The relevant scale is
developer velocity — specifically, how quickly an agent can make correct changes.

| Concern | Now (~2,800 lines) | After harness (~3,200 lines) |
|---------|-------------------|------------------------------|
| Agent context for any task | CLAUDE.md fully loaded | CLAUDE.md as map; deep docs loaded on demand |
| Violation detection | Manual code review | CI blocks merge; structural tests catch regressions |
| New contributor orientation | Read CLAUDE.md + source | Read CLAUDE.md + relevant docs/ file |
| Dependency CVE exposure | Manual | Daily automated audit |

## Anti-Patterns

### Anti-Pattern 1: Bloated CLAUDE.md

**What people do:** Add module-level documentation, code snippets, full directory listings,
and per-command explanations into CLAUDE.md because "it's the agent's entry point."

**Why it's wrong:** CLAUDE.md enters every session context. Content irrelevant to the current
task burns tokens and dilutes attention. OpenAI's harness engineering article shows their
CLAUDE.md is under 60 lines. The current CLAUDE.md is 56 lines and well-structured — the risk
is adding to it instead of creating proper docs/.

**Do this instead:** Keep CLAUDE.md as a pointer map. Move any explanation longer than 2 lines
into docs/. Reference it with a one-sentence description and file path.

### Anti-Pattern 2: Documentation-Only Architectural Rules

**What people do:** Write "crypto/ must not import profile/" in CLAUDE.md or a docs file,
then rely on code review to enforce it.

**Why it's wrong:** High-velocity agent-generated code is never reviewed at the granularity
of individual import statements. The rule drifts silently. By the time it is caught, multiple
modules violate it.

**Do this instead:** Express the rule as a test in tests/arch.rs. When the rule is violated,
`cargo test` fails with a deterministic error message that an agent can act on immediately.

### Anti-Pattern 3: Monolithic CI Workflow

**What people do:** Put all CI steps (fmt, check, clippy, test, audit) in a single sequential
workflow job with `cargo test` at the end.

**Why it's wrong:** If `cargo fmt --check` fails (a 2-second job), the agent waits for the
entire test suite to run before seeing the error. This wastes CI minutes and agent context.

**Do this instead:** Use parallel jobs in ci.yml. Fmt, check, clippy, and test run concurrently.
The failing job is surfaced immediately. Audit runs in a separate workflow on a cron schedule
so it does not block feature CI runs.

### Anti-Pattern 4: Unsafe Code Without Mechanical Denial

**What people do:** Trust that agents won't introduce `unsafe` blocks. Document "don't use
unsafe" in CLAUDE.md.

**Why it's wrong:** An agent under pressure to make a test pass may introduce `unsafe` to
bypass the borrow checker. A documented rule is ignored; a compiler error is not.

**Do this instead:** Add `unsafe_code = "forbid"` to `[lints.rust]` in Cargo.toml. The build
fails at `cargo check` if unsafe appears anywhere. Zero unsafe is correct for this project
(all OS interaction goes through safe abstractions in the keyring and sysinfo crates).

### Anti-Pattern 5: Using clippy.toml for Lint Levels

**What people do:** Set `warn = ["clippy::pedantic"]` or similar in clippy.toml.

**Why it's wrong:** clippy.toml is for Clippy's tool configuration (msrv, disallowed names,
function complexity thresholds). Lint levels in clippy.toml are not the canonical location
and may be ignored in some contexts.

**Do this instead:** Set lint levels in Cargo.toml `[lints.clippy]`. Use clippy.toml only for
the small set of configuration options that are genuinely tool config (msrv is the primary one).

## Integration Points

### lib.rs Requirement for arch_test

**Problem:** arch_test analyzes module trees starting from a library entry point (`src/lib.rs`).
The current project has only `src/main.rs`.

**Solution:** Add a thin `src/lib.rs` that re-exports the module tree:

```rust
// src/lib.rs — exists only for arch_test and library-style unit testing
pub mod cli;
pub mod config;
pub mod crypto;
pub mod error;
pub mod guard;
pub mod paths;
pub mod profile;
pub mod tui;
```

This does not change the binary behavior. `main.rs` already declares these modules; `lib.rs`
mirrors them. The binary target in Cargo.toml stays unchanged. arch_test builds against the
library target.

**Cargo.toml change required:** Add a `[lib]` section or confirm Cargo auto-detects `src/lib.rs`.
Cargo auto-detects `src/lib.rs` as the library target when present alongside `src/main.rs`,
so no Cargo.toml change is required.

### Cargo.toml [lints] Section

**Problem:** Currently no lints section in Cargo.toml.

**Recommended addition:**

```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
# Deny common correctness issues
unwrap_used = "warn"
expect_used = "warn"
```

Note: `unwrap_used` and `expect_used` produce warnings (not errors) because the existing
codebase uses them in test helpers. Upgrading to "deny" requires a small cleanup pass first.
Start with "warn" and tighten after the audit. `unsafe_code = "forbid"` can be "forbid"
immediately — the existing codebase has zero unsafe blocks.

### rustfmt.toml

**Recommended configuration for edition 2021 project:**

```toml
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
```

The existing codebase uses 100-char line width and 4-space indentation. This makes the
configuration explicit and reproducible across contributors and CI.

### clippy.toml

**Recommended minimal configuration:**

```toml
msrv = "1.74.0"
```

Only set msrv here (the minimum supported Rust version). Lint levels go in Cargo.toml.
The msrv is set to 1.74.0 because that is when the `[lints]` section in Cargo.toml was
stabilized (November 2023), and it is the floor for this feature set.

### GitHub Actions: ci.yml

**Recommended structure (HIGH confidence):**

```yaml
name: CI

on:
  push:
    branches: [main]
  pull_request:

env:
  RUSTFLAGS: "-Dwarnings"
  CARGO_TERM_COLOR: always

jobs:
  fmt:
    name: Rustfmt
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - run: cargo fmt --all -- --check

  check:
    name: Cargo Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo check --all-targets

  clippy:
    name: Clippy
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - run: cargo clippy --all-targets

  test:
    name: Tests
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - run: cargo test
```

Notes:
- `dtolnay/rust-toolchain@stable` is the current community-standard action, preferred over
  the archived `actions-rs/toolchain`.
- `RUSTFLAGS: "-Dwarnings"` makes clippy and check fail on any warning.
- Four parallel jobs means the fastest-failing job is surfaced immediately.
- keyring integration tests require OS keychain; `cargo test --lib` is sufficient for CI.
  If integration tests need keychain, they need to be gated with `#[ignore]` or feature flags.

**Keychain CI concern (MEDIUM confidence):** The keyring crate accesses the OS keychain.
On GitHub-hosted Linux runners (ubuntu-latest), there is no GNOME keyring daemon running by
default. Tests that call `OsKeyStore` will fail in CI. The existing tests use `MockKeyStore`
everywhere, so `cargo test --lib` should pass. The integration tests in `tests/integration.rs`
only test `--help` and `--version`, which do not touch the keychain. This should be safe.
Verify by checking which tests invoke `OsKeyStore` directly.

### GitHub Actions: audit.yml

**Recommended structure (HIGH confidence):**

```yaml
name: Security Audit

on:
  push:
    paths:
      - '**/Cargo.toml'
      - '**/Cargo.lock'
  schedule:
    - cron: '0 0 * * 1'   # Weekly on Monday midnight UTC
  workflow_dispatch:

jobs:
  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/audit@v1
        name: Audit Rust Dependencies
```

Note: Weekly cron (Monday) rather than daily. This project has 11 stable, mainstream
dependencies. Daily audits are overkill. Weekly catches new advisories promptly.

### CLAUDE.md Structure (Rewrite)

**Current CLAUDE.md is 56 lines.** The rewrite keeps it under 80 lines by:
1. Keeping the "Build & Test Commands" section verbatim (it is the correct density)
2. Replacing the "Architecture" prose section with a pointer table to docs/
3. Keeping "Key constraints" — it is exactly the right kind of content for CLAUDE.md

**Pointer table to add:**

```markdown
## Documentation

| File | What it covers |
|------|---------------|
| docs/ARCHITECTURE.md | Layer diagram, module map, dependency direction rules |
| docs/SECURITY.md | Encryption model, key storage, threat surface |
| docs/TESTING.md | Test strategy, how to use mocks, adding new tests |
| docs/adr/ | Why each key design decision was made |
```

**Architecture prose section to remove from CLAUDE.md:**
The 25-line "Architecture" section in CLAUDE.md that describes the TUI state machine, crypto
flow, switch lifecycle, and input validation. This moves verbatim (and expanded) into
docs/ARCHITECTURE.md. The pointer table replaces it.

## Build Order for Implementation

This ordering respects dependencies between components:

### Step 1: Create lib.rs (unblocks arch_test)
`src/lib.rs` is a prerequisite for structural tests. It is a pure re-export file with no
logic changes. Create it first; verify `cargo build` still passes.

### Step 2: Cargo.toml and config files (unblock CI and enforcement)
Add `[lints.rust]` and `[lints.clippy]` sections to Cargo.toml. Create `rustfmt.toml` and
`clippy.toml`. Run `cargo fmt`, `cargo clippy`, and `cargo test` locally to confirm zero
regressions before CI is wired up.

### Step 3: Add arch_test dev-dependency + tests/arch.rs
Add `arch_test` to `[dev-dependencies]`. Write the structural test file. Run `cargo test --test arch`
to confirm tests pass with the current clean architecture.

### Step 4: Write docs/ (unblocks CLAUDE.md rewrite)
Create `docs/ARCHITECTURE.md`, `docs/SECURITY.md`, `docs/TESTING.md`, and at least one ADR.
These must exist before CLAUDE.md can meaningfully point to them.

### Step 5: Rewrite CLAUDE.md
Replace the architecture prose with the pointer table. Keep build commands and key constraints.
Verify the resulting file is under 80 lines.

### Step 6: GitHub Actions workflows
Add `.github/workflows/ci.yml` and `.github/workflows/audit.yml`. Push to a branch and verify
all CI jobs pass on the first run.

## Sources

- [OpenAI Harness Engineering (blog summary)](https://alexlavaee.me/blog/openai-agent-first-codebase-learnings/) — MEDIUM confidence (secondary summary, primary article behind 403)
- [Writing a good CLAUDE.md — HumanLayer](https://www.humanlayer.dev/blog/writing-a-good-claude-md) — MEDIUM confidence
- [Skill Issue: Harness Engineering — HumanLayer](https://www.humanlayer.dev/blog/skill-issue-harness-engineering-for-coding-agents) — MEDIUM confidence
- [arch_test crate — GitHub](https://github.com/tdymel/arch_test) — HIGH confidence (primary source)
- [Clippy GitHub Actions — Official Rust Docs](https://doc.rust-lang.org/nightly/clippy/continuous_integration/github_actions.html) — HIGH confidence
- [actions-rust-lang/audit — GitHub](https://github.com/actions-rust-lang/audit) — HIGH confidence
- [Cargo.toml lints section — Rust RFC 3389](https://rust-lang.github.io/rfcs/3389-manifest-lint.html) — HIGH confidence
- [Clippy configuration — Official Rust Docs](https://doc.rust-lang.org/clippy/configuration.html) — HIGH confidence
- [Configuring rustfmt — Official Rust Docs](https://rust-lang.github.io/rustfmt/) — HIGH confidence
- [GitHub Actions for Rust — shift.click](https://shift.click/blog/github-actions-rust/) — MEDIUM confidence

---
*Architecture research for: harness engineering alignment, sub-swap Rust CLI*
*Researched: 2026-04-02*
