# Phase 3: Documentation Knowledge Base - Research

**Researched:** 2026-04-08
**Domain:** Technical documentation writing — Rust CLI tool, agent-legible docs, ADR format
**Confidence:** HIGH

---

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Agent-first documentation — each file leads with constraints, decisions, and boundaries in the first 50 lines. Implementation narrative follows for human readers.
- **D-02:** Use structured sections with clear headers. Tables for quick-reference data. Prose only where rationale needs explaining.
- **D-03:** Create exactly 4 ADRs: AES-256-GCM encryption, OS keychain for key storage, path injection for testability, offline-only constraint.
- **D-04:** ADR format: Title, Status, Context, Decision, Consequences. Each ADR concise (under 100 lines), referencing the specific source files that implement the decision.
- **D-05:** ADR directory structure: `docs/decisions/` with files `001-aes-256-gcm.md`, `002-os-keychain.md`, `003-path-injection.md`, `004-offline-only.md`.
- **D-06:** ARCHITECTURE.md must include: module layout diagram (text-based), dependency graph matching Foundation → Core → Business → Orchestration layer rules, and the specific boundary rules enforced by `tests/arch.rs`.
- **D-07:** Reference the verified dependency map from Phase 2 context (02-CONTEXT.md §code_context) as the source of truth for current module import relationships.
- **D-08:** SECURITY.md must cover: encryption model (AES-256-GCM with nonce|ciphertext|tag format), key management (256-bit key via CSPRNG, hex-encoded in OS keychain), threat model, and the 0600 file permission constraint with rationale.
- **D-09:** Include the profile switch lifecycle as a security-relevant workflow. Note the atomic swap property (backup → write → cleanup on success / restore on failure).
- **D-10:** TESTING.md uses template-based approach — show copyable code patterns for each testing abstraction: `Paths::from_temp(tempdir)`, `MockKeyStore`, `MockGuard`.
- **D-11:** Include a "How to Add a New Test" recipe covering: test location choice, setting up temp paths, mocking external deps, asserting outcomes.
- **D-12:** Document the structural test approach (`tests/arch.rs`) and how to add new architectural boundary rules.
- **D-13:** Each docs/ file is self-contained — no dependency on reading CLAUDE.md first. Some overlap with CLAUDE.md is acceptable until Phase 4.

### Claude's Discretion

- Exact wording and section ordering within each document
- Whether to include Mermaid diagrams or ASCII art for architecture visualization
- Level of detail in threat model (pragmatic scope appropriate for a local-only CLI tool)
- Whether ADRs include "Alternatives Considered" sections or keep to the minimal format

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.
</user_constraints>

---

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| DOCS-01 | `docs/ARCHITECTURE.md` describes module layout, dependency graph, and layer boundaries | Verified full dependency map from 02-CONTEXT.md; all 13 module files inspected |
| DOCS-02 | `docs/SECURITY.md` documents encryption model, key management, threat model, and file permissions | Verified from `crypto/mod.rs`, `crypto/keychain.rs`, `profile/switch.rs`, `profile/store.rs`, design spec |
| DOCS-03 | `docs/TESTING.md` documents test strategy, coverage approach, and how to add new tests | Verified from `paths.rs`, `guard.rs`, `crypto/keychain.rs`, `tests/arch.rs`, `tests/integration.rs` |
| DOCS-04 | `docs/decisions/` contains ADRs for key settled choices | Verified 4 decisions are fully implemented in codebase; ADR content sourced from design spec and source |
</phase_requirements>

---

## Summary

Phase 3 creates four documentation files (ARCHITECTURE.md, SECURITY.md, TESTING.md) and four ADRs under `docs/decisions/`. This is a pure documentation phase — no code changes. The `docs/` directory already exists with a `superpowers/` subdirectory; new files go alongside it at the `docs/` root.

All content is already fully determined by the existing codebase. Every claim in the documentation can be verified by reading source files that have already been inspected. The risk in this phase is quality, not discovery: documents must lead with constraints for agents, use tables over prose, and be self-contained without forward references to files that don't exist yet.

