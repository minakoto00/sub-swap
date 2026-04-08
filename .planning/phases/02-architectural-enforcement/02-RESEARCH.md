# Phase 2: Architectural Enforcement - Research

**Researched:** 2026-04-07
**Domain:** Rust integration tests, source-level static analysis, Cargo.toml parsing
**Confidence:** HIGH

## Summary

Phase 2 adds a single file — `tests/arch.rs` — containing pure Rust integration tests that
enforce module layer rules, crypto purity, and the network-free constraint. The tests work
by reading source files and `Cargo.toml` as text at test-run time, scanning for prohibited
patterns, and failing with structured remediation messages when violations are found.

No proc-macros, build scripts, or external tools are needed. The `toml` crate (already in
`[dependencies]`) can parse `Cargo.toml` for the network-crate check. Module import checks
use `std::fs::read_to_string` and simple string matching against `use crate::` lines. The
`env!("CARGO_MANIFEST_DIR")` macro provides the absolute path to `Cargo.toml` and `src/`
at test compile time, making tests portable across machines.

Current codebase is violation-free: no existing `use crate::` imports break the proposed
layer rules, and `crypto/mod.rs` has no `std::fs`, `std::net`, or `std::process` imports.
[VERIFIED: codebase grep]

**Primary recommendation:** Write `tests/arch.rs` with three test groups (one per ARCH-0x
requirement), parse source files and `Cargo.toml` directly with `std::fs` and `toml`, and
use a consistent three-part failure message: VIOLATION / FOUND / HOW TO FIX.

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Layered dependency direction enforced. Foundation (`error`, `paths`) — std and
  external only; `paths` may import `error`. Core (`crypto`, `config`, `guard`) — foundation
  only. Business (`profile`) — core and foundation; NOT `cli`, `tui`, or `guard`. Orchestration
  (`cli`, `tui`) — anything.
- **D-02:** Structural tests parse source files (`use crate::` statements). No proc-macro or
  build-time enforcement.
- **D-03:** ARCH-02 applies to `crypto/mod.rs` only — verify absence of `std::fs`,
  `std::io::Write`, `std::net`, `std::process`.
- **D-04:** `crypto/keychain.rs` is explicitly exempted from the purity constraint.
- **D-05:** ARCH-03 deny-list: `reqwest`, `hyper`, `tokio`, `async-std`, `surf`, `ureq`,
  `attohttpc`, `isahc`, `curl`, `tungstenite`, `websocket`.
- **D-06:** Also deny async runtimes: `tokio`, `async-std`, `smol`.
- **D-07:** Check `[dependencies]` and `[dev-dependencies]` in `Cargo.toml`. No transitive
  checking via `cargo metadata`.
- **D-08:** Three-part failure message format:
  ```
  VIOLATION: [Rule name]
  FOUND: [Specific file, line, or crate]
  HOW TO FIX: [1-2 actionable steps]
  ```
- **D-09:** Messages must name specific files and modules — no abstract descriptions.

### Claude's Discretion

- Exact deny-list contents for network crates
- Whether to use string parsing or a TOML parsing crate for Cargo.toml checks
- Test function naming conventions in `tests/arch.rs`
- Whether to group tests by requirement (ARCH-01, ARCH-02, ARCH-03) or by concern

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ARCH-01 | `tests/arch.rs` exists with structural tests validating module dependency directions and layer boundaries | Layer rule table below; source parsing pattern in Code Examples |
| ARCH-02 | Structural test enforces that `crypto/` module has no filesystem I/O and no side effects | Verified `crypto/mod.rs` is already pure; forbidden import list confirmed |
| ARCH-03 | Structural test verifies no network crates exist in the dependency tree | `toml` crate already available; deny-list confirmed against Cargo.toml |
| OBSV-04 | Structural test failure messages include agent-readable remediation instructions | Three-part message format; `panic!` usage with format string confirmed as idiomatic |
</phase_requirements>

---

## Standard Stack

