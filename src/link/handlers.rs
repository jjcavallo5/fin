use crate::daemon;
use crate::db;
use crate::entity;
use crate::link::types;
use crate::logging;
use crate::plaid;
use crate::plaid::get_plaid_account;
use axum::extract::State;
use axum::Json;
use sea_orm::ActiveModelTrait;

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
            logging::error("failed to create link token");
            std::process::exit(1);
        });

    let plaid_auth_response: types::PlaidAuthResponse = resp.json().await.unwrap_or_else(|_| {
        logging::error("response from Plaid was malformed");
        std::process::exit(1);
    });

    return axum::Json(plaid_auth_response);
}

async fn save_asset_account(
    account: &plaid::types::Account,
    instituion_name: String,
    nonce: String,
    ciphertext: String,
) -> Result<(), sea_orm::DbErr> {
    let acct_entry = entity::asset_accounts::ActiveModel {
        institution_name: sea_orm::ActiveValue::Set(instituion_name),
        name: sea_orm::ActiveValue::Set(account.name.clone()),
        nonce: sea_orm::ActiveValue::Set(nonce),
        encrypted_token: sea_orm::ActiveValue::Set(ciphertext),
        ..Default::default()
    };
    let db = db::get_db().await;
    return acct_entry.insert(&db).await.map(|_| ());
}

async fn save_liability_account(
    account: &plaid::types::Account,
    instituion_name: String,
    nonce: String,
    ciphertext: String,
) -> Result<(), sea_orm::DbErr> {
    let acct_entry = entity::liability_accounts::ActiveModel {
        institution_name: sea_orm::ActiveValue::Set(instituion_name),
        name: sea_orm::ActiveValue::Set(account.name.clone()),
        nonce: sea_orm::ActiveValue::Set(nonce),
        encrypted_token: sea_orm::ActiveValue::Set(ciphertext),
        ..Default::default()
    };
    let db = db::get_db().await;
    return acct_entry.insert(&db).await.map(|_| ());
}

pub async fn exchange_token(
    State(state): State<std::sync::Arc<types::LinkServerState>>,
    Json(payload): Json<types::PublicTokenRequest>,
) {
    println!("[EXCHANGE TOKEN]: exchange token called");
    let (client_id, secret) = plaid::load_env();

    let request = types::TokenExchangeRequest {
        client_id: client_id.clone(),
        secret: secret.clone(),
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
            logging::error("failed to exchange token");
            std::process::exit(1);
        });

    let access_token: types::TokenExchangeResponse = resp.json().await.unwrap_or_else(|_| {
        logging::error("response from Plaid was malformed");
        std::process::exit(1);
    });

    // Get encrypted token from daemon
    let (nonce, ciphertext) = daemon::encrypt_token(access_token.access_token.clone()).unwrap();
    let plaid_item =
        get_plaid_account(&client_id, &secret, &access_token.access_token, &client).await;

    // Save encrypted token to DB
    for account in plaid_item.accounts {
        let res = match account.account_type {
            plaid::types::AccountType::Loan | plaid::types::AccountType::Credit => {
                save_liability_account(
                    &account,
                    plaid_item.item.institution_name.clone(),
                    nonce.clone(),
                    ciphertext.clone(),
                )
                .await
            }
            plaid::types::AccountType::Investment
            | plaid::types::AccountType::Depository
            | plaid::types::AccountType::Brokerage
            | plaid::types::AccountType::Other => {
                save_asset_account(
                    &account,
                    plaid_item.item.institution_name.clone(),
                    nonce.clone(),
                    ciphertext.clone(),
                )
                .await
            }
        };

        match res {
            Ok(_) => logging::success(&format!("saved account {}", account.name)),
            Err(_) => logging::error("failed to save account"),
        }
    }

    // Graceful server shutdown after response to client
    if let Some(tx) = state.shutdown_tx.lock().await.take() {
        let _ = tx.send(());
    }
}
