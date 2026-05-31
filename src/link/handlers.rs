use crate::cache;
use crate::link::types;
use crate::plaid;
use crate::utils;
use axum::extract::State;
use axum::Json;

pub async fn get_link_token() -> axum::Json<types::PlaidAuthResponse> {
    println!("[GET TOKEN]: get token called");
    let (client_id, secret) = plaid::load_env();

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

pub async fn exchange_token(
    State(state): State<std::sync::Arc<types::LinkServerState>>,
    Json(payload): Json<types::PublicTokenRequest>,
) {
    println!("[EXCHANGE TOKEN]: exchange token called");
    let (client_id, secret) = plaid::load_env();

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
    cache::save_encrypt_token(access_token.access_token);

    utils::print_success("account linked successfully");

    // Graceful server shutdown after response to client
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
}
