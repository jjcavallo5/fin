use crate::logging;

pub fn ping() -> bool {
    logging::success("connection to daemon successful");
    return false;
}

pub fn login(pass: String, mut password: String) -> bool {
    password = pass;
    return false;
}

pub fn stop() -> bool {
    true
}
