// Thread message encryption: AES-256-GCM for content and HMAC-SHA256 for signatures.
// Keys are 32-byte secrets encoded as standard base64.

use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use openssl::symm::{decrypt_aead, encrypt_aead, Cipher};
use sha2::{Digest, Sha256};

use crate::{AppError, AppResult};
use deadpool_postgres::Pool;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;
const TAG_LEN: usize = 16;

pub struct EncryptionService {
    pool: Pool,
}

impl EncryptionService {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }

    /// Confirm the thread exists, then return a fresh `(aes_key, hmac_key)` pair.
    pub async fn generate_thread_key_pair(&self, thread_id: i64) -> AppResult<(String, String)> {
        let client = self.pool.get().await?;
        let exists = client
            .query_opt(
                "SELECT 1 FROM message_threads WHERE thread_id = $1 AND is_active = TRUE",
                &[&thread_id],
            )
            .await?;
        if exists.is_none() {
            return Err(AppError::NotFound("Thread not found".to_string()));
        }
        let aes = random_key()?;
        let hmac = random_key()?;
        Ok((BASE64.encode(aes), BASE64.encode(hmac)))
    }

    pub async fn encrypt_message(
        &self,
        content: &str,
        thread_key: &str,
    ) -> AppResult<(String, String)> {
        let (ciphertext, nonce) = encrypt(content, thread_key)?;
        Ok((ciphertext, nonce))
    }

    pub async fn decrypt_message(
        &self,
        encrypted_content: &str,
        nonce: &str,
        thread_key: &str,
    ) -> AppResult<String> {
        decrypt(encrypted_content, nonce, thread_key)
    }

    pub async fn sign_message(&self, message: &str, private_key: &str) -> AppResult<String> {
        sign(message, private_key)
    }

    pub async fn verify_signature(
        &self,
        message: &str,
        signature: &str,
        public_key: &str,
    ) -> AppResult<bool> {
        verify(message, signature, public_key)
    }
}

fn random_key() -> AppResult<[u8; KEY_LEN]> {
    let mut key = [0u8; KEY_LEN];
    openssl::rand::rand_bytes(&mut key).map_err(|e| AppError::Internal(e.to_string()))?;
    Ok(key)
}

fn decode_key(key: &str) -> AppResult<Vec<u8>> {
    let bytes = BASE64
        .decode(key.trim())
        .map_err(|_| AppError::BadRequest("Encryption key must be standard base64".into()))?;
    if bytes.len() != KEY_LEN {
        return Err(AppError::BadRequest(
            "Encryption key must be 32 bytes".into(),
        ));
    }
    Ok(bytes)
}

fn encrypt(content: &str, thread_key: &str) -> AppResult<(String, String)> {
    let key = decode_key(thread_key)?;
    let mut nonce = [0u8; NONCE_LEN];
    openssl::rand::rand_bytes(&mut nonce).map_err(|e| AppError::Internal(e.to_string()))?;
    let mut tag = [0u8; TAG_LEN];
    let mut ciphertext = encrypt_aead(
        Cipher::aes_256_gcm(),
        &key,
        Some(&nonce),
        &[],
        content.as_bytes(),
        &mut tag,
    )
    .map_err(|e| AppError::Internal(format!("Encrypt failed: {e}")))?;
    ciphertext.extend_from_slice(&tag);
    Ok((BASE64.encode(ciphertext), BASE64.encode(nonce)))
}

fn decrypt(encrypted_content: &str, nonce: &str, thread_key: &str) -> AppResult<String> {
    let key = decode_key(thread_key)?;
    let nonce = BASE64
        .decode(nonce.trim())
        .map_err(|_| AppError::BadRequest("Invalid nonce".into()))?;
    let payload = BASE64
        .decode(encrypted_content.trim())
        .map_err(|_| AppError::BadRequest("Invalid ciphertext".into()))?;
    if payload.len() < TAG_LEN {
        return Err(AppError::BadRequest("Ciphertext is too short".into()));
    }
    let (ciphertext, tag) = payload.split_at(payload.len() - TAG_LEN);
    let plain = decrypt_aead(
        Cipher::aes_256_gcm(),
        &key,
        Some(&nonce),
        &[],
        ciphertext,
        tag,
    )
    .map_err(|_| AppError::BadRequest("Decryption failed".into()))?;
    String::from_utf8(plain)
        .map_err(|_| AppError::BadRequest("Decrypted message is not text".into()))
}

fn hmac_sha256(key: &[u8], message: &[u8]) -> [u8; 32] {
    const BLOCK: usize = 64;
    let mut block_key = [0u8; BLOCK];
    if key.len() > BLOCK {
        let hashed = Sha256::digest(key);
        block_key[..hashed.len()].copy_from_slice(&hashed);
    } else {
        block_key[..key.len()].copy_from_slice(key);
    }
    let mut ipad = [0u8; BLOCK];
    let mut opad = [0u8; BLOCK];
    for i in 0..BLOCK {
        ipad[i] = block_key[i] ^ 0x36;
        opad[i] = block_key[i] ^ 0x5c;
    }
    let mut inner = Sha256::new();
    inner.update(ipad);
    inner.update(message);
    let inner_hash = inner.finalize();
    let mut outer = Sha256::new();
    outer.update(opad);
    outer.update(inner_hash);
    let digest = outer.finalize();
    let mut mac = [0u8; 32];
    mac.copy_from_slice(&digest);
    mac
}

fn sign(message: &str, private_key: &str) -> AppResult<String> {
    let key = decode_key(private_key)?;
    Ok(BASE64.encode(hmac_sha256(&key, message.as_bytes())))
}

fn verify(message: &str, signature: &str, public_key: &str) -> AppResult<bool> {
    let key = decode_key(public_key)?;
    let signature = match BASE64.decode(signature.trim()) {
        Ok(bytes) => bytes,
        Err(_) => return Ok(false),
    };
    let expected = hmac_sha256(&key, message.as_bytes());
    if signature.len() != expected.len() {
        return Ok(false);
    }
    let mut diff = 0u8;
    for (left, right) in signature.iter().zip(expected.iter()) {
        diff |= left ^ right;
    }
    Ok(diff == 0)
}

#[cfg(test)]
mod tests {
    use super::{decrypt, encrypt, hmac_sha256, sign, verify};

    #[test]
    fn hmac_matches_rfc4231_case1() {
        let key = [0x0bu8; 20];
        let mac = hmac_sha256(&key, b"Hi There");
        assert_eq!(
            mac,
            [
                0xb0, 0x34, 0x4c, 0x61, 0xd8, 0xdb, 0x38, 0x53, 0x5c, 0xa8, 0xaf, 0xce, 0xaf, 0x0b,
                0xf1, 0x2b, 0x88, 0x1d, 0xc2, 0x00, 0xc9, 0x83, 0x3d, 0xa7, 0x26, 0xe9, 0x37, 0x6c,
                0x2e, 0x32, 0xcf, 0xf7,
            ]
        );
    }

    #[test]
    fn round_trip_encrypts_and_signs() {
        let key = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [7u8; 32]);
        let (ciphertext, nonce) = encrypt("coi ro do", &key).unwrap();
        assert_ne!(ciphertext, "coi ro do");
        assert_eq!(decrypt(&ciphertext, &nonce, &key).unwrap(), "coi ro do");
        let signature = sign("coi ro do", &key).unwrap();
        assert!(verify("coi ro do", &signature, &key).unwrap());
        assert!(!verify("coi do", &signature, &key).unwrap());
    }
}
