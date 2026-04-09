use std::io::{self, BufRead, Write};

use crate::config::{AppConfig, KeyBackend};
use crate::crypto::keychain::OsKeyStore;
use crate::error::{validate_profile_name, Result};
#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::error::SubSwapError;
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
        let (key_backend, passphrase_kdf, key) =
            match crate::key::initialize_native_backend(&OsKeyStore::new()) {
                Ok(key) => (KeyBackend::Native, None, key),
                Err(native_err) => {
                    #[cfg(any(target_os = "linux", target_os = "windows"))]
                    {
                        println!("Native key storage unavailable: {native_err}");
                        let passphrase = prompt_passphrase_twice_wizard()?;
                        let (passphrase_kdf, key) =
                            crate::key::initialize_passphrase_backend(&passphrase)?;
                        (KeyBackend::Passphrase, Some(passphrase_kdf), key)
                    }

                    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
                    {
                        return Err(native_err);
                    }
                }
            };

        let config = AppConfig {
            encryption_enabled: true,
            key_backend: Some(key_backend.clone()),
            passphrase_kdf,
        };
        config.save(paths)?;
        println!(
            "Encryption enabled using {}.",
            crate::key::backend_label(&key_backend)
        );

        Ok((key, true))
    } else {
        println!("{}", skipped_encryption_message());

        let config = AppConfig {
            encryption_enabled: false,
            key_backend: None,
            passphrase_kdf: None,
        };
        config.save(paths)?;

        let dummy_key = [0u8; 32];
        Ok((dummy_key, false))
    }
}

fn skipped_encryption_message() -> &'static str {
    "Skipping encryption setup. You can enable it later with `sub-swap config set encryption true`."
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

#[cfg(any(target_os = "linux", target_os = "windows"))]
fn prompt_passphrase_twice_wizard() -> Result<String> {
    let first = rpassword::prompt_password("Enter passphrase: ")?;
    let second = rpassword::prompt_password("Confirm passphrase: ")?;

    if first != second {
        return Err(SubSwapError::Crypto(
            "passphrase confirmation did not match".to_string(),
        ));
    }

    if first.is_empty() {
        return Err(SubSwapError::Crypto(
            "passphrase cannot be empty".to_string(),
        ));
    }

    Ok(first)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_later_enable_message_uses_real_cli_command() {
        let message = skipped_encryption_message();
        assert!(message.contains("sub-swap config set encryption true"));
    }
}
