use crate::cache;
use crate::plaid;
use crate::tui;
use crate::utils;
use axum::{
    routing::{get, post},
    Router,
};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
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
        utils::print_error("failed to launch browser");
        std::process::exit(1);
    });
    let _ = axum::serve(listener, router)
        .with_graceful_shutdown(async {
            let _ = shutdown_rx.await;
        })
        .await;
}

pub async fn unlink() {
    let linked_items = plaid::get_linked_items().await;
    let names = linked_items
        .iter()
        .map(|item| item.item.institution_name.clone())
        .collect();
    let (_, idx) = tui::tui(names);

    let (client_id, secret) = plaid::load_env();
    let request = types::RemoveAccountRequest {
        client_id,
        secret,
        access_token: linked_items[idx].access_token.clone(),
    };
    let client = reqwest::Client::new();
    client
        .post("https://sandbox.plaid.com/item/remove")
        .json(&request)
        .send()
        .await
        .unwrap_or_else(|_| {
            utils::print_error("failed to remove token");
            std::process::exit(1)
        });

    cache::remove_token(linked_items[idx].access_token.clone());
    utils::print_success(&format!(
        "{} removed successfully.",
        linked_items[idx].item.institution_name,
    ))
}

pub async fn list() {
    let linked_items = plaid::get_linked_items().await;
    linked_items
        .iter()
        .for_each(|item| println!("{}", item.item.institution_name));
}
