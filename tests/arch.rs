//! Structural tests enforcing module boundary rules, crypto purity,
//! and the network-free dependency constraint.
//!
//! These tests read source files and Cargo.toml at test-runtime and fail
//! with agent-readable remediation messages (VIOLATION / FOUND / HOW TO FIX).
//!
//! Assumption: All `use crate::` imports are single-line (no multi-line grouped imports).
//! If multi-line grouped imports are introduced, these tests may miss violations.

use std::path::Path;

// ── Deny-list of network and async-runtime crates (ARCH-03) ──────────────────

const NETWORK_CRATES: &[&str] = &[
    "reqwest",
    "hyper",
    "surf",
    "ureq",
    "attohttpc",
    "isahc",
    "curl",
    "tungstenite",
    "websocket",
    "tokio",
    "async-std",
    "async_std",
    "smol",
];

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Read a source file relative to the crate manifest directory.
///
/// Panics with a clear message if the file cannot be read.
fn read_source(rel_path: &str) -> String {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");
    let full_path = Path::new(manifest_dir).join(rel_path);
    std::fs::read_to_string(&full_path).unwrap_or_else(|e| {
        panic!(
            "Could not read source file '{}': {}",
            full_path.display(),
            e
        )
    })
}

/// Assert that `source` contains no `use crate::{forbidden}` imports.
///
/// Scans every line that starts with `use crate::` (after trimming leading whitespace)
/// and panics with a three-part remediation message on the first violation found.
fn assert_no_crate_import(file: &str, source: &str, forbidden_modules: &[&str]) {
    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with("use crate::") {
            continue;
        }
        for forbidden in forbidden_modules {
            // Match `use crate::{forbidden}` or `use crate::{forbidden}::` or
            // `use crate::{forbidden};`
            let prefix_colon = format!("use crate::{forbidden}::");
            let prefix_semi = format!("use crate::{forbidden};");
            let prefix_brace = format!("use crate::{{{forbidden}");
            if trimmed.starts_with(&prefix_colon)
                || trimmed.starts_with(&prefix_semi)
                || trimmed.starts_with(&prefix_brace)
            {
                panic!(
                    "VIOLATION: Layer boundary — {file} must not import {forbidden}\n\
                     FOUND: {trimmed}\n\
                     HOW TO FIX: Remove the `use crate::{forbidden}` import from {file}. \
                     {file} is in a lower layer than {forbidden}. \
                     If this logic is needed, move it to the orchestration layer (cli.rs or tui/) \
                     or restructure the dependency."
                );
            }
        }
    }
}

/// Extract crate names from a TOML section (e.g., `[dependencies]` or `[dev-dependencies]`).
fn dep_names(cargo: &toml::Table, section: &str) -> Vec<String> {
    cargo
        .get(section)
        .and_then(|t| t.as_table())
        .map(|t| t.keys().cloned().collect())
        .unwrap_or_default()
}

// ── ARCH-01: Layer boundary tests ─────────────────────────────────────────────

/// Foundation layer: `error` must have no internal imports at all.
///
/// `error` is the lowest module — nothing internal should flow into it.
#[test]
fn arch_01_foundation_error_has_no_internal_imports() {
    let source = read_source("src/error.rs");
    assert_no_crate_import(
        "src/error.rs",
        &source,
        &[
            "paths", "crypto", "config", "guard", "profile", "cli", "tui",
        ],
    );
}

/// Foundation layer: `paths` may only import `error` — no higher modules.
#[test]
fn arch_01_foundation_paths_imports_only_error() {
    let source = read_source("src/paths.rs");
    assert_no_crate_import(
        "src/paths.rs",
        &source,
        &["crypto", "config", "guard", "profile", "cli", "tui"],
    );
}

/// Core layer: `crypto/mod.rs` may only import `error` — no `paths`, config, guard, or higher.
#[test]
fn arch_01_core_crypto_imports_only_error() {
    let source = read_source("src/crypto/mod.rs");
    assert_no_crate_import(
        "src/crypto/mod.rs",
        &source,
        &["paths", "config", "guard", "profile", "cli", "tui"],
    );
}

/// Core layer: `crypto/keychain.rs` may only import `error` — no `paths`, config, guard, or higher.
#[test]
fn arch_01_core_keychain_imports_only_error() {
    let source = read_source("src/crypto/keychain.rs");
    assert_no_crate_import(
        "src/crypto/keychain.rs",
        &source,
        &["paths", "config", "guard", "profile", "cli", "tui"],
    );
}

