use crate::utils;
use axum::Router;
use serde::{Deserialize, Serialize};
use std::{env, process};
use tokio::net::TcpListener;
use webbrowser;

fn load_env() -> (String, String) {
    let client_id = env::var("PLAID_CLIENT_ID").unwrap_or_else(|_e| {
        utils::print_error("PLAID_CLIENT_ID environment variable not set");
        process::exit(1);
    });
    let secret = env::var("PLAID_SECRET").unwrap_or_else(|_e| {
        utils::print_error("PLAID_SECRET environment variable not set");
        process::exit(1);
    });

    return (client_id, secret);
}

#[derive(Serialize)]
struct User {
    client_user_id: String,
}

#[derive(Serialize)]
struct LinkRequest {
    client_id: String,
    secret: String,
    client_name: String,
    country_codes: Vec<String>,
    language: String,
    products: Vec<String>,
    user: User,
}

#[derive(Deserialize, Serialize)]
struct PlaidAuthResponse {
    link_token: String,
}

async fn get_link_token() -> axum::Json<PlaidAuthResponse> {
    println!("[GET TOKEN]: get token called");
    let (client_id, secret) = load_env();

    let request = LinkRequest {
        client_id,
        secret,
        client_name: "FIN".to_string(),
        country_codes: vec!["US".to_string()],
        language: "en".to_string(),
        products: vec!["auth".to_string()],
        user: User {
            client_user_id: "Jeremy".to_string(),
        },
    };

    let client = reqwest::Client::new();

    let resp = client
        .post("https://sandbox.plaid.com/link/token/create")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            utils::print_error("failed to create link token");
            process::exit(1);
        });

    let plaid_auth_response: PlaidAuthResponse = resp.json().await.unwrap_or_else(|_| {
        utils::print_error("response from Plaid was malformed");
        process::exit(1);
    });

    return axum::Json(plaid_auth_response);
}

pub async fn link() {
    // Set up serving of the frontend react app
    let serve_dir = tower_http::services::ServeDir::new("web/dist");
    let router: Router = axum::Router::new()
        .route("/create-token", axum::routing::get(get_link_token))
        .fallback_service(serve_dir);

    // Create server URL on OS-specified port, save address in addr var
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: std::net::SocketAddr = listener.local_addr().unwrap();

    // Serve & open web browser to url
    let url = format!("http://127.0.0.1:{}", addr.port());
    webbrowser::open(&url).unwrap_or_else(|_| {
        utils::print_error("failed to launch browser");
        process::exit(1);
    });
    let _server = axum::serve(listener, router).await;

    let _link_token = get_link_token().await;
}
