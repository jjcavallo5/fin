pub mod encryption;
mod handlers;
mod types;
use crate::{db, logging};
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;

fn socket_path() -> std::path::PathBuf {
    dirs::home_dir().unwrap().join(".fin").join("fin.sock")
}

async fn handle_request(
    buffer: Vec<u8>,
    session: &mut handlers::Session,
    db_salt: &[u8; encryption::SALT_LEN],
) -> types::DaemonResponse {
    let decoded_req: types::DaemonRequest = serde_json::from_slice(&buffer).unwrap();

    let should_exit = match decoded_req {
        types::DaemonRequest::Ping => handlers::ping(),
        types::DaemonRequest::Login {
            pass,
            plaid_client_id,
            plaid_secret,
        } => handlers::login(pass, plaid_client_id, plaid_secret, session, db_salt),
        types::DaemonRequest::Stop => handlers::stop(),
        types::DaemonRequest::CreateLinkToken { product } => {
            handlers::create_link_token(product, session).await
        }
        types::DaemonRequest::ExchangePublicToken { public_token } => {
            handlers::exchange_public_token(public_token, session).await
        }
        types::DaemonRequest::GetPlaidAccount { nonce, ciphertext } => {
            handlers::get_plaid_account(nonce, ciphertext, session).await
        }
        types::DaemonRequest::RemovePlaidItem { nonce, ciphertext } => {
            handlers::remove_plaid_item(nonce, ciphertext, session).await
        }
    };

    should_exit
}

pub async fn run_daemon() {
    let db = db::get_db().await;
    let db_salt = db::get_db_salt(&db).await;

    let path = socket_path();
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::remove_file(&path).ok();
    let listener = match tokio::net::UnixListener::bind(&path) {
        Ok(proc) => proc,
        Err(_) => {
            logging::error("failed to start unix listener");
            std::process::exit(1)
        }
    };
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();

    let mut session = handlers::Session::new();

    while let Ok((mut socket, _)) = listener.accept().await {
        let mut buffer: Vec<u8> = Vec::new();
        tokio::io::AsyncReadExt::read_to_end(&mut socket, &mut buffer)
            .await
            .unwrap();
        match handle_request(buffer, &mut session, &db_salt).await {
            types::DaemonResponse::Quit => break,
            response => {
                let bytes = serde_json::to_vec(&response).unwrap();
                tokio::io::AsyncWriteExt::write_all(&mut socket, &bytes)
                    .await
                    .unwrap_or_else(|_| {
                        logging::error("failed to return daemon response");
                        std::process::exit(1);
                    });
            }
        }
    }

    std::fs::remove_file(path).ok();
    logging::info("exiting daemon...")
}

fn connect() -> std::os::unix::net::UnixStream {
    match std::os::unix::net::UnixStream::connect(socket_path()) {
        Ok(str) => str,
        Err(_) => {
            logging::error("connection failed");
            std::process::exit(1);
        }
    }
}

pub fn login() {
    // Get password
    spawn_daemon();
    println!("Enter encryption password: ");
    let mut password = String::new();
    std::io::stdin()
        .read_line(&mut password)
        .expect("Incorrect password");
    println!("Enter Plaid client ID: ");
    let mut plaid_client_id = String::new();
    std::io::stdin()
        .read_line(&mut plaid_client_id)
        .expect("Failed to read Plaid client ID");
    println!("Enter Plaid secret: ");
    let mut plaid_secret = String::new();
    std::io::stdin()
        .read_line(&mut plaid_secret)
        .expect("Failed to read Plaid secret");

    // Send login request with password
    let mut stream = connect();
    let req = types::DaemonRequest::Login {
        pass: password.trim().to_string(),
        plaid_client_id: plaid_client_id.trim().to_string(),
        plaid_secret: plaid_secret.trim().to_string(),
    };
    let bytes = serde_json::to_vec(&req).unwrap();
    stream.write_all(&bytes).unwrap_or_else(|_| {
        logging::error("failed to login");
        std::process::exit(1);
    });
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    match read_response(&mut stream) {
        types::DaemonResponse::Ok => logging::success("logged in"),
        types::DaemonResponse::Error { message } => logging::error(&message),
        _ => logging::error("unexpected daemon response"),
    }
}

