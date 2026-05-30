use crate::link::types;
use crate::utils;
use axum;

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

pub async fn exchange_token() {
    println!("EXCHANGE!")
}
