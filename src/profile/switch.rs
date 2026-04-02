use std::fs;

use crate::crypto;
use crate::error::{validate_profile_name, Result, SubSwapError};
use crate::paths::Paths;
use crate::profile::Profile;
use crate::profile::store::ProfileStore;

/// Switch the active profile to `target`.
///
/// Lifecycle:
/// 1. Load ProfileStore from profiles.json
/// 2. If target is already active, return Ok (no-op)
/// 3. If target doesn't exist in index, return ProfileNotFound error
/// 4. Load target profile files from ~/.sub-swap/profiles/<target>/
/// 5. If target files are encrypted (.enc), decrypt them in memory using the key
/// 6. Read current active profile from ~/.codex/ (auth.json + config.toml)
/// 7. Save current active profile back to ~/.sub-swap/profiles/<old>/ — encrypt if encrypt is true
/// 8. Write target profile's plaintext to ~/.codex/auth.json and ~/.codex/config.toml
/// 9. Set 0600 permissions on the codex files (Unix only)
/// 10. Update index: set_active(target), save
pub fn switch_profile(paths: &Paths, target: &str, key: &[u8; 32], encrypt: bool) -> Result<()> {
    let mut store = ProfileStore::load(paths)?;

    // No-op if already active
    if store.index.active_profile.as_deref() == Some(target) {
        return Ok(());
    }

    // Validate target exists
    if store.index.get(target).is_none() {
        return Err(SubSwapError::ProfileNotFound(target.to_string()));
    }

    // Load target profile files
    let (target_auth_raw, target_config_raw) = ProfileStore::load_profile_files(paths, target)?;

    // Decrypt target if encrypted
    let target_is_enc = ProfileStore::profile_is_encrypted(paths, target);
    let target_auth = if target_is_enc {
        crypto::decrypt(&target_auth_raw, key)?
    } else {
        target_auth_raw
    };
    let target_config = if target_is_enc {
        crypto::decrypt(&target_config_raw, key)?
    } else {
        target_config_raw
    };

    // Read current active profile from codex dir
    let old_name = store.index.active_profile.clone();
    let cur_auth = fs::read(paths.codex_auth())?;
    let cur_config = if paths.codex_config().exists() {
        fs::read(paths.codex_config())?
    } else {
        Vec::new()
    };

    // Save current active profile back to its profile dir
    if let Some(ref old) = old_name {
        let auth_to_save = if encrypt {
            crypto::encrypt(&cur_auth, key)?
        } else {
            cur_auth
        };
        let config_to_save = if encrypt {
            crypto::encrypt(&cur_config, key)?
        } else {
            cur_config
        };
        ProfileStore::save_profile_files(paths, old, &auth_to_save, &config_to_save, encrypt)?;
    }

    // Write target plaintext to codex dir
    fs::write(paths.codex_auth(), &target_auth)?;
    fs::write(paths.codex_config(), &target_config)?;

    // Set 0600 permissions on the codex files (Unix only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(paths.codex_auth(), fs::Permissions::from_mode(0o600))?;
        fs::set_permissions(paths.codex_config(), fs::Permissions::from_mode(0o600))?;
    }

    // Update index
    store.index.set_active(target);
    store.save(paths)?;

    Ok(())
}

/// Add a new profile by importing the current ~/.codex/ files.
pub fn add_profile_from_codex(
    paths: &Paths,
    store: &mut ProfileStore,
    name: &str,
    notes: Option<String>,
    key: &[u8; 32],
    encrypt: bool,
) -> Result<()> {
    validate_profile_name(name)?;
    // Check name doesn't already exist
    if store.index.get(name).is_some() {
        return Err(SubSwapError::ProfileExists(name.to_string()));
    }

    // Check codex auth.json exists
    if !paths.codex_auth().exists() {
        return Err(SubSwapError::NoCodexConfig);
    }

    // Read auth.json and config.toml from ~/.codex/
    let auth_data = fs::read(paths.codex_auth())?;
    let config_data = if paths.codex_config().exists() {
        fs::read(paths.codex_config())?
    } else {
        Vec::new()
    };

    // Encrypt if requested
    let auth_to_save = if encrypt {
        crypto::encrypt(&auth_data, key)?
    } else {
        auth_data
    };
    let config_to_save = if encrypt {
        crypto::encrypt(&config_data, key)?
    } else {
        config_data
    };

    // Save to ~/.sub-swap/profiles/<name>/
    ProfileStore::save_profile_files(paths, name, &auth_to_save, &config_to_save, encrypt)?;

    // Add profile to index, set as active, save
    store.index.add(Profile::new(name, notes));
    store.index.set_active(name);
    store.save(paths)?;

    Ok(())
}

