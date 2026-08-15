use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, oneshot};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub enum LinkProduct {
    Bank,
    Investment,
    Liability,
}

impl LinkProduct {
    pub fn plaid_name(self) -> &'static str {
        match self {
            Self::Bank => "auth",
            Self::Investment => "investments",
            Self::Liability => "liabilities",
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

    #[test]
    fn investment_uses_plaid_investments() {
        assert_eq!(LinkProduct::Investment.plaid_name(), "investments");
    }

    #[test]
    fn liability_uses_plaid_liabilities() {
        assert_eq!(LinkProduct::Liability.plaid_name(), "liabilities");
    }
}