pub fn quit() {
    let mut stream = connect();

    let bytes = serde_json::to_vec(&types::DaemonRequest::Stop).unwrap();
    match stream.write_all(&bytes) {
        Ok(_) => logging::success("exited daemon"),
        Err(_) => logging::error("failed to quit daemon"),
    }
}

pub fn ping() {
    let mut stream = connect();

    let bytes = serde_json::to_vec(&types::DaemonRequest::Ping).unwrap();
    if stream.write_all(&bytes).is_err() {
        logging::error("failed to ping daemon");
        return;
    }
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    match read_response(&mut stream) {
        types::DaemonResponse::Ok => (),
        types::DaemonResponse::Error { message } => logging::error(&message),
        _ => logging::error("unexpected daemon response"),
    }
}

pub fn spawn_daemon() {
    logging::info("starting daemon...");

    let current_exe = std::env::current_exe().unwrap_or_else(|_| {
        logging::error("failed to start daemon");
        std::process::exit(1)
    });

    let daemon_proc = std::process::Command::new(current_exe)
        .arg("daemon")
        .stdout(std::io::stdout())
        .spawn();

    match daemon_proc {
        Ok(proc) => logging::success(format!("daemon started with pid {}", proc.id()).as_str()),
        Err(_) => {
            logging::error("failed to start daemon");
            std::process::exit(1);
        }
    }
}

fn send_request(req: types::DaemonRequest) -> types::DaemonResponse {
    let mut stream = connect();
    let bytes = serde_json::to_vec(&req).unwrap();
    stream.write_all(&bytes).unwrap_or_else(|_| {
        logging::error("failed to send daemon request");
        std::process::exit(1);
    });
    stream.shutdown(std::net::Shutdown::Write).unwrap();

    read_response(&mut stream)
}

fn read_response(stream: &mut std::os::unix::net::UnixStream) -> types::DaemonResponse {
    let mut buffer = Vec::new();
    stream.read_to_end(&mut buffer).unwrap();
    serde_json::from_slice(&buffer).unwrap_or_else(|_| {
        logging::error("failed to parse daemon response");
        std::process::exit(1);
    })
}

pub fn create_link_token(product: crate::link::types::LinkProduct) -> Option<String> {
    match send_request(types::DaemonRequest::CreateLinkToken { product }) {
        types::DaemonResponse::LinkToken { token } => Some(token),
        types::DaemonResponse::Error { message } => {
            logging::error(&message);
            None
        }
        _ => {
            logging::error("unexpected daemon response");
            None
        }
    }
}

pub fn exchange_public_token(
    public_token: String,
) -> Option<(String, String, crate::plaid::types::PlaidItem)> {
    match send_request(types::DaemonRequest::ExchangePublicToken { public_token }) {
        types::DaemonResponse::ExchangedToken {
            nonce,
            ciphertext,
            item,
        } => Some((nonce, ciphertext, item)),
        types::DaemonResponse::Error { message } => {
            logging::error(&message);
            None
        }
        _ => {
            logging::error("unexpected daemon response");
            None
        }
    }
}

pub fn get_plaid_account(
    nonce: String,
    ciphertext: String,
) -> Option<crate::plaid::types::PlaidItem> {
    match send_request(types::DaemonRequest::GetPlaidAccount { nonce, ciphertext }) {
        types::DaemonResponse::PlaidAccount { item } => Some(item),
        types::DaemonResponse::Error { message } => {
            logging::error(&message);
            None
        }
        _ => {
            logging::error("unexpected daemon response");
            None
        }
    }
}

pub fn remove_plaid_item(nonce: String, ciphertext: String) -> bool {
    match send_request(types::DaemonRequest::RemovePlaidItem { nonce, ciphertext }) {
        types::DaemonResponse::Ok => true,
        types::DaemonResponse::Error { message } => {
            logging::error(&message);
            false
        }
        _ => {
            logging::error("unexpected daemon response");
            false
        }
    }
}
