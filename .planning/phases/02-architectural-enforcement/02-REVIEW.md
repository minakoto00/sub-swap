---
phase: 02-architectural-enforcement
reviewed: 2026-04-07T00:00:00Z
depth: standard
files_reviewed: 1
files_reviewed_list:
  - tests/arch.rs
findings:
  critical: 0
  warning: 3
  info: 2
  total: 5
status: issues_found
---

# Phase 02: Code Review Report

**Reviewed:** 2026-04-07
**Depth:** standard
**Files Reviewed:** 1
**Status:** issues_found

## Summary

`tests/arch.rs` implements three structural enforcement test suites (ARCH-01 layer boundaries, ARCH-02 crypto purity, ARCH-03 network crate prohibition). The overall design is sound and the test scaffolding is clear. However, there are three correctness gaps that could allow real violations to slip past the tests undetected, and two informational issues worth noting.

The most significant issue is that `assert_no_crate_import` only matches a forbidden module when it is the _first_ item in a brace-grouped import — a single-line multi-item import like `use crate::{paths, cli}` would not be caught for the second and later items. The ARCH-02 purity check uses `str::contains` against raw source text, making it vulnerable to false negatives (pattern in a string literal) and false positives (pattern in a comment). The ARCH-03 deny-list has notable gaps for the strictly-offline constraint.

---

## Warnings

### WR-01: `assert_no_crate_import` misses non-first items in single-line brace groups

**File:** `tests/arch.rs:57-76`

**Issue:** The three match patterns checked per forbidden module are:
- `use crate::{forbidden}::` — catches `use crate::cli::Foo`
- `use crate::{forbidden};` — catches `use crate::cli;`
- `use crate::{{{forbidden}` — catches `use crate::{cli` only when `cli` is the first item in the brace list

A single-line import such as `use crate::{paths, cli};` would pass all three checks for `cli` because the line starts with `use crate::{paths` — not `use crate::{cli`. This is a real gap: the layer boundary enforcement would silently miss the violation. The file's own comment on line 7 acknowledges the multi-line case but does not address this single-line multi-item case.

**Fix:** Replace the `prefix_brace` heuristic with a check that splits the brace contents and tests each item individually:

```rust
fn assert_no_crate_import(file: &str, source: &str, forbidden_modules: &[&str]) {
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        if !trimmed.starts_with("use crate::") {
            continue;
        }
        // Collect all imported module names from the line.
        // Handles: `use crate::foo;`, `use crate::foo::Bar;`,
        //          `use crate::{foo, bar, baz::Qux};`
        let after_crate = &trimmed["use crate::".len()..];
        let modules_in_line: Vec<&str> = if after_crate.starts_with('{') {
            // Brace group: extract contents between { and }
            after_crate
                .trim_start_matches('{')
                .split('}')
                .next()
                .unwrap_or("")
                .split(',')
                .map(|s| s.trim().split("::").next().unwrap_or(""))
                .collect()
        } else {
            // Simple: `use crate::foo` or `use crate::foo::Bar`
            vec![after_crate.split("::").next().unwrap_or("")]
        };

        for forbidden in forbidden_modules {
            if modules_in_line.contains(forbidden) {
                panic!(
                    "VIOLATION: Layer boundary — {file}:{} must not import {forbidden}\n\
                     FOUND: {trimmed}\n\
                     HOW TO FIX: Remove the `use crate::{forbidden}` import from {file}.",
                    line_num + 1
                );
            }
        }
    }
}
```

---

### WR-02: ARCH-02 crypto purity check uses raw `str::contains` — fragile against comments and string literals

**File:** `tests/arch.rs:193-203`

**Issue:** The check `source.contains(pattern)` scans the full raw source text including comments and string literals. This creates two failure modes:

1. **False positive:** A comment in `crypto/mod.rs` such as `// do not use std::fs here` would cause the test to fail spuriously, even though no actual import or usage exists.
2. **False negative:** A forbidden API accessed through a type alias or re-export would not appear verbatim in the source.

