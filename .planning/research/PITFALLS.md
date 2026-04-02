# Pitfalls Research

**Domain:** Harness engineering infrastructure added to an existing Rust CLI/TUI tool
**Researched:** 2026-04-02
**Confidence:** HIGH

This document covers common mistakes when adding CI/CD, linting, docs, and structural tests
to the sub-swap codebase — an existing Rust project with 43+ passing tests, consistent manual
formatting, encryption-focused code (aes-gcm, keyring), and platform-specific dependencies
(keyring, sysinfo). Each pitfall is specific to this codebase, not generic Rust advice.

---

## Critical Pitfalls

### Pitfall 1: rustfmt.toml imports options break import style silently

**What goes wrong:**
Adding `imports_granularity` or `group_imports` to rustfmt.toml rewrites every import block in
the codebase. The current code uses a deliberate style: standard library imports at the top,
then blank line, then crate internals (`use crate::...`). Setting `group_imports = "StdExternalCrate"`
or `imports_granularity = "Module"` destroys this grouping and produces a mass-reformatting commit
that pollutes `git blame` for every source file.

Additionally, `imports_granularity` combined with `group_imports` is known to produce
non-idempotent output (running `cargo fmt` twice yields different results), which causes the
`cargo fmt --check` CI step to permanently fail.

**Why it happens:**
Developers copy a rustfmt.toml from another project or a blog post without verifying whether the
options reformat the existing code. The options look reasonable in isolation. The breakage only
appears when you run `cargo fmt` the first time.

**How to avoid:**
1. Run `cargo fmt -- --check` (dry run) before committing rustfmt.toml.
2. Restrict rustfmt.toml to options with default-identical behavior for the existing code:
   - `edition = "2021"` — required for correct parse, no reformatting
   - `max_width = 100` — matches existing style
   - `trailing_comma = "Vertical"` — this is already the default, makes it explicit
3. Do NOT set `imports_granularity` or `group_imports` unless you are prepared to accept a
   mass-reformat commit and have verified idempotence.
4. If a mass-reformat commit is acceptable, do it as a standalone commit with message
   `chore: apply rustfmt formatting` so `git blame --ignore-rev` can skip it.

**Warning signs:**
- `cargo fmt` changes more than 5 files on first run
- Running `cargo fmt` twice produces different file content
- Import blocks are reordered in files you did not touch

**Phase to address:** Phase 1 (Code Quality Enforcement) — before CI is wired up

---

### Pitfall 2: keyring CI failures on Linux because there is no keychain daemon

**What goes wrong:**
The `keyring` crate (3.x) on Linux uses the D-Bus secret service (GNOME Keyring). In a standard
GitHub Actions `ubuntu-latest` runner, no D-Bus session and no GNOME Keyring daemon are running.
Any test or integration step that calls `OsKeyStore` will fail with a keyring error, not a test
assertion failure. The error message mentions "org.freedesktop.DBus" or "No such interface",
which looks like a misconfiguration rather than a test failure, making the root cause hard to
diagnose.

This project already works around this at the test level by using `MockKeyStore` for all unit
tests. The risk is in integration tests or new structural tests that instantiate `OsKeyStore`
directly, or any CI step that runs the actual binary against the OS keychain.

**Why it happens:**
On macOS (developer machines), the keychain "just works". The difference only surfaces in CI.
The keyring crate's own CI uses a non-trivial setup script that starts gnome-keyring-daemon with
a known password and a D-Bus session — but this is not obvious from the crate documentation.

**How to avoid:**
1. Keep all tests that touch key storage using `MockKeyStore` — never instantiate `OsKeyStore`
   in test code.
2. In GitHub Actions, restrict keyring-touching integration tests to `macos-latest` only, or
   skip them with `#[cfg_attr(ci, ignore)]` on Linux.
3. If Linux keychain testing is required, add this to the workflow before running tests:
   ```yaml
   - name: Install keyring dependencies (Linux)
     if: runner.os == 'Linux'
     run: |
       sudo apt-get install -y gnome-keyring dbus-x11
       eval $(dbus-launch --sh-syntax)
       echo "test" | gnome-keyring-daemon --unlock
   ```
