# Architecture

## Constraints and Invariants

| Rule | Enforced By | Consequence of Violation |
|------|-------------|--------------------------|
| Foundation modules import only `std`/external crates (exception: `paths` may import `error`) | `tests/arch.rs` arch_01 tests (10 tests total across foundation/core/business boundary checks) | `cargo test` fails with VIOLATION/FOUND/HOW TO FIX message |
| Core modules import only Foundation | `tests/arch.rs` arch_01 tests | `cargo test` fails |
| Business modules import Core + Foundation; never Orchestration or `guard` | `tests/arch.rs` arch_01 tests | `cargo test` fails |
| Orchestration modules may import anything | No restriction | N/A |
| `crypto/mod.rs` has no filesystem I/O (`std::fs`, `std::io::Write`, `std::net`, `std::process`) | `tests/arch.rs` arch_02 test | `cargo test` fails |
| `crypto/keychain.rs` is exempt from purity — side effects are its purpose | Documented exemption (D-04 from Phase 2) | N/A |
| Zero network crates in `[dependencies]` or `[dev-dependencies]` | `tests/arch.rs` arch_03 test | `cargo test` fails |
| All `use crate::` imports must be single-line (not multi-line grouped) | Structural test assumption | Multi-line grouped imports bypass boundary detection |

> **WARNING:** `tests/arch.rs` parses `use crate::` statements assuming single-line imports.
> If multi-line grouped imports (e.g., `use crate::{foo, bar}`) are introduced, boundary
> tests may miss violations. See the comment at the top of `tests/arch.rs`.

## Module Layout

```
Orchestration    cli    key    tui/mod    tui/wizard    tui/widgets
                  |      |        |           |              |
Business        profile/mod   profile/store   profile/switch
                  |               |                |
Core           crypto/mod  crypto/keychain  crypto/passphrase  config  guard
                  |               |                |              |      |
Foundation     error                                         paths
```

## Dependency Map

The following table is the verified source of truth for current module import relationships (sourced from Phase 2 architectural analysis, `02-CONTEXT.md §code_context`):

