use crate::utils;
use axum::{routing::get, Router};
use tokio::net::TcpListener;
mod handlers;
mod types;

pub async fn link() {
    // Set up serving of the frontend react app
    let serve_dir = tower_http::services::ServeDir::new("web/dist");
    let router: Router = axum::Router::new()
        .route("/create-token", get(handlers::get_link_token))
        .fallback_service(serve_dir);

    // Create server URL on OS-specified port, save address in addr var
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr: std::net::SocketAddr = listener.local_addr().unwrap();

    // Serve & open web browser to url
    let url = format!("http://127.0.0.1:{}", addr.port());
    webbrowser::open(&url).unwrap_or_else(|_| {
        utils::print_error("failed to launch browser");
        std::process::exit(1);
    });
    let _server = axum::serve(listener, router).await;
}
