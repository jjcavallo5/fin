use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, oneshot};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LinkProduct {
    Bank,
}

impl LinkProduct {
    pub fn plaid_name(self) -> &'static str {
        match self {
            Self::Bank => "auth",
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PlaidAuthResponse {
    pub link_token: String,
}

#[derive(Deserialize)]
pub struct PublicTokenRequest {
    pub public_token: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenExchangeResponse {
    pub access_token: String,
}

pub struct LinkServerState {
    pub shutdown_tx: std::sync::Arc<Mutex<Option<oneshot::Sender<()>>>>,
    pub product: LinkProduct,
}

#[cfg(test)]
mod tests {
    use super::LinkProduct;

    #[test]
    fn bank_uses_plaid_auth() {
        assert_eq!(LinkProduct::Bank.plaid_name(), "auth");
    }
}
