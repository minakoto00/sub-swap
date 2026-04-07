use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Paths {
    pub codex_dir: PathBuf,
    pub sub_swap_dir: PathBuf,
}

impl Default for Paths {
    fn default() -> Self {
        Self::new()
    }
}

impl Paths {
    pub fn new() -> Self {
        let home = dirs::home_dir().expect("Could not determine home directory");
        Self {
            codex_dir: home.join(".codex"),
            sub_swap_dir: home.join(".sub-swap"),
        }
    }

    pub fn profiles_dir(&self) -> PathBuf {
        self.sub_swap_dir.join("profiles")
    }

    pub fn profile_dir(&self, name: &str) -> PathBuf {
        self.profiles_dir().join(name)
    }

    pub fn profiles_json(&self) -> PathBuf {
        self.sub_swap_dir.join("profiles.json")
    }

    pub fn config_json(&self) -> PathBuf {
        self.sub_swap_dir.join("config.json")
    }

    pub fn codex_auth(&self) -> PathBuf {
        self.codex_dir.join("auth.json")
    }

    pub fn codex_config(&self) -> PathBuf {
        self.codex_dir.join("config.toml")
    }
}

#[cfg(test)]
impl Paths {
    pub fn from_temp(base: &std::path::Path) -> Self {
        Self {
            codex_dir: base.join("codex"),
            sub_swap_dir: base.join("sub-swap"),
        }
    }
}
