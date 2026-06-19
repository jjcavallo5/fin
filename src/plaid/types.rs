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
#[serde(rename_all = "lowercase")]
pub enum AccountType {
    Investment,
    Credit,
    Depository,
    Loan,
    Brokerage,
    Other,
}

#[derive(Deserialize)]
pub struct Account {
    pub account_id: String,
    pub balances: Balance,
    pub name: String,

    #[serde(rename = "type")]
    pub account_type: AccountType,

    #[serde(rename = "subtype")]
    pub account_subtype: String,
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

pub struct LinkedAccount {
    pub account_id: i32,
    pub plaid_item: PlaidItem,
}
