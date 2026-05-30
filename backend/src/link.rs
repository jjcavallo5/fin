use crate::utils;
use std::{env, process};

fn load_env() -> (String, String) {
    let client_id = env::var("PLAID_CLIENT_ID").unwrap_or_else(|_e| {
        utils::print_error("PLAID_CLIENT_ID environment variable not set");
        process::exit(1);
    });
    let secret = env::var("PLAID_SECRET").unwrap_or_else(|_e| {
        utils::print_error("PLAID_SECRET environment variable not set");
        process::exit(1);
    });

    return (client_id, secret);
}

pub fn link() {
    let (client_id, secret) = load_env();

    println!("{}, {}", client_id, secret)
}
