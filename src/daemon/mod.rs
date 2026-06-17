mod handlers;
use crate::logging;
use serde::{Deserialize, Serialize};
use std::io::{Read, Write};

const SOCKET_PTH: &str = "/tmp/fin.sock";

#[derive(Debug, Serialize, Deserialize)]
enum DaemonRequest {
    Ping,
    Stop,
    Login { pass: String },
}

fn handle_request(buffer: Vec<u8>) -> bool {
    let decoded_req: DaemonRequest = serde_json::from_slice(&buffer).unwrap();
    logging::success(format!("daemon received: {:?}", decoded_req).as_str());

    let mut password = String::new();

    let should_exit = match decoded_req {
        DaemonRequest::Ping => handlers::ping(),
        DaemonRequest::Login { pass } => handlers::login(pass, password),
        DaemonRequest::Stop => handlers::stop(),
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

    loop {
        match listener.accept() {
            Ok((mut socket, _)) => {
                let mut buffer: Vec<u8> = Vec::new();
                socket.read_to_end(&mut buffer).unwrap();
                let should_break = handle_request(buffer);

                if should_break {
                    break;
                }
            }
            Err(_) => break,
        }
    }

    logging::info("exiting daemon...")
}

pub fn login() {
    println!("Enter password: ");
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .expect("Incorrect password");
    spawn_daemon();
}

pub fn quit() {
    let mut stream = match std::os::unix::net::UnixStream::connect(SOCKET_PTH) {
        Ok(str) => str,
        Err(_) => {
            logging::error("daemon not running");
            std::process::exit(1);
        }
    };

    let bytes = serde_json::to_vec(&DaemonRequest::Stop).unwrap();
    match stream.write_all(&bytes) {
        Ok(_) => logging::success("exited daemon"),
        Err(_) => logging::error("failed to quit daemon"),
    }
}

pub fn ping() {
    let mut stream = match std::os::unix::net::UnixStream::connect(SOCKET_PTH) {
        Ok(str) => str,
        Err(_) => {
            logging::error("ping failed");
            std::process::exit(1);
        }
    };

    let bytes = serde_json::to_vec(&DaemonRequest::Ping).unwrap();
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