### Core (already available — no new dependencies needed)

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `std::fs` | stdlib | Read source files and Cargo.toml as strings | Zero-dep; always available in test context |
| `toml` | 1.1.2 | Parse `Cargo.toml` into typed table | Already in `[dependencies]`; `toml::from_str` is the canonical API [VERIFIED: Cargo.toml] |
| `env!("CARGO_MANIFEST_DIR")` | builtin macro | Absolute path to project root at compile time | Standard Rust pattern; portable across machines [ASSUMED] |

### No New Dependencies

`tests/arch.rs` requires zero new crate additions. `toml` is already a production dependency
and is accessible from integration tests automatically.

**Installation:** No `cargo add` needed.

---

## Architecture Patterns

### Recommended File Structure

```
tests/
├── integration.rs       # existing — binary smoke tests
└── arch.rs              # new — structural / static-analysis tests
```

`arch.rs` is a flat list of `#[test]` functions. No sub-modules needed. Suggested grouping:

```
arch_01_*  →  module layer boundary tests
arch_02_*  →  crypto purity tests
arch_03_*  →  network crate prohibition tests
```

### Pattern 1: Read Source File and Scan for Prohibited Imports

**What:** Open a `.rs` source file, collect `use crate::X` lines, assert none match the
prohibited set for that module's layer.

**When to use:** ARCH-01 layer boundary enforcement.

```rust
// Source: codebase observation + stdlib docs [ASSUMED pattern]
fn read_source(rel_path: &str) -> String {
    let root = env!("CARGO_MANIFEST_DIR");
    let path = std::path::Path::new(root).join(rel_path);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("Cannot read {rel_path}: {e}"))
}

fn crate_imports(source: &str) -> Vec<String> {
    source
        .lines()
        .filter(|l| l.trim_start().starts_with("use crate::"))
        .map(|l| l.trim().to_string())
        .collect()
}
```

### Pattern 2: Assert No Prohibited Module Import

**What:** Given a list of `use crate::` lines from a source file, assert none reference a
prohibited module. Fail with a VIOLATION / FOUND / HOW TO FIX message.

```rust
// Source: codebase observation [ASSUMED pattern]
fn assert_no_import(file: &str, source: &str, forbidden_modules: &[&str]) {
    let imports = crate_imports(source);
    for import in &imports {
        for forbidden in forbidden_modules {
            if import.contains(&format!("crate::{forbidden}")) {
                panic!(
                    "\nVIOLATION: Layer boundary — {file} must not import {forbidden}\n\
                     FOUND: {import}\n\
                     HOW TO FIX: Remove or move the import. \
                     {file} is in a lower layer than {forbidden}. \
                     If this logic is needed, move it up to the orchestration layer \
                     (cli.rs or tui/mod.rs).\n"
                );
            }
        }
    }
}
```

### Pattern 3: Parse Cargo.toml with `toml` Crate

**What:** Parse `Cargo.toml`, extract `[dependencies]` and `[dev-dependencies]` table keys,
check each key against a deny-list.

```rust
// Source: toml crate docs [ASSUMED — toml::Value API is stable but not verified via Context7]
fn read_cargo_toml() -> toml::Value {
    let root = env!("CARGO_MANIFEST_DIR");
    let content = std::fs::read_to_string(
        std::path::Path::new(root).join("Cargo.toml")
    ).expect("Cargo.toml must be readable");
    content.parse::<toml::Value>().expect("Cargo.toml must be valid TOML")
}

fn dep_names(cargo: &toml::Value, section: &str) -> Vec<String> {
    cargo
        .get(section)
        .and_then(|t| t.as_table())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default()
}
```

### Pattern 4: Three-Part Failure Message

Every `panic!` in `arch.rs` uses this template. No assertions with default messages.

```
VIOLATION: [rule name — concise, e.g. "Crypto purity — crypto/mod.rs must not import std::fs"]
FOUND: [specific import line or crate name found in violation]
HOW TO FIX: [1-2 sentences. What to remove/change. Where to move it instead.]
```

