use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GetAccountRequest {
    pub client_id: String,
    pub secret: String,
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct Balance {
    pub available: f32,
    pub current: f32,
}

#[derive(Deserialize)]
pub struct Account {
    pub balances: Balance,
    pub name: String,
}

#[derive(Deserialize)]
pub struct Item {
    pub institution_name: String,
}

#[derive(Deserialize)]
pub struct GetAccountResponse {
    pub accounts: Vec<Account>,
    pub item: Item,
}

pub struct PlaidItem {
    pub accounts: Vec<Account>,
    pub item: Item,
    pub access_token: String,
}
