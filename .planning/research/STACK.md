# Stack Research

**Domain:** Rust CLI — harness engineering infrastructure (CI/CD, mechanical enforcement, structural tests, docs)
**Researched:** 2026-04-02
**Confidence:** HIGH

## Context

This is a SUBSEQUENT MILESTONE research. The existing stack (Rust 2021, clap 4.6, ratatui 0.30, aes-gcm, keyring, etc.) is validated and NOT re-researched. This document covers ONLY the new infrastructure tooling needed for harness engineering alignment.

---

## Recommended Stack

### CI/CD — GitHub Actions

| Technology | Version | Purpose | Why Recommended |
|------------|---------|---------|-----------------|
| `dtolnay/rust-toolchain` | `@stable` / `@nightly` | Install Rust toolchain in CI | The de-facto standard; replaces the deprecated `actions-rs` suite. Supports components (clippy, rustfmt) and targets inline. |
| `Swatinem/rust-cache` | `@v2` | Cache Cargo registry + build artifacts | Purpose-built for Rust; smarter than raw `actions/cache` — auto-cleans stale deps, keys on Cargo.lock hash + rustc version. Widely used across the ecosystem. |
| `actions-rust-lang/audit` | `@v1` | Run `cargo audit` against RustSec advisory DB | Official actions-rust-lang org action, actively maintained. Creates GitHub issues for findings. Use over the deprecated `actions-rs/audit`. |
| `EmbarkStudios/cargo-deny-action` | `@v2` | License + supply chain checks | Broader than `cargo-audit`: checks licenses, duplicate deps, and source restrictions. Single deny.toml config. |

**What NOT to use:** `actions-rs/*` — the entire actions-rs organization is unmaintained as of 2023. Every action in that suite (`actions-rs/toolchain`, `actions-rs/cargo`, `actions-rs/clippy-check`, `actions-rs/tarpaulin`) should be replaced.

#### Recommended Workflow Structure

Three separate jobs (not steps) running in parallel after checkout:

```yaml
# .github/workflows/ci.yml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo test --locked

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - uses: Swatinem/rust-cache@v2
      - run: cargo fmt --check
      - run: cargo clippy --locked -- -D warnings

  audit:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions-rust-lang/audit@v1
```

**Rationale for separate jobs:** Parallel execution. A fmt failure shouldn't block the audit job. Each job surfaces its own failure clearly. `--locked` flag on test/clippy ensures Cargo.lock is respected (prevents dependency drift in CI).

---

### Mechanical Code Quality Enforcement

#### rustfmt.toml

No new dependencies needed — `rustfmt` ships with the stable toolchain.

**Recommended `rustfmt.toml` for this project:**

```toml
edition = "2021"
max_width = 100
imports_granularity = "Crate"
group_imports = "StdExternalCrate"
reorder_imports = true
trailing_comma = "Vertical"
wrap_comments = false
```

**Rationale for each setting:**
- `edition = "2021"` — matches Cargo.toml edition, required for correct parsing
- `max_width = 100` — community standard (rustfmt default is 100; being explicit prevents surprise if defaults change)
- `imports_granularity = "Crate"` — merges same-crate imports into one `use` block; reduces noise in diffs
- `group_imports = "StdExternalCrate"` — separates `std`, external crates, and local modules with blank lines; improves scanability for an agent reading the code
- `reorder_imports = true` — alphabetic ordering; deterministic, eliminates bikeshedding
- `trailing_comma = "Vertical"` — only adds trailing commas in multi-line constructs; default, but explicit for stability
- `wrap_comments = false` — preserves intentional comment line breaks; avoids mangling long doc comment URLs

All settings are STABLE (no `unstable_features = true` required). This is important for CI compatibility.

#### clippy.toml

No new dependencies needed — `clippy` ships with the stable toolchain.

**Recommended `clippy.toml` for this project:**

```toml
# Minimum supported Rust version — keeps lint suggestions compatible
msrv = "1.74.0"

# Cognitive complexity threshold (default: 25, lower = stricter)
cognitive-complexity-threshold = 20
```

**Rationale:** The `clippy.toml` file controls lint *behavior* (thresholds, MSRV), not which lints are active. Which lints are active belongs in Cargo.toml `[lints]` table (see below). Keep `clippy.toml` minimal. MSRV 1.74 is the minimum required for the Cargo.toml `[lints]` table feature.

#### Cargo.toml `[lints]` Table

No new dependencies — requires Rust 1.74+ (Cargo feature, stable since Oct 2023).

**Recommended additions to `Cargo.toml`:**

```toml
[lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"

[lints.clippy]
# Enable the full "all" group at warn, then selectively deny
all = { level = "warn", priority = -1 }
# Correctness — these are bugs, not style
correctness = { level = "deny", priority = 0 }
# Deny common security-relevant patterns
unwrap_used = "warn"
expect_used = "warn"
panic = "warn"
```