/// Core layer: `crypto/passphrase.rs` may only import `error` — no `paths`, config, guard, or higher.
#[test]
fn arch_01_core_passphrase_imports_only_error() {
    let source = read_source("src/crypto/passphrase.rs");
    assert_no_crate_import(
        "src/crypto/passphrase.rs",
        &source,
        &["paths", "config", "guard", "profile", "cli", "tui"],
    );
}

/// Core layer: `config` may only import `error` and `paths` — no profile, cli, tui, guard, crypto.
#[test]
fn arch_01_core_config_imports_only_error_and_paths() {
    let source = read_source("src/config.rs");
    assert_no_crate_import(
        "src/config.rs",
        &source,
        &["profile", "cli", "tui", "guard", "crypto"],
    );
}

/// Core layer: `guard` may only import `error` — no profile, cli, tui, config, crypto, paths.
#[test]
fn arch_01_core_guard_imports_only_error() {
    let source = read_source("src/guard.rs");
    assert_no_crate_import(
        "src/guard.rs",
        &source,
        &["profile", "cli", "tui", "config", "crypto", "paths"],
    );
}

/// Business layer: `profile/mod.rs` must not import orchestration layer.
///
/// `cli`, `tui`, and `guard` are forbidden — profile logic must not call into orchestration.
#[test]
fn arch_01_business_profile_mod_no_orchestration() {
    let source = read_source("src/profile/mod.rs");
    assert_no_crate_import("src/profile/mod.rs", &source, &["cli", "tui", "guard"]);
}

/// Business layer: `profile/store.rs` must not import orchestration layer.
#[test]
fn arch_01_business_profile_store_no_orchestration() {
    let source = read_source("src/profile/store.rs");
    assert_no_crate_import("src/profile/store.rs", &source, &["cli", "tui", "guard"]);
}

/// Business layer: `profile/switch.rs` must not import orchestration layer.
#[test]
fn arch_01_business_profile_switch_no_orchestration() {
    let source = read_source("src/profile/switch.rs");
    assert_no_crate_import("src/profile/switch.rs", &source, &["cli", "tui", "guard"]);
}

// ── ARCH-02: Crypto purity enforcement ───────────────────────────────────────

/// `crypto/mod.rs` must contain only pure functions with no filesystem or network side effects.
///
/// Exemption: `crypto/keychain.rs` is NOT tested here — its purpose is OS keychain I/O.
/// See D-04 in 02-CONTEXT.md.
#[test]
fn arch_02_crypto_mod_has_no_filesystem_io() {
    let source = read_source("src/crypto/mod.rs");
    let forbidden_patterns = ["std::fs", "std::io::Write", "std::net", "std::process"];

    for pattern in &forbidden_patterns {
        assert!(
            !source.contains(pattern),
            "VIOLATION: Crypto purity — crypto/mod.rs must contain only pure functions\n\
             FOUND: import or use of `{pattern}` in src/crypto/mod.rs\n\
             HOW TO FIX: Remove the `{pattern}` usage from crypto/mod.rs. \
             Filesystem and network operations belong in crypto/keychain.rs (side-effect layer) \
             or in the calling module, not in the pure encrypt/decrypt functions."
        );
    }
}

// ── ARCH-03: Network crate prohibition ───────────────────────────────────────

/// sub-swap must have no network or async-runtime crates in any Cargo.toml section.
///
/// Checks both `[dependencies]` and `[dev-dependencies]` against the NETWORK_CRATES deny-list.
/// Does not check transitive dependencies — direct dependencies only.
#[test]
fn arch_03_no_network_crates_in_dependencies() {
    let content = read_source("Cargo.toml");
    let cargo: toml::Table = content.parse().expect("Cargo.toml must be valid TOML");

    for section in &["dependencies", "dev-dependencies"] {
        for crate_name in dep_names(&cargo, section) {
            assert!(
                !NETWORK_CRATES.contains(&crate_name.as_str()),
                "VIOLATION: Network-free constraint — sub-swap must have no network or async-runtime crates\n\
                 FOUND: `{crate_name}` in [{section}] of Cargo.toml\n\
                 HOW TO FIX: Remove `{crate_name}` from Cargo.toml [{section}]. \
                 sub-swap is strictly offline. If HTTP or async behavior is needed, \
                 use synchronous alternatives or reconsider the design constraint."
            );
        }
    }
}