The `std::io::Write` pattern is particularly fragile: `std::fmt::Write` is already used in `crypto/keychain.rs` (line 9 — `use std::fmt::Write`) and while that is a different file, future refactors could introduce a similar `use` in `crypto/mod.rs` that the test would conflate with `std::io::Write` if the patterns were ever adjusted incorrectly.

**Fix:** Restrict the check to `use` statement lines only, mirroring the approach used in `assert_no_crate_import`:

```rust
fn assert_no_stdlib_side_effect_import(file: &str, source: &str, forbidden: &[&str]) {
    for (line_num, line) in source.lines().enumerate() {
        let trimmed = line.trim();
        // Only inspect `use` lines and direct qualified access patterns,
        // skip doc comments and inline comments
        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            continue;
        }
        for pattern in forbidden {
            if trimmed.contains(pattern) {
                panic!(
                    "VIOLATION: Crypto purity — {file} must contain only pure functions\n\
                     FOUND: `{pattern}` at {file}:{}\n\
                     HOW TO FIX: Remove the `{pattern}` usage.",
                    line_num + 1
                );
            }
        }
    }
}
```

---

### WR-03: ARCH-03 deny-list has gaps — several offline-violating crates are not covered

**File:** `tests/arch.rs:14-28`

**Issue:** The `NETWORK_CRATES` deny-list covers HTTP clients and major async runtimes but omits several crates that would also violate the strictly-offline constraint:

- `futures` / `futures-util` — async primitives that pair with executors
- `mio` — low-level async I/O (often pulled transitively, but a direct dep would be a red flag)
- `axum`, `warp`, `rocket`, `actix-web`, `actix-rt` — server frameworks
- `h2`, `h3`, `http` — HTTP protocol crates
- `rustls`, `native-tls` — TLS libraries (no legitimate use in an offline tool)
- `trust-dns`, `hickory-dns` — DNS resolver crates

If any of these were accidentally added as a direct dependency, the test would not catch it.

**Fix:** Extend the deny-list:

```rust
const NETWORK_CRATES: &[&str] = &[
    // existing entries ...
    "futures",
    "futures-util",
    "mio",
    "axum",
    "warp",
    "rocket",
    "actix-web",
    "actix-rt",
    "h2",
    "h3",
    "http",
    "rustls",
    "native-tls",
    "trust-dns-resolver",
    "hickory-dns",
];
```

---

## Info

### IN-01: `arch_01_core_crypto_imports_only_error` does not cover `crypto/keychain.rs` importing `crypto` (self-referential sibling)

**File:** `tests/arch.rs:126-135`

**Issue:** `arch_01_core_keychain_imports_only_error` (line 127) correctly forbids `keychain.rs` from importing higher-layer modules. However, `keychain.rs` tests (line 130 in `crypto/keychain.rs`) import `use crate::crypto::generate_key` — a sibling module call. This is architecturally sound (sibling within the same module tree), but it is worth noting this pattern is intentionally excluded from the boundary check by design.

No action required unless the architecture document explicitly prohibits intra-`crypto/` sibling imports. Document the intent if it is not already captured.

---

### IN-02: Line numbers are not reported in `assert_no_crate_import` violation messages

**File:** `tests/arch.rs:67-73`

**Issue:** When a violation is found, the panic message shows the full violating line text (`FOUND: {trimmed}`) but does not include the line number within the file. For large source files this makes it harder to locate the violation quickly.

**Fix:** Track line numbers using `enumerate()` and include them in the message:

```rust
for (line_num, line) in source.lines().enumerate() {
    // ...
    panic!(
        "VIOLATION: Layer boundary — {file}:{line} must not import {forbidden}\n\
         FOUND: {trimmed}\n\
         HOW TO FIX: ...",
        line = line_num + 1,
    );
}
```

This aligns with the remediation message design goal stated in the module-level doc comment (line 4).

---

_Reviewed: 2026-04-07_
_Reviewer: Claude (gsd-code-reviewer)_
_Depth: standard_
