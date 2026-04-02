use std::fs;

use crate::error::Result;
use crate::paths::Paths;
use crate::profile::ProfileIndex;

pub struct ProfileStore {
    pub index: ProfileIndex,
}

impl ProfileStore {
    pub fn init(paths: &Paths) -> Result<Self> {
        let index = ProfileIndex::new();
        let store = Self { index };
        store.save(paths)?;
        Ok(store)
    }

    pub fn load(paths: &Paths) -> Result<Self> {
        let data = fs::read_to_string(paths.profiles_json())?;
        let index: ProfileIndex = serde_json::from_str(&data)?;
        Ok(Self { index })
    }

    pub fn load_or_init(paths: &Paths) -> Result<Self> {
        if paths.profiles_json().exists() {
            Self::load(paths)
        } else {
            Self::init(paths)
        }
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        let json = serde_json::to_string_pretty(&self.index)?;
        fs::write(paths.profiles_json(), json)?;
        Ok(())
    }

    pub fn save_profile_files(
        paths: &Paths,
        name: &str,
        auth_data: &[u8],
        config_data: &[u8],
        encrypted: bool,
    ) -> Result<()> {
        let profile_dir = paths.profile_dir(name);
        fs::create_dir_all(&profile_dir)?;

        let (auth_path, auth_plain_path) = if encrypted {
            (
                profile_dir.join("auth.json.enc"),
                profile_dir.join("auth.json"),
            )
        } else {
            (
                profile_dir.join("auth.json"),
                profile_dir.join("auth.json.enc"),
            )
        };

        let (config_path, config_plain_path) = if encrypted {
            (
                profile_dir.join("config.toml.enc"),
                profile_dir.join("config.toml"),
            )
        } else {
            (
                profile_dir.join("config.toml"),
                profile_dir.join("config.toml.enc"),
            )
        };

        fs::write(&auth_path, auth_data)?;
        fs::write(&config_path, config_data)?;

        // Remove opposite-suffix files if they exist
        if auth_plain_path.exists() {
            fs::remove_file(&auth_plain_path)?;
        }
        if config_plain_path.exists() {
            fs::remove_file(&config_plain_path)?;
        }

        // Set 0600 permissions on Unix
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&auth_path, fs::Permissions::from_mode(0o600))?;
            fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    pub fn load_profile_files(paths: &Paths, name: &str) -> Result<(Vec<u8>, Vec<u8>)> {
        let profile_dir = paths.profile_dir(name);

        // Check .enc first, fall back to plaintext
        let auth_data = if profile_dir.join("auth.json.enc").exists() {
            fs::read(profile_dir.join("auth.json.enc"))?
        } else {
            fs::read(profile_dir.join("auth.json"))?
        };

        let config_data = if profile_dir.join("config.toml.enc").exists() {
            fs::read(profile_dir.join("config.toml.enc"))?
        } else {
            fs::read(profile_dir.join("config.toml"))?
        };

        Ok((auth_data, config_data))
    }

    pub fn profile_is_encrypted(paths: &Paths, name: &str) -> bool {
        paths.profile_dir(name).join("auth.json.enc").exists()
    }

    pub fn delete_profile_dir(paths: &Paths, name: &str) -> Result<()> {
        let profile_dir = paths.profile_dir(name);
        fs::remove_dir_all(profile_dir)?;
        Ok(())
    }

    pub fn rename_profile_dir(paths: &Paths, old: &str, new: &str) -> Result<()> {
        let old_dir = paths.profile_dir(old);
        let new_dir = paths.profile_dir(new);
        fs::rename(old_dir, new_dir)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::profile::Profile;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Paths) {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        fs::create_dir_all(paths.profiles_dir()).unwrap();
        // Also create sub_swap_dir so profiles.json can be written there
        fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        (tmp, paths)
    }

    #[test]
    fn test_init_creates_empty_index() {
        let (_tmp, paths) = setup();

        let store = ProfileStore::init(&paths).unwrap();

        assert!(paths.profiles_json().exists());
        assert!(store.index.profiles.is_empty());
        assert!(store.index.active_profile.is_none());
    }

    #[test]
    fn test_load_existing_index() {
        let (_tmp, paths) = setup();

        // Create a store, add a profile, save, then reload
        let mut store = ProfileStore::init(&paths).unwrap();
        store.index.add(Profile::new("work", Some("Work account".to_string())));
        store.save(&paths).unwrap();

        let loaded = ProfileStore::load(&paths).unwrap();

        assert!(loaded.index.get("work").is_some());
        assert_eq!(loaded.index.get("work").unwrap().name, "work");
        assert_eq!(
            loaded.index.get("work").unwrap().notes.as_deref(),
            Some("Work account")
        );
    }

    #[test]
    fn test_save_profile_files() {
        let (_tmp, paths) = setup();

        let auth_data = b"{\"token\": \"abc123\"}";
        let config_data = b"model = \"gpt-4\"";

        ProfileStore::save_profile_files(&paths, "personal", auth_data, config_data, false)
            .unwrap();

        let profile_dir = paths.profile_dir("personal");
        assert!(profile_dir.join("auth.json").exists());
        assert!(profile_dir.join("config.toml").exists());
        // No .enc files
        assert!(!profile_dir.join("auth.json.enc").exists());
        assert!(!profile_dir.join("config.toml.enc").exists());
    }

    #[test]
    fn test_save_profile_files_encrypted() {
        let (_tmp, paths) = setup();

        let auth_data = b"\xde\xad\xbe\xef\x00\x01\x02\x03";
        let config_data = b"\xca\xfe\xba\xbe\x04\x05\x06\x07";

        ProfileStore::save_profile_files(&paths, "secure", auth_data, config_data, true).unwrap();

        let profile_dir = paths.profile_dir("secure");
        assert!(profile_dir.join("auth.json.enc").exists());
        assert!(profile_dir.join("config.toml.enc").exists());
        // No plaintext files
        assert!(!profile_dir.join("auth.json").exists());
        assert!(!profile_dir.join("config.toml").exists());
    }

    #[test]
    fn test_load_profile_files_plaintext() {
        let (_tmp, paths) = setup();

        let auth_data = b"{\"token\": \"mytoken\"}";
        let config_data = b"model = \"claude\"";

        ProfileStore::save_profile_files(&paths, "myprofile", auth_data, config_data, false)
            .unwrap();

        let (loaded_auth, loaded_config) =
            ProfileStore::load_profile_files(&paths, "myprofile").unwrap();

        assert_eq!(loaded_auth, auth_data);
        assert_eq!(loaded_config, config_data);
    }

    #[test]
    fn test_delete_profile_dir() {
        let (_tmp, paths) = setup();

        let auth_data = b"{}";
        let config_data = b"";

        ProfileStore::save_profile_files(&paths, "todelete", auth_data, config_data, false)
            .unwrap();

        let profile_dir = paths.profile_dir("todelete");
        assert!(profile_dir.exists());

        ProfileStore::delete_profile_dir(&paths, "todelete").unwrap();

        assert!(!profile_dir.exists());
    }

    #[test]
    fn test_rename_profile_dir() {
        let (_tmp, paths) = setup();

        let auth_data = b"{}";
        let config_data = b"";

        ProfileStore::save_profile_files(&paths, "oldname", auth_data, config_data, false)
            .unwrap();

        let old_dir = paths.profile_dir("oldname");
        assert!(old_dir.exists());

        ProfileStore::rename_profile_dir(&paths, "oldname", "newname").unwrap();

        assert!(!old_dir.exists());
        assert!(paths.profile_dir("newname").exists());
    }
}
