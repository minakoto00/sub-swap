use crate::error::{Result, SubSwapError};
use keyring::Entry;

const SERVICE: &str = "sub-swap";
const ACCOUNT: &str = "encryption-key";

fn encode_key(key: &[u8; 32]) -> String {
    key.iter().fold(String::with_capacity(64), |mut s, b| {
        use std::fmt::Write;
        let _ = write!(s, "{b:02x}");
        s
    })
}

fn decode_key(s: &str) -> Result<[u8; 32]> {
    if s.len() != 64 {
        return Err(SubSwapError::Keychain(format!(
            "stored key has wrong hex length: expected 64 chars, got {}",
            s.len()
        )));
    }
    let mut key = [0u8; 32];
    for (i, chunk) in s.as_bytes().chunks(2).enumerate() {
        let hex_pair = std::str::from_utf8(chunk)
            .map_err(|e| SubSwapError::Keychain(format!("invalid UTF-8 in key hex: {e}")))?;
        key[i] = u8::from_str_radix(hex_pair, 16)
            .map_err(|e| SubSwapError::Keychain(format!("invalid hex in stored key: {e}")))?;
    }
    Ok(key)
}

// ── Trait ─────────────────────────────────────────────────────────────────────

pub trait KeyStore {
    fn get_key(&self) -> Result<[u8; 32]>;
    fn set_key(&self, key: &[u8; 32]) -> Result<()>;
}

// ── OsKeyStore ────────────────────────────────────────────────────────────────

pub struct OsKeyStore;

impl OsKeyStore {
    pub fn new() -> Self {
        Self
    }
}

impl Default for OsKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

impl KeyStore for OsKeyStore {
    fn get_key(&self) -> Result<[u8; 32]> {
        let entry =
            Entry::new(SERVICE, ACCOUNT).map_err(|e| SubSwapError::Keychain(e.to_string()))?;

        let raw = entry
            .get_password()
            .map_err(|e| SubSwapError::Keychain(format!("failed to retrieve key: {e}")))?;

        decode_key(&raw)
    }

    fn set_key(&self, key: &[u8; 32]) -> Result<()> {
        let entry =
            Entry::new(SERVICE, ACCOUNT).map_err(|e| SubSwapError::Keychain(e.to_string()))?;

        let encoded = encode_key(key);
        entry
            .set_password(&encoded)
            .map_err(|e| SubSwapError::Keychain(format!("failed to store key: {e}")))
    }
}

// ── MockKeyStore ──────────────────────────────────────────────────────────────

#[cfg(test)]
pub struct MockKeyStore {
    key: std::cell::RefCell<Option<[u8; 32]>>,
}

#[cfg(test)]
impl MockKeyStore {
    pub fn new() -> Self {
        Self {
            key: std::cell::RefCell::new(None),
        }
    }
}

#[cfg(test)]
impl Default for MockKeyStore {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl KeyStore for MockKeyStore {
    fn get_key(&self) -> Result<[u8; 32]> {
        self.key
            .borrow()
            .ok_or_else(|| SubSwapError::Keychain("no key stored".to_string()))
    }

    fn set_key(&self, key: &[u8; 32]) -> Result<()> {
        *self.key.borrow_mut() = Some(*key);
        Ok(())
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::generate_key;

    #[test]
    fn test_mock_keystore_roundtrip() {
        let store = MockKeyStore::new();
        let key = generate_key();
        store.set_key(&key).expect("set_key should succeed");
        let retrieved = store.get_key().expect("get_key should succeed");
        assert_eq!(retrieved, key);
    }

    #[test]
    fn test_mock_keystore_get_without_set_fails() {
        let store = MockKeyStore::new();
        let result = store.get_key();
        assert!(
            result.is_err(),
            "get_key on empty store should return error"
        );
    }

    #[test]
    fn test_mock_keystore_overwrite() {
        let store = MockKeyStore::new();
        let key1 = generate_key();
        let key2 = generate_key();

        store.set_key(&key1).expect("set key1 should succeed");
        store.set_key(&key2).expect("set key2 should succeed");

        let retrieved = store
            .get_key()
            .expect("get_key after overwrite should succeed");
        assert_eq!(retrieved, key2, "should return latest key after overwrite");
        assert_ne!(
            retrieved, key1,
            "should not return first key after overwrite"
        );
    }
}
