use std::env;

fn load_env() -> (String, String) {
    let client_id =
        env::var("PLAID_CLIENT_ID").unwrap_or_else(|err| panic!("Error: {}", err.to_string()));
    let secret =
        env::var("PLAID_SECRET").unwrap_or_else(|err| panic!("Error: {}", err.to_string()));

    return (client_id, secret);
}

pub fn link() {
    let (client_id, secret) = load_env();

    println!("{}, {}", client_id, secret)
}
