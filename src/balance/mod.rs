use crate::plaid;

pub async fn balance() {
    let linked_items = plaid::get_linked_accounts().await;

    for item in linked_items {
        println!(
            "\n\x1B[1m{}:\x1B[0m\n",
            item.plaid_item.item.institution_name
        );

        for account in item.plaid_item.accounts {
            match account.account_type {
                plaid::types::AccountType::Brokerage
                | plaid::types::AccountType::Investment
                | plaid::types::AccountType::Other
                | plaid::types::AccountType::Depository => println!(
                    "  {} ({}): ${} (${})",
                    account.name,
                    account.account_subtype,
                    format_cents(account.balances.current_cents),
                    account
                        .balances
                        .available_cents
                        .map(format_cents)
                        .unwrap_or_else(|| "unavailable".to_string())
                ),
                plaid::types::AccountType::Credit | plaid::types::AccountType::Loan => println!(
                    "  {} ({}): -${} (-${})",
                    account.name,
                    account.account_subtype,
                    format_cents(account.balances.current_cents),
                    account
                        .balances
                        .available_cents
                        .map(format_cents)
                        .unwrap_or_else(|| "unavailable".to_string())
                ),
            }
        }
    }
    println!();
}

pub async fn net_worth() {
    let linked_items = plaid::get_linked_accounts().await;

    let mut net_worth = 0_i64;

    for item in linked_items {
        println!(
            "\n\x1B[1m{}:\x1B[0m\n",
            item.plaid_item.item.institution_name
        );

        for account in item.plaid_item.accounts {
            match account.account_type {
                plaid::types::AccountType::Brokerage
                | plaid::types::AccountType::Investment
                | plaid::types::AccountType::Other
                | plaid::types::AccountType::Depository => {
                    println!(
                        "  {} ({}): \x1b[32;1m+${}\x1b[0m",
                        account.name,
                        account.account_subtype,
                        format_cents(account.balances.current_cents),
                    );
                    net_worth += account.balances.current_cents;
                }
                plaid::types::AccountType::Credit | plaid::types::AccountType::Loan => {
                    println!(
                        "  {} ({}): \x1b[31;1m-${}\x1b[0m",
                        account.name,
                        account.account_subtype,
                        format_cents(account.balances.current_cents),
                    );
                    net_worth -= account.balances.current_cents;
                }
            }
        }
    }

    println!("\n\x1B[1mNet Worth: ${}\x1B[0m\n", format_cents(net_worth));
}

fn format_cents(cents: i64) -> String {
    let sign = if cents < 0 { "-" } else { "" };
    let absolute = cents.unsigned_abs();
    format!("{sign}{}.{:02}", absolute / 100, absolute % 100)
}
