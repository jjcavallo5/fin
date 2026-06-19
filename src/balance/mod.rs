use crate::plaid;

pub async fn balance() {
    let linked_items = plaid::get_linked_accounts().await;

    for item in linked_items {
        println!(
            "\n\x1B[1m{}:\x1B[0m\n",
            item.plaid_item.item.institution_name
        );

        for account in item.plaid_item.accounts {
            println!(
                "  {}: ${} (${})",
                account.name, account.balances.current, account.balances.available
            );
        }
    }
    println!();
}