4. Alternatively, use the `KEYRING_MOCK` environment variable pattern: check for this in
   `main.rs` or wire a `MockKeyStore` when the env var is set, allowing CI to skip the OS
   keychain without changing test structure.

**Warning signs:**
- CI fails with "org.freedesktop.DBus" or "no such interface" errors on `ubuntu-latest`
- Tests pass locally on macOS but fail in CI
- `cargo test` output shows a panic inside the `keyring` crate internals, not in your code

**Phase to address:** Phase 2 (CI/CD) — when writing the GitHub Actions workflow

---

### Pitfall 3: cargo-audit noise from "unmaintained" transitive dependencies

**What goes wrong:**
`cargo audit` (or `cargo deny --check advisories`) will flag transitive dependencies that are
marked "unmaintained" in the RustSec advisory database. In practice, several widely-used crates
(like `instant`, `spin`, older `proc-macro` helpers) accumulate unmaintained advisories even
though they are stable and safe. If the CI step is configured to `--deny warnings`, these
advisories block CI with no actionable fix — the crate is not a security vulnerability, just
unsupported.

This is a documented operational pain point: many GitHub Action templates set `--deny warnings`
by default, turning informational unmaintained notices into hard CI failures for every advisory
that appears in the RustSec database, including in crates deep in the dependency tree.

**Why it happens:**
The distinction between "unmaintained advisory" and "security advisory" is not obvious. CI
templates copy-paste `--deny all` or `--deny warnings` without considering that unmaintained
advisories are common and often irrelevant to a project's security posture.

**How to avoid:**
1. Use `cargo deny` with a `deny.toml` that explicitly separates vulnerability handling from
   unmaintained handling:
   ```toml
   [advisories]
   vulnerability = "deny"
   unmaintained = "warn"   # not "deny"
   unsound = "deny"
   notice = "warn"
   ignore = []
   ```
2. Never use `cargo audit --deny warnings` in CI without the above distinction.
3. Review and document any advisories you ignore by adding them to the `ignore` array with a
   comment explaining the reason.
4. Use `cargo deny check` not `cargo audit` for new projects — cargo-deny provides finer control
   and is more actively maintained as of 2025.

**Warning signs:**
- CI audit step fails immediately after adding it, before any code changes
- Failing advisory ID starts with "RUSTSEC-" but the crate is not a direct dependency
- `cargo tree -i <crate>` shows the flagged crate is pulled in 3+ levels deep

**Phase to address:** Phase 2 (CI/CD) — when configuring the security audit step

---

### Pitfall 4: Clippy pedantic lints create noise in encryption code and test utilities

**What goes wrong:**
Enabling `clippy::pedantic` in `clippy.toml` or `Cargo.toml [lints]` fires on several patterns
that are intentional in this codebase:

- `clippy::missing_errors_doc` fires on every `pub fn` that returns `Result<T>` without an
  `# Errors` section in its rustdoc comment. The crypto module has several of these
  (`encrypt`, `decrypt`, `generate_key`) where the error condition is obvious from the type
  signature and documentation would add noise, not value.
- `clippy::missing_panics_doc` fires on test helper functions that use `unwrap()`, including
  all 43+ existing tests. With `check-private-items = true`, this fires inside `#[cfg(test)]`
  blocks even though test functions do not benefit from panic documentation.
- `clippy::must_use_candidate` fires on pure functions like `generate_key() -> [u8; 32]` and
  `encode_key()` — these are used consistently, but the lint considers them "should be
  `#[must_use]`" because they return a value.
- `clippy::needless_pass_by_value` fires on some function signatures in the trait
  implementations where the current ownership semantics are intentional for API clarity.

Enabling pedantic and treating it as `deny` will immediately break the build with 15-30 new
lint failures, none of which represent bugs.

**Why it happens:**
Pedantic is intended to be used with `#[allow]` sprinkled throughout the code. Enabling it at
the `deny` level in `Cargo.toml` without first auditing each lint creates a hard failure for
every existing function signature.

**How to avoid:**
1. Enable clippy pedantic lints at `warn` level first, not `deny`:
   ```toml
   [lints.clippy]
   pedantic = { level = "warn", priority = -1 }
   ```
