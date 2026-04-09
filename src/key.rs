use crate::config::{AppConfig, KeyBackend, PassphraseKdfConfig};
use crate::crypto;
use crate::crypto::keychain::KeyStore;
use crate::crypto::passphrase::{
    PassphraseParams, decode_salt_b64, default_params_with_random_salt, derive_key,
    encode_salt_b64,
};
use crate::error::{Result, SubSwapError};

pub fn resolve_key(
    config: &AppConfig,
    keystore: &impl KeyStore,
    passphrase: Option<&str>,
) -> Result<[u8; 32]> {
    if !config.encryption_enabled {
        return Ok([0u8; 32]);
    }

    match config.key_backend {
        Some(KeyBackend::Native) => keystore.get_key(),
        Some(KeyBackend::Passphrase) => {
            let passphrase = passphrase.ok_or_else(|| {
                SubSwapError::Crypto(
                    "passphrase backend is configured but no passphrase was provided".to_string(),
                )
            })?;
            let kdf = config.passphrase_kdf.as_ref().ok_or_else(|| {
                SubSwapError::Crypto("passphrase backend requires KDF metadata".to_string())
            })?;
            let params = PassphraseParams {
                salt: decode_salt_b64(&kdf.salt_b64)?,
                memory_kib: kdf.memory_kib,
                iterations: kdf.iterations,
                parallelism: kdf.parallelism,
            };
            derive_key(passphrase, &params)
        }
        None => Err(SubSwapError::Crypto(
            "encryption is enabled but no key backend is configured".to_string(),
        )),
    }
}

pub fn initialize_native_backend(keystore: &impl KeyStore) -> Result<[u8; 32]> {
    if let Ok(existing_key) = keystore.get_key() {
        return Ok(existing_key);
    }

    let key = crypto::generate_key();
    keystore.set_key(&key)?;
    Ok(key)
}

pub fn initialize_passphrase_backend(passphrase: &str) -> Result<(PassphraseKdfConfig, [u8; 32])> {
    let params = default_params_with_random_salt();
    let key = derive_key(passphrase, &params)?;
    let config = PassphraseKdfConfig {
        salt_b64: encode_salt_b64(&params.salt),
        memory_kib: params.memory_kib,
        iterations: params.iterations,
        parallelism: params.parallelism,
    };
    Ok((config, key))
}

pub fn backend_label(backend: &KeyBackend) -> &'static str {
    match backend {
        KeyBackend::Native => native_backend_label(),
        KeyBackend::Passphrase => "passphrase-derived key",
    }
}

#[cfg(target_os = "macos")]
fn native_backend_label() -> &'static str {
    "macOS Keychain"
}

#[cfg(target_os = "linux")]
fn native_backend_label() -> &'static str {
    "system keyring (Secret Service)"
}

#[cfg(target_os = "windows")]
fn native_backend_label() -> &'static str {
    "Windows Credential Manager"
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn native_backend_label() -> &'static str {
    "OS keychain"
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::keychain::{KeyStore, MockKeyStore};

    #[test]
    fn test_resolve_key_returns_zero_key_when_encryption_disabled() {
        let config = AppConfig {
            encryption_enabled: false,
            key_backend: None,
            passphrase_kdf: None,
        };

        let store = MockKeyStore::new();
        let key = resolve_key(&config, &store, None).unwrap();
        assert_eq!(key, [0u8; 32]);
    }

    #[test]
    fn test_resolve_key_reads_native_key_from_store() {
        let config = AppConfig {
            encryption_enabled: true,
            key_backend: Some(KeyBackend::Native),
            passphrase_kdf: None,
        };

        let store = MockKeyStore::new();
        let expected = crate::crypto::generate_key();
        store.set_key(&expected).unwrap();

        let actual = resolve_key(&config, &store, None).unwrap();
        assert_eq!(actual, expected);
    }

    #[test]
    fn test_resolve_key_derives_passphrase_backend() {
        let config = AppConfig {
            encryption_enabled: true,
            key_backend: Some(KeyBackend::Passphrase),
            passphrase_kdf: Some(PassphraseKdfConfig {
                salt_b64: "CQkJCQkJCQkJCQkJCQkJCQ==".to_string(),
                memory_kib: 65_536,
                iterations: 3,
                parallelism: 1,
            }),
        };

        let store = MockKeyStore::new();
        let derived = resolve_key(&config, &store, Some("hunter2")).unwrap();
        let repeated = resolve_key(&config, &store, Some("hunter2")).unwrap();
        assert_eq!(derived, repeated);
    }

    #[test]
    fn test_resolve_key_passphrase_backend_requires_passphrase() {
        let config = AppConfig {
            encryption_enabled: true,
            key_backend: Some(KeyBackend::Passphrase),
            passphrase_kdf: Some(PassphraseKdfConfig {
                salt_b64: "CQkJCQkJCQkJCQkJCQkJCQ==".to_string(),
                memory_kib: 65_536,
                iterations: 3,
                parallelism: 1,
            }),
        };

        let store = MockKeyStore::new();
        let result = resolve_key(&config, &store, None);

        match result {
            Err(SubSwapError::Crypto(message)) => {
                assert_eq!(
                    message,
                    "passphrase backend is configured but no passphrase was provided"
                );
            }
            other => panic!("expected crypto error, got {other:?}"),
        }
    }

    #[test]
    fn test_initialize_native_backend_is_idempotent() {
        let store = MockKeyStore::new();

        let first = initialize_native_backend(&store).unwrap();
        let second = initialize_native_backend(&store).unwrap();

        assert_eq!(first, second);
    }

    #[test]
    fn test_initialize_passphrase_backend_returns_config_and_key() {
        let (kdf, key) = initialize_passphrase_backend("correct horse battery staple").unwrap();
        assert_eq!(kdf.memory_kib, 65_536);
        assert_eq!(kdf.iterations, 3);
        assert_eq!(kdf.parallelism, 1);
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_backend_label_for_passphrase_backend() {
        assert_eq!(
            backend_label(&KeyBackend::Passphrase),
            "passphrase-derived key"
        );
    }
}