/// Add a new profile by importing files from a source directory.
/// Unlike add_profile_from_codex, this does NOT set the profile as active.
pub fn add_profile_from_path(
    paths: &Paths,
    store: &mut ProfileStore,
    name: &str,
    source: &std::path::Path,
    notes: Option<String>,
    key: &[u8; 32],
    encrypt: bool,
) -> Result<()> {
    validate_profile_name(name)?;
    // Check name doesn't already exist
    if store.index.get(name).is_some() {
        return Err(SubSwapError::ProfileExists(name.to_string()));
    }

    // Read auth.json from source directory
    let source_auth = source.join("auth.json");
    if !source_auth.exists() {
        return Err(SubSwapError::NoCodexConfig);
    }

    let auth_data = fs::read(&source_auth)?;
    let source_config = source.join("config.toml");
    let config_data = if source_config.exists() {
        fs::read(&source_config)?
    } else {
        Vec::new()
    };

    // Encrypt if requested
    let auth_to_save = if encrypt {
        crypto::encrypt(&auth_data, key)?
    } else {
        auth_data
    };
    let config_to_save = if encrypt {
        crypto::encrypt(&config_data, key)?
    } else {
        config_data
    };

    // Save to ~/.sub-swap/profiles/<name>/
    ProfileStore::save_profile_files(paths, name, &auth_to_save, &config_to_save, encrypt)?;

    // Add to index (but do NOT set as active)
    store.index.add(Profile::new(name, notes));
    store.save(paths)?;

    Ok(())
}

