use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct GetAccountRequest {
    pub client_id: String,
    pub secret: String,
    pub access_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Balance {
    pub available_cents: Option<i64>,
    pub current_cents: i64,
}

#[derive(Deserialize)]
struct PlaidBalance {
    #[serde(default, deserialize_with = "optional_dollars_to_cents")]
    available: Option<i64>,
    #[serde(deserialize_with = "dollars_to_cents")]
    current: i64,
}

#[derive(Debug, Deserialize, Serialize)]
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

#[derive(Debug, Deserialize, Serialize)]
pub struct Account {
    pub account_id: String,
    pub balances: Balance,
    pub name: String,

    #[serde(rename = "type")]
    pub account_type: AccountType,

    #[serde(rename = "subtype")]
    pub account_subtype: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Item {
    pub institution_name: String,
}

#[derive(Deserialize)]
pub struct GetAccountResponse {
    accounts: Vec<PlaidAccount>,
    pub item: Item,
}

#[derive(Deserialize)]
struct PlaidAccount {
    account_id: String,
    balances: PlaidBalance,
    name: String,
    #[serde(rename = "type")]
    account_type: AccountType,
    #[serde(rename = "subtype")]
    account_subtype: String,
}

impl GetAccountResponse {
    pub fn into_plaid_item(self) -> PlaidItem {
        PlaidItem {
            accounts: self
                .accounts
                .into_iter()
                .map(|account| Account {
                    account_id: account.account_id,
                    balances: Balance {
                        available_cents: account.balances.available,
                        current_cents: account.balances.current,
                    },
                    name: account.name,
                    account_type: account.account_type,
                    account_subtype: account.account_subtype,
                })
                .collect(),
            item: self.item,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaidItem {
    pub accounts: Vec<Account>,
    pub item: Item,
}

pub struct LinkedAccount {
    pub account_id: i32,
    pub plaid_item: PlaidItem,
    pub nonce: String,
    pub encrypted_token: String,
}

#[cfg(test)]
mod tests {
    use super::{GetAccountResponse, PlaidItem};

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

        let item = response.into_plaid_item();
        assert_eq!(item.accounts[0].balances.available_cents, Some(1011));
        assert_eq!(item.accounts[0].balances.current_cents, 123_456);
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

        let item = response.into_plaid_item();
        assert_eq!(item.accounts[0].balances.available_cents, None);
        assert_eq!(item.accounts[0].balances.current_cents, 4_201);
    }

    #[test]
    fn daemon_round_trip_does_not_reconvert_cents() {
        let response: GetAccountResponse = serde_json::from_str(
            r#"{
                "accounts": [{
                    "account_id": "checking",
                    "balances": {"available": 1000.25, "current": 1234.56},
                    "name": "Checking",
                    "type": "depository",
                    "subtype": "checking"
                }],
                "item": {"institution_name": "Bank"}
            }"#,
        )
        .unwrap();
        let item = response.into_plaid_item();

        let encoded = serde_json::to_vec(&item).unwrap();
        let decoded: PlaidItem = serde_json::from_slice(&encoded).unwrap();

        assert_eq!(decoded.accounts[0].balances.available_cents, Some(100_025));
        assert_eq!(decoded.accounts[0].balances.current_cents, 123_456);
    }
}
