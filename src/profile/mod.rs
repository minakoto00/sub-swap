pub mod store;
pub mod switch;

use std::collections::BTreeMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};

use crate::error::{Result, SubSwapError};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub notes: Option<String>,
    pub created_at: String,
    pub last_used: Option<String>,
}

impl Profile {
    pub fn new(name: impl Into<String>, notes: Option<String>) -> Self {
        Self {
            name: name.into(),
            notes,
            created_at: Utc::now().to_rfc3339(),
            last_used: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProfileIndex {
    pub version: u32,
    pub active_profile: Option<String>,
    pub profiles: BTreeMap<String, Profile>,
}

impl ProfileIndex {
    pub fn new() -> Self {
        Self {
            version: 1,
            active_profile: None,
            profiles: BTreeMap::new(),
        }
    }

    pub fn add(&mut self, profile: Profile) {
        self.profiles.insert(profile.name.clone(), profile);
    }

    pub fn get(&self, name: &str) -> Option<&Profile> {
        self.profiles.get(name)
    }

    pub fn remove(&mut self, name: &str) -> Result<Profile> {
        if self.active_profile.as_deref() == Some(name) {
            return Err(SubSwapError::ActiveProfile(name.to_string()));
        }
        self.profiles
            .remove(name)
            .ok_or_else(|| SubSwapError::ProfileNotFound(name.to_string()))
    }

    pub fn rename(&mut self, old: &str, new: &str) -> Result<()> {
        if self.profiles.contains_key(new) {
            return Err(SubSwapError::ProfileExists(new.to_string()));
        }
        let mut profile = self
            .profiles
            .remove(old)
            .ok_or_else(|| SubSwapError::ProfileNotFound(old.to_string()))?;
        profile.name = new.to_string();
        if self.active_profile.as_deref() == Some(old) {
            self.active_profile = Some(new.to_string());
        }
        self.profiles.insert(new.to_string(), profile);
        Ok(())
    }

    pub fn set_active(&mut self, name: &str) {
        self.active_profile = Some(name.to_string());
        if let Some(profile) = self.profiles.get_mut(name) {
            profile.last_used = Some(Utc::now().to_rfc3339());
        }
    }

    pub fn set_note(&mut self, name: &str, note: Option<String>) -> Result<()> {
        let profile = self
            .profiles
            .get_mut(name)
            .ok_or_else(|| SubSwapError::ProfileNotFound(name.to_string()))?;
        profile.notes = note;
        Ok(())
    }

    pub fn names(&self) -> Vec<&str> {
        self.profiles.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for ProfileIndex {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_profile() {
        let profile = Profile::new("work", Some("Work account".to_string()));
        assert_eq!(profile.name, "work");
        assert_eq!(profile.notes, Some("Work account".to_string()));
        assert!(!profile.created_at.is_empty());
        assert!(profile.last_used.is_none());
    }

    #[test]
    fn test_profile_index_add_and_get() {
        let mut index = ProfileIndex::new();
        let profile = Profile::new("personal", None);
        index.add(profile);

        let retrieved = index.get("personal");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "personal");
    }

    #[test]
    fn test_profile_index_remove() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("alpha", None));
        index.add(Profile::new("beta", None));
        index.set_active("alpha");

        // Cannot remove the active profile
        let err = index.remove("alpha");
        assert!(err.is_err());
        match err.unwrap_err() {
            SubSwapError::ActiveProfile(name) => assert_eq!(name, "alpha"),
            other => panic!("expected ActiveProfile, got {other}"),
        }

        // Can remove a non-active profile
        let removed = index.remove("beta");
        assert!(removed.is_ok());
        assert_eq!(removed.unwrap().name, "beta");
        assert!(index.get("beta").is_none());
    }

    #[test]
    fn test_profile_index_rename() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("old", None));

        index.rename("old", "new").unwrap();

        assert!(index.get("old").is_none());
        let renamed = index.get("new");
        assert!(renamed.is_some());
        assert_eq!(renamed.unwrap().name, "new");
    }

    #[test]
    fn test_profile_index_rename_to_existing_fails() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("foo", None));
        index.add(Profile::new("bar", None));

        let err = index.rename("foo", "bar");
        assert!(err.is_err());
        match err.unwrap_err() {
            SubSwapError::ProfileExists(name) => assert_eq!(name, "bar"),
            other => panic!("expected ProfileExists, got {other}"),
        }
    }

    #[test]
    fn test_profile_index_set_active_updates_last_used() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("myprofile", None));
        assert!(index.get("myprofile").unwrap().last_used.is_none());

        index.set_active("myprofile");

        assert_eq!(index.active_profile.as_deref(), Some("myprofile"));
        assert!(index.get("myprofile").unwrap().last_used.is_some());
    }

    #[test]
    fn test_profile_index_serialization_roundtrip() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("serialized", Some("a note".to_string())));
        index.set_active("serialized");

        let json = serde_json::to_string(&index).unwrap();
        let deserialized: ProfileIndex = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.version, 1);
        assert_eq!(deserialized.active_profile.as_deref(), Some("serialized"));
        let p = deserialized.get("serialized").unwrap();
        assert_eq!(p.notes.as_deref(), Some("a note"));
        assert!(p.last_used.is_some());
    }

    #[test]
    fn test_profile_names_returns_sorted() {
        let mut index = ProfileIndex::new();
        index.add(Profile::new("zebra", None));
        index.add(Profile::new("apple", None));
        index.add(Profile::new("mango", None));

        let names = index.names();
        assert_eq!(names, vec!["apple", "mango", "zebra"]);
    }
}
