use serde::{Deserialize, Serialize};
use tokio::sync::{oneshot, Mutex};

#[derive(Serialize)]
pub struct GetAccountRequest {
    pub client_id: String,
    pub secret: String,
    pub access_token: String,
}
