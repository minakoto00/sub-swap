use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SubSwapError {
    Io(io::Error),
    Json(serde_json::Error),
    Crypto(String),
    Keychain(String),
    ProfileNotFound(String),
    ProfileExists(String),
    ActiveProfile(String),
    NoCodexConfig,
    CodexRunning(Vec<u32>),
    InvalidProfileName(String),
}

impl fmt::Display for SubSwapError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Json(e) => write!(f, "JSON error: {e}"),
            Self::Crypto(msg) => write!(f, "Encryption error: {msg}"),
            Self::Keychain(msg) => write!(f, "Keychain error: {msg}"),
            Self::ProfileNotFound(name) => {
                write!(f, "Profile '{name}' not found. Run `sub-swap list` to see available profiles.")
            }
            Self::ProfileExists(name) => {
                write!(f, "Profile '{name}' already exists. Use a different name or `sub-swap remove {name}` first.")
            }
            Self::ActiveProfile(name) => {
                write!(f, "Cannot remove the active profile '{name}'. Switch to another profile first.")
            }
            Self::NoCodexConfig => write!(f, "No auth.json found in ~/.codex/. Nothing to import."),
            Self::CodexRunning(pids) => {
                let pids_str: Vec<String> = pids.iter().map(ToString::to_string).collect();
                write!(
                    f,
                    "Codex is currently running (PID {}). Switching profiles may cause unexpected behavior.",
                    pids_str.join(", ")
                )
            }
            Self::InvalidProfileName(name) => {
                write!(f, "Invalid profile name '{name}'. Names must be non-empty, alphanumeric (with hyphens and underscores allowed), and cannot contain path separators.")
            }
        }
    }
}

impl std::error::Error for SubSwapError {}

/// Validate that a profile name is safe to use as a filesystem path component.
///
/// Rules:
/// - Must be non-empty
/// - Must not contain `/`, `\`, or `..`
/// - Must not start with `.`
/// - Characters must be alphanumeric, `-`, or `_`
pub fn validate_profile_name(name: &str) -> Result<()> {
    if name.is_empty() {
        return Err(SubSwapError::InvalidProfileName(name.to_string()));
    }
    if name.contains('/') || name.contains('\\') || name.contains("..") || name.starts_with('.') {
        return Err(SubSwapError::InvalidProfileName(name.to_string()));
    }
    // Only allow alphanumeric, hyphens, underscores
    if !name.chars().all(|c| c.is_alphanumeric() || c == '-' || c == '_') {
        return Err(SubSwapError::InvalidProfileName(name.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_profile_names() {
        assert!(validate_profile_name("work").is_ok());
        assert!(validate_profile_name("my-profile").is_ok());
        assert!(validate_profile_name("test_123").is_ok());
    }

    #[test]
    fn test_invalid_profile_names() {
        assert!(validate_profile_name("").is_err());
        assert!(validate_profile_name("../etc").is_err());
        assert!(validate_profile_name("foo/bar").is_err());
        assert!(validate_profile_name("foo\\bar").is_err());
        assert!(validate_profile_name(".hidden").is_err());
        assert!(validate_profile_name("has space").is_err());
    }
}

impl From<io::Error> for SubSwapError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<serde_json::Error> for SubSwapError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

pub type Result<T> = std::result::Result<T, SubSwapError>;
