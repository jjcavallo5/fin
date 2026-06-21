pub mod encryption;
mod handlers;
mod types;
use crate::{db, logging};
use std::io::{Read, Write};

const SOCKET_PTH: &str = "/tmp/fin.sock";

fn handle_request(
    buffer: Vec<u8>,
    password: &mut String,
    db_salt: &[u8; encryption::SALT_LEN],
) -> types::DaemonResponse {
    let decoded_req: types::DaemonRequest = serde_json::from_slice(&buffer).unwrap();

    let should_exit = match decoded_req {
        types::DaemonRequest::Ping => handlers::ping(),
        types::DaemonRequest::Login { pass } => handlers::login(pass, password),
        types::DaemonRequest::Stop => handlers::stop(),
        types::DaemonRequest::Encrypt { token } => handlers::encrypt(token, password, db_salt),
        types::DaemonRequest::Decrypt { nonce, ciphertext } => {
            handlers::decrypt(nonce, ciphertext, password, db_salt)
        }
    };

    return should_exit;
}

pub async fn run_daemon() {
    let db = db::get_db().await;
    let db_salt = db::get_db_salt(&db).await;

    std::fs::remove_file(SOCKET_PTH).ok();
    let listener = match std::os::unix::net::UnixListener::bind(SOCKET_PTH) {
        Ok(proc) => proc,
        Err(_) => {
            logging::error("failed to start unix listener");
            std::process::exit(1)
        }
    };

    let mut password = String::new();

    loop {
        match listener.accept() {
            Ok((mut socket, _)) => {
                let mut buffer: Vec<u8> = Vec::new();
                socket.read_to_end(&mut buffer).unwrap();
                match handle_request(buffer, &mut password, &db_salt) {
                    types::DaemonResponse::Quit => break,
                    response => {
                        let bytes = serde_json::to_vec(&response).unwrap();
                        socket.write_all(&bytes).unwrap_or_else(|_| {
                            logging::error("failed to return daemon response");
                            std::process::exit(1);
                        });
                    }
                }
            }
            Err(_) => break,
        }
    }

    logging::info("exiting daemon...")
}

fn connect() -> std::os::unix::net::UnixStream {
    return match std::os::unix::net::UnixStream::connect(SOCKET_PTH) {
        Ok(str) => str,
        Err(_) => {
            logging::error("connection failed");
            std::process::exit(1);
        }
    };
}

pub fn login() {
    // Get password
    spawn_daemon();
    println!("Enter password: ");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Incorrect password");

    // Send login request with password
    let mut stream = connect();
    let req = types::DaemonRequest::Login {
        pass: buffer.trim().to_string(),
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

pub fn encrypt_token(token: String) -> Option<(String, String)> {
    match send_request(types::DaemonRequest::Encrypt { token }) {
        types::DaemonResponse::Encrypted { nonce, ciphertext } => Some((nonce, ciphertext)),
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

pub fn decrypt_token(nonce: String, ciphertext: String) -> Option<String> {
    match send_request(types::DaemonRequest::Decrypt { nonce, ciphertext }) {
        types::DaemonResponse::Data { token } => Some(token),
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
