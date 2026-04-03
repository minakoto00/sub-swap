# Phase 1: Code Quality Foundation - Research

**Researched:** 2026-04-03
**Domain:** Rust tooling — rustfmt, clippy, Cargo lints, lib.rs dual-target, justfile
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

**D-01:** Enable `clippy::pedantic` as `warn`, suppress exactly these lints:
- `module_name_repetitions`
- `must_use_candidate`
- `missing_errors_doc`
- `missing_panics_doc`
- `doc_markdown`
- `return_self_not_must_use`

**D-02:** Start with that allow-list; fix real issues, suppress false positives found during first pass.

**D-03:** Re-export all top-level modules (`cli`, `config`, `crypto`, `error`, `guard`, `paths`, `profile`, `tui`) as `pub mod` in lib.rs.

**D-04:** Keep lib.rs thin — just `pub mod` declarations, no logic. main.rs switches to using the library crate.

**D-05:** Implement exactly the commands required: `check`, `test`, `lint`, `fmt`, `validate`. No extras.

**D-06:** `just validate` runs: fmt-check → clippy → test. Stops on first failure using `&&` chaining.

**D-07:** Two-commit approach: (1) config files commit, (2) `cargo fmt` formatting commit.

### Claude's Discretion
- Exact `clippy.toml` MSRV value and cognitive-complexity threshold (use 1.94.1 and 25)
- Specific rustfmt options beyond `edition` and `max_width` (keep minimal)
- Whether lib.rs uses glob re-exports or explicit `pub mod` (use `pub mod` for clarity)

### Deferred Ideas (OUT OF SCOPE)
None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| QUAL-01 | Cargo.toml `[lints]` table with `unsafe_code = "forbid"`, `clippy::all = "warn"`, `clippy::pedantic = "warn"` (with targeted allows) | D-01/D-02 decisions; clippy audit findings below document exactly which violations need fixing vs. suppression |
| QUAL-02 | `rustfmt.toml` exists with `edition = "2021"` and `max_width = 100` | 32 format diff chunks exist; `cargo fmt` will reformat to satisfy `--check` |
| QUAL-03 | `clippy.toml` exists with MSRV setting and cognitive-complexity threshold | Rust stable is 1.94.1; default cognitive-complexity threshold is 25 |
| QUAL-04 | `src/lib.rs` exists as a thin re-export module enabling structural tests and `cargo doc` | Critical: `Cli`/`Commands`/`ConfigAction` are in main.rs and must move to `cli.rs` before lib.rs is viable |
| OBSV-01 | `justfile` exists with `check`, `test`, `lint`, `fmt`, `validate` commands | `just` 1.48.1 is available via homebrew but NOT currently installed |
| OBSV-02 | `just validate` runs fmt check + clippy + test in sequence, stops on first failure | D-06: use `&&` chaining; research confirms this is correct justfile pattern |
</phase_requirements>

---

## Summary

Phase 1 is a pure tooling and configuration phase with no business logic changes. The project already
builds cleanly (`cargo build` succeeds, 43 unit tests pass, 2 integration tests pass). The challenge
is bringing the codebase to zero-warning status under the new lint configuration without breaking
existing tests.

There are **two non-trivial technical problems** that require careful sequencing. First, enabling
`clippy::pedantic` as `warn` with `-D warnings` in the `[lints]` table will promote the existing
6 `dead_code` warnings plus ~27 pedantic warnings to errors. These must all be fixed before the
`[lints]` table is committed. Second, creating `lib.rs` requires moving `Cli`, `Commands`, and
`ConfigAction` out of `main.rs` into `cli.rs` (or a new `cli/args.rs`) because `cli.rs` already
imports them with `use crate::{Cli, Commands, ConfigAction}` — those symbols must come from a
module visible to the library crate, not from `main.rs`.

`just` is NOT currently installed. The plan must include an installation step before the justfile
can be verified.

