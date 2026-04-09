use crate::error::{Result, SubSwapError};
use crate::paths::Paths;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum KeyBackend {
    Native,
    Passphrase,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PassphraseKdfConfig {
    pub salt_b64: String,
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AppConfig {
    pub encryption_enabled: bool,
    #[serde(default)]
    pub key_backend: Option<KeyBackend>,
    #[serde(default)]
    pub passphrase_kdf: Option<PassphraseKdfConfig>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
            key_backend: Some(KeyBackend::Native),
            passphrase_kdf: None,
        }
    }
}

impl AppConfig {
    pub fn load(paths: &Paths) -> Result<Self> {
        let path = paths.config_json();
        if !path.exists() {
            return Ok(Self::default());
        }
        let data = fs::read_to_string(&path)?;
        let mut config: Self = serde_json::from_str(&data)?;
        config.apply_legacy_defaults();
        config.validate()?;
        Ok(config)
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
        self.validate()?;
        let path = paths.config_json();
        let data = serde_json::to_string_pretty(self)?;
        fs::write(&path, data)?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
        }

        Ok(())
    }

    fn apply_legacy_defaults(&mut self) {
        if self.encryption_enabled && self.key_backend.is_none() {
            self.key_backend = Some(KeyBackend::Native);
        }
    }

    fn validate(&self) -> Result<()> {
        if !self.encryption_enabled {
            return Ok(());
        }
        match &self.key_backend {
            None => Err(SubSwapError::Crypto(
                "encryption is enabled but no key backend is configured".to_string(),
            )),
            Some(KeyBackend::Native) => {
                if self.passphrase_kdf.is_some() {
                    Err(SubSwapError::Crypto(
                        "native backend must not store passphrase KDF metadata".to_string(),
                    ))
                } else {
                    Ok(())
                }
            }
            Some(KeyBackend::Passphrase) => {
                if self.passphrase_kdf.is_some() {
                    Ok(())
                } else {
                    Err(SubSwapError::Crypto(
                        "passphrase backend requires KDF metadata".to_string(),
                    ))
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::paths::Paths;
    use tempfile::TempDir;
    use std::fs;

    #[test]
    fn test_default_config_has_encryption_enabled() {
        let config = AppConfig::default();
        assert!(config.encryption_enabled);
        assert_eq!(config.key_backend, Some(KeyBackend::Native));
        assert!(config.passphrase_kdf.is_none());
    }

    #[test]
    fn test_save_and_load_config() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        let config = AppConfig {
            encryption_enabled: false,
            key_backend: None,
            passphrase_kdf: None,
        };
        config.save(&paths).unwrap();
        let loaded = AppConfig::load(&paths).unwrap();
        assert_eq!(loaded, config);
    }

    #[test]
    fn test_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        let loaded = AppConfig::load(&paths).unwrap();
        assert_eq!(loaded, AppConfig::default());
    }

    #[test]
    fn test_load_legacy_encrypted_config_defaults_to_native_backend() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        fs::write(
            paths.config_json(),
            r#"{
  "encryption_enabled": true
}"#,
        )
        .unwrap();

        let loaded = AppConfig::load(&paths).unwrap();
        assert_eq!(loaded.key_backend, Some(KeyBackend::Native));
        assert!(loaded.passphrase_kdf.is_none());
    }

    #[test]
    fn test_load_passphrase_backend_requires_kdf_metadata() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        fs::write(
            paths.config_json(),
            r#"{
  "encryption_enabled": true,
  "key_backend": "passphrase"
}"#,
        )
        .unwrap();

        let result = AppConfig::load(&paths);
        assert!(result.is_err(), "passphrase backend must reject missing KDF metadata");
    }

    #[test]
    fn test_save_and_load_passphrase_backend_roundtrip() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();

        let config = AppConfig {
            encryption_enabled: true,
            key_backend: Some(KeyBackend::Passphrase),
            passphrase_kdf: Some(PassphraseKdfConfig {
                salt_b64: "c2FsdC1ieXRlcy1mb3ItdGVzdA==".to_string(),
                memory_kib: 65_536,
                iterations: 3,
                parallelism: 1,
            }),
        };

        config.save(&paths).unwrap();
        let loaded = AppConfig::load(&paths).unwrap();
        assert_eq!(loaded, config);
    }
}
