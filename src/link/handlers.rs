use crate::daemon;
use crate::db;
use crate::entity;
use crate::link::types;
use crate::logging;
use crate::plaid;
use axum::Json;
use axum::extract::State;
use sea_orm::ActiveModelTrait;

pub async fn get_link_token() -> axum::Json<types::PlaidAuthResponse> {
    println!("[GET TOKEN]: get token called");
    let token = daemon::create_link_token().unwrap_or_else(|| {
        std::process::exit(1);
    });
    axum::Json(types::PlaidAuthResponse { link_token: token })
}

async fn save_plaid_item(
    item: &plaid::types::PlaidItem,
    nonce: String,
    ciphertext: String,
) -> entity::plaid_item::Model {
    let item_entry = entity::plaid_item::ActiveModel {
        institution_name: sea_orm::ActiveValue::Set(item.item.institution_name.clone()),
        nonce: sea_orm::ActiveValue::Set(nonce),
        encrypted_token: sea_orm::ActiveValue::Set(ciphertext),
        ..Default::default()
    };
    let db = db::get_db().await;
    return item_entry.insert(&db).await.unwrap_or_else(|_| {
        logging::error("Failed to insert plaid item into database");
        std::process::exit(1);
    });
}

async fn save_asset_account(
    account: &plaid::types::Account,
    plaid_item_id: i32,
) -> Result<(), sea_orm::DbErr> {
    let acct_entry = entity::asset_accounts::ActiveModel {
        account_id: sea_orm::ActiveValue::Set(account.account_id.clone()),
        name: sea_orm::ActiveValue::Set(account.name.clone()),
        account_type: sea_orm::ActiveValue::Set(match account.account_type {
            plaid::types::AccountType::Depository => entity::types::AssetAccountType::Depository,
            plaid::types::AccountType::Investment => entity::types::AssetAccountType::Investment,
            plaid::types::AccountType::Brokerage => entity::types::AssetAccountType::Brokerage,
            plaid::types::AccountType::Other => entity::types::AssetAccountType::Other,
            _ => unreachable!("liability account passed to save_asset_account"),
        }),
        account_subtype: sea_orm::ActiveValue::Set(account.account_subtype.clone()),
        plaid_item_id: sea_orm::ActiveValue::Set(plaid_item_id),
        ..Default::default()
    };
    let db = db::get_db().await;
    return acct_entry.insert(&db).await.map(|_| ());
}

async fn save_liability_account(
    account: &plaid::types::Account,
    plaid_item_id: i32,
) -> Result<(), sea_orm::DbErr> {
    let acct_entry = entity::liability_accounts::ActiveModel {
        account_id: sea_orm::ActiveValue::Set(account.account_id.clone()),
        name: sea_orm::ActiveValue::Set(account.name.clone()),
        account_type: sea_orm::ActiveValue::Set(match account.account_type {
            plaid::types::AccountType::Credit => entity::types::LiabilityAccountType::Credit,
            plaid::types::AccountType::Loan => entity::types::LiabilityAccountType::Loan,
            _ => unreachable!("asset account passed to save_liability_account"),
        }),
        account_subtype: sea_orm::ActiveValue::Set(account.account_subtype.clone()),
        plaid_item_id: sea_orm::ActiveValue::Set(plaid_item_id),
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
    let (nonce, ciphertext, plaid_item) = daemon::exchange_public_token(payload.public_token)
        .unwrap_or_else(|| std::process::exit(1));

    let saved_plaid_item = save_plaid_item(&plaid_item, nonce, ciphertext).await;

    // Save accounts to DB
    for account in plaid_item.accounts {
        let res = match &account.account_type {
            plaid::types::AccountType::Loan | plaid::types::AccountType::Credit => {
                save_liability_account(&account, saved_plaid_item.id).await
            }
            plaid::types::AccountType::Investment
            | plaid::types::AccountType::Depository
            | plaid::types::AccountType::Brokerage
            | plaid::types::AccountType::Other => {
                save_asset_account(&account, saved_plaid_item.id).await
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
