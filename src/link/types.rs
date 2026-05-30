use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct User {
    pub client_user_id: String,
}

#[derive(Serialize)]
pub struct LinkRequest {
    pub client_id: String,
    pub secret: String,
    pub client_name: String,
    pub country_codes: Vec<String>,
    pub language: String,
    pub products: Vec<String>,
    pub user: User,
}

#[derive(Deserialize, Serialize)]
pub struct PlaidAuthResponse {
    pub link_token: String,
}

#[derive(Deserialize)]
pub struct PublicTokenRequest {
    pub public_token: String,
}

#[derive(Serialize)]
pub struct TokenExchangeRequest {
    pub client_id: String,
    pub secret: String,
    pub public_token: String,
}

#[derive(Deserialize, Serialize)]
pub struct TokenExchangeResponse {
    pub access_token: String,
}