### Anti-Patterns to Avoid

- **`assert!(condition)` with no message:** Fails silently without VIOLATION/FOUND/HOW TO FIX.
  Use `assert!(condition, "VIOLATION: ...")` or `if !condition { panic!(...) }` instead.
- **Relative file paths:** `"src/crypto/mod.rs"` fails when tests run from a different cwd.
  Always join with `env!("CARGO_MANIFEST_DIR")`.
- **Regex for import detection:** Overkill. `str::contains` and `str::starts_with` on
  trimmed lines is sufficient for well-formatted Rust source.
- **`cargo metadata` for dep checking:** Too heavy. Direct `Cargo.toml` parse is sufficient
  per D-07.
- **Checking `use super::` or `use self::`:** Not relevant to cross-module layer violations.
  Only `use crate::` imports cross module boundaries.

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| TOML parsing | Custom regex/split parser | `toml` crate (already in deps) | Edge cases: inline tables, dotted keys, quoted crate names like `aes-gcm` |
| File path resolution | Hardcoded absolute paths | `env!("CARGO_MANIFEST_DIR")` | Portable; works on any machine, any cwd |
| Module graph analysis | AST walking, syn crate | Source-level string grep | D-02 explicitly chooses simplicity; syn adds significant compile time |

**Key insight:** The entire test infrastructure is stdlib + `toml`. No new crates, no build
scripts, no proc-macros. Simplicity is a first-class constraint from D-02.

---

## Common Pitfalls

### Pitfall 1: Multi-line `use` blocks escape single-line scanning

**What goes wrong:** `use crate::{crypto, config};` on one line is fine, but a multi-line
`use crate::{` block spread over several lines may have the module name on a line that does
not start with `use crate::`.

**Why it happens:** The current codebase does not use multi-line grouped imports — all
`use crate::` statements are single-line. [VERIFIED: codebase grep]

**How to avoid:** Verify once during implementation with `grep -n "use crate::" src/**/*.rs`
that no multi-line blocks exist. The current codebase is clean. Add a comment in `arch.rs`
noting this assumption so future maintainers know.

**Warning signs:** A module that "should" violate a rule but the test doesn't catch it.

### Pitfall 2: `env!("CARGO_MANIFEST_DIR")` resolves to workspace root, not crate root

**What goes wrong:** In a Cargo workspace with multiple crates, `CARGO_MANIFEST_DIR` points
to the workspace root, not the crate directory. `Cargo.toml` at that path would be the
workspace manifest.

**Why it happens:** This project is a single-crate repo (no workspace). [VERIFIED: Cargo.toml]

**How to avoid:** No action needed — single crate confirmed. Still: do not add a workspace
without updating the path logic in `arch.rs`.

### Pitfall 3: Test catches real violations but passes when the violation is removed

**What goes wrong:** A test is written against a real violation. The violation is fixed.
The test now passes — but it may also pass vacuously if the file path was wrong.

**Why it happens:** `read_to_string` panics if the file is missing (correct), so path errors
will surface. But if a violation is introduced in a file the test doesn't scan, it won't
be caught.

**How to avoid:** The ARCH-01 layer table must cover every `src/*.rs` file and every
`src/*/mod.rs` file. Use the verified dependency map from CONTEXT.md `<code_context>` as
the ground truth.

### Pitfall 4: Crate names in Cargo.toml use hyphens, but the deny-list may use underscores

**What goes wrong:** Cargo normalizes `aes-gcm` to `aes_gcm` internally, but `Cargo.toml`
uses hyphens. The `toml` crate returns keys exactly as written in the file — with hyphens.

**Why it happens:** Rust ecosystem has hyphen/underscore duality for crate names.

