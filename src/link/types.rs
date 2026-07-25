use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, oneshot};

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
}