2. Audit each warning class before escalating it to `deny`. For this codebase, the following
   should be explicitly allowed at the crate level in `lib.rs` or `main.rs`:
   ```rust
   #![allow(clippy::missing_errors_doc)]    // crypto API: error conditions are obvious
   #![allow(clippy::missing_panics_doc)]    // test utilities: not public API
   ```
3. For `must_use_candidate`, evaluate each case: add `#[must_use]` where it genuinely helps
   callers, or add `#[allow]` with a comment where ownership is intentional.
4. Do not use `clippy::pedantic = "deny"` as the initial configuration; earn it incrementally.

**Warning signs:**
- `cargo clippy -- -D clippy::pedantic` produces more than 10 new warnings
- Warnings are clustered in `crypto/mod.rs` and test helper functions
- Lint errors reference missing doc sections on functions that have no public callers

**Phase to address:** Phase 1 (Code Quality Enforcement) — when writing clippy.toml

---

### Pitfall 5: Cargo.toml `[lints]` priority interaction silently disables individual lints

**What goes wrong:**
The `[lints]` table in Cargo.toml (stable since Rust 1.74) uses lexicographic ordering when
multiple lint groups overlap. If you enable `clippy::pedantic = "warn"` and then try to
`deny` a specific pedantic lint (e.g., `clippy::unwrap_used`), the group-level setting may
override the individual setting unless you set a lower `priority` on the group.

The result is that `clippy::unwrap_used = "deny"` appears to be set but is never enforced
because the group `pedantic = { level = "warn", priority = 0 }` comes before it alphabetically
and the individual lint has no explicit priority.