**Primary recommendation:** Fix all clippy violations in source code first (most are real
improvements), then commit config files, then apply `cargo fmt`, then install `just` and verify
the justfile. This ordering prevents the `[lints]` table from creating a broken intermediate state.

---

## Standard Stack

### Core Tooling
| Tool | Version | Purpose | Notes |
|------|---------|---------|-------|
| rustfmt | 1.8.0-stable (ships with toolchain) | Code formatting | Already available |
| clippy | 0.1.94 (ships with toolchain) | Linting | Already available |
| just | 1.48.1 | Command runner / justfile | NOT installed — needs `cargo install just` or `brew install just` |

### Rust Toolchain
| Property | Value |
|----------|-------|
| Channel | stable |
| Version | 1.94.1 (e408947bf 2026-03-25) |
| Edition | 2021 |
| Target | aarch64-apple-darwin |

### Installation
```bash
# just is not installed — choose one:
brew install just          # preferred: system-wide, faster
cargo install just         # alternative: Rust ecosystem install
```

---

## Architecture Patterns

### lib.rs + main.rs Dual-Target Pattern

Rust allows a package to have both a binary (`src/main.rs`) and a library (`src/lib.rs`) target.
The library target is the "public API" of the crate. When both exist:
- Library crate: `sub_swap` — can be depended on by integration tests and `cargo doc`
- Binary crate: `sub-swap` — links against the library, provides `main()`

**CRITICAL ISSUE: `Cli`/`Commands`/`ConfigAction` are currently in `main.rs`.**

`src/cli.rs` line 11: `use crate::{Cli, Commands, ConfigAction};`

When `lib.rs` is added, `crate` in `main.rs` still refers to the binary crate root, not the library. This import in `cli.rs` will break when `cli.rs` is re-exported from `lib.rs` because the library crate doesn't define those types.

**Solution:** Move `Cli`, `Commands`, and `ConfigAction` into `src/cli.rs` before creating lib.rs. Then `main.rs` switches to `use sub_swap::cli::{Cli, Commands, ConfigAction}` (or equivalent). The `cli.rs` import `use crate::{Cli, Commands, ConfigAction}` becomes a self-reference that should be replaced with the types being defined in the same file.

### Recommended lib.rs Structure
```rust
// src/lib.rs — thin re-export module only
pub mod cli;
pub mod config;
pub mod crypto;
pub mod error;
pub mod guard;
pub mod paths;
pub mod profile;
pub mod tui;
```

### Recommended main.rs After Refactor
```rust
// src/main.rs — entry point only
use sub_swap::cli::{run, Cli};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

### Dead Code Fix: `#[cfg(test)]`-Gate Mock Types

The 6 existing `dead_code` warnings come from `MockKeyStore`, `MockGuard`, `has_key()`, and
`NotInitialized`. These items exist for testing but are only used inside inline `#[cfg(test)]`
modules. From the binary target's perspective they appear unused.

Two valid approaches:
1. Add `#[cfg(test)]` to the entire mock struct/impl blocks (cleanest — removes dead items from
   production compilation entirely). Note: `has_key()` on `OsKeyStore` is used in production; only
   the `MockKeyStore` copy triggers the warning.
2. Mark with `#[allow(dead_code)]` at item level (less preferred — keeps dead items in binary).

After lib.rs is added, external tests can access `MockKeyStore` and `MockGuard` from `sub_swap::crypto::keychain` and `sub_swap::guard` — but the inline `#[cfg(test)]` gating is still the right fix because the binary target should not carry test infrastructure.

For `NotInitialized` variant in `SubSwapError`: it's a legitimate future-use variant. Add
`#[allow(dead_code)]` at the variant level or remove it if not planned for the near term.

### Cargo `[lints]` Table Pattern (Rust 1.73+)

```toml
# Cargo.toml — [lints] table approach (QUAL-01)
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
pedantic = "warn"
# Allow-list for intentional patterns (D-01)
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
doc_markdown = "allow"
return_self_not_must_use = "allow"
```

