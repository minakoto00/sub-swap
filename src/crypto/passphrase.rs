use argon2::{Algorithm, Argon2, Params, Version};
use base64::{engine::general_purpose::STANDARD, Engine as _};
use rand::Rng;

use crate::error::{Result, SubSwapError};

pub const SALT_LEN: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PassphraseParams {
    pub salt: [u8; SALT_LEN],
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
}

pub fn default_params_with_random_salt() -> PassphraseParams {
    let mut salt = [0u8; SALT_LEN];
    rand::rng().fill_bytes(&mut salt);
    PassphraseParams {
        salt,
        memory_kib: 65_536,
        iterations: 3,
        parallelism: 1,
    }
}

pub fn derive_key(passphrase: &str, params: &PassphraseParams) -> Result<[u8; 32]> {
    let argon_params = Params::new(
        params.memory_kib,
        params.iterations,
        params.parallelism,
        Some(32),
    )
    .map_err(|e| SubSwapError::Crypto(format!("invalid Argon2 parameters: {e}")))?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, argon_params);
    let mut output = [0u8; 32];
    argon2
        .hash_password_into(passphrase.as_bytes(), &params.salt, &mut output)
        .map_err(|e| SubSwapError::Crypto(format!("failed to derive encryption key: {e}")))?;
    Ok(output)
}

pub fn encode_salt_b64(salt: &[u8; SALT_LEN]) -> String {
    STANDARD.encode(salt)
}

pub fn decode_salt_b64(raw: &str) -> Result<[u8; SALT_LEN]> {
    let decoded = STANDARD
        .decode(raw)
        .map_err(|e| SubSwapError::Crypto(format!("invalid base64 salt: {e}")))?;

    if decoded.len() != SALT_LEN {
        return Err(SubSwapError::Crypto(format!(
            "invalid salt length: expected {SALT_LEN} bytes, got {}",
            decoded.len()
        )));
    }

    let mut salt = [0u8; SALT_LEN];
    salt.copy_from_slice(&decoded);
    Ok(salt)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::SubSwapError;

    const EXPECTED_FIXED_DERIVED_KEY: [u8; 32] = [
        106, 209, 10, 249, 127, 23, 68, 17, 155, 215, 19, 92, 133, 18, 29, 197, 137, 121, 79, 156, 93,
        100, 98, 0, 184, 173, 77, 107, 236, 241, 80, 132,
    ];

    fn default_test_params() -> PassphraseParams {
        PassphraseParams {
            salt: [7u8; SALT_LEN],
            memory_kib: 65_536,
            iterations: 3,
            parallelism: 1,
        }
    }

    #[test]
    fn test_derive_key_matches_fixed_output() {
        let params = default_test_params();
        let derived =
            derive_key("correct horse battery staple", &params).expect("derive key should succeed");
        assert_eq!(derived, EXPECTED_FIXED_DERIVED_KEY);
    }

    #[test]
    fn test_derive_key_changes_when_salt_changes() {
        let base = PassphraseParams {
            salt: [1u8; SALT_LEN],
            memory_kib: 65_536,
            iterations: 3,
            parallelism: 1,
        };
        let different = PassphraseParams {
            salt: [2u8; SALT_LEN],
            memory_kib: 65_536,
            iterations: 3,
            parallelism: 1,
        };

        let first = derive_key("same-passphrase", &base).unwrap();
        let second = derive_key("same-passphrase", &different).unwrap();

        assert_ne!(first, second);
    }

    #[test]
    fn test_decode_salt_b64_rejects_malformed_input() {
        let result = decode_salt_b64("not-base64!");
        match result {
            Err(SubSwapError::Crypto(msg)) => {
                assert!(
                    msg.contains("invalid base64 salt"),
                    "unexpected error message: {msg}"
                );
            }
            _ => panic!("expected base64 error"),
        }
    }

    #[test]
    fn test_decode_salt_b64_rejects_wrong_length() {
        let encoded_short = STANDARD.encode(&[1u8, 2, 3, 4, 5]);
        let result = decode_salt_b64(&encoded_short);
        match result {
            Err(SubSwapError::Crypto(msg)) => {
                assert!(
                    msg.contains("invalid salt length"),
                    "unexpected error message: {msg}"
                );
            }
            _ => panic!("expected salt length error"),
        }
    }

    #[test]
    fn test_decode_salt_b64_roundtrip() {
        let salt = [9u8; SALT_LEN];
        let encoded = encode_salt_b64(&salt);
        let decoded = decode_salt_b64(&encoded).unwrap();
        assert_eq!(decoded, salt);
    }

    #[test]
    fn test_derive_key_rejects_invalid_params() {
        let params = PassphraseParams {
            salt: [7u8; SALT_LEN],
            memory_kib: 0,
            iterations: 3,
            parallelism: 1,
        };

        let result = derive_key("any-passphrase", &params);
        match result {
            Err(SubSwapError::Crypto(msg)) => {
                assert!(
                    msg.contains("invalid Argon2 parameters"),
                    "unexpected error message: {msg}"
                );
            }
            _ => panic!("expected Argon2 parameter error"),
        }
    }
}
