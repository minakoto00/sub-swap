# ADR-003: Path Injection for Testability

**Status:** Accepted
**Date:** 2026-04-02

## Context

sub-swap reads and writes files in two fixed locations: `~/.codex/` (active profile) and `~/.sub-swap/` (stored profiles, config, index). Functions hardcoding these paths would be untestable — tests would modify the user's real home directory.

## Decision

Inject a `Paths` struct into every function that performs filesystem I/O:
- `Paths` holds `codex_dir` and `sub_swap_dir` fields plus derived methods (`profiles_dir()`, etc.)
- Production: `Paths::new()` resolves to real home directory paths
- Tests: `Paths::from_temp(tmp.path())` maps to a temp directory (`#[cfg(test)]` gated)
- All filesystem functions take `&Paths` as a parameter instead of computing paths internally

## Consequences

**Easier:**
- 100% of filesystem operations are testable without touching the real home directory
- Production and test code share identical function signatures — no conditional compilation in business logic
- Adding new file paths is straightforward (add a method to `Paths`)

**Harder:**
- Every function that does file I/O must accept a `&Paths` parameter (threading it through call chains)
- Tests must manually create subdirectories after calling `from_temp` (it creates the mapping, not the directories)

**Constrained:**
- `Paths::from_temp` is `#[cfg(test)]` gated — cannot be used outside test builds
- `tempfile` crate is a dev-dependency for `TempDir`

## Implementation

- `src/paths.rs` — `Paths` struct with `new()` and `from_temp()` constructors
- Used in: `src/config.rs`, `src/profile/store.rs`, `src/profile/switch.rs`
- Test pattern: `let tmp = TempDir::new().unwrap(); let paths = Paths::from_temp(tmp.path());`