The `[lints]` table was stabilized in Rust 1.73.0 (2023-10-05). Rust 1.94.1 supports it fully.
The `priority` field is available for overriding group settings — not needed here.

**Important:** With `[lints]` in Cargo.toml, `cargo clippy` respects these lints without needing
to pass `-D warnings` on the command line. However, `cargo clippy -- -D warnings` still works and
overrides in the same direction. The `[lints]` table replaces the need for a `build.rs` or
`.cargo/config.toml` approach.

### rustfmt.toml Minimal Config Pattern

```toml
# rustfmt.toml — QUAL-02 compliant
edition = "2021"
max_width = 100
```

**Current state:** The codebase has 32 format diff chunks across 8 files. The majority are
`use` statement import reordering (serde before std crate) and function argument wrapping.
`max_width = 100` instead of the default 80 will produce fewer line-wraps but the existing
code was written closer to 80 — expect moderate reformatting.

### clippy.toml Config Pattern

```toml
# clippy.toml — QUAL-03 compliant
msrv = "1.94.1"
cognitive-complexity-threshold = 25
```

**MSRV note:** Using current stable (1.94.1) as MSRV. This is not a library so MSRV stability
across Rust versions is not a concern. If this is updated later the clippy.toml should be
updated to match.

### Justfile Pattern for Rust Projects

```just
# justfile — OBSV-01/02 compliant
set shell := ["bash", "-c"]

# Fast compile check
check:
    cargo check

# Run all tests
test:
    cargo test

# Run linter
lint:
    cargo clippy -- -D warnings

# Apply formatting
fmt:
    cargo fmt

# Validate everything (fmt-check → clippy → test), stops on first failure
validate:
    cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

**Justfile notes:**
- `set shell` is optional but makes behavior explicit
- Recipe names must not conflict with `just` built-in commands (none of these do)
- `just validate` will stop at the first `&&`-chained failure and exit non-zero
- `just --list` shows all recipes — useful for discovery
- No shebang needed; just runs commands line-by-line by default

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Code formatter | Custom formatting script | `cargo fmt` / rustfmt | Built into toolchain, understands Rust AST |
| Lint runner | Shell script | `cargo clippy` | Semantic analysis, auto-fix suggestions |
| Command runner | Makefile | `just` | Simpler syntax, cross-platform, no implicit rules |
| Import ordering | Manual convention | rustfmt (`imports_granularity`) | Automatic, enforced at commit |

**Key insight:** All tooling for this phase ships with the Rust toolchain or has a single-command
install. There is nothing to build.

---

## Common Pitfalls

### Pitfall 1: `Cli`/`Commands`/`ConfigAction` in `main.rs` breaks lib target
**What goes wrong:** `lib.rs` re-exports `cli` module. `cli.rs` has `use crate::{Cli, Commands, ConfigAction}`. When compiled as library, `crate` is the lib root — those types don't exist there since they're in `main.rs`. Compilation fails.
**Why it happens:** Types defined in `main.rs` only exist in the binary crate root, not in the library crate root. Both share the same source files but `crate` refers to different roots depending on which target is being compiled.
**How to avoid:** Move `Cli`, `Commands`, `ConfigAction` into `cli.rs` BEFORE adding `lib.rs`. Update `main.rs` to import them from `cli`.
**Warning signs:** `cargo build --lib` fails with "failed to resolve: use of undeclared type `Cli`" or similar.

### Pitfall 2: `[lints]` table activating `-D warnings` breaks on existing dead code
**What goes wrong:** Committing `clippy::pedantic = "warn"` in `[lints]` with `all = "warn"` promotes the 6 existing `dead_code` warnings to lint-category warnings, then `cargo clippy -- -D warnings` fails.
**Why it happens:** `dead_code` is part of the `unused` lint group, which is part of `warnings`. The `[lints]` table `all = "warn"` already captures it. Dead code that was invisible at default lint level now blocks the check gate.
**How to avoid:** Fix all existing clippy warnings BEFORE committing the `[lints]` table.
**Warning signs:** `cargo clippy` (without `-D warnings`) currently shows 6 warnings — these will become errors under the new config.

### Pitfall 3: `doc_markdown` allow doesn't cover inline-test warnings
**What goes wrong:** The allow-list includes `doc_markdown = "allow"` but doc comments in code that are already flagged (e.g., `switch.rs` `ProfileStore`, `set_active(target)`) may still appear as errors because the `[lints]` table apply after the allow overrides.
**Why it happens:** The `doc_markdown` lint is in `clippy::pedantic`. The allow in `[lints]` should silence it project-wide. Verify after applying `[lints]` that no doc_markdown violations remain.
**How to avoid:** This is already in the allow-list per D-01. Verify with `cargo clippy -- -D warnings` after config commit.

### Pitfall 4: `just` not installed
**What goes wrong:** Committing a justfile and then verifying `just validate` passes requires `just` to be installed. The current machine does NOT have `just` installed.
**Why it happens:** `just` is a third-party command runner, not part of the Rust toolchain.
**How to avoid:** Install `just` as an explicit plan step before any justfile verification step.
**Warning signs:** `command not found: just` when running `just validate`.

### Pitfall 5: `unsafe_code = "forbid"` is in `[lints.rust]` not `[lints.clippy]`
**What goes wrong:** Placing `unsafe_code = "forbid"` under `[lints.clippy]` silently does nothing; it's a rustc lint, not a clippy lint.
**Why it happens:** The lint namespace differs: `[lints.rust]` for rustc lints, `[lints.clippy]` for clippy lints.
**How to avoid:** Use the correct section. `unsafe_code` goes under `[lints.rust]`.

### Pitfall 6: Two-commit strategy ordering
**What goes wrong:** Committing `cargo fmt` changes in the same commit as config files makes git blame dirty and makes diffs harder to review.
**Why it happens:** `cargo fmt` can touch many files and obscures what was an intentional code change vs. a mechanical format change.
**How to avoid:** Follow D-07: (1) commit config files + source fixes, (2) separate commit for `cargo fmt` output.

---

## Code Examples

### Cargo.toml `[lints]` Table (verified pattern)
```toml
[lints.rust]
unsafe_code = "forbid"

