use std::path::{Path, PathBuf};
use std::process;

use crate::link::types;
use crate::utils;
use axum::extract::State;
use axum::Json;
use dirs;
use keycrypt;

fn load_env() -> (String, String) {
    let client_id = std::env::var("PLAID_CLIENT_ID").unwrap_or_else(|_e| {
        utils::print_error("PLAID_CLIENT_ID environment variable not set");
        std::process::exit(1);
    });
    let secret = std::env::var("PLAID_SECRET").unwrap_or_else(|_e| {
        utils::print_error("PLAID_SECRET environment variable not set");
        std::process::exit(1);
    });

    return (client_id, secret);
}

pub async fn get_link_token() -> axum::Json<types::PlaidAuthResponse> {
    println!("[GET TOKEN]: get token called");
    let (client_id, secret) = load_env();

    let request = types::LinkRequest {
        client_id,
        secret,
        client_name: "FIN".to_string(),
        country_codes: vec!["US".to_string()],
        language: "en".to_string(),
        products: vec!["auth".to_string()],
        user: types::User {
            client_user_id: "Jeremy".to_string(),
        },
    };

    let client = reqwest::Client::new();

    let resp = client
        .post("https://sandbox.plaid.com/link/token/create")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            utils::print_error("failed to create link token");
            std::process::exit(1);
        });

    let plaid_auth_response: types::PlaidAuthResponse = resp.json().await.unwrap_or_else(|_| {
        utils::print_error("response from Plaid was malformed");
        std::process::exit(1);
    });

    return axum::Json(plaid_auth_response);
}

fn get_token_cache_path() -> PathBuf {
    let fin_dir = dirs::home_dir().unwrap().join(".fin");
    std::fs::create_dir(fin_dir).err();
    return dirs::home_dir().unwrap().join(".fin/tokens.enc");
}

fn write_token_file(cache: types::EncryptedTokenCache) {
    let file_path = get_token_cache_path();
    let file_contents = serde_json::to_string_pretty(&cache).expect("Failed to read tokens file");
    let encrypted_contents =
        keycrypt::encrypt(file_contents).expect("Failed to encrypt token file");
    std::fs::write(&file_path, encrypted_contents).expect("Failed to write token file");
}

fn read_token_file() -> types::EncryptedTokenCache {
    let file_path = get_token_cache_path();
    let file_res = std::fs::read_to_string(&file_path);

    match file_res {
        Ok(contents) => {
            return serde_json::from_str(&contents).expect("Failed to parse token contents")
        }
        Err(_) => return types::EncryptedTokenCache { tokens: vec![] },
    }
}

fn save_encrypt_token(token: String) {
    let mut cache = read_token_file();
    cache.tokens.push(token);
    write_token_file(cache);
}

pub async fn exchange_token(
    State(state): State<std::sync::Arc<types::LinkServerState>>,
    Json(payload): Json<types::PublicTokenRequest>,
) {
    println!("[EXCHANGE TOKEN]: exchange token called");
    let (client_id, secret) = load_env();

    let request = types::TokenExchangeRequest {
        client_id,
        secret,
        public_token: payload.public_token,
    };

    let client = reqwest::Client::new();
    let resp = client
        .post("https://sandbox.plaid.com/item/public_token/exchange")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            utils::print_error("failed to exchange token");
            std::process::exit(1);
        });

    let access_token: types::TokenExchangeResponse = resp.json().await.unwrap_or_else(|_| {
        utils::print_error("response from Plaid was malformed");
        std::process::exit(1);
    });

    // Save token to encrypted file
    save_encrypt_token(access_token.access_token);

    utils::print_success("account linked successfully");

    // Graceful server shutdown after response to client
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
}
