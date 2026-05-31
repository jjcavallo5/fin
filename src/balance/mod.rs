use crate::cache;
use crate::plaid;
use crate::utils;
mod types;

pub async fn balance() {
    println!("[GET TOKEN]: get token called");
    let (client_id, secret) = plaid::load_env();

    let token_cache = cache::read_token_file();

    let client = reqwest::Client::new();
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

        let body = resp.text().await.unwrap();
        println!("{body:?}")
    }
}