[lints.clippy]
all = "warn"
pedantic = "warn"
# Intentional suppressions (see docs/decisions/ for rationale)
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
doc_markdown = "allow"
return_self_not_must_use = "allow"
```

### MockKeyStore Properly Gated
```rust
// Before: struct and impl at module level → dead_code warning from binary
pub struct MockKeyStore { ... }
impl MockKeyStore { ... }

// After: gated so test infra doesn't appear in binary compilation
#[cfg(test)]
pub struct MockKeyStore { ... }
#[cfg(test)]
impl MockKeyStore { ... }
#[cfg(test)]
impl KeyStore for MockKeyStore { ... }
```

### lib.rs Minimal Re-export
```rust
// src/lib.rs
pub mod cli;
pub mod config;
pub mod crypto;
pub mod error;
pub mod guard;
pub mod paths;
pub mod profile;
pub mod tui;
```

### main.rs After Struct Migration
```rust
// src/main.rs — after Cli/Commands/ConfigAction move to cli.rs
use sub_swap::cli::{run, Cli};
use clap::Parser;

fn main() {
    let cli = Cli::parse();
    if let Err(e) = run(cli) {
        eprintln!("Error: {e}");
        std::process::exit(1);
    }
}
```

### Justfile Complete
```just
# justfile

# Fast compile check
check:
    cargo check

# Run all tests
test:
    cargo test

# Run clippy with warnings as errors
lint:
    cargo clippy -- -D warnings

# Apply formatting in-place
fmt:
    cargo fmt

# Validate all quality gates in sequence; stops on first failure
validate:
    cargo fmt --check && cargo clippy -- -D warnings && cargo test
