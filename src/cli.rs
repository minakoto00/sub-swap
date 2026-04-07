use std::path::Path;

use clap::{Parser, Subcommand};

use crate::config::AppConfig;
use crate::crypto;
use crate::crypto::keychain::{get_or_default_key, OsKeyStore};
use crate::error::{validate_profile_name, Result, SubSwapError};
use crate::guard::{CodexGuard, OsGuard};
use crate::paths::Paths;
use crate::profile::store::ProfileStore;
use crate::profile::switch;

// ── CLI types ─────────────────────────────────────────────────────────────────

#[derive(Parser)]
#[command(
    name = "sub-swap",
    version,
    about = "Manage multiple ~/.codex/ profiles"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// List all profiles
    List {
        #[arg(short, long)]
        verbose: bool,
    },
    /// Switch to a profile
    Use {
        name: String,
        #[arg(short, long)]
        force: bool,
    },
    /// Import current ~/.codex/ config as a new profile
    Add {
        name: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(short, long)]
        note: Option<String>,
    },
    /// Delete a stored profile
    Remove { name: String },
    /// Rename a profile
    Rename { old: String, new: String },
    /// Set or update a profile's note
    Note { name: String, text: String },
    /// View decrypted profile contents (stdout only)
    Decrypt { name: String },
    /// Manage global settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },
}

#[derive(Subcommand)]
pub enum ConfigAction {
    /// Set a config value
    Set { key: String, value: String },
    /// Show current config
    Show,
}

// ── Entry point ───────────────────────────────────────────────────────────────

pub fn run(cli: Cli) -> Result<()> {
    let paths = Paths::new()?;

    match cli.command {
        None => {
            // No subcommand: first-launch wizard if not initialized, else TUI
            if paths.profiles_json().exists() {
                crate::tui::run_tui(&paths)?;
            } else {
                crate::tui::wizard::run_first_launch(&paths)?;
            }
        }
        Some(Commands::List { verbose }) => cmd_list(&paths, verbose)?,
        Some(Commands::Use { name, force }) => cmd_use(&paths, &name, force)?,
        Some(Commands::Add { name, from, note }) => cmd_add(&paths, &name, from.as_deref(), note)?,
        Some(Commands::Remove { name }) => cmd_remove(&paths, &name)?,
        Some(Commands::Rename { old, new }) => cmd_rename(&paths, &old, &new)?,
        Some(Commands::Note { name, text }) => cmd_note(&paths, &name, &text)?,
        Some(Commands::Decrypt { name }) => cmd_decrypt(&paths, &name)?,
        Some(Commands::Config { action }) => cmd_config(&paths, action)?,
    }

    Ok(())
}

// ── Command implementations ───────────────────────────────────────────────────

fn cmd_list(paths: &Paths, verbose: bool) -> Result<()> {
    let store = ProfileStore::load(paths)?;
    let active = store.index.active_profile.as_deref();

    if store.index.profiles.is_empty() {
        println!("No profiles stored. Use `sub-swap add <name>` to create one.");
        return Ok(());
    }

    for name in store.index.names() {
        let marker = if active == Some(name) { "*" } else { " " };
        if verbose {
            let profile = store.index.get(name).unwrap();
            let last_used = profile.last_used.as_deref().unwrap_or("never");
            let note = profile.notes.as_deref().unwrap_or("");
            println!("{marker} {name}  (last used: {last_used})  {note}");
        } else {
            let profile = store.index.get(name).unwrap();
            let notes = profile.notes.as_deref().unwrap_or("");
            println!("{marker} {name:<16} {notes}");
        }
    }

    Ok(())
}

fn cmd_use(paths: &Paths, name: &str, force: bool) -> Result<()> {
    validate_profile_name(name)?;
    let store = ProfileStore::load(paths)?;

    // Already active — no-op
    if store.index.active_profile.as_deref() == Some(name) {
        println!("Profile '{name}' is already active.");
        return Ok(());
    }

    // Validate the profile exists before doing anything else
    if store.index.get(name).is_none() {
        return Err(SubSwapError::ProfileNotFound(name.to_string()));
    }

    // Process guard (skip if --force)
    if !force {
        let guard = OsGuard::new();
        guard.check()?;
    }

    let config = AppConfig::load(paths)?;
    let keystore = OsKeyStore::new();
    let key = get_or_default_key(&keystore, config.encryption_enabled)?;

    switch::switch_profile(paths, name, &key, config.encryption_enabled)?;
    println!("Switched to profile '{name}'.");

    Ok(())
}

