use std::io::{self, BufRead, Write};

use crate::config::AppConfig;
use crate::crypto;
use crate::crypto::keychain::{KeyStore, OsKeyStore};
use crate::error::{validate_profile_name, Result};
use crate::paths::Paths;
use crate::profile::store::ProfileStore;
use crate::profile::switch;

pub fn run_first_launch(paths: &Paths) -> Result<()> {
    println!();
    println!("Welcome to sub-swap!");
    println!("====================");
    println!("sub-swap lets you manage multiple Codex profiles and switch between them.");
    println!();

    // Create ~/.sub-swap/profiles/ directory
    std::fs::create_dir_all(paths.profiles_dir())?;

    let codex_found = paths.codex_auth().exists() && paths.codex_config().exists();

    if codex_found {
        println!("Found existing Codex configuration:");
        println!("  auth.json  : {}", paths.codex_auth().display());
        println!("  config.toml: {}", paths.codex_config().display());
        println!();

        let save_profile = prompt_yn("Save it as your first profile?", true)?;

        if save_profile {
            let name = loop {
                let input = prompt_string("Profile name", Some("default"))?;
                match validate_profile_name(&input) {
                    Ok(()) => break input,
                    Err(e) => {
                        println!("{e}");
                        println!("Please try again.");
                    }
                }
            };
            let note = prompt_string_optional("Optional note (press Enter to skip)")?;

            let (key, encryption_enabled) = setup_encryption(paths)?;

            let mut store = ProfileStore::load_or_init(paths)?;
            switch::add_profile_from_codex(
                paths,
                &mut store,
                &name,
                note,
                &key,
                encryption_enabled,
            )?;

            println!();
            println!("Profile '{name}' created successfully.");
            if encryption_enabled {
                println!("Encryption is enabled — your profile files are encrypted at rest.");
            } else {
                println!("Encryption is disabled — profile files are stored in plaintext.");
            }
        } else {
            let (_key, _encryption_enabled) = setup_encryption(paths)?;
        }
    } else {
        println!("No Codex configuration found.");
        println!("Once you have configured Codex, run `sub-swap add <name>` to create a profile.");
        println!();

        let (_key, _encryption_enabled) = setup_encryption(paths)?;
    }

    // Always create profiles.json so the wizard does not re-run on next launch.
    ProfileStore::load_or_init(paths)?;

    println!();
    println!("Run `sub-swap` for the interactive wizard or `sub-swap --help` for commands.");
    println!();

    Ok(())
}

fn setup_encryption(paths: &Paths) -> Result<([u8; 32], bool)> {
    println!();
    let generate = prompt_yn("Generate encryption key now?", true)?;

    if generate {
        let key = crypto::generate_key();
        OsKeyStore::new().set_key(&key)?;

        #[cfg(target_os = "macos")]
        println!("Encryption key stored in macOS Keychain (Keychain Access > sub-swap).");

        #[cfg(target_os = "linux")]
        println!("Encryption key stored in the system keyring (e.g. GNOME Keyring or KWallet).");

        #[cfg(target_os = "windows")]
        println!("Encryption key stored in Windows Credential Manager.");

        #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
        println!("Encryption key stored in the OS keychain.");

        let config = AppConfig {
            encryption_enabled: true,
        };
        config.save(paths)?;

        Ok((key, true))
    } else {
        println!(
            "Skipping encryption setup. You can enable it later with `sub-swap config --encrypt`."
        );

        let config = AppConfig {
            encryption_enabled: false,
        };
        config.save(paths)?;

        let dummy_key = [0u8; 32];
        Ok((dummy_key, false))
    }
}

fn prompt_yn(question: &str, default_yes: bool) -> Result<bool> {
    let choices = if default_yes { "[Y/n]" } else { "[y/N]" };
    print!("{question} {choices} ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next().transpose()?.unwrap_or_default();
    let trimmed = line.trim().to_lowercase();

    let answer = match trimmed.as_str() {
        "y" | "yes" => true,
        "n" | "no" => false,
        _ => default_yes,
    };

    Ok(answer)
}

fn prompt_string(question: &str, default: Option<&str>) -> Result<String> {
    match default {
        Some(d) => print!("{question} [{d}]: "),
        None => print!("{question}: "),
    }
    io::stdout().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next().transpose()?.unwrap_or_default();
    let trimmed = line.trim().to_string();

    if trimmed.is_empty() {
        Ok(default.unwrap_or("").to_string())
    } else {
        Ok(trimmed)
    }
}

fn prompt_string_optional(question: &str) -> Result<Option<String>> {
    print!("{question}: ");
    io::stdout().flush()?;

    let stdin = io::stdin();
    let line = stdin.lock().lines().next().transpose()?.unwrap_or_default();
    let trimmed = line.trim().to_string();

    if trimmed.is_empty() {
        Ok(None)
    } else {
        Ok(Some(trimmed))
    }
}