**Rationale:**
- `unsafe_code = "forbid"` — this project has no need for unsafe code; forbid (not deny) prevents `#[allow(unsafe_code)]` overrides
- `missing_docs = "warn"` — not `deny` yet; warn first to expose gaps without blocking CI immediately
- `clippy::all` at `warn` with `priority = -1` — enables broad lint coverage without failing builds on style issues
- `clippy::correctness` at `deny` with `priority = 0` — correctness lints are effectively bugs; these should block CI
- `unwrap_used`, `expect_used`, `panic` at `warn` — for a security CLI, panics in non-test code are a design smell; warn surfaces them
- Priority system: negative priority = baseline; higher priority = override. This is the `[lints]` table priority mechanism (Rust 1.74+).

**What NOT to put in Cargo.toml lints:** Don't deny `clippy::pedantic` globally — pedantic generates many false positives and will create friction without clear benefit for a 2,800-line codebase. Use warn if desired.

---

### Structural / Architectural Boundary Tests

#### Option A: Hand-Written Tests (RECOMMENDED for this project)

No new dependencies. Use standard `#[test]` in `tests/` or inline test modules.

**Why recommended over arch_test_core:** The sub-swap codebase is 2,800 lines across 14 files. The architectural invariants to enforce are:
1. `crypto/` functions are pure (no I/O)
2. `profile/` never calls `tui/`
3. No `std::net` or async runtime imports anywhere

For a small codebase with clear module boundaries, hand-written tests using `std::fs::read_to_string` to parse source files, or compile-time checks using `pub(crate)` visibility rules, are simpler and more maintainable than a framework dependency.

**Example structural test pattern:**

```rust
// tests/architecture.rs
#[test]
fn crypto_module_has_no_io_imports() {
    // Check source files in crypto/ don't import std::io or std::fs
    let crypto_files = ["src/crypto/mod.rs", "src/crypto/keychain.rs"];
    for path in &crypto_files {
        let src = std::fs::read_to_string(path).unwrap();
        assert!(!src.contains("std::fs"), "{path} must not import std::fs");
        assert!(!src.contains("std::io::"), "{path} must not import std::io directly");
    }
}

#[test]
fn no_network_crates_in_source() {
    // Verify offline constraint: no net-related imports
    let output = std::process::Command::new("grep")
        .args(["-r", "reqwest\\|hyper\\|tokio\\|async-std", "src/"])
        .output()
        .unwrap();
    assert!(output.stdout.is_empty(), "Network crate imports found in src/");
}
```

#### Option B: arch_test_core (if complexity grows)

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `arch_test_core` | `0.x` | Rule-based architecture layer enforcement | If the codebase grows beyond 6-8 modules with complex dependency rules; overkill for current size |

**Verdict for sub-swap v1.0:** Do NOT add `arch_test_core` as a dev-dependency. Hand-written tests in `tests/architecture.rs` are sufficient, faster to execute, and have zero dependency overhead. Revisit if the codebase expands significantly.

---

### Documentation Structure Enforcement

No new tooling needed. Use standard Rust mechanisms:

| Mechanism | How | When |
|-----------|-----|------|
| `missing_docs = "warn"` in `[lints.rust]` | Already covered above | Enforces doc coverage on public items |
| `cargo doc --no-deps` step in CI | Add to lint job | Ensures docs compile without errors |
| Rustdoc lints (`broken_intra_doc_links`) | Built into `cargo doc` | Catches dead documentation links |

**Recommended `cargo doc` CI step:**
```yaml
- run: cargo doc --no-deps --document-private-items 2>&1 | grep -v "^$" && cargo doc --no-deps 2>&1 | grep "warning" && exit 0 || exit 1
```

Simpler approach: add `RUSTDOCFLAGS="-D warnings"` environment variable:
```yaml
- run: cargo doc --no-deps
  env:
    RUSTDOCFLAGS: "-D warnings"
```

This fails CI on any broken doc link or rustdoc warning.

---

## Installation

No new runtime dependencies. All tools are either:
- Shipped with the Rust toolchain (`rustfmt`, `clippy`, `cargo doc`)
- GitHub Actions (no local install needed)
- Installed via `cargo install` in CI (cargo-audit, cargo-deny)

For local development, contributors run:
```bash
# Install cargo-audit locally (optional, CI covers it)
cargo install cargo-audit

# Install cargo-deny locally (optional, CI covers it)
cargo install cargo-deny

# Run all checks locally
cargo fmt --check
cargo clippy --locked -- -D warnings
cargo test --locked
cargo audit
```

---

## Alternatives Considered

