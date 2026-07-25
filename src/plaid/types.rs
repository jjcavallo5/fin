use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GetAccountRequest {
    pub client_id: String,
    pub secret: String,
    pub access_token: String,
}

#[derive(Deserialize)]
pub struct Balance {
    #[serde(
        rename = "available",
        default,
        deserialize_with = "optional_dollars_to_cents"
    )]
    pub available_cents: Option<i64>,
    #[serde(rename = "current", deserialize_with = "dollars_to_cents")]
    pub current_cents: i64,
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

fn decimal_to_cents<E: de::Error>(value: &str) -> Result<i64, E> {
    crate::money::parse_dollars_to_cents(value).map_err(E::custom)
}

fn dollars_to_cents<'de, D: Deserializer<'de>>(deserializer: D) -> Result<i64, D::Error> {
    let value = serde_json::Number::deserialize(deserializer)?;
    decimal_to_cents(&value.to_string())
}

fn optional_dollars_to_cents<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<i64>, D::Error> {
    let value = Option::<serde_json::Number>::deserialize(deserializer)?;
    value
        .map(|number| decimal_to_cents(&number.to_string()))
        .transpose()
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

#[cfg(test)]
mod tests {
    use super::GetAccountResponse;

    #[test]
    fn deserializes_balances_as_exact_cents() {
        let response: GetAccountResponse = serde_json::from_str(
            r#"{
                "accounts": [{
                    "account_id": "checking",
                    "balances": {"available": 10.105, "current": 1234.56},
                    "name": "Checking",
                    "type": "depository",
                    "subtype": "checking"
                }],
                "item": {"institution_name": "Bank"}
            }"#,
        )
        .unwrap();

        assert_eq!(response.accounts[0].balances.available_cents, Some(1011));
        assert_eq!(response.accounts[0].balances.current_cents, 123_456);
    }

    #[test]
    fn accepts_a_null_available_balance() {
        let response: GetAccountResponse = serde_json::from_str(
            r#"{
                "accounts": [{
                    "account_id": "card",
                    "balances": {"available": null, "current": 42.01},
                    "name": "Card",
                    "type": "credit",
                    "subtype": "credit card"
                }],
                "item": {"institution_name": "Bank"}
            }"#,
        )
        .unwrap();

        assert_eq!(response.accounts[0].balances.available_cents, None);
        assert_eq!(response.accounts[0].balances.current_cents, 4_201);
    }
}