/// Decrypt a profile's files and return them as strings without writing to disk.
pub fn decrypt_profile_to_stdout(
    paths: &Paths,
    name: &str,
    key: &[u8; 32],
) -> Result<(String, String)> {
    let store = ProfileStore::load(paths)?;

    // Verify profile exists
    if store.index.get(name).is_none() {
        return Err(SubSwapError::ProfileNotFound(name.to_string()));
    }

    // Load profile files
    let (auth_raw, config_raw) = ProfileStore::load_profile_files(paths, name)?;

    // Decrypt if encrypted
    let is_enc = ProfileStore::profile_is_encrypted(paths, name);
    let auth_bytes = if is_enc {
        crypto::decrypt(&auth_raw, key)?
    } else {
        auth_raw
    };
    let config_bytes = if is_enc {
        crypto::decrypt(&config_raw, key)?
    } else {
        config_raw
    };

    let auth_str = String::from_utf8(auth_bytes)
        .map_err(|e| SubSwapError::Crypto(format!("auth.json is not valid UTF-8: {e}")))?;
    let config_str = String::from_utf8(config_bytes)
        .map_err(|e| SubSwapError::Crypto(format!("config.toml is not valid UTF-8: {e}")))?;

    Ok((auth_str, config_str))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;
    use crate::profile::store::ProfileStore;
    use tempfile::TempDir;

    fn setup_with_profiles() -> (TempDir, Paths, [u8; 32]) {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(paths.profiles_dir()).unwrap();
        std::fs::create_dir_all(&paths.codex_dir).unwrap();
        // Also create sub_swap_dir so profiles.json can be written there
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();

        // Set up codex files for "work" profile
        std::fs::write(paths.codex_auth(), r#"{"key": "work-key"}"#).unwrap();
        std::fs::write(paths.codex_config(), "model = \"gpt-5\"").unwrap();

        // Create profile index with "work" as active
        let mut store = ProfileStore::init(&paths).unwrap();
        store.index.add(Profile::new("work", Some("Work profile".into())));
        store.index.set_active("work");
        store.save(&paths).unwrap();

        // Save work profile files (plaintext)
        ProfileStore::save_profile_files(
            &paths,
            "work",
            br#"{"key": "work-key"}"#,
            b"model = \"gpt-5\"",
            false,
        )
        .unwrap();

        // Save personal profile files (plaintext)
        ProfileStore::save_profile_files(
            &paths,
            "personal",
            br#"{"key": "personal-key"}"#,
            b"model = \"gpt-4\"",
            false,
        )
        .unwrap();
        store.index.add(Profile::new("personal", Some("Personal".into())));
        store.save(&paths).unwrap();

        let key = crypto::generate_key();
        (tmp, paths, key)
    }

    #[test]
    fn test_switch_updates_codex_files() {
        let (_tmp, paths, key) = setup_with_profiles();

        switch_profile(&paths, "personal", &key, false).unwrap();

        let auth_content = std::fs::read_to_string(paths.codex_auth()).unwrap();
        let config_content = std::fs::read_to_string(paths.codex_config()).unwrap();

        assert!(
            auth_content.contains("personal-key"),
            "auth.json should contain personal-key, got: {auth_content}"
        );
        assert!(
            config_content.contains("gpt-4"),
            "config.toml should contain gpt-4, got: {config_content}"
        );
    }

    #[test]
    fn test_switch_encrypts_old_profile() {
        let (_tmp, paths, key) = setup_with_profiles();

        // Switch to personal with encrypt=true — old "work" profile should be encrypted
        switch_profile(&paths, "personal", &key, true).unwrap();

        assert!(
            ProfileStore::profile_is_encrypted(&paths, "work"),
            "work profile should now be encrypted"
        );
    }

    #[test]
    fn test_switch_updates_index() {
        let (_tmp, paths, key) = setup_with_profiles();

        switch_profile(&paths, "personal", &key, false).unwrap();

        let store = ProfileStore::load(&paths).unwrap();
        assert_eq!(
            store.index.active_profile.as_deref(),
            Some("personal"),
            "active_profile should be 'personal'"
        );
    }

    #[test]
    fn test_switch_to_already_active_is_noop() {
        let (_tmp, paths, key) = setup_with_profiles();

        // "work" is already active — should return Ok without changing anything
        let result = switch_profile(&paths, "work", &key, false);
        assert!(result.is_ok(), "switching to already-active profile should be Ok");

        let store = ProfileStore::load(&paths).unwrap();
        assert_eq!(store.index.active_profile.as_deref(), Some("work"));
    }

    #[test]
    fn test_switch_to_nonexistent_fails() {
        let (_tmp, paths, key) = setup_with_profiles();

        let result = switch_profile(&paths, "nonexistent", &key, false);
        assert!(result.is_err(), "switching to nonexistent profile should fail");

        match result.unwrap_err() {
            SubSwapError::ProfileNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("expected ProfileNotFound, got {other}"),
        }
    }

    #[test]
    fn test_switch_without_encryption() {
        let (_tmp, paths, key) = setup_with_profiles();

        // Switch with encrypt=false — old "work" profile files should remain plaintext
        switch_profile(&paths, "personal", &key, false).unwrap();

        assert!(
            !ProfileStore::profile_is_encrypted(&paths, "work"),
            "work profile should remain plaintext when encrypt=false"
        );

        let (auth_raw, _) = ProfileStore::load_profile_files(&paths, "work").unwrap();
        let auth_str = String::from_utf8(auth_raw).unwrap();
        assert!(
            auth_str.contains("work-key"),
            "work auth.json should still contain work-key"
        );
    }

    #[test]
    fn test_add_profile_from_codex() {
        let (_tmp, paths, key) = setup_with_profiles();
        let mut store = ProfileStore::load(&paths).unwrap();

        // Add a new profile called "first"
        add_profile_from_codex(&paths, &mut store, "first", Some("First profile".into()), &key, true).unwrap();

        // Reload store to confirm persistence
        let reloaded = ProfileStore::load(&paths).unwrap();

        assert!(
            reloaded.index.get("first").is_some(),
            "'first' profile should be in the index"
        );
        assert_eq!(
            reloaded.index.active_profile.as_deref(),
            Some("first"),
            "active profile should be 'first'"
        );
        assert!(
            ProfileStore::profile_is_encrypted(&paths, "first"),
            "'first' profile files should be encrypted"
        );
    }
}