**How to avoid:** The deny-list in D-05 uses unhyphenated names (`reqwest`, `tokio`, etc.).
All network crates in the deny-list use hyphens in some (e.g., `async-std`, `aes-gcm`) but
the network crate deny-list as given in D-05 is all unhyphenated. Include both hyphenated
and underscore forms for any deny-listed crate that might appear either way (e.g.,
`async-std` and `async_std`).

### Pitfall 5: `profile/switch.rs` imports `crypto` — verify this is allowed

**What goes wrong:** `profile/switch.rs` uses `crate::crypto` (it calls `crypto::encrypt`
and `crypto::decrypt`). The layer rule for Business layer says it CAN import core modules
including `crypto`. This is allowed. A test checking `profile/` must NOT flag `crypto`
as a violation.

**Why it happens:** Confusion between "profile can't import cli/tui/guard" and "profile
can import crypto." [VERIFIED: CONTEXT.md D-01 + codebase grep]

**How to avoid:** The forbidden set for `profile/` files is specifically `{cli, tui, guard}`.
Not `crypto`. Double-check the deny-list for each layer in the test implementation.

---

## Complete Layer Rule Table

Derived from D-01. This is the ground truth for ARCH-01 test implementation.

| File(s) | Layer | Forbidden `crate::` imports |
|---------|-------|-----------------------------|
| `src/error.rs` | Foundation | _(any internal import)_ |
| `src/paths.rs` | Foundation | Any except `error` |
| `src/crypto/mod.rs` | Core | `profile`, `cli`, `tui`, `guard`, `config`, `paths` (only `error` allowed) |
| `src/crypto/keychain.rs` | Core | `profile`, `cli`, `tui`, `guard`, `config`, `paths` (only `error` allowed) |
| `src/config.rs` | Core | `profile`, `cli`, `tui`, `guard`, `crypto` (only `error`, `paths` allowed) |
| `src/guard.rs` | Core | `profile`, `cli`, `tui`, `config`, `crypto`, `paths` (only `error` allowed) |
| `src/profile/mod.rs` | Business | `cli`, `tui`, `guard` |
| `src/profile/store.rs` | Business | `cli`, `tui`, `guard` |
| `src/profile/switch.rs` | Business | `cli`, `tui`, `guard` |
| `src/cli.rs` | Orchestration | _(nothing forbidden)_ |
| `src/tui/mod.rs` | Orchestration | _(nothing forbidden)_ |
| `src/tui/wizard.rs` | Orchestration | _(nothing forbidden)_ |
| `src/tui/widgets.rs` | Orchestration | _(nothing forbidden)_ |

**Note:** Orchestration files have no forbidden imports — skip them in ARCH-01 tests.
Only Foundation, Core, and Business layers need enforcement.

---

## Crypto Purity Verified State

`src/crypto/mod.rs` current imports [VERIFIED: codebase grep]:

```
use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::Rng;
use crate::error::{Result, SubSwapError};
```

No `std::fs`, `std::io`, `std::net`, or `std::process`. The file is currently pure.
ARCH-02 test will verify this stays true.

Forbidden import patterns to check in `crypto/mod.rs`:
- `std::fs`
- `std::io::Write` (or `std::io` broadly)
- `std::net`
- `std::process`

---

## Network Crate Deny-List (ARCH-03)

Final deny-list combining D-05 and D-06, with hyphen variants added:

```rust
const NETWORK_CRATES: &[&str] = &[
    // HTTP clients
    "reqwest", "hyper", "surf", "ureq", "attohttpc", "isahc", "curl",
    // WebSocket
    "tungstenite", "websocket",
    // Async runtimes (imply network capability)
    "tokio", "async-std", "async_std", "smol",
];
```

Current `[dependencies]` in Cargo.toml [VERIFIED: Cargo.toml]:
`clap`, `ratatui`, `crossterm`, `aes-gcm`, `rand`, `keyring`, `sysinfo`, `serde`,
`serde_json`, `toml`, `chrono`, `dirs`

Current `[dev-dependencies]`: `tempfile`, `assert_cmd`, `predicates`

