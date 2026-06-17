use crate::logging;

pub fn ping() -> bool {
    logging::success("connection to daemon successful");
    return false;
}

pub fn login(pass: String, password: &mut String) -> bool {
    password.clear();
    password.push_str(pass.as_str());
    return false;
}

pub fn stop() -> bool {
    true
}

pub fn encrypt(token: String, password: &String) -> bool {
    logging::info(format!("token: {}, pass: {}", token, password).as_str());
    return false;
}

pub fn decrypt(token: String, password: &String) -> bool {
    logging::info(format!("token: {}, pass: {}", token, password).as_str());
    return false;
}