```

---

## Exact Clippy Violations to Fix (Verified by `cargo clippy --pedantic`)

These are ALL real violations found by running `cargo clippy -- -W clippy::pedantic` with the
planned allow-list applied. The plan must address each one.

**Dead code (6 warnings → fix with `#[cfg(test)]` gating):**
- `src/crypto/keychain.rs`: `has_key()` method on trait, `MockKeyStore` struct, `MockKeyStore::new()`
- `src/guard.rs`: `MockGuard` struct, `MockGuard::new()`
- `src/error.rs`: `NotInitialized` variant

**Pedantic violations (all fixable in source, none require suppression):**
- `src/cli.rs:21` — `if_not_else`: invert condition to avoid `if !x { ... } else { ... }`
- `src/cli.rs:106` — `single_match_else`: replace `match from { Some(...) => ..., ... }` with `if let`
- `src/crypto/keychain.rs:8` — `format_collect`: use `fold` + `write!` instead of `map(format!()).collect()`
- `src/error.rs:37` — `redundant_closure_for_method_calls`: `|p| p.to_string()` → `ToString::to_string`
- `src/profile/mod.rs:96` — `redundant_closure_for_method_calls`: `|s| s.as_str()` → `String::as_str`
- `src/profile/switch.rs:12,14,21,146` — `doc_markdown`: doc comments with unbackticked identifiers (SUPPRESSED per D-01 allow-list)
- `src/tui/mod.rs:185,222,272,361,417` — `single_match_else`: multiple match → if let conversions
- `src/tui/mod.rs:254,292,350,381` — `redundant_closure_for_method_calls`: `|s| s.to_string()` → method ref
- `src/tui/mod.rs:256,294,352,383` — `assigning_clones`: `.clone()` assign → `.clone_from()`
- `src/tui/widgets.rs:38,54` — `redundant_closure_for_method_calls`: two closures
- `src/tui/wizard.rs:48,118,142,143,164` — `uninlined_format_args`: use format string variables directly
- `src/tui/wizard.rs:133` — `match_same_arms`: identical match arms → collapse to single wildcard

**Note on `doc_markdown`:** The 4 violations in `switch.rs` are covered by the allow-list (`doc_markdown = "allow"`). They will NOT appear as errors. No source change needed for those.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| rustfmt | QUAL-02, cargo fmt | Yes | 1.8.0-stable | None needed |
| clippy | QUAL-01, cargo clippy | Yes | 0.1.94 | None needed |
| cargo | All | Yes | 1.94.1 | None needed |
| rustc | All | Yes | 1.94.1 | None needed |
| just | OBSV-01, OBSV-02 | NO | 1.48.1 available | Must install |

**Missing dependencies with no fallback:**
- `just` — blocks verification of OBSV-01 and OBSV-02. Must install via `brew install just` or `cargo install just` before justfile can be validated.

**Missing dependencies with fallback:**
- None.

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | None (no `nextest.toml` or custom config) |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| QUAL-01 | `cargo clippy -- -D warnings` passes with `[lints]` table active | smoke | `cargo clippy -- -D warnings` | N/A (tool check) |
| QUAL-02 | `cargo fmt --check` passes with no diff | smoke | `cargo fmt --check` | N/A (tool check) |
| QUAL-03 | `clippy.toml` accepted by clippy (MSRV parses, no errors) | smoke | `cargo clippy 2>&1 | grep error` | N/A (tool check) |
| QUAL-04 | `cargo build --lib` succeeds after lib.rs added | smoke | `cargo build --lib` | N/A (build check) |
| OBSV-01 | `just check/test/lint/fmt` each work as standalone | smoke | `just check && just test && just lint` | N/A (tool check) |
| OBSV-02 | `just validate` stops on first failure | smoke | `just validate` (verify exit code) | N/A (tool check) |

### Sampling Rate
- **Per task commit:** `cargo check` (fast, catches compile errors)
- **Per wave merge:** `cargo test` (full suite)
- **Phase gate:** `cargo fmt --check && cargo clippy -- -D warnings && cargo test && just validate` all green before phase complete