| Module | Imports From | Layer |
|--------|-------------|-------|
| `error` | (nothing internal) | Foundation |
| `paths` | `error` | Foundation |
| `crypto/mod.rs` | `error` | Core |
| `crypto/keychain.rs` | `error` | Core |
| `crypto/passphrase.rs` | `error` | Core |
| `config` | `error`, `paths` | Core |
| `guard` | `error` | Core |
| `profile/mod.rs` | `error` | Business |
| `profile/store.rs` | `error`, `paths`, `profile` | Business |
| `profile/switch.rs` | `crypto`, `error`, `paths`, `profile` | Business |
| `key.rs` | `config`, `crypto`, `error` | Orchestration |
| `cli` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/mod.rs` | `config`, `crypto`, `error`, `guard`, `paths`, `profile` | Orchestration |
| `tui/wizard.rs` | `config`, `crypto`, `error`, `paths`, `profile` | Orchestration |
| `tui/widgets.rs` | `profile` | Orchestration |

## Layer Definitions

### Foundation

**Modules:** `error`, `paths`

**Responsibility:** Provide error types and path abstractions with zero coupling to business logic.

- `error` — Defines `SubSwapError` enum and `Result<T>` type alias. Has no internal imports whatsoever. Every other module depends on `error`, so it must remain at the bottom of the dependency graph.
- `paths` — Defines the `Paths` struct that maps logical file locations (`~/.codex/`, `~/.sub-swap/`) to concrete filesystem paths. May import `error` only.

**Import rule:** Foundation modules may only import `std` and external crates. `paths` may also import `error`. No other internal imports.

**Rationale:** Foundation exists so all other layers have a shared error vocabulary without creating circular dependencies. Keeping it import-free ensures it can be used anywhere without pulling in business logic.

### Core

**Modules:** `crypto/mod.rs`, `crypto/keychain.rs`, `crypto/passphrase.rs`, `config`, `guard`

**Responsibility:** Provide reusable infrastructure that business logic depends on — encryption primitives, key derivation, keychain access, configuration, and process detection.

- `crypto/mod.rs` — Pure AES-256-GCM encrypt/decrypt/key-generation functions. Imports `error` only. No filesystem I/O (enforced by arch_02).
- `crypto/keychain.rs` — OS keychain abstraction (`KeyStore` trait, `OsKeyStore`, `MockKeyStore`). Imports `error` only. Exempt from purity constraint — side effects are its purpose.
- `crypto/passphrase.rs` — Pure Argon2id passphrase derivation and salt metadata helpers. Imports `error` only. No backend selection or I/O; this module exists only to turn passphrase + parameters into key bytes.
- `config` — Application configuration (`AppConfig`). Imports `error`, `paths`.
- `guard` — Process detection abstraction (`CodexGuard` trait, `OsGuard`, `MockGuard`). Imports `error` only.

**Import rule:** Core modules may import Foundation only. Never Business, Orchestration, or each other (except as explicitly listed in the dependency map).

### Business

**Modules:** `profile/mod.rs`, `profile/store.rs`, `profile/switch.rs`

**Responsibility:** Profile lifecycle management — the core product behavior.

- `profile/mod.rs` — Defines `Profile` and `ProfileIndex` data types. Imports `error`.
- `profile/store.rs` — Filesystem I/O for profile files: load, save, detect encryption. Imports `error`, `paths`, `profile`.
- `profile/switch.rs` — The full profile switch lifecycle including backup/restore atomicity. Imports `crypto`, `error`, `paths`, `profile`.

**Import rule:** Business modules may import Core and Foundation. They must never import `cli`, `tui`, or `guard`. This ensures business logic can be tested without the TUI or process detection layers.

### Orchestration

**Modules:** `cli`, `key.rs`, `tui/mod.rs`, `tui/wizard.rs`, `tui/widgets.rs`

**Responsibility:** User-facing entry points and orchestration helpers that choose how lower-level services are combined.

- `cli` — 8 subcommands dispatched via clap derive macros. Thin layer that orchestrates business logic.
- `key.rs` — Backend selection helper shared by CLI/TUI flows. It chooses between native OS key storage and passphrase derivation, but delegates the actual derivation to `crypto/passphrase.rs` and the actual keychain I/O to `crypto/keychain.rs`.
- `tui/mod.rs` — ratatui interactive menu with a 7-screen state machine (`AppScreen` enum). Event loop reads keys, dispatches to per-screen handlers, re-renders.
- `tui/wizard.rs` — First-launch wizard using simple stdin/stdout prompts (not ratatui). Triggered when `profiles.json` does not exist.
- `tui/widgets.rs` — Reusable TUI widget definitions. Imports `profile` only.

**Import rule:** Orchestration modules may import anything. They are the top of the dependency graph and have no import restrictions.

## Boundary Enforcement

`tests/arch.rs` enforces all architectural rules as deterministic `cargo test` failures. Every failure message uses the three-part format:

```
VIOLATION: [Rule name — what boundary was crossed]
FOUND: [Specific file, line, or crate that violates]
HOW TO FIX: [1-2 actionable steps an agent can follow without additional context]
```

### How the tests work

- `read_source(path)` reads the source file at test runtime using `CARGO_MANIFEST_DIR` to locate files relative to the crate root.
- `assert_no_crate_import(file, source, forbidden)` scans each line for `use crate::{forbidden}::`, `use crate::{forbidden};`, or `use crate::{forbidden{` patterns and panics on the first match.
- `dep_names(table)` parses `Cargo.toml` via `toml::Table` to extract dependency crate names from `[dependencies]` and `[dev-dependencies]` sections.

### All 12 structural test functions

| Test | What It Checks |
|------|----------------|
| `arch_01_foundation_error_has_no_internal_imports` | `error` has zero internal imports |
| `arch_01_foundation_paths_imports_only_error` | `paths` cannot import `crypto`, `config`, `guard`, `profile`, `cli`, `tui` |
| `arch_01_core_crypto_imports_only_error` | `crypto/mod.rs` cannot import `paths`, `config`, `guard`, `profile`, `cli`, `tui` |
| `arch_01_core_keychain_imports_only_error` | `crypto/keychain.rs` cannot import `paths`, `config`, `guard`, `profile`, `cli`, `tui` |
| `arch_01_core_passphrase_imports_only_error` | `crypto/passphrase.rs` cannot import `paths`, `config`, `guard`, `profile`, `cli`, `tui` |
| `arch_01_core_config_imports_only_error_and_paths` | `config` cannot import `profile`, `cli`, `tui`, `guard`, `crypto` |
| `arch_01_core_guard_imports_only_error` | `guard` cannot import `profile`, `cli`, `tui`, `config`, `crypto`, `paths` |
| `arch_01_business_profile_mod_no_orchestration` | `profile/mod.rs` cannot import `cli`, `tui`, `guard` |
| `arch_01_business_profile_store_no_orchestration` | `profile/store.rs` cannot import `cli`, `tui`, `guard` |
| `arch_01_business_profile_switch_no_orchestration` | `profile/switch.rs` cannot import `cli`, `tui`, `guard` |
| `arch_02_crypto_mod_has_no_filesystem_io` | `crypto/mod.rs` forbids `std::fs`, `std::io::Write`, `std::net`, `std::process` |
| `arch_03_no_network_crates_in_dependencies` | `Cargo.toml` deny-list check against known network/async-runtime crates |

Note: `crypto/keychain.rs` is exempt from the purity checks in `arch_02` — its purpose is OS keychain I/O and side effects are intentional (see D-04 in Phase 2 context).

## Entry Points

sub-swap has two entry modes that share all core business logic:

**CLI (`cli.rs`):** 8 subcommands dispatched via clap derive macros — `add`, `list`, `use`, `rename`, `delete`, `decrypt`, `config`, `tui`. Each subcommand is a thin orchestration layer that calls into Business and Core modules. Argument parsing is fully type-safe through clap's derive API.

**TUI (`tui/mod.rs`):** An interactive ratatui terminal interface with a 7-screen state machine controlled by the `AppScreen` enum (defined in `tui/widgets.rs`). The event loop reads keyboard events and dispatches to per-screen handler functions, then re-renders on every state change.

**First-launch wizard (`tui/wizard.rs`):** Simple stdin/stdout prompts triggered when `profiles.json` does not exist. This is not a ratatui interface — it uses standard I/O to collect the initial profile name and settings before the full TUI is available.

## External Dependencies

| Crate | Purpose | Why Not Hand-Rolled |
|-------|---------|---------------------|
| `aes-gcm` | AES-256-GCM encryption | Timing-safe, audited; CVE-2023-42811 patched in 0.10.3 |
| `keyring` | OS keychain abstraction | macOS/Linux/Windows portability via a single API |
| `sysinfo` | Process detection | Cross-platform (macOS/Linux/Windows), no shell-out required |
| `clap` | CLI argument parsing | Derive macro generates type-safe subcommand dispatch |
| `ratatui` | Terminal UI framework | Retained-mode state machine rendering |
| `toml` | TOML parsing | Config file reading and Cargo.toml parsing in structural tests |
| `serde`/`serde_json` | Serialization | `profiles.json` index and `auth.json` credential files |
| `rand` | CSPRNG | Nonce generation per encryption call, and key generation |
