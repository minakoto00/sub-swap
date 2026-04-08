# ADR-004: Offline-Only Constraint

**Status:** Accepted
**Date:** 2026-04-02

## Context

sub-swap is a credential manager — it handles API tokens and authentication files. Network-capable code in a credential manager dramatically increases the attack surface: data exfiltration, man-in-the-middle attacks, and unintended telemetry all become possible. The design principle is that a tool managing secrets should have the smallest possible attack surface.

## Decision

Zero network crates in the entire dependency tree:
- No HTTP clients (`reqwest`, `hyper`, `surf`, `ureq`, `attohttpc`, `isahc`, `curl`)
- No WebSocket libraries (`tungstenite`, `websocket`)
- No async runtimes (`tokio`, `async-std`, `smol`) — these imply network capability
- All operations are synchronous
- Enforced by `tests/arch.rs` arch_03 test which deny-lists 13 network/async crates in `[dependencies]` and `[dev-dependencies]`

## Consequences

**Easier:**
- Attack surface is limited to local filesystem and OS keychain
- No network-related CVEs can affect the application
- Simpler mental model — all operations are local and synchronous
- `arch_03_no_network_crates_in_dependencies` prevents accidental re-introduction

**Harder:**
- Cross-machine profile sharing is impossible without a future export/import feature
- No auto-update mechanism
- No remote backup capability

**Constrained:**
- Adding any network crate will cause `cargo test` to fail (arch_03 enforced)
- Overriding requires modifying the `NETWORK_CRATES` constant in `tests/arch.rs` and justifying the addition

## Implementation

- `Cargo.toml` — absence of network crates (enforced by test)
- `tests/arch.rs` — `arch_03_no_network_crates_in_dependencies` test with `NETWORK_CRATES` deny-list
