mod handlers;
mod types;
use crate::logging;
use std::io::{Read, Write};

const SOCKET_PTH: &str = "/tmp/fin.sock";

fn handle_request(buffer: Vec<u8>, password: &mut String) -> types::DaemonResponse {
    let decoded_req: types::DaemonRequest = serde_json::from_slice(&buffer).unwrap();

    let should_exit = match decoded_req {
        types::DaemonRequest::Ping => handlers::ping(),
        types::DaemonRequest::Login { pass } => handlers::login(pass, password),
        types::DaemonRequest::Stop => handlers::stop(),
        types::DaemonRequest::Encrypt { token } => handlers::encrypt(token, password),
        types::DaemonRequest::Decrypt { token } => handlers::decrypt(token, password),
    };

    return should_exit;
}

pub fn run_daemon() {
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
                match handle_request(buffer, &mut password) {
                    types::DaemonResponse::Ok => logging::success("ok"),
                    types::DaemonResponse::Error { message } => logging::error(&message),
                    types::DaemonResponse::Data { token } => logging::success(&token),
                    types::DaemonResponse::Quit => break,
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
    match stream.write_all(&bytes) {
        Ok(_) => logging::success("connection to daemon successful"),
        Err(_) => logging::error("failed to ping daemon"),
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

pub fn encrypt_token(token: String) -> String {
    let mut stream = connect();

    let req = types::DaemonRequest::Encrypt { token };
    let bytes = serde_json::to_vec(&req).unwrap();
    return String::new();
}
