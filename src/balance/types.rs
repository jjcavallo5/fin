use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GetAccountRequest {
    pub client_id: String,
    pub secret: String,
    pub access_token: String,
}

#[derive(Deserialize)]
struct Balance {
    balance: f32,
}

#[derive(Deserialize)]
struct Account {
    account_id: String,
    balances: Balance,
    name: String,
    official_name: String,
    subtype: String,
}

#[derive(Deserialize)]
struct Item {
    institution_name: String,
}

#[derive(Deserialize)]
pub struct GetAccountResponse {
    accounts: Vec<Account>,
    item: Item,
    request_id: String,
}
