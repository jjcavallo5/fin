use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    Ping,
    Stop,
    Login {
        pass: String,
        plaid_client_id: String,
        plaid_secret: String,
    },
    CreateLinkToken {
        product: crate::link::types::LinkProduct,
    },
    ExchangePublicToken {
        public_token: String,
    },
    GetPlaidAccount {
        nonce: String,
        ciphertext: String,
    },
    RemovePlaidItem {
        nonce: String,
        ciphertext: String,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonResponse {
    Ok,
    Quit,
    Error {
        message: String,
    },
    LinkToken {
        token: String,
    },
    PlaidAccount {
        item: crate::plaid::types::PlaidItem,
    },
    ExchangedToken {
        nonce: String,
        ciphertext: String,
        item: crate::plaid::types::PlaidItem,
    },
}