This is a confirmed Clippy issue (rust-lang/rust-clippy#11237) where `[lints.clippy]` from
Cargo.toml is not obeyed for some lints when group membership causes ordering conflicts.

**Why it happens:**
The priority mechanic is not obvious from the Cargo.toml documentation. Most examples show
simple lint settings without priority, which works for single lints but breaks for the
group-then-override pattern.

**How to avoid:**
Use explicit priorities when mixing group-level and individual lint settings:
```toml
[lints.clippy]
pedantic = { level = "warn", priority = -1 }   # lower priority than individual lints
unwrap_used = "deny"                            # overrides pedantic's warn for this lint
```
The priority field is documented in RFC 3389. Group settings should always have negative
priority if individual lint overrides are needed.

**Warning signs:**
- A lint you explicitly set to `deny` is not reported even on code that violates it
- Running `cargo clippy` produces no output but `cargo clippy --message-format json` shows the
  lint is registered with a different level
- Adding `#![deny(clippy::some_lint)]` to source code does enforce it but the Cargo.toml
  setting does not

**Phase to address:** Phase 1 (Code Quality Enforcement) — when writing Cargo.toml lints

---

## Technical Debt Patterns

Shortcuts that seem reasonable but create long-term problems.

| Shortcut | Immediate Benefit | Long-term Cost | When Acceptable |
|----------|-------------------|----------------|-----------------|
| `#![allow(clippy::all)]` at crate root | Silences all clippy | No enforcement — defeats the purpose | Never |
| Mass `#[allow(clippy::X)]` on every function | Unblocks CI quickly | Makes lints meaningless; hides real future issues | Only for genuinely noisy pedantic lints with a comment explaining why |
| Structural tests that assert specific file names exist | Simple to write | Brittle — any rename breaks the test without a logical regression | Never for file names; use module path assertions instead |
| Copying rustfmt.toml from another project without review | Fast | May cause mass reformatting diff that pollutes git blame permanently | Never without running `cargo fmt -- --check` first |
| Setting `unmaintained = "deny"` in cargo-deny | Appears thorough | Causes CI to fail on every new RustSec advisory regardless of risk | Never — use `"warn"` for unmaintained, `"deny"` for vulnerabilities only |
| Writing docs/ content as prose paragraphs describing what the code does | Feels thorough | Docs drift from code immediately; agent reads outdated information | Never — docs should describe WHY and constraints, not WHAT the code does |

---

## Integration Gotchas

Common mistakes when connecting the harness to this codebase's external dependencies.

| Integration | Common Mistake | Correct Approach |
|-------------|----------------|------------------|
| `keyring` in GitHub Actions Linux | Running any test that calls `OsKeyStore` on ubuntu-latest | All tests use `MockKeyStore`; restrict OS keychain tests to macos-latest |
| `sysinfo` in GitHub Actions | Running process-detection tests in parallel (hits test-limit race conditions) | Mark `OsGuard` tests `#[ignore]` in CI; `MockGuard` already covers the logic |
| `cargo deny` advisories | Using `--all` deny level causing unmaintained crate blocks | Separate `vulnerability = "deny"` from `unmaintained = "warn"` in deny.toml |
| `rustfmt --check` in CI | Adding after reformatting, then CI fails on PRs that don't reformat | Run mass reformat commit first, then enable `--check` CI step |
| `cargo doc` with `deny(missing_docs)` | Enabling globally; fires on private items and test helpers | Use `#![warn(missing_docs)]` not `deny`, apply only to pub API modules |

---

## Performance Traps

Not applicable at this scale (single-binary CLI, no server workload). Skipped intentionally.

---

## Security Mistakes

Domain-specific mistakes beyond general security hygiene.

| Mistake | Risk | Prevention |
|---------|------|------------|
| Structural test that reads profile files to check encryption status | Test leaks encrypted key material into test output/logs if assertion fails | Use `profile_is_encrypted()` predicate (byte-level check), never assert on decrypted content |
| Adding `KEYRING_MOCK=1` env var check in production code paths | Allows disabling encryption in production via environment | Mock injection must be `#[cfg(test)]` only; production code should never branch on env vars for security features |
| Running `cargo audit` without pinning the RustSec advisory database version | CI fails intermittently on new unmaintained advisories not related to security | Pin advisory DB or use `unmaintained = "warn"` in cargo-deny |
| Documenting the encryption key format (hex-encoded 32 bytes in keychain) in public docs | Explains to an attacker exactly what to look for in the keychain | Key storage implementation detail belongs in internal docs only, never in README or public ARCHITECTURE.md |

---

## UX Pitfalls

These apply to developer experience (DX) of the harness infrastructure itself.

| Pitfall | Developer Impact | Better Approach |
|---------|-----------------|-----------------|
| CI workflow with no job names, only step names | Hard to find failing step in GitHub Actions UI | Give every job a descriptive `name:` field |
| Structural tests with assertion messages that say "assert failed" | Developer must read the test code to understand what boundary was violated | Every `assert!` in structural tests must include a message: `assert!(condition, "explanation of what architectural rule was violated")` |
| CLAUDE.md map that points to docs/ files that don't exist yet | Agent reads the map, follows the pointer, gets 404, falls back to training data | Create stub files with a one-line description before adding pointers to CLAUDE.md |
| docs/ files that describe the current implementation rather than the constraints and decisions | Agent re-implements things differently because the "why" is missing | Each docs/ file must lead with constraints and decisions, not with code walkthrough |

---

## "Looks Done But Isn't" Checklist

Things that appear complete but are missing critical pieces.

- [ ] **rustfmt.toml added:** Verify `cargo fmt -- --check` passes on ALL files without changes — don't just check that the file was created
- [ ] **clippy.toml added:** Verify `cargo clippy` produces zero warnings, not just zero errors — warnings become future debt
- [ ] **GitHub Actions workflow added:** Verify the workflow actually runs (push a commit with a deliberate fmt error) — YAML syntax errors silently skip jobs
- [ ] **cargo-deny configured:** Verify `cargo deny check` passes, not just that deny.toml exists — the configuration must be valid TOML with correct section names
- [ ] **Structural tests added:** Verify each test fails when the boundary is violated (temporarily move a module) — a test that never fails is not a test
- [ ] **CLAUDE.md restructured:** Verify every pointer in the new map leads to an existing file — a dead link in CLAUDE.md causes agent to ignore the section entirely
- [ ] **docs/ files created:** Verify each docs/ file answers "why" not just "what" — "what" is already in the code

---

## Recovery Strategies

When pitfalls occur despite prevention, how to recover.

| Pitfall | Recovery Cost | Recovery Steps |
|---------|---------------|----------------|
| Mass rustfmt reformat committed on a feature branch | LOW | Rebase interactively to isolate the reformat commit; use `git blame --ignore-rev <sha>` going forward |
| CI keyring failure on Linux | LOW | Add `if: runner.os != 'Linux'` to the affected job, or install gnome-keyring in the workflow |
| cargo-deny blocks CI on unmaintained advisory | LOW | Change `unmaintained = "deny"` to `unmaintained = "warn"` in deny.toml; add the specific advisory ID to `ignore` array with comment |
| Clippy pedantic produces 30+ warnings after enabling | MEDIUM | Revert to `warn` level, run `cargo clippy 2>&1 | grep warning | sort | uniq -c | sort -rn` to categorize, then allow the top 2-3 lint categories at crate level with explanatory comments |
| Structural test fails on every rename | MEDIUM | Replace file-path assertions with module-path or trait-existence assertions; structural tests should test interfaces and boundaries, not file layout |
| CLAUDE.md restructure loses build commands | HIGH | Restore the Build & Test Commands section verbatim in the new CLAUDE.md map — this is the highest-ROI section for an agent first working in the repo; never defer it to a child file |

---

## Pitfall-to-Phase Mapping

How roadmap phases should address these pitfalls.

| Pitfall | Prevention Phase | Verification |
|---------|------------------|--------------|
| rustfmt.toml import options breaking style | Phase 1: Code Quality Enforcement | `cargo fmt -- --check` passes with zero diffs on all files |
| Cargo.toml `[lints]` priority interaction | Phase 1: Code Quality Enforcement | Introduce a deliberate `unwrap_used` violation, confirm `cargo clippy` catches it |
| Clippy pedantic noise in crypto code | Phase 1: Code Quality Enforcement | `cargo clippy -- -W clippy::pedantic` produces zero warnings or only allowed categories |
| keyring CI failures on Linux | Phase 2: CI/CD | CI green on both ubuntu-latest and macos-latest |
| sysinfo parallel test race | Phase 2: CI/CD | `cargo test` with `--test-threads=1` or ignored OS-level tests passes reliably |
| cargo-audit unmaintained advisory noise | Phase 2: CI/CD | `cargo deny check` passes without manual ignores of security advisories |
| Structural tests that are too brittle | Phase 3: Structural Tests | Temporarily rename a module, confirm exactly one structural test fails, then revert |
| CLAUDE.md restructure losing context | Phase 4: CLAUDE.md Restructure | Run a fresh agent session, ask it to find the build command — it must succeed in under 2 reads |
| docs/ files describing "what" not "why" | Phase 4: Documentation | Each docs/ file reviewed for at least one "decision" entry explaining a constraint that is not obvious from code |

---

## Sources

- [Rustfmt Configurations reference](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md) — stable vs unstable options, imports defaults
- [rustfmt non-idempotent imports issue](https://github.com/rust-lang/rustfmt/issues/6195) — imports_granularity + group_imports bug
- [Clippy false-positive label tracker](https://github.com/rust-lang/rust-clippy/labels/I-false-positive) — known false positive categories
- [missing_panics_doc fires in test functions](https://github.com/rust-lang/rust-clippy/issues/12265) — check-private-items + tests
- [Cargo.toml lints not obeyed issue](https://github.com/rust-lang/rust-clippy/issues/11237) — priority interaction bug
- [RFC 3389: manifest lint table](https://rust-lang.github.io/rfcs/3389-manifest-lint.html) — priority field documentation
- [keyring-rs Linux headless CI](https://github.com/open-source-cooperative/keyring-rs) — their CI uses gnome-keyring unlock workaround
- [actions/runner-images gnome-keyring IPC_LOCK issue](https://github.com/actions/runner-images/issues/6683) — known Linux CI limitation
- [cargo-deny advisories config](https://embarkstudios.github.io/cargo-deny/checks/advisories/cfg.html) — unmaintained vs vulnerability separation
- [Writing a good CLAUDE.md](https://www.humanlayer.dev/blog/writing-a-good-claude-md) — context loss patterns in restructuring
- [Stop bloating your CLAUDE.md](https://alexop.dev/posts/stop-bloating-your-claude-md-progressive-disclosure-ai-coding-tools/) — progressive disclosure anti-patterns
- [OpenAI harness engineering](https://openai.com/index/harness-engineering/) — AGENTS.md as map pattern

---
*Pitfalls research for: harness engineering infrastructure addition to existing Rust CLI (sub-swap)*
*Researched: 2026-04-02*
