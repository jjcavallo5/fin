use argon2::{Algorithm, Argon2, Params, Version};
use chacha20poly1305::{
    aead::{Aead, KeyInit},
    Key, XChaCha20Poly1305, XNonce,
};
use rand_core::{OsRng, RngCore};

pub const SALT_LEN: usize = 32;
pub const KEY_LEN: usize = 32;
pub const NONCE_LEN: usize = 24;

#[derive(Debug, Clone)]
pub struct EncryptedBlob {
    pub nonce: [u8; NONCE_LEN],
    pub ciphertext: Vec<u8>,
}

/// Generate this once when initializing the local DB.
/// Store it in the DB header/config.
pub fn generate_db_salt() -> [u8; SALT_LEN] {
    let mut salt = [0u8; SALT_LEN];
    OsRng.fill_bytes(&mut salt);
    salt
}

/// Derive a session key from the user password + stored DB salt.
pub fn derive_key(
    password: &str,
    db_salt: &[u8; SALT_LEN],
) -> Result<[u8; KEY_LEN], argon2::Error> {
    let params = Params::new(
        64 * 1024, // memory cost: 64 MiB
        3,         // iterations
        1,         // parallelism
        Some(KEY_LEN),
    )?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);

    let mut key = [0u8; KEY_LEN];
    argon2.hash_password_into(password.as_bytes(), db_salt, &mut key)?;

    Ok(key)
}

pub fn encrypt_token(key_bytes: &[u8; KEY_LEN], token: &str) -> Result<EncryptedBlob, String> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_bytes));

    let mut nonce = [0u8; NONCE_LEN];
    OsRng.fill_bytes(&mut nonce);

    let ciphertext = cipher
        .encrypt(XNonce::from_slice(&nonce), token.as_bytes())
        .map_err(|_| "encryption failed".to_string())?;

    Ok(EncryptedBlob { nonce, ciphertext })
}

pub fn decrypt_token(key_bytes: &[u8; KEY_LEN], blob: &EncryptedBlob) -> Result<String, String> {
    let cipher = XChaCha20Poly1305::new(Key::from_slice(key_bytes));

    let plaintext = cipher
        .decrypt(XNonce::from_slice(&blob.nonce), blob.ciphertext.as_ref())
        .map_err(|_| "wrong password or corrupted ciphertext".to_string())?;

    String::from_utf8(plaintext).map_err(|_| "decrypted token was not valid UTF-8".to_string())
}
