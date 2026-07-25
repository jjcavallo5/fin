#[cfg(debug_assertions)]
pub const PLAID_BASE_URL: &str = "https://sandbox.plaid.com";
#[cfg(not(debug_assertions))]
pub const PLAID_BASE_URL: &str = "https://production.plaid.com";

#[cfg(debug_assertions)]
pub const DATABASE_FILENAME: &str = "sandbox.db";
#[cfg(not(debug_assertions))]
pub const DATABASE_FILENAME: &str = "fin.db";

pub fn plaid_endpoint(path: &str) -> String {
    format!("{PLAID_BASE_URL}/{path}")
}

#[cfg(test)]
mod tests {
    use super::{DATABASE_FILENAME, PLAID_BASE_URL, plaid_endpoint};

    #[test]
    fn build_selects_matching_plaid_and_database_environments() {
        if cfg!(debug_assertions) {
            assert_eq!(PLAID_BASE_URL, "https://sandbox.plaid.com");
            assert_eq!(DATABASE_FILENAME, "sandbox.db");
        } else {
            assert_eq!(PLAID_BASE_URL, "https://production.plaid.com");
            assert_eq!(DATABASE_FILENAME, "fin.db");
        }
    }

    #[test]
    fn builds_plaid_endpoint() {
        assert_eq!(
            plaid_endpoint("accounts/get"),
            format!("{PLAID_BASE_URL}/accounts/get")
        );
    }
}