The switch lifecycle in `profile/switch.rs` is more sophisticated than the design spec described — it includes a backup-and-restore mechanism that gives it a true atomic-swap property. SECURITY.md must document this accurately.

**Primary recommendation:** Write all five files in a single task wave. Each file is independent and there are no ordering dependencies between ARCHITECTURE.md, SECURITY.md, TESTING.md, and the ADRs.

---

## Standard Stack

This phase has no library dependencies — it is documentation only. No `npm install` or `cargo add` required.

**Tools used by the planner to write docs:**
- Standard Markdown with GitHub-flavored fenced code blocks
- ASCII art for module diagrams (decision D-02 implies no Mermaid dependency; Claude's discretion allows either)
- Tables for module boundary quick-reference

---

## Architecture Patterns

### Document File Placement

```
docs/
├── ARCHITECTURE.md       # Module layout + layer rules (new)
├── SECURITY.md           # Encryption + key management + threat model (new)
├── TESTING.md            # Test patterns + recipes (new)
├── decisions/            # ADRs (new directory)
│   ├── 001-aes-256-gcm.md
│   ├── 002-os-keychain.md
│   ├── 003-path-injection.md
│   └── 004-offline-only.md
└── superpowers/          # Existing — do not touch
    └── specs/
        └── 2026-04-02-sub-swap-design.md
```

### Pattern: Agent-First Document Structure

Every document opens with a "Constraints & Invariants" block within the first 50 lines. This is the machine-readable section. Human narrative (rationale, examples, history) follows.

**ARCHITECTURE.md structure (order matters):**
1. Constraints & invariants table (layer rules, what is forbidden, enforced by which test)
2. Module layout ASCII diagram
3. Dependency table (verified map from 02-CONTEXT.md)
4. Layer definitions with boundary rules
5. How arch.rs enforces boundaries (human narrative)

**SECURITY.md structure:**
1. Constraints & invariants block (encryption defaults, file permissions, key never on disk)
2. Encryption model (cipher, format, key size)
3. Key management (generation, storage, retrieval, loss)
4. Threat model (what is protected, what is not)
5. Profile switch lifecycle (security-relevant steps including backup/restore)
6. Input validation rules

**TESTING.md structure:**
1. Constraints & invariants (what must be true for every test)
2. Test infrastructure (three abstractions with copyable templates)
3. How to add a new unit test (recipe)
4. How to add a new architectural boundary rule (recipe)
5. Integration test pattern
6. Test location map

**ADR structure (minimal format per D-04):**
```
# ADR-NNN: Title

**Status:** Accepted
**Date:** 2026-04-02

## Context
[Why this decision was needed]

## Decision
[What was decided]

## Consequences
[What becomes easier, harder, or constrained by this choice]

## Implementation
[Source files that implement this decision]
```

---

## Content Inventory (Verified from Source)

This section documents exactly what the planner needs to write each document. All items are `[VERIFIED]` from source file inspection.

### ARCHITECTURE.md Content

**Layer model** [VERIFIED: 02-CONTEXT.md D-01]:
| Layer | Modules | Can Import |
|-------|---------|-----------|
| Foundation | `error`, `paths` | `std` + external crates; `paths` may import `error` |
| Core | `crypto/mod.rs`, `crypto/keychain.rs`, `config`, `guard` | Foundation only |
| Business | `profile/mod.rs`, `profile/store.rs`, `profile/switch.rs` | Core + Foundation; cannot import `cli`, `tui`, `guard` |
| Orchestration | `cli`, `tui/mod.rs`, `tui/wizard.rs`, `tui/widgets.rs` | Anything |

**Full verified dependency map** [VERIFIED: 02-CONTEXT.md §code_context]:
| Module | Imports From | Layer |
|--------|-------------|-------|
| `error` | (nothing internal) | Foundation |
| `paths` | `error` | Foundation |
| `crypto/mod.rs` | `error` | Core |
| `crypto/keychain.rs` | `error` | Core |
| `config` | `error`, `paths` | Core |
| `guard` | `error` | Core |
| `profile/mod.rs` | `error` | Business |
| `profile/store.rs` | `error`, `paths`, `profile` | Business |
| `profile/switch.rs` | `crypto`, `error`, `paths`, `profile` | Business |
| `cli` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/mod.rs` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/wizard.rs` | `config`, `crypto`, `error`, `paths`, `profile` | Orchestration |
| `tui/widgets.rs` | `profile` | Orchestration |

**Structural tests in `tests/arch.rs`** [VERIFIED: tests/arch.rs]:
- `arch_01_foundation_error_has_no_internal_imports` — error has zero internal imports
- `arch_01_foundation_paths_imports_only_error` — paths cannot import core or above
- `arch_01_core_crypto_imports_only_error` — crypto/mod.rs cannot import paths, config, guard, profile, cli, tui
- `arch_01_core_keychain_imports_only_error` — keychain cannot import paths, config, guard, profile, cli, tui
- `arch_01_core_config_imports_only_error_and_paths` — config cannot import profile, cli, tui, guard, crypto
- `arch_01_core_guard_imports_only_error` — guard cannot import profile, cli, tui, config, crypto, paths
- `arch_01_business_profile_mod_no_orchestration` — profile/mod.rs cannot import cli, tui, guard
- `arch_01_business_profile_store_no_orchestration` — same for store.rs
- `arch_01_business_profile_switch_no_orchestration` — same for switch.rs
- `arch_02_crypto_mod_has_no_filesystem_io` — crypto/mod.rs forbids std::fs, std::io::Write, std::net, std::process
- `arch_03_no_network_crates_in_dependencies` — Cargo.toml deny-list check

**Important arch.rs note** [VERIFIED: tests/arch.rs line 8]:
> "Assumption: All `use crate::` imports are single-line (no multi-line grouped imports). If multi-line grouped imports are introduced, these tests may miss violations."
ARCHITECTURE.md must document this limitation.

### SECURITY.md Content

**Encryption model** [VERIFIED: src/crypto/mod.rs]:
- Algorithm: AES-256-GCM (authenticated encryption with associated data)
- Nonce: 96-bit (12 bytes), randomly generated per encryption call via OS CSPRNG (`rand::rng().fill_bytes`)
- Output format: `[12-byte nonce][ciphertext + 16-byte GCM tag]` concatenated
- Each file (auth.json, config.toml) is encrypted independently
- Pure functions: `encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>` and `decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>>`
- Decryption validates minimum length (28 bytes = 12 nonce + 16 tag)

**Key management** [VERIFIED: src/crypto/keychain.rs]:
- Key size: 256-bit (32 bytes), generated via `rand::rng().fill_bytes`
- Storage: hex-encoded (64 chars) in OS keychain via `keyring` crate
- Service name: `"sub-swap"`, account: `"encryption-key"`
- Platform backends: macOS Keychain Services, Linux secret-service (GNOME Keyring / KWallet), Windows Credential Manager
- Key never written to filesystem; lives only in OS keychain and in-memory during operations
- `get_or_default_key`: returns zeroed 32-byte array when encryption is disabled (avoids keychain call)

**File permissions** [VERIFIED: src/profile/store.rs, src/profile/switch.rs, src/config.rs]:
- All files under `~/.sub-swap/` created with mode `0600` (Unix) via `fs::Permissions::from_mode(0o600)`
- Applied to: `profiles.json`, `config.json`, per-profile `auth.json(.enc)`, `config.toml(.enc)`, codex files during switch
- `#[cfg(unix)]` gating — no-op on Windows (Windows has different permission model)
- `profiles.json` itself is not encrypted but is 0600: it contains no secrets (only metadata), but restricts access anyway

**Profile switch lifecycle** [VERIFIED: src/profile/switch.rs]:
The actual implementation is more robust than the design spec. Steps in order:
1. Load `ProfileStore` from `profiles.json`
2. If target is already active → return Ok (no-op)
3. If target doesn't exist in index → return `ProfileNotFound` error
4. Load target profile files from `~/.sub-swap/profiles/<target>/`
5. Detect if encrypted (`.enc` suffix present)
6. Decrypt target files in memory if encrypted
7. Read current active profile from `~/.codex/`
8. Encrypt old active profile files (if `encrypt=true`), write to `~/.sub-swap/profiles/<old>/`
9. **Create backups**: `~/.codex/auth.json.bak` and `~/.codex/config.toml.bak`
10. Write target plaintext to `~/.codex/auth.json` and `~/.codex/config.toml`
11. Set 0600 permissions on codex files (Unix)
12. Update `profiles.json` index: set_active(target), save
13. **On success**: Remove backup files
14. **On any failure in steps 10-12**: Restore from backups, remove backups, return error

The backup/restore mechanism is the atomic-swap property. SECURITY.md should note that there is a brief window (steps 9-10) where both backup and codex files exist simultaneously.

**Input validation** [VERIFIED: src/error.rs `validate_profile_name`]:
- Rules: non-empty, no `/`, `\`, `..`, no leading `.`, characters restricted to alphanumeric + `-` + `_`
- Called at: CLI dispatch (before any operation) and in `add_profile_from_*` functions

**Threat model scope** (for a local-only offline CLI tool):
What IS protected:
- Inactive credentials encrypted at rest with AES-256-GCM
- Encryption key stored in OS keychain, not filesystem
- All files 0600 (Unix owner-only)
- No network attack surface (zero network crates)
- Path traversal blocked by `validate_profile_name`

What is NOT protected (honest scope):
- Active profile credentials in `~/.codex/auth.json` are plaintext (Codex requires this)
- Memory zeroization is not implemented (keys and plaintext exist in heap until GC)
- No protection against root/same-user access
- Key loss means encrypted profiles are unrecoverable (no backup mechanism)
- No protection against malicious code running as the same user

### TESTING.md Content

**Three testing abstractions** [VERIFIED from source]:

**1. `Paths::from_temp`** [VERIFIED: src/paths.rs lines 51-58]:
```rust
// Only available in #[cfg(test)] builds
// Maps base/codex → ~/.codex equivalent
//      base/sub-swap → ~/.sub-swap equivalent
let tmp = TempDir::new().unwrap();
let paths = Paths::from_temp(tmp.path());
// All operations using &paths write to tmp, not the real home dir
```
Note: callers must manually create required subdirectories:
```rust
std::fs::create_dir_all(paths.profiles_dir()).unwrap();
std::fs::create_dir_all(&paths.codex_dir).unwrap();
std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
```

**2. `MockKeyStore`** [VERIFIED: src/crypto/keychain.rs lines 90-123]:
```rust
// Only available in #[cfg(test)] builds
// Backed by RefCell<Option<[u8; 32]>>; starts with no key stored
let store = MockKeyStore::new();
store.set_key(&key).expect("set_key should succeed");
let retrieved = store.get_key().expect("get_key should succeed");
// get_key() before set_key() returns Err (simulates missing keychain entry)
```

**3. `MockGuard`** [VERIFIED: src/guard.rs lines 52-78]:
```rust
// Only available in #[cfg(test)] builds
// Takes a fixed Vec<u32> of PIDs at construction time
let guard_no_process = MockGuard::new(vec![]);      // check() returns Ok
let guard_with_process = MockGuard::new(vec![12345]); // check() returns Err(CodexRunning([12345]))
```

**Integration test pattern** [VERIFIED: tests/integration.rs]:
Uses `env!("CARGO_BIN_EXE_sub-swap")` to get the compiled binary path, then `std::process::Command` to invoke it. Integration tests require `cargo build` first (or run via `cargo test --test integration` which builds automatically).

**Unit test location** [VERIFIED: all source files]:
All unit tests live in `#[cfg(test)] mod tests` blocks within the source file they test. This is standard Rust convention. New tests for `crypto/mod.rs` go in `src/crypto/mod.rs`; new tests for `profile/switch.rs` go in `src/profile/switch.rs`.

**Structural test extension** [VERIFIED: tests/arch.rs]:
- New layer boundary rules: add a new `#[test]` function calling `assert_no_crate_import` with the file and forbidden module list
- New purity checks: add a new `#[test]` function scanning for forbidden string patterns
- New network crates: extend `NETWORK_CRATES` constant at the top of `tests/arch.rs`

### ADR Content Summary

**ADR 001: AES-256-GCM** [VERIFIED: crypto/mod.rs, design spec §4.1]:
- Context: need authenticated encryption for credential files
- Decision: AES-256-GCM with per-operation random 96-bit nonce; output format `[nonce|ciphertext|tag]`
- Consequences: tampering detected (authentication tag), no key-reuse attack (random nonce), unrecoverable without original key
- Implementation: `src/crypto/mod.rs`

**ADR 002: OS Keychain** [VERIFIED: crypto/keychain.rs, design spec §4.2]:
- Context: need to store 256-bit key without putting it on the filesystem
- Decision: `keyring` crate abstracting macOS Keychain, Linux secret-service, Windows Credential Manager; service=`"sub-swap"`, account=`"encryption-key"`
- Consequences: machine-bound key, portable via single crate, no filesystem exposure, loss = unrecoverable encrypted profiles
- Implementation: `src/crypto/keychain.rs`, `KeyStore` trait

**ADR 003: Path Injection** [VERIFIED: paths.rs, all test files]:
- Context: every function that touches `~/.codex/` or `~/.sub-swap/` would be untestable without isolation
- Decision: `Paths` struct injected into every function that performs filesystem I/O; `Paths::from_temp` constructor available in `#[cfg(test)]` for test isolation
- Consequences: 100% of filesystem operations are testable without touching real home directory; production code and test code share the same function signatures
- Implementation: `src/paths.rs`, used throughout `src/config.rs`, `src/profile/store.rs`, `src/profile/switch.rs`

**ADR 004: Offline-Only** [VERIFIED: Cargo.toml, design spec §2, tests/arch.rs]:
- Context: a credential manager has a high-value attack surface; network code dramatically increases it
- Decision: zero network crates in dependency tree; no async runtime; all operations synchronous; `tests/arch.rs` ARCH-03 enforces this via deny-list check on `Cargo.toml`
- Consequences: cross-machine profile sharing is impossible without a future export/import feature; attack surface is limited to local filesystem and OS keychain; `arch_03_no_network_crates_in_dependencies` prevents accidental re-introduction
- Implementation: `Cargo.toml` (absence of network crates), `tests/arch.rs` ARCH-03

---

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Encryption | Custom AES | `aes-gcm` crate | Timing-safe, audited, CVE-2023-42811 patched in 0.10.3 |
| Key storage | Filesystem key file | `keyring` crate | OS keychain provides OS-level access control |
| Process detection | Shell `ps` or `/proc` parsing | `sysinfo` crate | Cross-platform (macOS/Linux/Windows), no shell-out |

This phase is documentation — no new libraries. The above table is included for ARCHITECTURE.md context so readers know why these crates appear in Cargo.toml.

---

## Common Pitfalls

### Pitfall 1: Documenting the Design Spec Instead of the Code
**What goes wrong:** Using `docs/superpowers/specs/2026-04-02-sub-swap-design.md` as the primary source leads to inaccurate documentation. The design spec describes intent; the code describes reality.
**Why it happens:** The design spec is well-organized and easy to quote.
**How to avoid:** Cross-reference every factual claim against the source file. Example: the design spec describes the switch lifecycle as 5 steps; the actual implementation has a backup-and-restore mechanism the spec doesn't mention.
**Warning signs:** Documentation that doesn't mention `auth.json.bak`, `config.toml.bak`, or the backup/restore error path in the switch lifecycle.

### Pitfall 2: Describing MockKeyStore and MockGuard as Always Available
**What goes wrong:** Documentation states agents can use `MockKeyStore::new()` from any code, but it's gated behind `#[cfg(test)]`.
**Why it happens:** The `#[cfg(test)]` attribute is easy to overlook when reading source.
**How to avoid:** TESTING.md must explicitly state that `MockKeyStore`, `MockGuard`, and `Paths::from_temp` are only available in test builds.
**Warning signs:** Code examples that don't show them inside `#[cfg(test)]` contexts.

### Pitfall 3: Omitting the arch.rs Single-Line Import Limitation
**What goes wrong:** ARCHITECTURE.md implies `tests/arch.rs` is a complete boundary enforcer; an agent adds a multi-line grouped import and the test doesn't catch it.
**Why it happens:** The limitation is documented only as a comment at the top of `tests/arch.rs`.
**How to avoid:** ARCHITECTURE.md must call out the single-line-import assumption explicitly.

### Pitfall 4: Asserting `profiles.json` is Plaintext Because It Contains No Secrets
**What goes wrong:** Partially correct — `profiles.json` contains no secrets, but it IS 0600 on Unix. Documentation that says "not encrypted" without noting the 0600 permission is misleading.
**How to avoid:** SECURITY.md should explain both properties: not encrypted (contents are metadata only) AND 0600 restricted (access limited to owner).

### Pitfall 5: Misstating the 0600 Constraint as Universal
**What goes wrong:** SECURITY.md states "all files created with 0600" without noting the `#[cfg(unix)]` gate. On Windows this code is a no-op.
**How to avoid:** Include `#[cfg(unix)]` qualifier whenever the 0600 constraint is stated.

---

## Code Examples

These are exact patterns agents will copy into TESTING.md.

### Minimal Unit Test With Paths Injection
```rust
// Source: src/profile/store.rs tests module (verified pattern)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;
    use tempfile::TempDir;

    #[test]
    fn test_example() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        std::fs::create_dir_all(&paths.codex_dir).unwrap();
        // ... test using &paths
    }
}
```

### Unit Test With MockKeyStore
```rust
// Source: src/crypto/keychain.rs tests module (verified pattern)
#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{generate_key, keychain::MockKeyStore};

    #[test]
    fn test_with_mock_keystore() {
        let store = MockKeyStore::new();
        let key = generate_key();
        store.set_key(&key).expect("set_key should succeed");
        let retrieved = store.get_key().expect("get_key should succeed");
        assert_eq!(retrieved, key);
    }
}
```

### Unit Test With MockGuard
```rust
// Source: src/guard.rs tests module (verified pattern)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guard_blocks_when_codex_running() {
        let guard = MockGuard::new(vec![99999]);
        let result = guard.check();
        assert!(result.is_err());
        match result.unwrap_err() {
            SubSwapError::CodexRunning(pids) => assert_eq!(pids, vec![99999]),
            other => panic!("Expected CodexRunning, got: {:?}", other),
        }
    }
}
```

### Adding a New Arch Boundary Test
```rust
// Source: tests/arch.rs (verified pattern)
// To add a rule: "profile/store.rs must not import tui"
#[test]
fn arch_01_profile_store_does_not_import_tui() {
    let source = read_source("src/profile/store.rs");
    assert_no_crate_import("src/profile/store.rs", &source, &["tui"]);
}
```

---

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in (`cargo test`) |
| Config file | none — uses default cargo test runner |
| Quick run command | `cargo test --lib` |
| Full suite command | `cargo test` |

### Phase Requirements → Test Map

This phase is documentation-only. The deliverables (`.md` files) are not executable.

| Req ID | Behavior | Test Type | Automated Command | Notes |
|--------|----------|-----------|-------------------|-------|
| DOCS-01 | `docs/ARCHITECTURE.md` exists and contains required sections | Manual review | — | File existence: `test -f docs/ARCHITECTURE.md` |
| DOCS-02 | `docs/SECURITY.md` exists and contains required sections | Manual review | — | File existence: `test -f docs/SECURITY.md` |
| DOCS-03 | `docs/TESTING.md` exists and contains required sections | Manual review | — | File existence: `test -f docs/TESTING.md` |
| DOCS-04 | `docs/decisions/` contains 4 ADR files | Shell check | `ls docs/decisions/ \| wc -l` outputs 4 | |

**Verification approach for planner:** The `/gsd-verify-work` step should confirm file existence, check required section headers, and confirm the ADR directory contains exactly 4 files. Content quality is a human review concern.

### Sampling Rate
- **Per task commit:** `cargo test` (ensures docs additions don't accidentally break existing tests — they shouldn't, but this is a safety net)
- **Per wave merge:** `cargo test` (same)
- **Phase gate:** `cargo test` green + manual section-header review before `/gsd-verify-work`

### Wave 0 Gaps
None — no test infrastructure changes needed. Existing `cargo test` suite covers the code being documented.

---

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| `cargo test` | Verifying docs don't break tests | ✓ (per CLAUDE.md) | — | — |

No external tools required to write Markdown files.

---

## Security Domain

This phase creates documentation — it introduces no new attack surface. Security is not applicable as a code-level concern. However, SECURITY.md is itself a security deliverable and must be accurate.

**ASVS categories not applicable:** No authentication, session management, access control, input validation, or cryptographic code is being written. The existing security controls are being documented, not changed.

---

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| A1 | `tui/mod.rs`, `tui/wizard.rs`, `tui/widgets.rs` exist and are within the Orchestration layer | Architecture content | ARCHITECTURE.md layer diagram would be incomplete; low risk — source file list is visible |
| A2 | The `docs/` directory exists at repo root (only `superpowers/` subdirectory visible from `ls`) | File placement | Plans would create a new `docs/` directory instead of adding to existing one |

Note: A2 was confirmed via Bash `ls` output showing `docs/superpowers` exists. New files go alongside `superpowers/` at `docs/` root level.

All other claims in this research were verified by direct source file inspection.

---

## Open Questions

None. All content required for planning is fully determined by the codebase scan.

---

## Sources

### Primary (HIGH confidence)
All findings are VERIFIED by direct source file inspection in this session:
- `src/crypto/mod.rs` — Encryption model, nonce format, output format
- `src/crypto/keychain.rs` — Key management, MockKeyStore implementation
- `src/profile/switch.rs` — Switch lifecycle including backup/restore mechanism
- `src/profile/store.rs` — File I/O, 0600 permissions, load/save patterns
- `src/paths.rs` — Paths struct, from_temp constructor, #[cfg(test)] gate
- `src/guard.rs` — CodexGuard trait, MockGuard implementation
- `src/error.rs` — validate_profile_name rules, SubSwapError variants
- `src/config.rs` — AppConfig, config.json 0600 permissions
- `src/profile/mod.rs` — ProfileIndex, Profile struct
- `src/lib.rs` — Module re-export list
- `tests/arch.rs` — All structural tests, remediation message format, single-line import assumption
- `tests/integration.rs` — Binary invocation test pattern
- `Cargo.toml` — All dependencies and versions
- `.planning/phases/02-architectural-enforcement/02-CONTEXT.md` — Verified dependency map
- `docs/superpowers/specs/2026-04-02-sub-swap-design.md` — Design intent, storage layout, platform table

### Secondary (MEDIUM confidence)
- `.planning/phases/03-documentation-knowledge-base/03-CONTEXT.md` — User decisions
- `.planning/REQUIREMENTS.md` — Requirement definitions

---

## Metadata

**Confidence breakdown:**
- ARCHITECTURE.md content: HIGH — full dependency map verified from two sources (02-CONTEXT.md + source scan)
- SECURITY.md content: HIGH — all claims verified from source files; switch lifecycle is more complete than design spec
- TESTING.md content: HIGH — all three abstractions inspected with their complete implementations
- ADR content: HIGH — all four decisions are fully implemented; implementation files confirmed

**Research date:** 2026-04-08
**Valid until:** Stable indefinitely (documentation of settled decisions; no moving-target libraries)