fn cmd_add(paths: &Paths, name: &str, from: Option<&str>, note: Option<String>) -> Result<()> {
    validate_profile_name(name)?;
    let mut store = ProfileStore::load_or_init(paths)?;
    let config = AppConfig::load(paths)?;
    let keystore = OsKeyStore::new();
    let key = get_or_default_key(&keystore, config.encryption_enabled)?;

    if let Some(source_path) = from {
        let source = Path::new(source_path);
        if !source.is_dir() {
            return Err(SubSwapError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Source path '{source_path}' is not a directory"),
            )));
        }
        switch::add_profile_from_path(
            paths,
            &mut store,
            name,
            Path::new(source_path),
            note,
            &key,
            config.encryption_enabled,
        )?;
        println!("Profile '{name}' imported from '{source_path}'.");
    } else {
        switch::add_profile_from_codex(
            paths,
            &mut store,
            name,
            note,
            &key,
            config.encryption_enabled,
        )?;
        println!("Profile '{name}' created from current ~/.codex/ config.");
    }

    Ok(())
}

fn cmd_remove(paths: &Paths, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut store = ProfileStore::load(paths)?;

    // remove() validates the profile exists and is not active
    store.index.remove(name)?;
    ProfileStore::delete_profile_dir(paths, name)?;
    store.save(paths)?;

    println!("Profile '{name}' removed.");
    Ok(())
}

fn cmd_rename(paths: &Paths, old: &str, new: &str) -> Result<()> {
    validate_profile_name(old)?;
    validate_profile_name(new)?;
    let mut store = ProfileStore::load(paths)?;

    // rename() validates old exists and new doesn't
    store.index.rename(old, new)?;
    ProfileStore::rename_profile_dir(paths, old, new)?;
    store.save(paths)?;

    println!("Profile '{old}' renamed to '{new}'.");
    Ok(())
}

fn cmd_note(paths: &Paths, name: &str, text: &str) -> Result<()> {
    validate_profile_name(name)?;
    let mut store = ProfileStore::load(paths)?;

    store.index.set_note(name, Some(text.to_string()))?;
    store.save(paths)?;

    println!("Note updated for profile '{name}'.");
    Ok(())
}

fn cmd_decrypt(paths: &Paths, name: &str) -> Result<()> {
    validate_profile_name(name)?;
    let config = AppConfig::load(paths)?;
    let keystore = OsKeyStore::new();
    let key = get_or_default_key(&keystore, config.encryption_enabled)?;

    let (auth_str, config_str) = switch::decrypt_profile_to_stdout(paths, name, &key)?;

    println!("=== auth.json ===");
    println!("{auth_str}");
    println!("=== config.toml ===");
    println!("{config_str}");

    Ok(())
}

fn cmd_config(paths: &Paths, action: ConfigAction) -> Result<()> {
    match action {
        ConfigAction::Show => {
            let config = AppConfig::load(paths)?;
            println!("encryption_enabled: {}", config.encryption_enabled);
        }
        ConfigAction::Set { key, value } => {
            let mut config = AppConfig::load(paths)?;
            match key.as_str() {
                "encryption" | "encryption_enabled" => {
                    let new_val = match value.to_lowercase().as_str() {
                        "true" | "1" | "yes" | "on" => true,
                        "false" | "0" | "no" | "off" => false,
                        _ => {
                            return Err(SubSwapError::Crypto(format!(
                                "Invalid value for encryption: '{value}'. Use true or false."
                            )));
                        }
                    };
                    if new_val == config.encryption_enabled {
                        println!("Encryption is already set to {new_val}.");
                        return Ok(());
                    }
                    let keystore = OsKeyStore::new();
                    let key_bytes = get_or_default_key(&keystore, new_val)?;
                    toggle_all_profiles(paths, &key_bytes, new_val)?;
                    config.encryption_enabled = new_val;
                    config.save(paths)?;
                    println!("Encryption set to {new_val}.");
                }
                _ => {
                    return Err(SubSwapError::Crypto(format!(
                        "Unknown config key '{key}'. Supported: encryption"
                    )));
                }
            }
        }
    }

    Ok(())
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Re-encrypt or decrypt all non-active profiles.
///
/// When `encrypt` is true:  plaintext profiles are encrypted with `key`.
/// When `encrypt` is false: encrypted profiles are decrypted to plaintext.
fn toggle_all_profiles(paths: &Paths, key: &[u8; 32], encrypt: bool) -> Result<()> {
    let store = ProfileStore::load(paths)?;
    let active = store.index.active_profile.as_deref();

    for name in store.index.names() {
        // Skip the active profile — its data lives in ~/.codex/, not in the store
        if active == Some(name) {
            continue;
        }

        let (auth_raw, config_raw) = ProfileStore::load_profile_files(paths, name)?;
        let currently_encrypted = ProfileStore::profile_is_encrypted(paths, name);

        let (new_auth, new_config) = if encrypt && !currently_encrypted {
            // Encrypt plaintext
            (
                crypto::encrypt(&auth_raw, key)?,
                crypto::encrypt(&config_raw, key)?,
            )
        } else if !encrypt && currently_encrypted {
            // Decrypt ciphertext
            (
                crypto::decrypt(&auth_raw, key)?,
                crypto::decrypt(&config_raw, key)?,
            )
        } else {
            // Already in the desired state
            continue;
        };

        ProfileStore::save_profile_files(paths, name, &new_auth, &new_config, encrypt)?;
    }

    Ok(())
}

