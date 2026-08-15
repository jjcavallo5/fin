use crate::daemon::encryption::{self, EncryptedBlob, NONCE_LEN, SALT_LEN};
use crate::daemon::types;
use crate::{environment, logging, plaid};
use serde_json::json;
use zeroize::Zeroizing;

pub struct Session {
    key: Option<Zeroizing<[u8; encryption::KEY_LEN]>>,
    plaid_client_id: Zeroizing<String>,
    plaid_secret: Zeroizing<String>,
}

impl Session {
    pub fn new() -> Self {
        Self {
            key: None,
            plaid_client_id: Zeroizing::new(String::new()),
            plaid_secret: Zeroizing::new(String::new()),
        }
    }

    fn credentials(&self) -> Result<(&str, &str), String> {
        if self.key.is_none() || self.plaid_client_id.is_empty() || self.plaid_secret.is_empty() {
            return Err("daemon is not logged in".to_string());
        }
        Ok((&self.plaid_client_id, &self.plaid_secret))
    }

    fn key(&self) -> Result<&[u8; encryption::KEY_LEN], String> {
        self.key
            .as_deref()
            .ok_or_else(|| "daemon is not logged in".to_string())
    }
}

pub fn ping() -> types::DaemonResponse {
    logging::success("connection to daemon successful");
    types::DaemonResponse::Ok
}

pub fn login(
    pass: String,
    plaid_client_id: String,
    plaid_secret: String,
    session: &mut Session,
    db_salt: &[u8; SALT_LEN],
) -> types::DaemonResponse {
    if plaid_client_id.is_empty() || plaid_secret.is_empty() {
        return types::DaemonResponse::Error {
            message: "Plaid credentials cannot be empty".to_string(),
        };
    }
    let key = match encryption::derive_key(&pass, db_salt) {
        Ok(key) => key,
        Err(_) => {
            return types::DaemonResponse::Error {
                message: "failed to derive encryption key".to_string(),
            };
        }
    };
    session.key = Some(Zeroizing::new(key));
    session.plaid_client_id = Zeroizing::new(plaid_client_id);
    session.plaid_secret = Zeroizing::new(plaid_secret);
    types::DaemonResponse::Ok
}

pub fn stop() -> types::DaemonResponse {
    types::DaemonResponse::Quit
}

fn encrypt(token: String, session: &Session) -> Result<(String, String), String> {
    let key = session.key()?;
    match encryption::encrypt_token(key, &token) {
        Ok(blob) => Ok((
            encryption::encode_hex(&blob.nonce),
            encryption::encode_hex(&blob.ciphertext),
        )),
        Err(message) => Err(message),
    }
}

fn decrypt_value(nonce: String, ciphertext: String, session: &Session) -> Result<String, String> {
    let key = session.key()?;
    let nonce_bytes = encryption::decode_hex(&nonce)?;
    if nonce_bytes.len() != NONCE_LEN {
        return Err(format!("nonce must be {NONCE_LEN} bytes"));
    }
    let blob = EncryptedBlob {
        nonce: nonce_bytes.try_into().unwrap(),
        ciphertext: encryption::decode_hex(&ciphertext)?,
    };
    encryption::decrypt_token(key, &blob)
}

async fn plaid_post<T: serde::de::DeserializeOwned>(
    path: &str,
    body: serde_json::Value,
) -> Result<T, String> {
    let response = reqwest::Client::new()
        .post(environment::plaid_endpoint(path))
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Plaid request failed: {e}"))?;
    if !response.status().is_success() {
        let status = response.status();
        let response_body = response
            .text()
            .await
            .unwrap_or_else(|e| format!("<failed to read response body: {e}>"));
        return Err(format!(
            "Plaid request to {path} failed: {status}: {response_body}"
        ));
    }
    response
        .json()
        .await
        .map_err(|e| format!("Plaid response was malformed: {e}"))
}

pub async fn create_link_token(
    product: crate::link::types::LinkProduct,
    session: &Session,
) -> types::DaemonResponse {
    let (client_id, secret) = match session.credentials() {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let result: Result<crate::link::types::PlaidAuthResponse, _> = plaid_post(
        "link/token/create",
        json!({
            "client_id": client_id, "secret": secret, "client_name": "FIN",
            "country_codes": ["US"], "language": "en", "products": [product.plaid_name()],
            "user": { "client_user_id": "Jeremy" }
        }),
    )
    .await;
    match result {
        Ok(response) => types::DaemonResponse::LinkToken {
            token: response.link_token,
        },
        Err(message) => types::DaemonResponse::Error { message },
    }
}

pub async fn exchange_public_token(
    public_token: String,
    session: &Session,
) -> types::DaemonResponse {
    let (client_id, secret) = match session.credentials() {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let exchanged: crate::link::types::TokenExchangeResponse = match plaid_post(
        "item/public_token/exchange",
        json!({
            "client_id": client_id, "secret": secret, "public_token": public_token
        }),
    )
    .await
    {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let item = match plaid::get_plaid_account(client_id, secret, &exchanged.access_token).await {
        Ok(item) => item,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    match encrypt(exchanged.access_token, session) {
        Ok((nonce, ciphertext)) => types::DaemonResponse::ExchangedToken {
            nonce,
            ciphertext,
            item,
        },
        Err(message) => types::DaemonResponse::Error { message },
    }
}

pub async fn get_plaid_account(
    nonce: String,
    ciphertext: String,
    session: &Session,
) -> types::DaemonResponse {
    let (client_id, secret) = match session.credentials() {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let token = match decrypt_value(nonce, ciphertext, session) {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    match plaid::get_plaid_account(client_id, secret, &token).await {
        Ok(item) => types::DaemonResponse::PlaidAccount { item },
        Err(message) => types::DaemonResponse::Error { message },
    }
}

pub async fn remove_plaid_item(
    nonce: String,
    ciphertext: String,
    session: &Session,
) -> types::DaemonResponse {
    let (client_id, secret) = match session.credentials() {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let token = match decrypt_value(nonce, ciphertext, session) {
        Ok(value) => value,
        Err(message) => return types::DaemonResponse::Error { message },
    };
    let result: Result<serde_json::Value, _> = plaid_post(
        "item/remove",
        json!({
            "client_id": client_id, "secret": secret, "access_token": token
        }),
    )
    .await;
    match result {
        Ok(_) => types::DaemonResponse::Ok,
        Err(message) => types::DaemonResponse::Error { message },
    }
}
