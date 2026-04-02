use serde::{Deserialize, Serialize};
use crate::error::Result;
use crate::paths::Paths;
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub encryption_enabled: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            encryption_enabled: true,
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
        let config: Self = serde_json::from_str(&data)?;
        Ok(config)
    }

    pub fn save(&self, paths: &Paths) -> Result<()> {
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;
    use crate::paths::Paths;

    #[test]
    fn test_default_config_has_encryption_enabled() {
        let config = AppConfig::default();
        assert!(config.encryption_enabled);
    }

    #[test]
    fn test_save_and_load_config() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        let config = AppConfig { encryption_enabled: false };
        config.save(&paths).unwrap();
        let loaded = AppConfig::load(&paths).unwrap();
        assert!(!loaded.encryption_enabled);
    }

    #[test]
    fn test_load_missing_returns_default() {
        let tmp = TempDir::new().unwrap();
        let paths = Paths::from_temp(tmp.path());
        std::fs::create_dir_all(&paths.sub_swap_dir).unwrap();
        let loaded = AppConfig::load(&paths).unwrap();
        assert!(loaded.encryption_enabled);
    }
}