| Recommended | Alternative | When to Use Alternative |
|-------------|-------------|-------------------------|
| `Swatinem/rust-cache@v2` | `actions/cache` with manual paths | When you need granular control over exactly what's cached; for most projects rust-cache's smart defaults are better |
| `actions-rust-lang/audit@v1` | `cargo-deny` (for security only) | If you only need vulnerability checking and don't want the license/source features of deny |
| Hand-written structural tests | `arch_test_core` | When codebase has 10+ modules with complex inter-layer dependency rules; too heavy for 14 files |
| `[lints]` in Cargo.toml | `#![deny(...)]` in `main.rs` | The `#![deny]` approach is fine for single-file overrides but Cargo.toml lints work for the whole crate uniformly and are visible without opening source files |
| `clippy::all` at warn | `clippy::pedantic` | Use pedantic when you want maximum strictness; warn-only to avoid blocking CI on style lints |

---

## What NOT to Add

| Avoid | Why | Use Instead |
|-------|-----|-------------|
| `actions-rs/*` actions | Entire org is unmaintained since 2023; security risk for GitHub Actions | `dtolnay/rust-toolchain`, `actions-rust-lang/*` |
| `cargo-tarpaulin` for coverage | Linux x86_64 only; uses ptrace (brittle); the actions-rs/tarpaulin action is also unmaintained | `cargo-llvm-cov` if coverage is needed (out of scope for this milestone) |
| `arch_test_core` dev-dependency | Adds external dependency for a problem hand-written tests solve in 20 lines | Hand-written `tests/architecture.rs` |
| `unstable_features = true` in rustfmt.toml | Unstable options break on stable Rust toolchain (CI uses stable) | Only stable rustfmt options |
| `clippy::pedantic` at `deny` | Generates high false-positive rate; blocks CI over style disagreements | `clippy::pedantic` at `warn` if desired at all |
| `RUSTFLAGS = "-D warnings"` globally | Breaks dependencies that have warnings in their own code | Use `[lints]` in Cargo.toml which applies only to the current package |
| Code coverage as a milestone goal | TUI test infrastructure is explicitly out of scope; coverage requires ratatui test harness | Defer to a future milestone |

---

## Version Compatibility

| Package | Compatible With | Notes |
|---------|-----------------|-------|
| `[lints]` table in Cargo.toml | Rust 1.74.0+ | Released Oct 2023; stable toolchain in 2026 satisfies this |
| `dtolnay/rust-toolchain@stable` | All current GitHub-hosted runners | Replaces rustup default on runner |
| `Swatinem/rust-cache@v2` | `dtolnay/rust-toolchain` | Cache key includes rustc version; must run toolchain setup before cache |
| `imports_granularity` in rustfmt.toml | rustfmt stable | Stable option; available since Rust 1.39 |
| `group_imports` in rustfmt.toml | rustfmt stable | Stable since Rust 1.62 |

---

## Integration with Existing Cargo Workflow

The existing `cargo test`, `cargo build`, `cargo check` commands are unchanged. New commands added:

| Command | When to Run | Purpose |
|---------|-------------|---------|
| `cargo fmt --check` | CI + pre-commit | Verify formatting without modifying files |
| `cargo fmt` | Local dev | Auto-fix formatting |
| `cargo clippy --locked -- -D warnings` | CI | Lint with warnings-as-errors |
| `cargo clippy` | Local dev | Lint without hard failure |
| `cargo audit` | CI (audit job) | Security advisory check |
| `cargo doc --no-deps` | CI (lint job) | Verify docs compile |

All integrate with the existing `cargo` workflow. No new build system or task runner needed.

---

## Sources

- [dtolnay/rust-toolchain GitHub](https://github.com/dtolnay/rust-toolchain) — toolchain action usage, version specifiers
- [Swatinem/rust-cache GitHub](https://github.com/Swatinem/rust-cache) — cache strategy, what gets cached
- [actions-rust-lang/audit GitHub](https://github.com/actions-rust-lang/audit) — v1.2.7, inputs reference
- [Clippy configuration docs](https://doc.rust-lang.org/clippy/configuration.html) — clippy.toml options, MSRV setting — HIGH confidence
- [rustfmt Configurations.md](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md) — stable option reference — HIGH confidence
- [Cargo lints RFC 3389](https://rust-lang.github.io/rfcs/3389-manifest-lint.html) — `[lints]` table syntax, workspace lints — HIGH confidence
- [arch_test GitHub](https://github.com/tdymel/arch_test) — architecture test library for Rust — MEDIUM confidence (low download count, niche adoption)
- [cargo-deny crates.io](https://crates.io/crates/cargo-deny) — license + supply chain checks — HIGH confidence
- [LogRocket: cargo-audit vs cargo-deny comparison](https://blog.logrocket.com/comparing-rust-supply-chain-safety-tools/) — comparison rationale — MEDIUM confidence
- [RustSec Advisory Database](https://rustsec.org/) — source of cargo-audit advisories — HIGH confidence
- [cargo-llvm-cov GitHub](https://github.com/taiki-e/cargo-llvm-cov) — coverage alternative to tarpaulin — MEDIUM confidence

---
*Stack research for: sub-swap v1.0 harness engineering infrastructure*
*Researched: 2026-04-02*
