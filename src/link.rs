use crate::utils;
use serde::{Deserialize, Serialize};
use std::{env, process};

fn load_env() -> (String, String) {
    let client_id = env::var("PLAID_CLIENT_ID").unwrap_or_else(|_e| {
        utils::print_error("PLAID_CLIENT_ID environment variable not set");
        process::exit(1);
    });
    let secret = env::var("PLAID_SECRET").unwrap_or_else(|_e| {
        utils::print_error("PLAID_SECRET environment variable not set");
        process::exit(1);
    });

    return (client_id, secret);
}

#[derive(Serialize)]
struct User {
    client_user_id: String,
}

#[derive(Serialize)]
struct LinkRequest {
    client_id: String,
    secret: String,
    client_name: String,
    country_codes: Vec<String>,
    language: String,
    products: Vec<String>,
    user: User,
}

#[derive(Deserialize)]
struct PlaidAuthResponse {
    expiration: String,
    link_token: String,
    request_id: String,
}

fn get_link_token() -> String {
    let (client_id, secret) = load_env();

    let request = LinkRequest {
        client_id,
        secret,
        client_name: "FIN".to_string(),
        country_codes: vec!["US".to_string()],
        language: "en".to_string(),
        products: vec!["auth".to_string()],
        user: User {
            client_user_id: "Jeremy".to_string(),
        },
    };

    let client = reqwest::blocking::Client::new();

    let resp = client
        .post("https://sandbox.plaid.com/link/token/create")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .unwrap_or_else(|_| {
            utils::print_error("failed to create link token");
            process::exit(1);
        });

    let plaid_auth_response: PlaidAuthResponse = resp.json().unwrap_or_else(|_| {
        utils::print_error("response from Plaid was malformed");
        process::exit(1);
    });

    return plaid_auth_response.link_token;
}

#[tokio::main]
pub async fn link() {
    tower_http::services::ServeDir::new()
    let link_token = get_link_token();
}
