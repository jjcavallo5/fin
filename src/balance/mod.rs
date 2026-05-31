use crate::plaid;
mod types;

pub async fn balance() {
    let linked_items = plaid::get_linked_items().await;

    for item in linked_items {
        println!("\x1B[1m{}:\x1B[0m\n", item.item.institution_name);

        for account in item.accounts {
            println!("  {}: ${}", account.name, account.balances.available);
        }
        println!()
    }
}
