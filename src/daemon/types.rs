use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonRequest {
    Ping,
    Stop,
    Login { pass: String },
    Encrypt { token: String },
    Decrypt { nonce: String, ciphertext: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum DaemonResponse {
    Ok,
    Quit,
    Data { token: String },
    Encrypted { nonce: String, ciphertext: String },
    Error { message: String },
}
