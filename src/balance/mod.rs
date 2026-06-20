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
                    account.balances.current,
                    account.balances.available
                ),
                plaid::types::AccountType::Credit | plaid::types::AccountType::Loan => println!(
                    "  {} ({}): -${} (-${})",
                    account.name,
                    account.account_subtype,
                    account.balances.current,
                    account.balances.available
                ),
            }
        }
    }
    println!();
}
