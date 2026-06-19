use crate::db;
use crate::entity;
use crate::logging;
use crate::plaid;
use crate::tui;
use axum::{
    Router,
    routing::{get, post},
};
use sea_orm::EntityTrait;
use tokio::net::TcpListener;
use tokio::sync::{Mutex, oneshot};
mod handlers;
mod types;

pub async fn link() {
    // Set up app state to recieve shutdown signal on success
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    // Set up serving of the frontend react app
    let server_state = types::LinkServerState {
        shutdown_tx: std::sync::Arc::new(Mutex::new(Some(shutdown_tx))),
    };
    let serve_dir = tower_http::services::ServeDir::new("web/dist");
    let router: Router = axum::Router::new()
        .route("/create-token", get(handlers::get_link_token))
        .route("/exchange-token", post(handlers::exchange_token))
        .fallback_service(serve_dir)
        .with_state(std::sync::Arc::new(server_state));

    // Create server URL on OS-specified port, save address in addr var
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: std::net::SocketAddr = listener.local_addr().unwrap();

    // Serve & open web browser to url
    let url = format!("http://127.0.0.1:{}", addr.port());
    webbrowser::open(&url).unwrap_or_else(|_| {
        logging::error("failed to launch browser");
        std::process::exit(1);
    });
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await;
}

pub async fn unlink() {
    // Get linked items
    let linked_items = plaid::get_linked_accounts().await;
    let names = linked_items
        .iter()
        .map(|item| item.plaid_item.item.institution_name.clone())
        .collect();
    let (_, idx) = tui::tui(names);
    let selected_item = &linked_items[idx];

    // Remove selected item from plaid
    let (client_id, secret) = plaid::load_env();
    let request = types::RemoveAccountRequest {
        client_id,
        secret,
        access_token: selected_item.plaid_item.access_token.clone(),
    };
    let client = reqwest::Client::new();
    client
        .post("https://sandbox.plaid.com/item/remove")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            logging::error("failed to remove token");
            std::process::exit(1)
        });

    // Remove selected item from DB
    let db = db::get_db().await;
    match entity::asset_accounts::Entity::delete_by_id(selected_item.account_id)
        .exec(&db)
        .await
    {
        Ok(_) => logging::success(&format!(
            "{} removed successfully.",
            selected_item.plaid_item.item.institution_name,
        )),
        Err(_) => logging::error("failed to remove account"),
    };
}

pub async fn list() {
    let linked_items = plaid::get_linked_accounts().await;
    linked_items
        .iter()
        .for_each(|item| println!("{}", item.plaid_item.item.institution_name));
}
