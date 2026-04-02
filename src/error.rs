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
    NotInitialized,
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
                let pids_str: Vec<String> = pids.iter().map(|p| p.to_string()).collect();
                write!(
                    f,
                    "Codex is currently running (PID {}). Switching profiles may cause unexpected behavior.",
                    pids_str.join(", ")
                )
            }
            Self::NotInitialized => {
                write!(f, "sub-swap is not initialized. Run `sub-swap` to set up.")
            }
        }
    }
}

impl std::error::Error for SubSwapError {}

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