None of these appear in the deny-list. ARCH-03 test will pass immediately on current codebase.

---

## Code Examples

### Complete ARCH-01 test skeleton

```rust
// Source: stdlib + derived from CONTEXT.md D-01 [ASSUMED structure]
#[test]
fn arch_01_foundation_error_has_no_internal_imports() {
    let src = read_source("src/error.rs");
    assert_no_crate_import("src/error.rs", &src, &[
        "paths", "crypto", "config", "guard", "profile", "cli", "tui",
    ]);
}

#[test]
fn arch_01_foundation_paths_imports_only_error() {
    let src = read_source("src/paths.rs");
    assert_no_crate_import("src/paths.rs", &src, &[
        "crypto", "config", "guard", "profile", "cli", "tui",
    ]);
}

#[test]
fn arch_01_core_crypto_imports_only_error() {
    let src = read_source("src/crypto/mod.rs");
    assert_no_crate_import("src/crypto/mod.rs", &src, &[
        "paths", "config", "guard", "profile", "cli", "tui",
    ]);
}

// ... repeat pattern for each Core and Business module
```

### Complete ARCH-02 test skeleton

```rust
// Source: stdlib [ASSUMED structure]
#[test]
fn arch_02_crypto_mod_has_no_filesystem_io() {
    let src = read_source("src/crypto/mod.rs");
    let forbidden = ["std::fs", "std::io::Write", "std::net", "std::process"];
    for pattern in &forbidden {
        if src.contains(pattern) {
            panic!(
                "\nVIOLATION: Crypto purity — crypto/mod.rs must contain only pure functions\n\
                 FOUND: import or use of `{pattern}` in src/crypto/mod.rs\n\
                 HOW TO FIX: Remove the `{pattern}` usage from crypto/mod.rs. \
                 Filesystem and network operations belong in keychain.rs (side-effect layer) \
                 or in the calling module, not in the pure encrypt/decrypt functions.\n"
            );
        }
    }
}
```

### Complete ARCH-03 test skeleton

```rust
// Source: toml crate [ASSUMED API — toml::Value::parse is stable]
#[test]
fn arch_03_no_network_crates_in_dependencies() {
    let root = env!("CARGO_MANIFEST_DIR");
    let content = std::fs::read_to_string(
        std::path::Path::new(root).join("Cargo.toml")
    ).expect("Cargo.toml must exist");
    let cargo: toml::Value = content.parse().expect("Cargo.toml must be valid TOML");

    let deny: &[&str] = &[
        "reqwest", "hyper", "surf", "ureq", "attohttpc", "isahc", "curl",
        "tungstenite", "websocket", "tokio", "async-std", "async_std", "smol",
    ];

    for section in &["dependencies", "dev-dependencies"] {
        if let Some(deps) = cargo.get(section).and_then(|t| t.as_table()) {
            for crate_name in deps.keys() {
                if deny.contains(&crate_name.as_str()) {
                    panic!(
                        "\nVIOLATION: Network-free constraint — sub-swap must have no network or async-runtime crates\n\
                         FOUND: `{crate_name}` in [{section}] of Cargo.toml\n\
                         HOW TO FIX: Remove `{crate_name}` from Cargo.toml [{section}]. \
                         sub-swap is strictly offline. If async behavior is needed, \
                         use synchronous alternatives. See docs/ARCHITECTURE.md for the offline constraint.\n"
                    );
                }
            }
        }
    }
}
```

---

## Validation Architecture

`workflow.nyquist_validation` is absent from `.planning/config.json` — treated as enabled.

### Test Framework

