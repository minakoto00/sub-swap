# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Test Commands

```bash
export PATH="$HOME/.cargo/bin:$PATH"  # cargo may not be in default PATH

cargo check                           # fast compile check
cargo build                           # debug build
cargo build --release                 # release binary at target/release/sub-swap
cargo test                            # run all tests (unit + integration)
cargo test --lib                      # unit tests only
cargo test --lib config::tests        # run tests in a specific module
cargo test --lib crypto::tests::test_encrypt_then_decrypt_roundtrip  # single test
cargo test --test integration         # integration tests only
```

```bash
just validate  # fmt + clippy + test in sequence
```

## Key Constraints

- **Strictly offline**: No network crates in dependency tree. No async runtime.
- **File permissions**: All files under `~/.sub-swap/` must be created with mode 0600 (Unix). Check `#[cfg(unix)]` blocks in `config.rs`, `profile/store.rs`, and `profile/switch.rs`.
- **`decrypt` command is view-only**: Prints to stdout, never writes decrypted data to disk.
- **Process guard**: `sub-swap use` checks for running Codex processes via sysinfo and blocks unless `--force` is passed.

## Architecture

sub-swap manages multiple `~/.codex/` profiles (auth.json + config.toml) with AES-256-GCM encryption at rest. Two entry modes (CLI and TUI) share core logic through layered modules. See docs/ for full details.

## Documentation

| File | Purpose |
|------|---------|
| [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md) | Module layout, layer rules, dependency graph, enforcement tests |
| [docs/SECURITY.md](docs/SECURITY.md) | Encryption model, key management, threat model, file permissions |
| [docs/TESTING.md](docs/TESTING.md) | Test infrastructure, mocks, recipes for adding tests |
| [docs/decisions/001-aes-256-gcm.md](docs/decisions/001-aes-256-gcm.md) | ADR: AES-256-GCM for authenticated encryption |
| [docs/decisions/002-os-keychain.md](docs/decisions/002-os-keychain.md) | ADR: OS keychain for encryption key storage |
| [docs/decisions/003-path-injection.md](docs/decisions/003-path-injection.md) | ADR: path injection for test isolation |
| [docs/decisions/004-offline-only.md](docs/decisions/004-offline-only.md) | ADR: strictly offline constraint |
