use sysinfo::{ProcessesToUpdate, System};

use crate::error::{Result, SubSwapError};

pub trait CodexGuard {
    fn check(&self) -> Result<()>;
    fn find_codex_pids(&self) -> Vec<u32>;
}

// ── Production guard ─────────────────────────────────────────────────────────

pub struct OsGuard;

impl OsGuard {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OsGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl CodexGuard for OsGuard {
    fn find_codex_pids(&self) -> Vec<u32> {
        let mut sys = System::new();
        sys.refresh_processes(ProcessesToUpdate::All, true);
        sys.processes()
            .values()
            .filter(|p| {
                let name = p.name().to_string_lossy().to_lowercase();
                name == "codex" || name == "codex.exe"
            })
            .map(|p| p.pid().as_u32())
            .collect()
    }

    fn check(&self) -> Result<()> {
        let pids = self.find_codex_pids();
        if pids.is_empty() {
            Ok(())
        } else {
            Err(SubSwapError::CodexRunning(pids))
        }
    }
}

// ── Test double ──────────────────────────────────────────────────────────────

pub struct MockGuard {
    pids: Vec<u32>,
}

impl MockGuard {
    pub fn new(pids: Vec<u32>) -> Self {
        Self { pids }
    }
}

impl CodexGuard for MockGuard {
    fn find_codex_pids(&self) -> Vec<u32> {
        self.pids.clone()
    }

    fn check(&self) -> Result<()> {
        let pids = self.find_codex_pids();
        if pids.is_empty() {
            Ok(())
        } else {
            Err(SubSwapError::CodexRunning(pids))
        }
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mock_guard_no_processes() {
        let guard = MockGuard::new(vec![]);
        assert!(guard.check().is_ok());
    }

    #[test]
    fn test_mock_guard_with_processes() {
        let guard = MockGuard::new(vec![12345, 67890]);
        let result = guard.check();
        assert!(result.is_err());
        match result.unwrap_err() {
            SubSwapError::CodexRunning(pids) => {
                assert_eq!(pids, vec![12345, 67890]);
            }
            other => panic!("Expected CodexRunning, got: {:?}", other),
        }
    }

    #[test]
    fn test_os_guard_constructs() {
        let _guard = OsGuard::new();
        // Just verifying construction doesn't panic
    }
}
