use crate::daemon::types;
use crate::logging;

pub fn ping() -> types::DaemonResponse {
    logging::success("connection to daemon successful");
    return types::DaemonResponse::Ok;
}

pub fn login(pass: String, password: &mut String) -> types::DaemonResponse {
    password.clear();
    password.push_str(pass.as_str());
    return types::DaemonResponse::Ok;
}

pub fn stop() -> types::DaemonResponse {
    return types::DaemonResponse::Quit;
}

pub fn encrypt(token: String, password: &String) -> types::DaemonResponse {
    logging::info(format!("token: {}, pass: {}", token, password).as_str());
    let mut encrypted_token = String::new();
    encrypted_token.push_str(&token);
    encrypted_token.push_str("ASLKFJAKSJFLASJF");
    return types::DaemonResponse::Data {
        token: encrypted_token,
    };
}

pub fn decrypt(token: String, password: &String) -> types::DaemonResponse {
    logging::info(format!("token: {}, pass: {}", token, password).as_str());
    return types::DaemonResponse::Data { token };
}
