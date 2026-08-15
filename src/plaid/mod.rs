use crate::daemon;
use crate::db;
use crate::entity;
use crate::environment;
pub mod types;
use sea_orm::EntityTrait;

pub async fn get_plaid_account(
    client_id: &str,
    secret: &str,
    token: &str,
) -> Result<types::PlaidItem, String> {
    let request = types::GetAccountRequest {
        client_id: client_id.to_string(),
        secret: secret.to_string(),
        access_token: token.to_string(),
    };
    let resp = reqwest::Client::new()
        .post(environment::plaid_endpoint("accounts/get"))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| format!("failed to get Plaid accounts: {e}"))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let response_body = resp
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {e}>"));
        return Err(format!(
            "Plaid accounts request failed: {status}: {response_body}"
        ));
    }

    let body: types::GetAccountResponse = resp
        .json()
        .await
        .map_err(|e| format!("Plaid accounts response was malformed: {e}"))?;

    Ok(body.into_plaid_item())
}

pub async fn get_linked_accounts() -> Vec<types::LinkedAccount> {
    let db = db::get_db().await;
    let items: Vec<entity::plaid_item::Model> =
        entity::plaid_item::Entity::find().all(&db).await.unwrap();

    let mut linked_accounts: Vec<types::LinkedAccount> = vec![];
    for item in items {
        let plaid_item =
            daemon::get_plaid_account(item.nonce.clone(), item.encrypted_token.clone())
                .unwrap_or_else(|| {
                    std::process::exit(1);
                });
        linked_accounts.push(types::LinkedAccount {
            account_id: item.id,
            plaid_item,
            nonce: item.nonce,
            encrypted_token: item.encrypted_token,
        })
    }

    return linked_accounts;
}
