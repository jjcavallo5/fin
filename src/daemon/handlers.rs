use crate::daemon::encryption::{self, EncryptedBlob, NONCE_LEN, SALT_LEN};
use crate::daemon::types;
use crate::logging;

pub fn ping() -> types::DaemonResponse {
    logging::success("connection to daemon successful");
    return types::DaemonResponse::Ok;
}

pub fn login(pass: String, password: &mut String) -> types::DaemonResponse {
    password.clear();
    password.push_str(pass.as_str());
    return types::DaemonResponse::Ok;
}

pub fn stop() -> types::DaemonResponse {
    return types::DaemonResponse::Quit;
}

fn derive_key(password: &String, db_salt: &[u8; SALT_LEN]) -> Result<[u8; 32], String> {
    if password.is_empty() {
        return Err("daemon is not logged in".to_string());
    }

    encryption::derive_key(password, db_salt)
        .map_err(|_| "failed to derive encryption key".to_string())
}

pub fn encrypt(
    token: String,
    password: &String,
    db_salt: &[u8; SALT_LEN],
) -> types::DaemonResponse {
    let key = match derive_key(password, db_salt) {
        Ok(key) => key,
        Err(message) => return types::DaemonResponse::Error { message },
    };

    match encryption::encrypt_token(&key, &token) {
        Ok(blob) => types::DaemonResponse::Encrypted {
            nonce: encryption::encode_hex(&blob.nonce),
            ciphertext: encryption::encode_hex(&blob.ciphertext),
        },
        Err(message) => types::DaemonResponse::Error { message },
    }
}

pub fn decrypt(
    nonce: String,
    ciphertext: String,
    password: &String,
    db_salt: &[u8; SALT_LEN],
) -> types::DaemonResponse {
    let key = match derive_key(password, db_salt) {
        Ok(key) => key,
        Err(message) => return types::DaemonResponse::Error { message },
    };

    let nonce_bytes = match encryption::decode_hex(&nonce) {
        Ok(bytes) if bytes.len() == NONCE_LEN => bytes,
        Ok(_) => {
            return types::DaemonResponse::Error {
                message: format!("nonce must be {} bytes", NONCE_LEN),
            };
        }
        Err(message) => return types::DaemonResponse::Error { message },
    };

    let ciphertext_bytes = match encryption::decode_hex(&ciphertext) {
        Ok(bytes) => bytes,
        Err(message) => return types::DaemonResponse::Error { message },
    };

    let nonce_array: [u8; NONCE_LEN] = nonce_bytes.try_into().unwrap();
    let blob = EncryptedBlob {
        nonce: nonce_array,
        ciphertext: ciphertext_bytes,
    };

    match encryption::decrypt_token(&key, &blob) {
        Ok(token) => types::DaemonResponse::Data { token },
        Err(message) => types::DaemonResponse::Error { message },
    }
}
