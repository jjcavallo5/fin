use clap::{Parser, Subcommand};
use tokio;
mod balance;
mod cache;
mod link;
mod plaid;
mod utils;

#[derive(Parser, Debug)]
#[command(name = "fin")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    Link,
    Balance,
}

#[tokio::main]
async fn main() {
    let args = Cli::parse();

    match &args.command {
        Commands::Link => link::link().await,
        Commands::Balance => balance::balance().await,
    }
}