### Wave 0 Gaps
None — this phase adds config files and fixes existing source code. No new test files are required. The existing 43 unit tests + 2 integration tests must continue to pass throughout.

---

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `.cargo/config.toml` `build.rustflags` for lints | `Cargo.toml [lints]` table | Rust 1.73 (Oct 2023) | Per-package, version-controlled, no env var needed |
| `Makefile` for command runner | `justfile` | Ongoing adoption | Simpler syntax, no implicit rules, cross-platform |
| `#![warn(clippy::pedantic)]` in `lib.rs` | `[lints.clippy] pedantic = "warn"` in `Cargo.toml` | Rust 1.73 | Centralized, not duplicated per file |

**Deprecated/outdated:**
- `#![warn(...)]` at crate root: Still works but `[lints]` table is now the canonical location per cargo docs. Don't add to lib.rs.
- `RUSTFLAGS=-D warnings` in CI: Still valid for CI but not needed for local development with `[lints]` table.

---

## Open Questions

1. **Should `NotInitialized` variant be removed or kept?**
   - What we know: It's never constructed in current code; adding `#[cfg(test)]` doesn't apply to an error variant.
   - What's unclear: Is it planned for future use? The switch lifecycle doesn't use it.
   - Recommendation: Remove it in the source-fix commit. If needed later, it can be re-added. Alternatively, add `#[expect(dead_code, reason = "reserved for future initialization check")]` — Rust 1.81+ supports `#[expect]` as a stricter alternative to `#[allow]` that warns if the suppression becomes unnecessary.

2. **`has_key()` on `KeyStore` trait — is it needed on the trait?**
   - What we know: `OsKeyStore::has_key()` is called in `cli.rs` to check if a key exists before generating. `MockKeyStore` implements it for test completeness. The trait method itself triggers `dead_code` warning.
   - What's unclear: Whether making the trait method dead triggers because no callers call it through the trait (only through concrete type).
   - Recommendation: Add `#[cfg(test)]` to `MockKeyStore`'s `has_key` impl only if the trait method warning persists after gating the struct. If `OsKeyStore::has_key()` is actually called in production code through `dyn KeyStore`, the trait method is not dead — verify this in `cli.rs`.

---

## Project Constraints (from CLAUDE.md)

The following CLAUDE.md directives are mandatory for all implementation in this phase:

| Directive | Applies To |
|-----------|------------|
| `cargo check` — fast compile check | Run after each source change |
| `cargo test` — run all tests | All 45 existing tests must pass throughout |
| `export PATH="$HOME/.cargo/bin:$PATH"` required before cargo commands | All shell invocations |
| **Strictly offline**: No network crates in dependency tree | No new dependencies allowed |
| **File permissions**: All files under `~/.sub-swap/` must be mode 0600 | Not applicable to this phase (config files only) |
| `src/lib.rs` adds a library target — run `cargo build` immediately after | Build verification step required |
| `pub(crate)` declarations in `main.rs` context may need adjustment | Verify after lib.rs added |

---

## Sources

### Primary (HIGH confidence)
- Direct code inspection — `cargo clippy` and `cargo fmt --check` run against actual codebase
- Rust toolchain — versions verified with `rustc --version`, `rustfmt --version`, `clippy --version`
- `cargo search just` — confirmed 1.48.1 current version from crates.io
- `brew info just` — confirmed 1.48.1 available via homebrew

### Secondary (MEDIUM confidence)
- Cargo docs — `[lints]` table stabilized Rust 1.73 (documented in Cargo Book)
- clippy documentation — cognitive-complexity-threshold default of 25

### Tertiary (LOW confidence)
- None

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — all versions verified against live toolchain
- Architecture: HIGH — lib.rs issue verified by reading actual source files
- Pitfalls: HIGH — all pitfalls verified by running tools against the actual codebase
- Clippy violations: HIGH — complete list from actual `cargo clippy` run

**Research date:** 2026-04-03
**Valid until:** 2026-05-03 (stable tooling, 30-day horizon)
