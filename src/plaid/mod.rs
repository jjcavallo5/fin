use crate::cache;
use crate::utils;
mod types;

pub fn load_env() -> (String, String) {
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

pub async fn get_linked_items() -> Vec<types::GetAccountResponse> {
    let (client_id, secret) = load_env();
    let token_cache = cache::read_token_file();
    let client = reqwest::Client::new();
    let mut linked_items: Vec<types::GetAccountResponse> = vec![];

    for token in token_cache.tokens {
        let request = types::GetAccountRequest {
            client_id: client_id.clone(),
            secret: secret.clone(),
            access_token: token,
        };
        let resp = client
            .post("https://sandbox.plaid.com/accounts/get")
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .unwrap_or_else(|_| {
                utils::print_error("failed to create link token");
                std::process::exit(1);
            });

        let body: types::GetAccountResponse = resp.json().await.unwrap_or_else(|_| {
            utils::print_error("Balance response was malformed");
            std::process::exit(1);
        });

        linked_items.push(body)
    }

    return linked_items;
}