| Property | Value |
|----------|-------|
| Framework | Rust built-in test harness |
| Config file | None — Cargo.toml `[[test]]` auto-discovers `tests/*.rs` |
| Quick run command | `cargo test --test arch` |
| Full suite command | `cargo test` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ARCH-01 | Layer boundary rules enforced | structural (file parse) | `cargo test --test arch arch_01` | No — Wave 0 |
| ARCH-02 | `crypto/mod.rs` is pure — no fs/net/process | structural (file parse) | `cargo test --test arch arch_02` | No — Wave 0 |
| ARCH-03 | No network crates in Cargo.toml | structural (file parse) | `cargo test --test arch arch_03` | No — Wave 0 |
| OBSV-04 | Failure messages include HOW TO FIX | verified by test message format | `cargo test --test arch` | No — Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test --test arch`
- **Per wave merge:** `cargo test`
- **Phase gate:** Full `cargo test` green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `tests/arch.rs` — covers ARCH-01, ARCH-02, ARCH-03, OBSV-04 (entire phase output)
- [ ] No new fixtures or shared helpers needed — tests are self-contained

---

## Environment Availability

Step 2.6: Checked — this phase is pure Rust source writing. No external tools beyond `cargo`.

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo` | All tests | Yes | see `rustup show` | — |
| `toml` crate | ARCH-03 | Yes (in deps) | 1.1.2 | string split (lower quality) |

No missing dependencies.

---

## Security Domain

This phase adds test infrastructure only — no production code changes. ASVS categories
are not applicable. The crypto purity check (ARCH-02) is itself a security control: it
ensures `encrypt`/`decrypt` cannot accidentally introduce side effects that would weaken
the encryption model.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `toml::Value` parses via `.parse::<toml::Value>()` and `.get(section).and_then(toml::Value::as_table)` returns table keys | Code Examples (ARCH-03) | Test would fail to compile — easy to fix by checking toml 1.x API |
| A2 | `env!("CARGO_MANIFEST_DIR")` resolves to `/Users/brainco/security/sub-swap` at test compile time | Architecture Patterns | Paths would be wrong; tests would panic on file read. Single-crate repo makes this safe. |
| A3 | No multi-line `use crate::{ ... }` blocks exist in any source file | Common Pitfalls | Scanner would miss violations in such blocks. Verified by grep — zero occurrences today. |

---

## Open Questions

1. **`src/error.rs` — does it truly have zero `use crate::` imports?**
   - What we know: grep showed no `use crate::` lines in `error.rs` [VERIFIED: codebase grep]
   - What's unclear: none — this is confirmed clean
   - Recommendation: Test can assert empty import list directly

2. **Should ARCH-01 tests for orchestration layer be omitted entirely?**
   - What we know: D-01 says orchestration (`cli`, `tui`) can import anything — no forbidden list
   - What's unclear: Whether a "positive" test (orchestration CAN import profile/crypto) adds value
   - Recommendation: Omit orchestration tests. They would only verify the positive case, which adds no enforcement value. Focus tests on the enforced constraints only.

---

## Sources

### Primary (HIGH confidence)
- [VERIFIED: codebase grep] — All `use crate::` imports across all `src/**/*.rs` files
- [VERIFIED: Cargo.toml] — Current dependency list, `toml` crate version, no network crates present
- [VERIFIED: codebase grep] — `crypto/mod.rs` has no `std::fs`, `std::net`, `std::process` imports
- [VERIFIED: cargo test output] — All existing tests pass; `tests/integration.rs` is the only test file

### Secondary (MEDIUM confidence)
- CONTEXT.md — Layer rules (D-01 through D-09) defined by user decisions; used as ground truth

### Tertiary (LOW confidence / ASSUMED)
- `toml::Value` API shape — based on training knowledge of toml 1.x crate; not verified via Context7

---

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH — everything is already in deps, stdlib only
- Layer rule table: HIGH — derived from verified codebase grep + CONTEXT.md D-01
- Architecture patterns: HIGH — source parsing approach verified against actual source structure
- Code examples: MEDIUM — structure is correct; `toml::Value` API shape is ASSUMED (A1)
- Pitfalls: HIGH — all verified against actual codebase state

**Research date:** 2026-04-07
**Valid until:** 2026-05-07 (stable Rust stdlib; toml 1.x API stable)
