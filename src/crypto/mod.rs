pub mod keychain;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use rand::Rng;

use crate::error::{Result, SubSwapError};

const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;
const MIN_CIPHERTEXT_LEN: usize = NONCE_LEN + TAG_LEN;

pub fn generate_key() -> [u8; 32] {
    let mut key = [0u8; 32];
    rand::rng().fill_bytes(&mut key);
    key
}

pub fn encrypt(plaintext: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| SubSwapError::Crypto(e.to_string()))?;

    let mut output = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);
    Ok(output)
}

pub fn decrypt(data: &[u8], key: &[u8; 32]) -> Result<Vec<u8>> {
    if data.len() < MIN_CIPHERTEXT_LEN {
        return Err(SubSwapError::Crypto(format!(
            "data too short: expected at least {} bytes, got {}",
            MIN_CIPHERTEXT_LEN,
            data.len()
        )));
    }

    let (nonce_bytes, ciphertext) = data.split_at(NONCE_LEN);
    let nonce = Nonce::from_slice(nonce_bytes);

    let aes_key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(aes_key);

    cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| SubSwapError::Crypto(e.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> [u8; 32] {
        let mut key = [0u8; 32];
        key[0] = 0x42;
        key[31] = 0xFF;
        key
    }

    #[test]
    fn test_encrypt_then_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello, sub-swap!";
        let encrypted = encrypt(plaintext, &key).expect("encrypt should succeed");
        let decrypted = decrypt(&encrypted, &key).expect("decrypt should succeed");
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_encrypted_data_differs_from_plaintext() {
        let key = test_key();
        let plaintext = b"secret message";
        let encrypted = encrypt(plaintext, &key).expect("encrypt should succeed");
        // Skip the 12-byte nonce prefix, compare ciphertext portion
        assert_ne!(&encrypted[12..], plaintext.as_ref());
    }

    #[test]
    fn test_encrypted_data_has_nonce_prefix() {
        let key = test_key();
        let plaintext = b"test data for length check";
        let encrypted = encrypt(plaintext, &key).expect("encrypt should succeed");
        // Output = 12-byte nonce + plaintext.len() + 16-byte tag
        assert_eq!(encrypted.len(), 12 + plaintext.len() + 16);
    }

    #[test]
    fn test_decrypt_with_wrong_key_fails() {
        let key = test_key();
        let plaintext = b"top secret";
        let encrypted = encrypt(plaintext, &key).expect("encrypt should succeed");

        let mut wrong_key = test_key();
        wrong_key[5] ^= 0xFF;
        let result = decrypt(&encrypted, &wrong_key);
        assert!(result.is_err(), "decrypt with wrong key should fail");
    }

    #[test]
    fn test_decrypt_tampered_data_fails() {
        let key = test_key();
        let plaintext = b"tamper me";
        let mut encrypted = encrypt(plaintext, &key).expect("encrypt should succeed");
        // Flip the last byte of the tag
        let last = encrypted.len() - 1;
        encrypted[last] ^= 0xFF;
        let result = decrypt(&encrypted, &key);
        assert!(result.is_err(), "decrypt of tampered data should fail");
    }

    #[test]
    fn test_decrypt_too_short_data_fails() {
        let key = test_key();
        let short_data = [0u8; 5];
        let result = decrypt(&short_data, &key);
        assert!(result.is_err(), "decrypt of too-short data should fail");
    }

    #[test]
    fn test_two_encryptions_produce_different_output() {
        let key = test_key();
        let plaintext = b"same plaintext";
        let enc1 = encrypt(plaintext, &key).expect("first encrypt should succeed");
        let enc2 = encrypt(plaintext, &key).expect("second encrypt should succeed");
        // Different nonces produce different ciphertext
        assert_ne!(enc1, enc2, "two encryptions of same plaintext should differ");
    }

    #[test]
    fn test_generate_key_is_32_bytes() {
        let key = generate_key();
        assert_eq!(key.len(), 32);
    }

    #[test]
    fn test_generate_key_is_random() {
        let key1 = generate_key();
        let key2 = generate_key();
        assert_ne!(key1, key2, "two generated keys should differ");
    }
}
