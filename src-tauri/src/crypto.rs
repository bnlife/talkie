use aes_gcm::{
    aead::{Aead, KeyInit},
    Aes256Gcm, Nonce,
};
use base64::{engine::general_purpose::STANDARD as B64, Engine};
use rand::RngCore;
use std::sync::Mutex;

const SERVICE_NAME: &str = "Talkie";
const KEY_NAME: &str = "master_key";
const ENCRYPTED_PREFIX: &str = "ENC:";

static MASTER_KEY: Mutex<Option<[u8; 32]>> = Mutex::new(None);

/// Get or create a 32-byte master key stored in the OS keyring.
/// Cached in memory after first access.
fn get_master_key() -> Result<[u8; 32], String> {
    let mut cached = MASTER_KEY.lock().map_err(|e| format!("lock fail: {e}"))?;
    if let Some(key) = *cached {
        return Ok(key);
    }

    let entry = keyring::Entry::new(SERVICE_NAME, KEY_NAME)
        .map_err(|e| format!("keyring entry fail: {e}"))?;

    let key = match entry.get_password() {
        Ok(encoded) => {
            let bytes = B64.decode(&encoded)
                .map_err(|e| format!("master key decode fail: {e}"))?;
            if bytes.len() != 32 {
                return Err("master key length mismatch".into());
            }
            let mut k = [0u8; 32];
            k.copy_from_slice(&bytes);
            k
        }
        Err(_) => {
            let mut k = [0u8; 32];
            rand::thread_rng().fill_bytes(&mut k);
            let encoded = B64.encode(k);
            entry.set_password(&encoded)
                .map_err(|e| format!("keyring set fail: {e}"))?;
            log::info!("RS::crypto | master key created");
            k
        }
    };

    *cached = Some(key);
    Ok(key)
}

/// Encrypt a plaintext string. Returns `ENC:<base64>`.
pub fn encrypt(plaintext: &str) -> Result<String, String> {
    let key = get_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("cipher init fail: {e}"))?;

    let mut nonce_bytes = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext.as_bytes())
        .map_err(|e| format!("encrypt fail: {e}"))?;

    // Prepend nonce to ciphertext
    let mut combined = Vec::with_capacity(12 + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(format!("{ENCRYPTED_PREFIX}{}", B64.encode(combined)))
}

/// Decrypt an `ENC:<base64>` string back to plaintext.
pub fn decrypt(encoded: &str) -> Result<String, String> {
    let b64 = encoded
        .strip_prefix(ENCRYPTED_PREFIX)
        .ok_or("not an ENC: value")?;

    let combined = B64.decode(b64)
        .map_err(|e| format!("base64 decode fail: {e}"))?;

    if combined.len() < 12 {
        return Err("encrypted data too short".into());
    }

    let (nonce_bytes, ciphertext) = combined.split_at(12);
    let nonce = Nonce::from_slice(nonce_bytes);

    let key = get_master_key()?;
    let cipher = Aes256Gcm::new_from_slice(&key)
        .map_err(|e| format!("cipher init fail: {e}"))?;

    let plaintext = cipher
        .decrypt(nonce, ciphertext)
        .map_err(|e| format!("decrypt fail: {e}"))?;

    String::from_utf8(plaintext)
        .map_err(|e| format!("utf8 fail: {e}"))
}

/// Check if a value is already encrypted.
pub fn is_encrypted(value: &str) -> bool {
    value.starts_with(ENCRYPTED_PREFIX)
}

/// Ensure an API key is encrypted. If already encrypted, return as-is.
pub fn ensure_encrypted(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    if is_encrypted(value) {
        Ok(value.to_string())
    } else {
        encrypt(value)
    }
}

/// Decrypt an API key if it's encrypted. If not encrypted (legacy), return as-is.
pub fn ensure_decrypted(value: &str) -> Result<String, String> {
    if value.is_empty() {
        return Ok(String::new());
    }
    if is_encrypted(value) {
        decrypt(value)
    } else {
        // Legacy plaintext value — pass through (will be encrypted on next save)
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encrypt_decrypt_roundtrip() {
        let plaintext = "sk-test-12345";
        let encrypted = encrypt(plaintext).unwrap();
        assert!(is_encrypted(&encrypted));
        let decrypted = decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn different_nonces_each_time() {
        let plaintext = "sk-same-key";
        let a = encrypt(plaintext).unwrap();
        let b = encrypt(plaintext).unwrap();
        // Same plaintext → different ciphertext (random nonce)
        assert_ne!(a, b);
        // Both decrypt to same plaintext
        assert_eq!(decrypt(&a).unwrap(), plaintext);
        assert_eq!(decrypt(&b).unwrap(), plaintext);
    }

    #[test]
    fn is_encrypted_detects_prefix() {
        assert!(is_encrypted("ENC:abc123"));
        assert!(!is_encrypted("sk-plaintext"));
        assert!(!is_encrypted(""));
    }

    #[test]
    fn ensure_encrypted_idempotent() {
        let encrypted = encrypt("sk-key").unwrap();
        let again = ensure_encrypted(&encrypted).unwrap();
        assert_eq!(encrypted, again);
    }

    #[test]
    fn ensure_decrypted_passthrough_plaintext() {
        let result = ensure_decrypted("sk-legacy").unwrap();
        assert_eq!(result, "sk-legacy");
    }

    #[test]
    fn ensure_decrypted_decrypts_encrypted() {
        let encrypted = encrypt("sk-secret").unwrap();
        let result = ensure_decrypted(&encrypted).unwrap();
        assert_eq!(result, "sk-secret");
    }

    #[test]
    fn empty_string_handled() {
        assert_eq!(ensure_encrypted("").unwrap(), "");
        assert_eq!(ensure_decrypted("").unwrap(), "");
    }

    #[test]
    fn tampered_ciphertext_fails() {
        let encrypted = encrypt("sk-key").unwrap();
        let mut tampered = encrypted.clone();
        tampered.push('X');
        assert!(decrypt(&tampered).is_err());
    }
}
