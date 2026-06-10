use crate::logging;
use std::io::Write;

const SOCKET_PTH: &str = "/tmp/fin.sock";

pub fn run_daemon() {
    let listener = match std::os::unix::net::UnixListener::bind(SOCKET_PTH) {
        Ok(proc) => proc,
        Err(_) => {
            logging::error("failed to start unix listened");
            std::process::exit(1)
        }
    };

    loop {
        match listener.accept() {
            Ok((socket, address)) => {
                logging::success(format!("socket: {:?}, address: {:?}", socket, address).as_str())
            }
            Err(_) => break,
        }
    }

    logging::info("exiting daemon...")
}

pub fn login() {
    spawn_daemon();
}

pub fn ping() {
    let mut stream = match std::os::unix::net::UnixStream::connect(SOCKET_PTH) {
        Ok(str) => str,
        Err(_) => {
            logging::error("ping failed");
            std::process::exit(1);
        }
    };

    let bytes = serde_json::to_vec("PING").unwrap();
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
