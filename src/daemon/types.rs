use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    Ping,
    Stop,
    Login { pass: String },
    Encrypt { token: String },
    Decrypt { token: String },
}
