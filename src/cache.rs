use crate::utils;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Deserialize, Serialize)]
pub struct EncryptedTokenCache {
    pub tokens: Vec<String>,
}

fn get_token_cache_path() -> PathBuf {
    let fin_dir = dirs::home_dir().unwrap().join(".fin");
    std::fs::create_dir(fin_dir).err();
    return dirs::home_dir().unwrap().join(".fin/tokens.enc");
}

fn write_token_file(cache: EncryptedTokenCache) {
    let file_path = get_token_cache_path();
    let file_contents = serde_json::to_string_pretty(&cache).expect("Failed to read tokens file");
    let encrypted_contents =
        keycrypt::encrypt(file_contents).expect("Failed to encrypt token file");
    std::fs::write(&file_path, encrypted_contents).expect("Failed to write token file");
}

pub fn read_token_file() -> EncryptedTokenCache {
    let file_path = get_token_cache_path();
    let file_res = std::fs::read_to_string(&file_path);

    match file_res {
        Ok(contents) => {
            let decrypted_contents = keycrypt::decrypt(contents).unwrap_or_else(|_| {
                utils::print_error("failed to load existing connections.");
                std::process::exit(1);
            });
            return serde_json::from_str(&decrypted_contents)
                .expect("Failed to parse token contents");
        }
        Err(_) => return EncryptedTokenCache { tokens: vec![] },
    }
}

pub fn save_encrypt_token(token: String) {
    let mut cache = read_token_file();
    cache.tokens.push(token);
    write_token_file(cache);
}

pub fn remove_token(token: String) {
    let mut cache = read_token_file();
    cache.tokens.retain(|t| t != &token);
    write_token_file(cache);
}
