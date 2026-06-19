use crate::daemon;
use crate::db;
use crate::entity;
use crate::logging;
pub mod types;
use sea_orm::EntityTrait;

pub fn load_env() -> (String, String) {
    let client_id = std::env::var("PLAID_CLIENT_ID").unwrap_or_else(|_e| {
        logging::error("PLAID_CLIENT_ID environment variable not set");
        std::process::exit(1);
    });
    let secret = std::env::var("PLAID_SECRET").unwrap_or_else(|_e| {
        logging::error("PLAID_SECRET environment variable not set");
        std::process::exit(1);
    });

    return (client_id, secret);
}

pub async fn get_plaid_account(
    client_id: &String,
    secret: &String,
    token: &String,
    client: &reqwest::Client,
) -> types::PlaidItem {
    let request = types::GetAccountRequest {
        client_id: client_id.clone(),
        secret: secret.clone(),
        access_token: token.clone(),
    };
    let resp = client
        .post("https://sandbox.plaid.com/accounts/get")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            logging::error("failed to create link token");
            std::process::exit(1);
        });

    let body: types::GetAccountResponse = resp.json().await.unwrap_or_else(|_| {
        logging::error("Balance response was malformed");
        std::process::exit(1);
    });

    return types::PlaidItem {
        access_token: token.clone(),
        accounts: body.accounts,
        item: body.item,
    };
}

pub async fn get_linked_accounts() -> Vec<types::LinkedAccount> {
    let (client_id, secret) = load_env();
    let client = reqwest::Client::new();
    let db = db::get_db().await;
    let items: Vec<entity::plaid_item::Model> =
        entity::plaid_item::Entity::find().all(&db).await.unwrap();

    let mut linked_accounts: Vec<types::LinkedAccount> = vec![];
    for item in items {
        let token = daemon::decrypt_token(item.nonce, item.encrypted_token).unwrap_or_else(|| {
            logging::error("failed to connect to daemon. Are you logged in?");
            std::process::exit(1);
        });
        linked_accounts.push(types::LinkedAccount {
            account_id: item.id,
            plaid_item: get_plaid_account(&client_id, &secret, &token, &client).await,
        })
    }

    return linked_accounts;
}
